
//! Autogenerated weights for `pallet_amm`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-10-19, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `ip-172-88-3-164`, CPU: `Intel(R) Xeon(R) Platinum 8124M CPU @ 3.00GHz`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("heiko-dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/parallel
// benchmark
// pallet
// --chain=heiko-dev
// --execution=wasm
// --wasm-execution=compiled
// --pallet=pallet_amm
// --extrinsic=*
// --steps=50
// --repeat=20
// --output=./runtime/heiko/src/weights/pallet_amm.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_amm`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_amm::WeightInfo for WeightInfo<T> {
	// Storage: AMM Pools (r:1 w:1)
	// Storage: AMM ProtocolFee (r:1 w:0)
	// Storage: Assets Asset (r:3 w:3)
	// Storage: Assets Account (r:5 w:5)
	fn add_liquidity() -> Weight {
		(162_017_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(10 as Weight))
			.saturating_add(T::DbWeight::get().writes(9 as Weight))
	}
	// Storage: AMM Pools (r:1 w:1)
	// Storage: AMM ProtocolFee (r:1 w:0)
	// Storage: Assets Asset (r:3 w:3)
	// Storage: Assets Account (r:5 w:5)
	fn remove_liquidity() -> Weight {
		(173_662_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(10 as Weight))
			.saturating_add(T::DbWeight::get().writes(9 as Weight))
	}
	// Storage: AMM Pools (r:1 w:1)
	// Storage: Assets Asset (r:3 w:3)
	// Storage: Assets Account (r:6 w:6)
	// Storage: System Account (r:2 w:2)
	// Storage: AMM ProtocolFee (r:1 w:0)
	fn create_pool() -> Weight {
		(216_781_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(13 as Weight))
			.saturating_add(T::DbWeight::get().writes(12 as Weight))
	}
	// Storage: AMM ProtocolFee (r:0 w:1)
	fn update_protocol_fee() -> Weight {
		(23_037_000 as Weight)
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: AMM ProtocolFeeReceiver (r:0 w:1)
	fn update_protocol_fee_receiver() -> Weight {
		(24_415_000 as Weight)
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
}
