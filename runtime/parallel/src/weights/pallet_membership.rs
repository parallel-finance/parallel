//! Autogenerated weights for pallet_membership
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 3.0.0
//! DATE: 2021-08-04, STEPS: `[50, ]`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("parallel"), DB CACHE: 128

// Executed Command:
// ./target/release/parallel
// benchmark
// --chain=parallel
// --execution=wasm
// --wasm-execution=compiled
// --pallet=pallet_membership
// --extrinsic=*
// --steps=50
// --repeat=20
// --raw
// --output=./runtime/parallel/src/weights//pallet_membership.rs

#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::all)]
#![rustfmt::skip]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for pallet_membership.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_membership::WeightInfo for WeightInfo<T> {
    fn add_member(m: u32) -> Weight {
        (36_358_000 as Weight)
            // Standard Error: 3_000
            .saturating_add((196_000 as Weight).saturating_mul(m as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    fn remove_member(m: u32) -> Weight {
        (43_655_000 as Weight)
            // Standard Error: 0
            .saturating_add((154_000 as Weight).saturating_mul(m as Weight))
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    fn swap_member(m: u32) -> Weight {
        (44_177_000 as Weight)
            // Standard Error: 0
            .saturating_add((174_000 as Weight).saturating_mul(m as Weight))
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    fn reset_member(m: u32) -> Weight {
        (44_439_000 as Weight)
            // Standard Error: 1_000
            .saturating_add((397_000 as Weight).saturating_mul(m as Weight))
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    fn change_key(m: u32) -> Weight {
        (46_274_000 as Weight)
            // Standard Error: 1_000
            .saturating_add((168_000 as Weight).saturating_mul(m as Weight))
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(4 as Weight))
    }
    fn set_prime(m: u32) -> Weight {
        (11_856_000 as Weight)
            // Standard Error: 0
            .saturating_add((122_000 as Weight).saturating_mul(m as Weight))
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    fn clear_prime(m: u32) -> Weight {
        (5_093_000 as Weight)
            // Standard Error: 0
            .saturating_add((2_000 as Weight).saturating_mul(m as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
}
