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

use std::sync::Arc;

pub use pallet_bridge_rpc_runtime_api::BridgeApi as BridgeRuntimeApi;

use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use primitives::{ChainId, ChainNonce};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};

#[rpc]
pub trait BridgeApi<BlockHash> {
    #[rpc(name = "bridge_hasBridged")]
    fn has_bridged(&self, id: ChainId, nonce: ChainNonce, at: Option<BlockHash>) -> Result<bool>;
}

/// A struct that implements the [`BridgeApi`].
pub struct Bridge<C, B> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<B>,
}

impl<C, B> Bridge<C, B> {
    /// Create new `Bridge` with the given reference to the client.
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

pub enum Error {
    RuntimeError,
    CheckProposalError,
}

impl From<Error> for i64 {
    fn from(e: Error) -> i64 {
        match e {
            Error::RuntimeError => 1,
            Error::CheckProposalError => 2,
        }
    }
}

impl<C, Block> BridgeApi<<Block as BlockT>::Hash> for Bridge<C, Block>
where
    Block: BlockT,
    C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: BridgeRuntimeApi<Block>,
{
    fn has_bridged(
        &self,
        chain_id: ChainId,
        chain_nonce: ChainNonce,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<bool> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or(
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash,
        ));
        api.has_bridged(&at, chain_id, chain_nonce)
            .map_err(runtime_error_into_rpc_error)?
            .map_err(check_proposal_error_into_rpc_error)
    }
}

/// Converts a runtime trap into an RPC error.
fn runtime_error_into_rpc_error(err: impl std::fmt::Debug) -> RpcError {
    RpcError {
        code: ErrorCode::ServerError(Error::RuntimeError.into()),
        message: "Runtime trapped".into(),
        data: Some(format!("{:?}", err).into()),
    }
}

/// Converts an  error into an RPC error.
fn check_proposal_error_into_rpc_error(err: impl std::fmt::Debug) -> RpcError {
    RpcError {
        code: ErrorCode::ServerError(Error::CheckProposalError.into()),
        message: "Not able to check if the poposal is finished".into(),
        data: Some(format!("{:?}", err).into()),
    }
}
