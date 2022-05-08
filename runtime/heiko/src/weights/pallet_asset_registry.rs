
//! Autogenerated weights for `pallet_asset_registry`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-05-08, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("heiko-dev"), DB CACHE: 1024

// Executed Command:
// ./parallel
// benchmark
// pallet
// --chain=heiko-dev
// --execution=wasm
// --wasm-execution=compiled
// --pallet=pallet_asset_registry
// --extrinsic=*
// --steps=50
// --repeat=20
// --output=./runtime/heiko/src/weights/pallet_asset_registry.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_asset_registry`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_asset_registry::WeightInfo for WeightInfo<T> {
	// Storage: AssetRegistry AssetIdType (r:1 w:1)
	// Storage: AssetRegistry AssetTypeId (r:0 w:1)
	fn register_asset() -> Weight {
		(37_335_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: AssetRegistry AssetTypeId (r:1 w:0)
	// Storage: AssetRegistry SupportedFeePaymentAssets (r:1 w:1)
	// Storage: AssetRegistry AssetTypeUnitsPerSecond (r:0 w:1)
	fn update_asset_units_per_second() -> Weight {
		(48_471_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: AssetRegistry SupportedFeePaymentAssets (r:1 w:1)
	// Storage: AssetRegistry AssetIdType (r:1 w:1)
	// Storage: AssetRegistry AssetTypeUnitsPerSecond (r:1 w:2)
	// Storage: AssetRegistry AssetTypeId (r:0 w:2)
	fn update_asset_type() -> Weight {
		(64_596_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(6 as Weight))
	}
	// Storage: AssetRegistry SupportedFeePaymentAssets (r:1 w:1)
	// Storage: AssetRegistry AssetTypeUnitsPerSecond (r:0 w:1)
	fn remove_fee_payment_asset() -> Weight {
		(40_493_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: AssetRegistry SupportedFeePaymentAssets (r:1 w:1)
	// Storage: AssetRegistry AssetIdType (r:1 w:1)
	// Storage: AssetRegistry AssetTypeUnitsPerSecond (r:0 w:1)
	// Storage: AssetRegistry AssetTypeId (r:0 w:1)
	fn deregister_asset() -> Weight {
		(50_378_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
	}
}
