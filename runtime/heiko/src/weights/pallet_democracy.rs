
//! Autogenerated weights for `pallet_democracy`
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
// --pallet=pallet_democracy
// --extrinsic=*
// --steps=50
// --repeat=20
// --output=./runtime/heiko/src/weights/pallet_democracy.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_democracy`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_democracy::WeightInfo for WeightInfo<T> {
	// Storage: Democracy PublicPropCount (r:1 w:1)
	// Storage: Democracy PublicProps (r:1 w:1)
	// Storage: Democracy Blacklist (r:1 w:0)
	// Storage: Democracy DepositOf (r:0 w:1)
	fn propose() -> Weight {
		// Minimum execution time: 84_420 nanoseconds.
		Weight::from_ref_time(87_886_000)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	// Storage: Democracy DepositOf (r:1 w:1)
	fn second() -> Weight {
		// Minimum execution time: 76_939 nanoseconds.
		Weight::from_ref_time(78_701_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Democracy ReferendumInfoOf (r:1 w:1)
	// Storage: Democracy VotingOf (r:1 w:1)
	// Storage: Balances Locks (r:1 w:1)
	fn vote_new() -> Weight {
		// Minimum execution time: 96_415 nanoseconds.
		Weight::from_ref_time(97_039_000)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	// Storage: Democracy ReferendumInfoOf (r:1 w:1)
	// Storage: Democracy VotingOf (r:1 w:1)
	// Storage: Balances Locks (r:1 w:1)
	fn vote_existing() -> Weight {
		// Minimum execution time: 96_539 nanoseconds.
		Weight::from_ref_time(97_273_000)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	// Storage: Democracy ReferendumInfoOf (r:1 w:1)
	// Storage: Democracy Cancellations (r:1 w:1)
	fn emergency_cancel() -> Weight {
		// Minimum execution time: 36_863 nanoseconds.
		Weight::from_ref_time(37_747_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: Democracy PublicProps (r:1 w:1)
	// Storage: Democracy DepositOf (r:1 w:1)
	// Storage: System Account (r:2 w:2)
	// Storage: Democracy NextExternal (r:1 w:1)
	// Storage: Democracy ReferendumInfoOf (r:1 w:1)
	// Storage: Democracy Blacklist (r:0 w:1)
	fn blacklist() -> Weight {
		// Minimum execution time: 149_625 nanoseconds.
		Weight::from_ref_time(151_657_000)
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(7))
	}
	// Storage: Democracy NextExternal (r:1 w:1)
	// Storage: Democracy Blacklist (r:1 w:0)
	fn external_propose() -> Weight {
		// Minimum execution time: 27_618 nanoseconds.
		Weight::from_ref_time(28_127_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Democracy NextExternal (r:0 w:1)
	fn external_propose_majority() -> Weight {
		// Minimum execution time: 8_350 nanoseconds.
		Weight::from_ref_time(8_654_000)
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Democracy NextExternal (r:0 w:1)
	fn external_propose_default() -> Weight {
		// Minimum execution time: 8_504 nanoseconds.
		Weight::from_ref_time(8_943_000)
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Democracy NextExternal (r:1 w:1)
	// Storage: Democracy ReferendumCount (r:1 w:1)
	// Storage: Democracy ReferendumInfoOf (r:0 w:1)
	fn fast_track() -> Weight {
		// Minimum execution time: 37_155 nanoseconds.
		Weight::from_ref_time(37_766_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	// Storage: Democracy NextExternal (r:1 w:1)
	// Storage: Democracy Blacklist (r:1 w:1)
	fn veto_external() -> Weight {
		// Minimum execution time: 45_184 nanoseconds.
		Weight::from_ref_time(46_382_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: Democracy PublicProps (r:1 w:1)
	// Storage: Democracy DepositOf (r:1 w:1)
	// Storage: System Account (r:2 w:2)
	fn cancel_proposal() -> Weight {
		// Minimum execution time: 127_480 nanoseconds.
		Weight::from_ref_time(130_326_000)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	// Storage: Democracy ReferendumInfoOf (r:0 w:1)
	fn cancel_referendum() -> Weight {
		// Minimum execution time: 24_464 nanoseconds.
		Weight::from_ref_time(24_888_000)
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Democracy LowestUnbaked (r:1 w:1)
	// Storage: Democracy ReferendumCount (r:1 w:0)
	// Storage: Democracy ReferendumInfoOf (r:2 w:0)
	/// The range of component `r` is `[0, 99]`.
	fn on_initialize_base(r: u32, ) -> Weight {
		// Minimum execution time: 11_236 nanoseconds.
		Weight::from_ref_time(14_842_952)
			// Standard Error: 5_204
			.saturating_add(Weight::from_ref_time(3_716_590).saturating_mul(r.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(r.into())))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Democracy LowestUnbaked (r:1 w:1)
	// Storage: Democracy ReferendumCount (r:1 w:0)
	// Storage: Democracy LastTabledWasExternal (r:1 w:0)
	// Storage: Democracy NextExternal (r:1 w:0)
	// Storage: Democracy PublicProps (r:1 w:0)
	// Storage: Democracy ReferendumInfoOf (r:2 w:0)
	/// The range of component `r` is `[0, 99]`.
	fn on_initialize_base_with_launch_period(r: u32, ) -> Weight {
		// Minimum execution time: 15_630 nanoseconds.
		Weight::from_ref_time(19_692_582)
			// Standard Error: 5_563
			.saturating_add(Weight::from_ref_time(3_718_003).saturating_mul(r.into()))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(r.into())))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Democracy VotingOf (r:3 w:3)
	// Storage: Balances Locks (r:1 w:1)
	// Storage: Democracy ReferendumInfoOf (r:2 w:2)
	/// The range of component `r` is `[0, 99]`.
	fn delegate(r: u32, ) -> Weight {
		// Minimum execution time: 74_416 nanoseconds.
		Weight::from_ref_time(82_689_875)
			// Standard Error: 8_213
			.saturating_add(Weight::from_ref_time(5_600_273).saturating_mul(r.into()))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(r.into())))
			.saturating_add(T::DbWeight::get().writes(4))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(r.into())))
	}
	// Storage: Democracy VotingOf (r:2 w:2)
	// Storage: Democracy ReferendumInfoOf (r:2 w:2)
	/// The range of component `r` is `[0, 99]`.
	fn undelegate(r: u32, ) -> Weight {
		// Minimum execution time: 44_648 nanoseconds.
		Weight::from_ref_time(47_188_871)
			// Standard Error: 7_770
			.saturating_add(Weight::from_ref_time(5_567_156).saturating_mul(r.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(r.into())))
			.saturating_add(T::DbWeight::get().writes(2))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(r.into())))
	}
	// Storage: Democracy PublicProps (r:0 w:1)
	fn clear_public_proposals() -> Weight {
		// Minimum execution time: 9_496 nanoseconds.
		Weight::from_ref_time(9_825_000)
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Democracy VotingOf (r:1 w:1)
	// Storage: Balances Locks (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	/// The range of component `r` is `[0, 99]`.
	fn unlock_remove(r: u32, ) -> Weight {
		// Minimum execution time: 42_856 nanoseconds.
		Weight::from_ref_time(54_333_578)
			// Standard Error: 2_716
			.saturating_add(Weight::from_ref_time(137_559).saturating_mul(r.into()))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	// Storage: Democracy VotingOf (r:1 w:1)
	// Storage: Balances Locks (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	/// The range of component `r` is `[0, 99]`.
	fn unlock_set(r: u32, ) -> Weight {
		// Minimum execution time: 51_587 nanoseconds.
		Weight::from_ref_time(53_375_363)
			// Standard Error: 1_537
			.saturating_add(Weight::from_ref_time(201_853).saturating_mul(r.into()))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	// Storage: Democracy ReferendumInfoOf (r:1 w:1)
	// Storage: Democracy VotingOf (r:1 w:1)
	/// The range of component `r` is `[1, 100]`.
	fn remove_vote(r: u32, ) -> Weight {
		// Minimum execution time: 28_377 nanoseconds.
		Weight::from_ref_time(31_506_120)
			// Standard Error: 1_622
			.saturating_add(Weight::from_ref_time(194_730).saturating_mul(r.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: Democracy ReferendumInfoOf (r:1 w:1)
	// Storage: Democracy VotingOf (r:1 w:1)
	/// The range of component `r` is `[1, 100]`.
	fn remove_other_vote(r: u32, ) -> Weight {
		// Minimum execution time: 28_098 nanoseconds.
		Weight::from_ref_time(31_538_319)
			// Standard Error: 1_701
			.saturating_add(Weight::from_ref_time(196_032).saturating_mul(r.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
}
