#![cfg_attr(not(feature = "std"), no_std)]

pub mod constants;
pub mod evm_migration;
pub mod precompiles;

pub use fp_rpc;
pub use fp_self_contained;
pub use pallet_evm_precompile_assets_erc20::AddressToAssetId;
pub use pallet_evm_precompile_balances_erc20::Erc20Metadata;

use frame_support::log;
use sp_std::{marker::PhantomData, result::Result};
use xcm::latest::{
    Instruction::{self, *},
    MultiLocation, Weight,
};
use xcm_executor::traits::{OnResponse, ShouldExecute};

/// TODO: Belowing code can be removed once upgrade to polkadot-v1.0.0
/// Copy from polkadot-v0.9.38
/// Polakdot-v1.0.0 have made changes of ShouldExecute
pub struct AllowKnownQueryResponses<ResponseHandler>(PhantomData<ResponseHandler>);
impl<ResponseHandler: OnResponse> ShouldExecute for AllowKnownQueryResponses<ResponseHandler> {
    fn should_execute<RuntimeCall>(
        origin: &MultiLocation,
        instructions: &mut [Instruction<RuntimeCall>],
        _max_weight: Weight,
        _weight_credit: &mut Weight,
    ) -> Result<(), ()> {
        log::trace!(
            target: "xcm::barriers",
            "Parallel AllowKnownQueryResponses origin: {:?}, instructions: {:?}, max_weight: {:?}, weight_credit: {:?}",
            origin, instructions, _max_weight, _weight_credit,
        );
        // ignore other instructions
        // ensure!(instructions.len() == 1, ());
        match instructions.first() {
            Some(QueryResponse {
                query_id, querier, ..
            }) if ResponseHandler::expecting_response(origin, *query_id, querier.as_ref()) => {
                Ok(())
            }
            _ => Err(()),
        }
    }
}
