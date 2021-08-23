// Copyright 2021 Parallel Finance Developer.
// This file is part of Parallel Finance.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
// http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
use primitives::*;
use sc_client_api::{Backend as BackendT, BlockchainEvents, KeyIterator};
use sp_api::{CallApiAt, NumberFor, ProvideRuntimeApi};
use sp_blockchain::HeaderBackend;
use sp_consensus::BlockStatus;
use sp_runtime::{
    generic::{self, SignedBlock},
    traits::{BlakeTwo256, Block as BlockT},
    Justifications,
};
use sp_storage::{ChildInfo, PrefixedStorageKey, StorageData, StorageKey};
use std::sync::Arc;

/// Header type.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;

/// Block type.
pub type Block = generic::Block<Header, sp_runtime::OpaqueExtrinsic>;

/// Block ID.
pub type BlockId = generic::BlockId<Block>;

/// A set of APIs that parallel-like runtimes must implement.
pub trait RuntimeApiCollection:
    sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
    + sp_api::ApiExt<Block>
    + sp_block_builder::BlockBuilder<Block>
    + frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index>
    + pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance>
    + orml_oracle_rpc::OracleRuntimeApi<Block, DataProviderId, CurrencyId, TimeStampedPrice>
    + sp_consensus_aura::AuraApi<Block, AuraId>
    + sp_api::Metadata<Block>
    + sp_offchain::OffchainWorkerApi<Block>
    + sp_session::SessionKeys<Block>
    + cumulus_primitives_core::CollectCollationInfo<Block>
    + pallet_loans_rpc::LoansRuntimeApi<Block, AccountId>
where
    <Self as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
{
}

impl<Api> RuntimeApiCollection for Api
where
    Api: sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
        + sp_api::ApiExt<Block>
        + sp_block_builder::BlockBuilder<Block>
        + frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index>
        + pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance>
        + orml_oracle_rpc::OracleRuntimeApi<Block, DataProviderId, CurrencyId, TimeStampedPrice>
        + sp_consensus_aura::AuraApi<Block, AuraId>
        + sp_api::Metadata<Block>
        + sp_offchain::OffchainWorkerApi<Block>
        + sp_session::SessionKeys<Block>
        + cumulus_primitives_core::CollectCollationInfo<Block>
        + pallet_loans_rpc::LoansRuntimeApi<Block, AccountId>,
    <Self as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
{
}

/// Trait that abstracts over all available client implementations.
///
/// For a concrete type there exists [`Client`].
pub trait AbstractClient<Block, Backend>:
    BlockchainEvents<Block>
    + Sized
    + Send
    + Sync
    + ProvideRuntimeApi<Block>
    + HeaderBackend<Block>
    + CallApiAt<Block, StateBackend = Backend::State>
where
    Block: BlockT,
    Backend: BackendT<Block>,
    Backend::State: sp_api::StateBackend<BlakeTwo256>,
    Self::Api: crate::client::RuntimeApiCollection<StateBackend = Backend::State>,
{
}

impl<Block, Backend, Client> AbstractClient<Block, Backend> for Client
where
    Block: BlockT,
    Backend: BackendT<Block>,
    Backend::State: sp_api::StateBackend<BlakeTwo256>,
    Client: BlockchainEvents<Block>
        + ProvideRuntimeApi<Block>
        + HeaderBackend<Block>
        + Sized
        + Send
        + Sync
        + CallApiAt<Block, StateBackend = Backend::State>,
    Client::Api: crate::client::RuntimeApiCollection<StateBackend = Backend::State>,
{
}

/// Execute something with the client instance.
///
/// As there exist multiple chains inside Parallel, like Parallel itself, Heiko etc,
/// there can exist different kinds of client types. As these client types differ in the generics
/// that are being used, we can not easily return them from a function. For returning them from a
/// function there exists [`Client`]. However, the problem on how to use this client instance still
/// exists. This trait "solves" it in a dirty way. It requires a type to implement this trait and
/// than the [`execute_with_client`](ExecuteWithClient::execute_with_client) function can be called
/// with any possible client instance.
///
/// In a perfect world, we could make a closure work in this way.
pub trait ExecuteWithClient {
    /// The return type when calling this instance.
    type Output;

    /// Execute whatever should be executed with the given client instance.
    fn execute_with_client<Client, Api, Backend>(self, client: Arc<Client>) -> Self::Output
    where
        <Api as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
        Backend: sc_client_api::Backend<Block>,
        Backend::State: sp_api::StateBackend<BlakeTwo256>,
        Api: RuntimeApiCollection<StateBackend = Backend::State>,
        Client: AbstractClient<Block, Backend, Api = Api> + 'static;
}

