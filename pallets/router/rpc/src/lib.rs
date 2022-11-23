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
use jsonrpsee::{
    core::{async_trait, Error as JsonRpseeError, RpcResult},
    proc_macros::rpc,
    types::error::{CallError, ErrorCode, ErrorObject},
};
use primitives::CurrencyId;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_rpc::number::NumberOrHex;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use sp_std::vec::Vec;

#[rpc(client, server)]
pub trait RouterApi<BlockHash, Balance>
where
    Balance: Codec + Copy + TryFrom<NumberOrHex>,
{
    #[method(name = "router_getBestRoute")]
    fn get_best_route(
        &self,
        amount: NumberOrHex,
        token_in: CurrencyId,
        token_out: CurrencyId,
        reversed: bool,
        at: Option<BlockHash>,
    ) -> RpcResult<(Vec<CurrencyId>, NumberOrHex)>;
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

impl From<Error> for i32 {
    fn from(e: Error) -> i32 {
        match e {
            Error::RuntimeError => 1,
            Error::RouterError => 2,
        }
    }
}

#[async_trait]
impl<C, Block, Balance> RouterApiServer<<Block as BlockT>::Hash, Balance> for Router<C, Block>
where
    Block: BlockT,
    C: Send + Sync + 'static,
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block>,
    C::Api: RouterRuntimeApi<Block, Balance>,
    Balance: Codec + Copy + TryFrom<NumberOrHex> + Into<NumberOrHex> + std::fmt::Display,
{
    fn get_best_route(
        &self,
        amount: NumberOrHex,
        token_in: CurrencyId,
        token_out: CurrencyId,
        reversed: bool,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<(Vec<CurrencyId>, NumberOrHex)> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or(self.client.info().best_hash));
        let (route, amt) = api
            .get_best_route(
                &at,
                decode_hex(amount, "balance")?,
                token_in,
                token_out,
                reversed,
            )
            .map_err(runtime_error_into_rpc_error)?
            .map_err(smart_route_rpc_error)?;
        Ok((route, try_into_rpc_balance(amt)?))
    }
}

/// Converts a runtime trap into an RPC error.
fn runtime_error_into_rpc_error(err: impl std::fmt::Debug) -> JsonRpseeError {
    JsonRpseeError::RuntimeCall(CallError::Custom(ErrorObject::owned(
        Error::RuntimeError.into(),
        "Runtime trapped",
        Some(format!("{:?}", err)),
    )))
}

fn smart_route_rpc_error(err: impl std::fmt::Debug) -> JsonRpseeError {
    JsonRpseeError::RuntimeCall(CallError::Custom(ErrorObject::owned(
        Error::RouterError.into(),
        "Smart router error",
        Some(format!("{:?}", err)),
    )))
}

fn decode_hex<H: std::fmt::Debug + Copy, T: TryFrom<H>>(
    from: H,
    name: &str,
) -> Result<T, JsonRpseeError> {
    from.try_into().map_err(|_| {
        JsonRpseeError::RuntimeCall(CallError::Custom(ErrorObject::owned(
            ErrorCode::InvalidParams.code(),
            format!("{:?} does not fit into the {} type", from, name),
            None::<()>,
        )))
    })
}

fn try_into_rpc_balance<T: std::fmt::Display + Copy + TryInto<NumberOrHex>>(
    value: T,
) -> Result<NumberOrHex, JsonRpseeError> {
    value.try_into().map_err(|_| {
        JsonRpseeError::RuntimeCall(CallError::Custom(ErrorObject::owned(
            ErrorCode::InvalidParams.code(),
            format!("{} doesn't fit in NumberOrHex representation", value),
            None::<()>,
        )))
    })
}
