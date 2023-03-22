
//! Autogenerated weights for `pallet_bridge`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-03-22, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `ip-172-88-3-164`, CPU: `Intel(R) Xeon(R) Platinum 8124M CPU @ 3.00GHz`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("parallel-dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/parallel
// benchmark
// pallet
// --chain=parallel-dev
// --execution=wasm
// --wasm-execution=compiled
// --pallet=pallet_bridge
// --extrinsic=*
// --steps=50
// --repeat=20
// --output=./runtime/parallel/src/weights/pallet_bridge.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_bridge`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_bridge::WeightInfo for WeightInfo<T> {
	// Storage: Bridge ChainNonces (r:1 w:1)
	// Storage: Bridge BridgeRegistry (r:0 w:1)
	fn register_chain() -> Weight {
		// Minimum execution time: 33_226 nanoseconds.
		Weight::from_ref_time(34_284_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: Bridge ChainNonces (r:1 w:1)
	// Storage: Bridge BridgeRegistry (r:0 w:1)
	fn unregister_chain() -> Weight {
		// Minimum execution time: 34_427 nanoseconds.
		Weight::from_ref_time(35_276_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: Bridge BridgeTokens (r:1 w:1)
	// Storage: Bridge AssetIds (r:1 w:1)
	fn register_bridge_token() -> Weight {
		// Minimum execution time: 35_767 nanoseconds.
		Weight::from_ref_time(36_381_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: Bridge AssetIds (r:1 w:1)
	// Storage: Bridge BridgeTokens (r:0 w:1)
	fn unregister_bridge_token() -> Weight {
		// Minimum execution time: 35_305 nanoseconds.
		Weight::from_ref_time(36_068_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: Bridge AssetIds (r:1 w:0)
	// Storage: Bridge BridgeTokens (r:1 w:1)
	fn set_bridge_token_fee() -> Weight {
		// Minimum execution time: 40_117 nanoseconds.
		Weight::from_ref_time(40_904_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Bridge AssetIds (r:1 w:0)
	// Storage: Bridge BridgeTokens (r:1 w:1)
	fn set_bridge_token_status() -> Weight {
		// Minimum execution time: 40_150 nanoseconds.
		Weight::from_ref_time(40_840_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Bridge AssetIds (r:1 w:0)
	// Storage: Bridge BridgeTokens (r:1 w:1)
	fn set_bridge_token_cap() -> Weight {
		// Minimum execution time: 39_857 nanoseconds.
		Weight::from_ref_time(40_549_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Bridge AssetIds (r:1 w:0)
	// Storage: Bridge BridgeTokens (r:1 w:1)
	fn clean_cap_accumulated_value() -> Weight {
		// Minimum execution time: 39_688 nanoseconds.
		Weight::from_ref_time(40_315_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Bridge ChainNonces (r:1 w:1)
	// Storage: Bridge AssetIds (r:1 w:0)
	// Storage: Bridge BridgeTokens (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	fn teleport() -> Weight {
		// Minimum execution time: 99_963 nanoseconds.
		Weight::from_ref_time(101_095_000)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	// Storage: Bridge ChainNonces (r:1 w:0)
	// Storage: Bridge BridgeRegistry (r:1 w:1)
	// Storage: Bridge AssetIds (r:1 w:0)
	// Storage: Bridge BridgeTokens (r:1 w:1)
	// Storage: Bridge ProposalVotes (r:1 w:1)
	// Storage: Bridge VoteThreshold (r:1 w:0)
	// Storage: BridgeMembership Members (r:1 w:0)
	// Storage: System Account (r:2 w:1)
	// Storage: Assets Metadata (r:1 w:0)
	fn materialize() -> Weight {
		// Minimum execution time: 157_410 nanoseconds.
		Weight::from_ref_time(158_875_000)
			.saturating_add(T::DbWeight::get().reads(10))
			.saturating_add(T::DbWeight::get().writes(4))
	}
}
