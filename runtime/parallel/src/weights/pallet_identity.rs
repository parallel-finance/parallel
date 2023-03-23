
//! Autogenerated weights for `pallet_identity`
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
// --pallet=pallet_identity
// --extrinsic=*
// --steps=50
// --repeat=20
// --output=./runtime/parallel/src/weights/pallet_identity.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_identity`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_identity::WeightInfo for WeightInfo<T> {
	// Storage: Identity Registrars (r:1 w:1)
	/// The range of component `r` is `[1, 19]`.
	fn add_registrar(r: u32, ) -> Weight {
		// Minimum execution time: 29_099 nanoseconds.
		Weight::from_ref_time(30_538_377)
			// Standard Error: 4_278
			.saturating_add(Weight::from_ref_time(333_798).saturating_mul(r.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Identity IdentityOf (r:1 w:1)
	/// The range of component `r` is `[1, 20]`.
	/// The range of component `x` is `[0, 100]`.
	fn set_identity(r: u32, x: u32, ) -> Weight {
		// Minimum execution time: 67_998 nanoseconds.
		Weight::from_ref_time(62_475_619)
			// Standard Error: 4_290
			.saturating_add(Weight::from_ref_time(355_081).saturating_mul(r.into()))
			// Standard Error: 837
			.saturating_add(Weight::from_ref_time(717_036).saturating_mul(x.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Identity IdentityOf (r:1 w:0)
	// Storage: Identity SubsOf (r:1 w:1)
	// Storage: Identity SuperOf (r:2 w:2)
	/// The range of component `s` is `[0, 100]`.
	fn set_subs_new(s: u32, ) -> Weight {
		// Minimum execution time: 19_043 nanoseconds.
		Weight::from_ref_time(50_032_198)
			// Standard Error: 7_754
			.saturating_add(Weight::from_ref_time(4_590_404).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(s.into())))
			.saturating_add(T::DbWeight::get().writes(1))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(s.into())))
	}
	// Storage: Identity IdentityOf (r:1 w:0)
	// Storage: Identity SubsOf (r:1 w:1)
	// Storage: Identity SuperOf (r:0 w:2)
	/// The range of component `p` is `[0, 100]`.
	fn set_subs_old(p: u32, ) -> Weight {
		// Minimum execution time: 18_918 nanoseconds.
		Weight::from_ref_time(49_991_859)
			// Standard Error: 7_699
			.saturating_add(Weight::from_ref_time(1_988_657).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(p.into())))
	}
	// Storage: Identity SubsOf (r:1 w:1)
	// Storage: Identity IdentityOf (r:1 w:1)
	// Storage: Identity SuperOf (r:0 w:100)
	/// The range of component `r` is `[1, 20]`.
	/// The range of component `s` is `[0, 100]`.
	/// The range of component `x` is `[0, 100]`.
	fn clear_identity(r: u32, s: u32, x: u32, ) -> Weight {
		// Minimum execution time: 100_732 nanoseconds.
		Weight::from_ref_time(67_033_125)
			// Standard Error: 8_899
			.saturating_add(Weight::from_ref_time(136_148).saturating_mul(r.into()))
			// Standard Error: 1_737
			.saturating_add(Weight::from_ref_time(1_913_520).saturating_mul(s.into()))
			// Standard Error: 1_737
			.saturating_add(Weight::from_ref_time(363_081).saturating_mul(x.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(s.into())))
	}
	// Storage: Identity Registrars (r:1 w:0)
	// Storage: Identity IdentityOf (r:1 w:1)
	/// The range of component `r` is `[1, 20]`.
	/// The range of component `x` is `[0, 100]`.
	fn request_judgement(r: u32, x: u32, ) -> Weight {
		// Minimum execution time: 69_971 nanoseconds.
		Weight::from_ref_time(63_562_435)
			// Standard Error: 5_276
			.saturating_add(Weight::from_ref_time(369_266).saturating_mul(r.into()))
			// Standard Error: 1_029
			.saturating_add(Weight::from_ref_time(731_381).saturating_mul(x.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Identity IdentityOf (r:1 w:1)
	/// The range of component `r` is `[1, 20]`.
	/// The range of component `x` is `[0, 100]`.
	fn cancel_request(r: u32, x: u32, ) -> Weight {
		// Minimum execution time: 63_111 nanoseconds.
		Weight::from_ref_time(58_304_921)
			// Standard Error: 4_614
			.saturating_add(Weight::from_ref_time(326_877).saturating_mul(r.into()))
			// Standard Error: 900
			.saturating_add(Weight::from_ref_time(734_884).saturating_mul(x.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Identity Registrars (r:1 w:1)
	/// The range of component `r` is `[1, 19]`.
	fn set_fee(r: u32, ) -> Weight {
		// Minimum execution time: 15_502 nanoseconds.
		Weight::from_ref_time(16_411_432)
			// Standard Error: 2_914
			.saturating_add(Weight::from_ref_time(239_686).saturating_mul(r.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Identity Registrars (r:1 w:1)
	/// The range of component `r` is `[1, 19]`.
	fn set_account_id(r: u32, ) -> Weight {
		// Minimum execution time: 15_935 nanoseconds.
		Weight::from_ref_time(16_760_329)
			// Standard Error: 2_616
			.saturating_add(Weight::from_ref_time(242_666).saturating_mul(r.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Identity Registrars (r:1 w:1)
	/// The range of component `r` is `[1, 19]`.
	fn set_fields(r: u32, ) -> Weight {
		// Minimum execution time: 15_560 nanoseconds.
		Weight::from_ref_time(16_391_981)
			// Standard Error: 2_527
			.saturating_add(Weight::from_ref_time(246_402).saturating_mul(r.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Identity Registrars (r:1 w:0)
	// Storage: Identity IdentityOf (r:1 w:1)
	/// The range of component `r` is `[1, 19]`.
	/// The range of component `x` is `[0, 100]`.
	fn provide_judgement(r: u32, x: u32, ) -> Weight {
		// Minimum execution time: 51_006 nanoseconds.
		Weight::from_ref_time(48_150_046)
			// Standard Error: 5_046
			.saturating_add(Weight::from_ref_time(309_199).saturating_mul(r.into()))
			// Standard Error: 933
			.saturating_add(Weight::from_ref_time(1_182_580).saturating_mul(x.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Identity SubsOf (r:1 w:1)
	// Storage: Identity IdentityOf (r:1 w:1)
	// Storage: System Account (r:2 w:2)
	// Storage: Identity SuperOf (r:0 w:100)
	/// The range of component `r` is `[1, 20]`.
	/// The range of component `s` is `[0, 100]`.
	/// The range of component `x` is `[0, 100]`.
	fn kill_identity(r: u32, s: u32, x: u32, ) -> Weight {
		// Minimum execution time: 121_761 nanoseconds.
		Weight::from_ref_time(88_592_049)
			// Standard Error: 9_023
			.saturating_add(Weight::from_ref_time(178_603).saturating_mul(r.into()))
			// Standard Error: 1_762
			.saturating_add(Weight::from_ref_time(1_904_737).saturating_mul(s.into()))
			// Standard Error: 1_762
			.saturating_add(Weight::from_ref_time(351_479).saturating_mul(x.into()))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(4))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(s.into())))
	}
	// Storage: Identity IdentityOf (r:1 w:0)
	// Storage: Identity SuperOf (r:1 w:1)
	// Storage: Identity SubsOf (r:1 w:1)
	/// The range of component `s` is `[0, 99]`.
	fn add_sub(s: u32, ) -> Weight {
		// Minimum execution time: 59_146 nanoseconds.
		Weight::from_ref_time(67_177_748)
			// Standard Error: 2_321
			.saturating_add(Weight::from_ref_time(198_063).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: Identity IdentityOf (r:1 w:0)
	// Storage: Identity SuperOf (r:1 w:1)
	/// The range of component `s` is `[1, 100]`.
	fn rename_sub(s: u32, ) -> Weight {
		// Minimum execution time: 24_899 nanoseconds.
		Weight::from_ref_time(27_679_772)
			// Standard Error: 1_150
			.saturating_add(Weight::from_ref_time(109_155).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Identity IdentityOf (r:1 w:0)
	// Storage: Identity SuperOf (r:1 w:1)
	// Storage: Identity SubsOf (r:1 w:1)
	/// The range of component `s` is `[1, 100]`.
	fn remove_sub(s: u32, ) -> Weight {
		// Minimum execution time: 64_877 nanoseconds.
		Weight::from_ref_time(69_386_385)
			// Standard Error: 1_862
			.saturating_add(Weight::from_ref_time(179_060).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: Identity SuperOf (r:1 w:1)
	// Storage: Identity SubsOf (r:1 w:1)
	/// The range of component `s` is `[0, 99]`.
	fn quit_sub(s: u32, ) -> Weight {
		// Minimum execution time: 45_057 nanoseconds.
		Weight::from_ref_time(49_840_179)
			// Standard Error: 1_544
			.saturating_add(Weight::from_ref_time(170_374).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
}
