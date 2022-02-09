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

// pub use pallet_loans_rpc_runtime_api::LoansApi as LoansRuntimeApi;

use codec::Codec;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use primitives::{Liquidity, Shortfall};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};

#[rpc]
pub trait AMMAnalyticsAPI<BlockHash, AccountId> {
    #[rpc(name = "amm_getPools")]
    fn get_pools(
        &self,
        at: Option<BlockHash>,
    ) -> Result<Vec<(CurrencyId, CurrencyId)>> ;
}

/// A struct that implements the [`AMM`].
pub struct AMM<C, B> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<B>,
}

impl<C, B> AMM<C, B> {
    /// Create new `Loans` with the given reference to the client.
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

pub enum Error {
    RuntimeError,
    AnalyticsError,
}

impl From<Error> for i64 {
    fn from(e: Error) -> i64 {
        match e {
            Error::RuntimeError => 1,
            Error::AnalyticsError => 2,
        }
    }
}

impl<C, Block, AccountId> LoansApi<<Block as BlockT>::Hash, AccountId> for Loans<C, Block>
where
    Block: BlockT,
    C: Send + Sync + 'static,
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block>,
    C::Api: LoansRuntimeApi<Block, AccountId>,
    AccountId: Codec,
{
    fn get_pools(
        &self,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Vec<(CurrencyId, CurrencyId)>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or(
            self.client.info().best_hash,
        ));
        api.get_account_liquidity(&at, account)
            .map_err(runtime_error_into_rpc_error)?
            .map_err(account_liquidity_error_into_rpc_error)
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

/// Converts an account liquidity error into an RPC error.
fn account_liquidity_error_into_rpc_error(err: impl std::fmt::Debug) -> RpcError {
    RpcError {
        code: ErrorCode::ServerError(Error::AccountLiquidityError.into()),
        message: "Not able to get account liquidity".into(),
        data: Some(format!("{:?}", err).into()),
    }
}
