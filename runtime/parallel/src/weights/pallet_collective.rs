
//! Autogenerated weights for `pallet_collective`
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
// --pallet=pallet_collective
// --extrinsic=*
// --steps=50
// --repeat=20
// --output=./runtime/parallel/src/weights/pallet_collective.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_collective`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_collective::WeightInfo for WeightInfo<T> {
	// Storage: TechnicalCommittee Members (r:1 w:1)
	// Storage: TechnicalCommittee Proposals (r:1 w:0)
	// Storage: TechnicalCommittee Prime (r:0 w:1)
	// Storage: TechnicalCommittee Voting (r:100 w:100)
	/// The range of component `m` is `[0, 100]`.
	/// The range of component `n` is `[0, 100]`.
	/// The range of component `p` is `[0, 100]`.
	fn set_members(m: u32, _n: u32, p: u32, ) -> Weight {
		// Minimum execution time: 27_310 nanoseconds.
		Weight::from_ref_time(27_504_000)
			// Standard Error: 95_202
			.saturating_add(Weight::from_ref_time(7_513_706).saturating_mul(m.into()))
			// Standard Error: 95_202
			.saturating_add(Weight::from_ref_time(11_688_776).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(p.into())))
			.saturating_add(T::DbWeight::get().writes(2))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(p.into())))
	}
	// Storage: TechnicalCommittee Members (r:1 w:0)
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[1, 100]`.
	fn execute(b: u32, m: u32, ) -> Weight {
		// Minimum execution time: 35_089 nanoseconds.
		Weight::from_ref_time(34_392_561)
			// Standard Error: 39
			.saturating_add(Weight::from_ref_time(2_243).saturating_mul(b.into()))
			// Standard Error: 410
			.saturating_add(Weight::from_ref_time(20_040).saturating_mul(m.into()))
			.saturating_add(T::DbWeight::get().reads(1))
	}
	// Storage: TechnicalCommittee Members (r:1 w:0)
	// Storage: TechnicalCommittee ProposalOf (r:1 w:0)
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[1, 100]`.
	fn propose_execute(b: u32, m: u32, ) -> Weight {
		// Minimum execution time: 39_742 nanoseconds.
		Weight::from_ref_time(38_344_327)
			// Standard Error: 51
			.saturating_add(Weight::from_ref_time(2_526).saturating_mul(b.into()))
			// Standard Error: 531
			.saturating_add(Weight::from_ref_time(27_997).saturating_mul(m.into()))
			.saturating_add(T::DbWeight::get().reads(2))
	}
	// Storage: TechnicalCommittee Members (r:1 w:0)
	// Storage: TechnicalCommittee ProposalOf (r:1 w:1)
	// Storage: TechnicalCommittee Proposals (r:1 w:1)
	// Storage: TechnicalCommittee ProposalCount (r:1 w:1)
	// Storage: TechnicalCommittee Voting (r:0 w:1)
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[2, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn propose_proposed(b: u32, m: u32, p: u32, ) -> Weight {
		// Minimum execution time: 49_178 nanoseconds.
		Weight::from_ref_time(52_484_105)
			// Standard Error: 222
			.saturating_add(Weight::from_ref_time(4_577).saturating_mul(b.into()))
			// Standard Error: 2_320
			.saturating_add(Weight::from_ref_time(36_915).saturating_mul(m.into()))
			// Standard Error: 2_290
			.saturating_add(Weight::from_ref_time(304_414).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	// Storage: TechnicalCommittee Members (r:1 w:0)
	// Storage: TechnicalCommittee Voting (r:1 w:1)
	/// The range of component `m` is `[5, 100]`.
	fn vote(m: u32, ) -> Weight {
		// Minimum execution time: 56_935 nanoseconds.
		Weight::from_ref_time(58_911_783)
			// Standard Error: 1_300
			.saturating_add(Weight::from_ref_time(50_736).saturating_mul(m.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: TechnicalCommittee Voting (r:1 w:1)
	// Storage: TechnicalCommittee Members (r:1 w:0)
	// Storage: TechnicalCommittee Proposals (r:1 w:1)
	// Storage: TechnicalCommittee ProposalOf (r:0 w:1)
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_early_disapproved(m: u32, p: u32, ) -> Weight {
		// Minimum execution time: 54_102 nanoseconds.
		Weight::from_ref_time(57_299_004)
			// Standard Error: 1_729
			.saturating_add(Weight::from_ref_time(42_870).saturating_mul(m.into()))
			// Standard Error: 1_686
			.saturating_add(Weight::from_ref_time(278_895).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	// Storage: TechnicalCommittee Voting (r:1 w:1)
	// Storage: TechnicalCommittee Members (r:1 w:0)
	// Storage: TechnicalCommittee ProposalOf (r:1 w:1)
	// Storage: TechnicalCommittee Proposals (r:1 w:1)
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_early_approved(b: u32, m: u32, p: u32, ) -> Weight {
		// Minimum execution time: 74_176 nanoseconds.
		Weight::from_ref_time(75_836_036)
			// Standard Error: 180
			.saturating_add(Weight::from_ref_time(4_753).saturating_mul(b.into()))
			// Standard Error: 1_911
			.saturating_add(Weight::from_ref_time(29_186).saturating_mul(m.into()))
			// Standard Error: 1_863
			.saturating_add(Weight::from_ref_time(309_329).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	// Storage: TechnicalCommittee Voting (r:1 w:1)
	// Storage: TechnicalCommittee Members (r:1 w:0)
	// Storage: TechnicalCommittee Prime (r:1 w:0)
	// Storage: TechnicalCommittee Proposals (r:1 w:1)
	// Storage: TechnicalCommittee ProposalOf (r:0 w:1)
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_disapproved(m: u32, p: u32, ) -> Weight {
		// Minimum execution time: 59_649 nanoseconds.
		Weight::from_ref_time(61_562_205)
			// Standard Error: 1_702
			.saturating_add(Weight::from_ref_time(41_938).saturating_mul(m.into()))
			// Standard Error: 1_660
			.saturating_add(Weight::from_ref_time(279_342).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	// Storage: TechnicalCommittee Voting (r:1 w:1)
	// Storage: TechnicalCommittee Members (r:1 w:0)
	// Storage: TechnicalCommittee Prime (r:1 w:0)
	// Storage: TechnicalCommittee ProposalOf (r:1 w:1)
	// Storage: TechnicalCommittee Proposals (r:1 w:1)
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_approved(b: u32, m: u32, p: u32, ) -> Weight {
		// Minimum execution time: 78_797 nanoseconds.
		Weight::from_ref_time(81_235_716)
			// Standard Error: 201
			.saturating_add(Weight::from_ref_time(4_116).saturating_mul(b.into()))
			// Standard Error: 2_127
			.saturating_add(Weight::from_ref_time(27_026).saturating_mul(m.into()))
			// Standard Error: 2_073
			.saturating_add(Weight::from_ref_time(305_312).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	// Storage: TechnicalCommittee Proposals (r:1 w:1)
	// Storage: TechnicalCommittee Voting (r:0 w:1)
	// Storage: TechnicalCommittee ProposalOf (r:0 w:1)
	/// The range of component `p` is `[1, 100]`.
	fn disapprove_proposal(p: u32, ) -> Weight {
		// Minimum execution time: 34_561 nanoseconds.
		Weight::from_ref_time(40_445_446)
			// Standard Error: 2_730
			.saturating_add(Weight::from_ref_time(294_540).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(3))
	}
}
