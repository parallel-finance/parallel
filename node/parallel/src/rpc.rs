#![warn(missing_docs)]

use std::sync::Arc;

use crate::client::Block;
use primitives::{AccountId, AssetId, Balance, DataProviderId, Index, TimeStampedPrice};
pub use sc_rpc_api::DenyUnsafe;
use sc_transaction_pool_api::TransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};

/// Full client dependencies.
pub struct FullDeps<C, P> {
    /// The client instance to use.
    pub client: Arc<C>,
    /// Transaction pool instance.
    pub pool: Arc<P>,
    /// Whether to deny unsafe calls
    pub deny_unsafe: DenyUnsafe,
}

/// Instantiate all full RPC extensions.
pub fn create_full<C, P>(
    deps: FullDeps<C, P>,
) -> Result<jsonrpc_core::IoHandler<sc_rpc::Metadata>, sc_service::Error>
where
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + 'static,
    C: Send + Sync + 'static,
    C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>,
    C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
    C::Api: orml_oracle_rpc::OracleRuntimeApi<Block, DataProviderId, AssetId, TimeStampedPrice>,
    C::Api: pallet_loans_rpc::LoansRuntimeApi<Block, AccountId>,
    C::Api: BlockBuilder<Block>,
    P: TransactionPool + 'static,
{
    use orml_oracle_rpc::{Oracle, OracleApi};
    use pallet_loans_rpc::{Loans, LoansApi};
    use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApi};
    use substrate_frame_rpc_system::{FullSystem, SystemApi};

    let mut io = jsonrpc_core::IoHandler::default();
    let FullDeps {
        client,
        pool,
        deny_unsafe,
    } = deps;

    io.extend_with(SystemApi::to_delegate(FullSystem::new(
        client.clone(),
        pool,
        deny_unsafe,
    )));

    io.extend_with(TransactionPaymentApi::to_delegate(TransactionPayment::new(
        client.clone(),
    )));

    // Extend this RPC with a custom API by using the following syntax.
    // `YourRpcStruct` should have a reference to a client, which is needed
    // to call into the runtime.
    // `io.extend_with(YourRpcTrait::to_delegate(YourRpcStruct::new(ReferenceToClient, ...)));`
    io.extend_with(OracleApi::to_delegate(Oracle::new(client.clone())));

    io.extend_with(LoansApi::to_delegate(Loans::new(client.clone())));

    Ok(io)
}
