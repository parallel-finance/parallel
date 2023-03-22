
//! Autogenerated weights for `pallet_farming`
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
// --pallet=pallet_farming
// --extrinsic=*
// --steps=50
// --repeat=20
// --output=./runtime/parallel/src/weights/pallet_farming.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_farming`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_farming::WeightInfo for WeightInfo<T> {
	// Storage: Farming Pools (r:1 w:1)
	fn create() -> Weight {
		// Minimum execution time: 38_804 nanoseconds.
		Weight::from_ref_time(39_612_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Farming Pools (r:1 w:1)
	fn set_pool_status() -> Weight {
		// Minimum execution time: 39_079 nanoseconds.
		Weight::from_ref_time(39_694_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Farming Pools (r:1 w:1)
	fn set_pool_cool_down_duration() -> Weight {
		// Minimum execution time: 39_297 nanoseconds.
		Weight::from_ref_time(39_998_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Farming Pools (r:1 w:1)
	fn reset_pool_unlock_height() -> Weight {
		// Minimum execution time: 40_162 nanoseconds.
		Weight::from_ref_time(40_765_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Farming Pools (r:1 w:1)
	// Storage: Farming Positions (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:2 w:2)
	// Storage: System Account (r:1 w:1)
	fn deposit() -> Weight {
		// Minimum execution time: 132_925 nanoseconds.
		Weight::from_ref_time(133_883_000)
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(6))
	}
	// Storage: Farming Pools (r:1 w:1)
	// Storage: Farming Positions (r:1 w:1)
	fn withdraw() -> Weight {
		// Minimum execution time: 84_745 nanoseconds.
		Weight::from_ref_time(86_054_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: Farming Pools (r:1 w:0)
	// Storage: Farming Positions (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:2 w:2)
	fn redeem() -> Weight {
		// Minimum execution time: 104_304 nanoseconds.
		Weight::from_ref_time(104_957_000)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	// Storage: Farming Pools (r:1 w:1)
	// Storage: Farming Positions (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:2 w:2)
	fn claim() -> Weight {
		// Minimum execution time: 125_473 nanoseconds.
		Weight::from_ref_time(126_892_000)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(5))
	}
	// Storage: Farming Pools (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:2 w:2)
	// Storage: System Account (r:1 w:1)
	fn dispatch_reward() -> Weight {
		// Minimum execution time: 115_914 nanoseconds.
		Weight::from_ref_time(117_103_000)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(5))
	}
}
