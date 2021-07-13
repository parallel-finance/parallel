use std::sync::Arc;

use codec::{Codec, Encode};
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use serde::{Deserialize, Serialize};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_core::Bytes;
use sp_runtime::FixedU128;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};

pub use pallet_loans_rpc_runtime_api::LoanApi as LoanRuntimeApi;

/// Retrieved MMR leaf and its proof.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AccountLiquidity<BlockHash> {
    /// Block hash the liquidity and shortfall were generated for.
    pub block_hash: BlockHash,
    /// SCALE-encoded liquidity data.
    pub liquidity: Bytes,
    /// SCALE-encoded shortfall data.
    pub shortfall: Bytes,
}

impl<BlockHash> AccountLiquidity<BlockHash> {
    /// Create new `AccountLiquidity` from given concrete `liquidity` and `shortfall`.
    pub fn new(block_hash: BlockHash, liquidity: FixedU128, shortfall: FixedU128) -> Self {
        Self {
            block_hash,
            liquidity: Bytes(liquidity.encode()),
            shortfall: Bytes(shortfall.encode()),
        }
    }
}

#[rpc]
pub trait LoanApi<BlockHash, AccountId> {
    #[rpc(name = "pallet_loans_get_account_liquidity")]
    fn get_account_liquidity(
        &self,
        account: AccountId,
        at: Option<BlockHash>,
    ) -> Result<AccountLiquidity<BlockHash>>;
}

/// A struct that implements the [`LoanApi`].
pub struct Loan<C, B> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<B>,
}

impl<C, B> Loan<C, B> {
    /// Create new `Loan` with the given reference to the client.
    pub fn new(client: Arc<C>) -> Self {
        Loan {
            client,
            _marker: Default::default(),
        }
    }
}

pub enum Error {
    RuntimeError,
}

impl From<Error> for i64 {
    fn from(e: Error) -> i64 {
        match e {
            Error::RuntimeError => 1,
        }
    }
}

impl<C, Block, AccountId> LoanApi<<Block as BlockT>::Hash, AccountId> for Loan<C, Block>
where
    Block: BlockT,
    C: Send + Sync + 'static,
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block>,
    C::Api: LoanRuntimeApi<Block, AccountId>,
    AccountId: Codec,
{
    fn get_account_liquidity(
        &self,
        account: AccountId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<AccountLiquidity<<Block as BlockT>::Hash>> {
        let api = self.client.runtime_api();
        let block_hash = at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash);

        let (liquidity, shortfall) = api
            .get_account_liquidity_with_context(
                &BlockId::hash(block_hash),
                sp_core::ExecutionContext::OffchainCall(None),
                account,
            )
            .unwrap()
            .map_err(runtime_error_into_rpc_error)?;

        Ok(AccountLiquidity::new(block_hash, liquidity, shortfall))
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