/// A handle to a Parallel client instance.
///
/// The Parallel service supports multiple different runtimes (Heiko, Parallel itself, etc). As each runtime has a
/// specialized client, we need to hide them behind a trait. This is this trait.
///
/// When wanting to work with the inner client, you need to use `execute_with`.
///
/// See [`ExecuteWithClient`](trait.ExecuteWithClient.html) for more information.
pub trait ClientHandle {
    /// Execute the given something with the client.
    fn execute_with<T: ExecuteWithClient>(&self, t: T) -> T::Output;
}

/// A client instance of Parallel.
///
/// See [`ExecuteWithClient`] for more information.
#[allow(dead_code)]
#[derive(Clone)]
pub enum Client {
    Parallel(
        Arc<
            crate::service::FullClient<
                parallel_runtime::RuntimeApi,
                crate::service::ParallelExecutor,
            >,
        >,
    ),
    Heiko(
        Arc<crate::service::FullClient<heiko_runtime::RuntimeApi, crate::service::HeikoExecutor>>,
    ),
}

impl ClientHandle for Client {
    fn execute_with<T: ExecuteWithClient>(&self, t: T) -> T::Output {
        match self {
            Self::Parallel(client) => {
                T::execute_with_client::<_, _, crate::service::FullBackend>(t, client.clone())
            }
            Self::Heiko(client) => {
                T::execute_with_client::<_, _, crate::service::FullBackend>(t, client.clone())
            }
        }
    }
}

impl sc_client_api::UsageProvider<Block> for Client {
    fn usage_info(&self) -> sc_client_api::ClientInfo<Block> {
        match self {
            Self::Parallel(client) => client.usage_info(),
            Self::Heiko(client) => client.usage_info(),
        }
    }
}

impl sc_client_api::BlockBackend<Block> for Client {
    fn block_body(
        &self,
        id: &BlockId,
    ) -> sp_blockchain::Result<Option<Vec<<Block as BlockT>::Extrinsic>>> {
        match self {
            Self::Parallel(client) => client.block_body(id),
            Self::Heiko(client) => client.block_body(id),
        }
    }

    fn block_indexed_body(&self, id: &BlockId) -> sp_blockchain::Result<Option<Vec<Vec<u8>>>> {
        match self {
            Self::Parallel(client) => client.block_indexed_body(id),
            Self::Heiko(client) => client.block_indexed_body(id),
        }
    }

    fn block(&self, id: &BlockId) -> sp_blockchain::Result<Option<SignedBlock<Block>>> {
        match self {
            Self::Parallel(client) => client.block(id),
            Self::Heiko(client) => client.block(id),
        }
    }

    fn block_status(&self, id: &BlockId) -> sp_blockchain::Result<BlockStatus> {
        match self {
            Self::Parallel(client) => client.block_status(id),
            Self::Heiko(client) => client.block_status(id),
        }
    }

    fn justifications(&self, id: &BlockId) -> sp_blockchain::Result<Option<Justifications>> {
        match self {
            Self::Parallel(client) => client.justifications(id),
            Self::Heiko(client) => client.justifications(id),
        }
    }

    fn block_hash(
        &self,
        number: NumberFor<Block>,
    ) -> sp_blockchain::Result<Option<<Block as BlockT>::Hash>> {
        match self {
            Self::Parallel(client) => client.block_hash(number),
            Self::Heiko(client) => client.block_hash(number),
        }
    }

    fn indexed_transaction(
        &self,
        hash: &<Block as BlockT>::Hash,
    ) -> sp_blockchain::Result<Option<Vec<u8>>> {
        match self {
            Self::Parallel(client) => client.indexed_transaction(hash),
            Self::Heiko(client) => client.indexed_transaction(hash),
        }
    }

    fn has_indexed_transaction(
        &self,
        hash: &<Block as BlockT>::Hash,
    ) -> sp_blockchain::Result<bool> {
        match self {
            Self::Parallel(client) => client.has_indexed_transaction(hash),
            Self::Heiko(client) => client.has_indexed_transaction(hash),
        }
    }
}

impl sc_client_api::StorageProvider<Block, crate::service::FullBackend> for Client {
    fn storage(
        &self,
        id: &BlockId,
        key: &StorageKey,
    ) -> sp_blockchain::Result<Option<StorageData>> {
        match self {
            Self::Parallel(client) => client.storage(id, key),
            Self::Heiko(client) => client.storage(id, key),
        }
    }

    fn storage_keys(
        &self,
        id: &BlockId,
        key_prefix: &StorageKey,
    ) -> sp_blockchain::Result<Vec<StorageKey>> {
        match self {
            Self::Parallel(client) => client.storage_keys(id, key_prefix),
            Self::Heiko(client) => client.storage_keys(id, key_prefix),
        }
    }

    fn storage_hash(
        &self,
        id: &BlockId,
        key: &StorageKey,
    ) -> sp_blockchain::Result<Option<<Block as BlockT>::Hash>> {
        match self {
            Self::Parallel(client) => client.storage_hash(id, key),
            Self::Heiko(client) => client.storage_hash(id, key),
        }
    }

