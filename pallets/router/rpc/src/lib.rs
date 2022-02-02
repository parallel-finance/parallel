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

pub use pallet_router_rpc_runtime_api::RouterApi as RouterRuntimeApi;

use codec::Codec;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use primitives::CurrencyId;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use sp_std::vec::Vec;

#[rpc]
pub trait RouterApi<BlockHash, AccountId> {
    #[rpc(name = "router_getBestRoute")]
    fn get_best_route(
        &self,
        token_in: CurrencyId,
        token_out: CurrencyId,
    ) -> Result<Vec<CurrencyId>>;
}

/// A struct that implements the [`RouteApi`].
pub struct Router<C, B> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<B>,
}

impl<C, B> Router<C, B> {
    /// Create new `Route` with the given reference to the client.
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

pub enum Error {
    RuntimeError,
    RouterError,
}

impl From<Error> for i64 {
    fn from(e: Error) -> i64 {
        match e {
            Error::RuntimeError => 1,
            Error::RouterError => 2,
        }
    }
}

impl<C, Block, AccountId> RouterApi<<Block as BlockT>::Hash, AccountId> for Router<C, Block>
where
    Block: BlockT,
    C: Send + Sync + 'static,
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block>,
    C::Api: RouterRuntimeApi<Block, AccountId>,
    AccountId: Codec,
{
    fn get_best_route(
        &self,
        token_in: CurrencyId,
        token_out: CurrencyId,
    ) -> Result<Vec<CurrencyId>> {
        let api = self.client.runtime_api();

        api.get_best_route(token_in, token_out)
            .map_err(runtime_error_into_rpc_error)?
            .map_error(other_rpc_error)
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

// TODO: Rename this
fn other_rpc_error(err: impl std::fmt::Debug) -> RpcError {
    RpcError {
        code: ErrorCode::ServerError(Error::AccountLiquidityError.into()),
        message: "Some error message here".into(),
        data: Some(format!("{:?}", err).into()),
    }
}
