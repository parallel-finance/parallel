#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::dispatch::DispatchResult;

pub trait EmergencyCallFilter<Call> {
    fn contains(call: &Call) -> bool;
}

// The registrar trait. We need to comply with this
pub trait AssetRegistrar<AssetId, Balance, AssetRegistrarMetadata> {
    // How to create an asset
    fn create_asset(
        asset: AssetId,
        min_balance: Balance,
        metadata: AssetRegistrarMetadata,
        // Wether or not an asset-receiving account increments the sufficient counter
        is_sufficient: bool,
    ) -> DispatchResult;
}