    fn storage_pairs(
        &self,
        id: &BlockId,
        key_prefix: &StorageKey,
    ) -> sp_blockchain::Result<Vec<(StorageKey, StorageData)>> {
        match self {
            Self::Parallel(client) => client.storage_pairs(id, key_prefix),
            Self::Heiko(client) => client.storage_pairs(id, key_prefix),
        }
    }

    fn storage_keys_iter<'a>(
        &self,
        id: &BlockId,
        prefix: Option<&'a StorageKey>,
        start_key: Option<&StorageKey>,
    ) -> sp_blockchain::Result<
        KeyIterator<
            'a,
            <crate::service::FullBackend as sc_client_api::Backend<Block>>::State,
            Block,
        >,
    > {
        match self {
            Self::Parallel(client) => client.storage_keys_iter(id, prefix, start_key),
            Self::Heiko(client) => client.storage_keys_iter(id, prefix, start_key),
        }
    }

    fn child_storage(
        &self,
        id: &BlockId,
        child_info: &ChildInfo,
        key: &StorageKey,
    ) -> sp_blockchain::Result<Option<StorageData>> {
        match self {
            Self::Parallel(client) => client.child_storage(id, child_info, key),
            Self::Heiko(client) => client.child_storage(id, child_info, key),
        }
    }

    fn child_storage_keys(
        &self,
        id: &BlockId,
        child_info: &ChildInfo,
        key_prefix: &StorageKey,
    ) -> sp_blockchain::Result<Vec<StorageKey>> {
        match self {
            Self::Parallel(client) => client.child_storage_keys(id, child_info, key_prefix),
            Self::Heiko(client) => client.child_storage_keys(id, child_info, key_prefix),
        }
    }

    fn child_storage_keys_iter<'a>(
        &self,
        id: &BlockId,
        child_info: ChildInfo,
        prefix: Option<&'a StorageKey>,
        start_key: Option<&StorageKey>,
    ) -> sp_blockchain::Result<
        KeyIterator<
            'a,
            <crate::service::FullBackend as sc_client_api::Backend<Block>>::State,
            Block,
        >,
    > {
        match self {
            Self::Parallel(client) => {
                client.child_storage_keys_iter(id, child_info, prefix, start_key)
            }
            Self::Heiko(client) => {
                client.child_storage_keys_iter(id, child_info, prefix, start_key)
            }
        }
    }

    fn child_storage_hash(
        &self,
        id: &BlockId,
        child_info: &ChildInfo,
        key: &StorageKey,
    ) -> sp_blockchain::Result<Option<<Block as BlockT>::Hash>> {
        match self {
            Self::Parallel(client) => client.child_storage_hash(id, child_info, key),
            Self::Heiko(client) => client.child_storage_hash(id, child_info, key),
        }
    }

    fn max_key_changes_range(
        &self,
        first: NumberFor<Block>,
        last: BlockId,
    ) -> sp_blockchain::Result<Option<(NumberFor<Block>, BlockId)>> {
        match self {
            Self::Parallel(client) => client.max_key_changes_range(first, last),
            Self::Heiko(client) => client.max_key_changes_range(first, last),
        }
    }

    fn key_changes(
        &self,
        first: NumberFor<Block>,
        last: BlockId,
        storage_key: Option<&PrefixedStorageKey>,
        key: &StorageKey,
    ) -> sp_blockchain::Result<Vec<(NumberFor<Block>, u32)>> {
        match self {
            Self::Parallel(client) => client.key_changes(first, last, storage_key, key),
            Self::Heiko(client) => client.key_changes(first, last, storage_key, key),
        }
    }
}

impl sp_blockchain::HeaderBackend<Block> for Client {
    fn header(&self, id: BlockId) -> sp_blockchain::Result<Option<Header>> {
        match self {
            Self::Parallel(client) => client.header(&id),
            Self::Heiko(client) => client.header(&id),
        }
    }

    fn info(&self) -> sp_blockchain::Info<Block> {
        match self {
            Self::Parallel(client) => client.info(),
            Self::Heiko(client) => client.info(),
        }
    }

    fn status(&self, id: BlockId) -> sp_blockchain::Result<sp_blockchain::BlockStatus> {
        match self {
            Self::Parallel(client) => client.status(id),
            Self::Heiko(client) => client.status(id),
        }
    }

    fn number(&self, hash: Hash) -> sp_blockchain::Result<Option<BlockNumber>> {
        match self {
            Self::Parallel(client) => client.number(hash),
            Self::Heiko(client) => client.number(hash),
        }
    }

    fn hash(&self, number: BlockNumber) -> sp_blockchain::Result<Option<Hash>> {
        match self {
            Self::Parallel(client) => client.hash(number),
            Self::Heiko(client) => client.hash(number),
        }
    }
}
