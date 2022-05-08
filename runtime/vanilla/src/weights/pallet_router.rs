
//! Autogenerated weights for `pallet_router`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-05-08, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("vanilla-dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/parallel
// benchmark
// pallet
// --chain=vanilla-dev
// --execution=wasm
// --wasm-execution=compiled
// --pallet=pallet_router
// --extrinsic=*
// --steps=50
// --repeat=20
// --output=./runtime/vanilla/src/weights/pallet_router.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_router`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_router::WeightInfo for WeightInfo<T> {
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: Assets Asset (r:2 w:2)
	// Storage: Assets Account (r:4 w:4)
	// Storage: AMM Pools (r:1 w:1)
	fn swap_exact_tokens_for_tokens() -> Weight {
		(189_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(8 as Weight))
			.saturating_add(T::DbWeight::get().writes(8 as Weight))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: AMM Pools (r:1 w:1)
	// Storage: Assets Account (r:4 w:4)
	// Storage: Assets Asset (r:2 w:2)
	fn swap_tokens_for_exact_tokens() -> Weight {
		(186_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(8 as Weight))
			.saturating_add(T::DbWeight::get().writes(8 as Weight))
	}
}
