
//! Autogenerated weights for `pallet_utility`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-03-23, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `ip-172-88-3-164`, CPU: `Intel(R) Xeon(R) Platinum 8124M CPU @ 3.00GHz`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("parallel-dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/parallel
// benchmark
// pallet
// --chain=parallel-dev
// --execution=wasm
// --wasm-execution=compiled
// --pallet=pallet_utility
// --extrinsic=*
// --steps=50
// --repeat=20
// --output=./runtime/parallel/src/weights/pallet_utility.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_utility`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_utility::WeightInfo for WeightInfo<T> {
	/// The range of component `c` is `[0, 1000]`.
	fn batch(c: u32, ) -> Weight {
		// Minimum execution time: 21_041 nanoseconds.
		Weight::from_ref_time(37_722_957)
			// Standard Error: 2_140
			.saturating_add(Weight::from_ref_time(7_780_596).saturating_mul(c.into()))
	}
	fn as_derivative() -> Weight {
		// Minimum execution time: 11_677 nanoseconds.
		Weight::from_ref_time(11_976_000)
	}
	/// The range of component `c` is `[0, 1000]`.
	fn batch_all(c: u32, ) -> Weight {
		// Minimum execution time: 21_157 nanoseconds.
		Weight::from_ref_time(31_688_389)
			// Standard Error: 2_072
			.saturating_add(Weight::from_ref_time(8_200_339).saturating_mul(c.into()))
	}
	fn dispatch_as() -> Weight {
		// Minimum execution time: 24_883 nanoseconds.
		Weight::from_ref_time(25_367_000)
	}
	/// The range of component `c` is `[0, 1000]`.
	fn force_batch(c: u32, ) -> Weight {
		// Minimum execution time: 21_128 nanoseconds.
		Weight::from_ref_time(32_997_782)
			// Standard Error: 2_313
			.saturating_add(Weight::from_ref_time(7_808_716).saturating_mul(c.into()))
	}
}
