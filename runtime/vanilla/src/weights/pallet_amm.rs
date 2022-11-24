
//! Autogenerated weights for `pallet_amm`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-05-30, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("vanilla-dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/parallel
// benchmark
// pallet
// --chain=vanilla-dev
// --execution=wasm
// --wasm-execution=compiled
// --pallet=pallet_amm
// --extrinsic=*
// --steps=50
// --repeat=20
// --output=./runtime/vanilla/src/weights/pallet_amm.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_amm`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_amm::WeightInfo for WeightInfo<T> {
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: AMM Pools (r:1 w:1)
	// Storage: Assets Asset (r:3 w:3)
	// Storage: Assets Account (r:5 w:5)
	fn add_liquidity() -> Weight {
		(209_348_000 as u64)
			.saturating_add(T::DbWeight::get().reads(10 as u64))
			.saturating_add(T::DbWeight::get().writes(10 as u64))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: AMM Pools (r:1 w:1)
	// Storage: Assets Asset (r:3 w:3)
	// Storage: Assets Account (r:5 w:5)
	fn remove_liquidity() -> Weight {
		(226_453_000 as u64)
			.saturating_add(T::DbWeight::get().reads(10 as u64))
			.saturating_add(T::DbWeight::get().writes(10 as u64))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: AMM Pools (r:1 w:1)
	// Storage: Assets Asset (r:3 w:3)
	// Storage: Assets Account (r:6 w:6)
	// Storage: System Account (r:2 w:2)
	fn create_pool() -> Weight {
		(278_326_000 as u64)
			.saturating_add(T::DbWeight::get().reads(13 as u64))
			.saturating_add(T::DbWeight::get().writes(13 as u64))
	}
	// Storage: AMM ProtocolFee (r:0 w:1)
	fn update_protocol_fee() -> Weight {
			(4_067_000 as u64).saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: AMM ProtocolFeeReceiver (r:0 w:1)
	fn update_protocol_fee_receiver() -> Weight {
			(4_114_000 as u64).saturating_add(T::DbWeight::get().writes(1 as u64))
	}
}
