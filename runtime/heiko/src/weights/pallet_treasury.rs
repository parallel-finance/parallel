
//! Autogenerated weights for `pallet_treasury`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-03-23, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `ip-172-88-3-164`, CPU: `Intel(R) Xeon(R) Platinum 8124M CPU @ 3.00GHz`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("heiko-dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/parallel
// benchmark
// pallet
// --chain=heiko-dev
// --execution=wasm
// --wasm-execution=compiled
// --pallet=pallet_treasury
// --extrinsic=*
// --steps=50
// --repeat=20
// --output=./runtime/heiko/src/weights/pallet_treasury.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_treasury`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_treasury::WeightInfo for WeightInfo<T> {
	fn spend() -> Weight {
		// Minimum execution time: 394 nanoseconds.
		Weight::from_ref_time(451_000)
	}
	// Storage: Treasury ProposalCount (r:1 w:1)
	// Storage: Treasury Proposals (r:0 w:1)
	fn propose_spend() -> Weight {
		// Minimum execution time: 49_261 nanoseconds.
		Weight::from_ref_time(50_384_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: Treasury Proposals (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	fn reject_proposal() -> Weight {
		// Minimum execution time: 57_114 nanoseconds.
		Weight::from_ref_time(57_876_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: Treasury Proposals (r:1 w:0)
	// Storage: Treasury Approvals (r:1 w:1)
	/// The range of component `p` is `[0, 99]`.
	fn approve_proposal(p: u32, ) -> Weight {
		// Minimum execution time: 17_982 nanoseconds.
		Weight::from_ref_time(25_246_156)
			// Standard Error: 2_865
			.saturating_add(Weight::from_ref_time(146_225).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Treasury Approvals (r:1 w:1)
	fn remove_approval() -> Weight {
		// Minimum execution time: 14_565 nanoseconds.
		Weight::from_ref_time(14_969_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: System Account (r:1 w:0)
	// Storage: Treasury Inactive (r:1 w:1)
	// Storage: Balances InactiveIssuance (r:1 w:1)
	// Storage: Treasury Approvals (r:1 w:1)
	// Storage: Treasury Proposals (r:2 w:2)
	/// The range of component `p` is `[0, 100]`.
	fn on_initialize_proposals(p: u32, ) -> Weight {
		// Minimum execution time: 52_455 nanoseconds.
		Weight::from_ref_time(76_727_324)
			// Standard Error: 14_704
			.saturating_add(Weight::from_ref_time(51_708_821).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().reads((3_u64).saturating_mul(p.into())))
			.saturating_add(T::DbWeight::get().writes(4))
			.saturating_add(T::DbWeight::get().writes((3_u64).saturating_mul(p.into())))
	}
}
