
//! Autogenerated weights for `pallet_identity`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-10-19, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
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
		(28_765_000 as Weight)
			// Standard Error: 3_000
			.saturating_add((314_000 as Weight).saturating_mul(r as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Identity IdentityOf (r:1 w:1)
	/// The range of component `r` is `[1, 20]`.
	/// The range of component `x` is `[1, 100]`.
	fn set_identity(r: u32, x: u32, ) -> Weight {
		(56_777_000 as Weight)
			// Standard Error: 16_000
			.saturating_add((356_000 as Weight).saturating_mul(r as Weight))
			// Standard Error: 3_000
			.saturating_add((687_000 as Weight).saturating_mul(x as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Identity IdentityOf (r:1 w:0)
	// Storage: Identity SubsOf (r:1 w:1)
	// Storage: Identity SuperOf (r:1 w:1)
	/// The range of component `s` is `[1, 100]`.
	fn set_subs_new(s: u32, ) -> Weight {
		(51_538_000 as Weight)
			// Standard Error: 4_000
			.saturating_add((6_080_000 as Weight).saturating_mul(s as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(s as Weight)))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(s as Weight)))
	}
	// Storage: Identity IdentityOf (r:1 w:0)
	// Storage: Identity SubsOf (r:1 w:1)
	// Storage: Identity SuperOf (r:0 w:1)
	/// The range of component `p` is `[1, 100]`.
	fn set_subs_old(p: u32, ) -> Weight {
		(50_586_000 as Weight)
			// Standard Error: 2_000
			.saturating_add((1_997_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(p as Weight)))
	}
	// Storage: Identity SubsOf (r:1 w:1)
	// Storage: Identity IdentityOf (r:1 w:1)
	// Storage: Identity SuperOf (r:0 w:100)
	/// The range of component `r` is `[1, 20]`.
	/// The range of component `s` is `[1, 100]`.
	/// The range of component `x` is `[1, 100]`.
	fn clear_identity(r: u32, s: u32, x: u32, ) -> Weight {
		(59_186_000 as Weight)
			// Standard Error: 9_000
			.saturating_add((216_000 as Weight).saturating_mul(r as Weight))
			// Standard Error: 1_000
			.saturating_add((2_031_000 as Weight).saturating_mul(s as Weight))
			// Standard Error: 1_000
			.saturating_add((355_000 as Weight).saturating_mul(x as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(s as Weight)))
	}
	// Storage: Identity Registrars (r:1 w:0)
	// Storage: Identity IdentityOf (r:1 w:1)
	/// The range of component `r` is `[1, 20]`.
	/// The range of component `x` is `[1, 100]`.
	fn request_judgement(r: u32, x: u32, ) -> Weight {
		(61_681_000 as Weight)
			// Standard Error: 5_000
			.saturating_add((339_000 as Weight).saturating_mul(r as Weight))
			// Standard Error: 1_000
			.saturating_add((686_000 as Weight).saturating_mul(x as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Identity IdentityOf (r:1 w:1)
	/// The range of component `r` is `[1, 20]`.
	/// The range of component `x` is `[1, 100]`.
	fn cancel_request(r: u32, x: u32, ) -> Weight {
		(56_404_000 as Weight)
			// Standard Error: 4_000
			.saturating_add((265_000 as Weight).saturating_mul(r as Weight))
			// Standard Error: 0
			.saturating_add((690_000 as Weight).saturating_mul(x as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Identity Registrars (r:1 w:1)
	/// The range of component `r` is `[1, 19]`.
	fn set_fee(r: u32, ) -> Weight {
		(13_718_000 as Weight)
			// Standard Error: 2_000
			.saturating_add((242_000 as Weight).saturating_mul(r as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Identity Registrars (r:1 w:1)
	/// The range of component `r` is `[1, 19]`.
	fn set_account_id(r: u32, ) -> Weight {
		(13_719_000 as Weight)
			// Standard Error: 2_000
			.saturating_add((251_000 as Weight).saturating_mul(r as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Identity Registrars (r:1 w:1)
	/// The range of component `r` is `[1, 19]`.
	fn set_fields(r: u32, ) -> Weight {
		(13_686_000 as Weight)
			// Standard Error: 2_000
			.saturating_add((240_000 as Weight).saturating_mul(r as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Identity Registrars (r:1 w:0)
	// Storage: Identity IdentityOf (r:1 w:1)
	/// The range of component `r` is `[1, 19]`.
	/// The range of component `x` is `[1, 100]`.
	fn provide_judgement(r: u32, x: u32, ) -> Weight {
		(43_166_000 as Weight)
			// Standard Error: 4_000
			.saturating_add((268_000 as Weight).saturating_mul(r as Weight))
			// Standard Error: 0
			.saturating_add((681_000 as Weight).saturating_mul(x as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Identity SubsOf (r:1 w:1)
	// Storage: Identity IdentityOf (r:1 w:1)
	// Storage: System Account (r:2 w:2)
	// Storage: Identity SuperOf (r:0 w:100)
	/// The range of component `r` is `[1, 20]`.
	/// The range of component `s` is `[1, 100]`.
	/// The range of component `x` is `[1, 100]`.
	fn kill_identity(r: u32, s: u32, x: u32, ) -> Weight {
		(82_381_000 as Weight)
			// Standard Error: 11_000
			.saturating_add((234_000 as Weight).saturating_mul(r as Weight))
			// Standard Error: 2_000
			.saturating_add((2_018_000 as Weight).saturating_mul(s as Weight))
			// Standard Error: 2_000
			.saturating_add((10_000 as Weight).saturating_mul(x as Weight))
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(s as Weight)))
	}
	// Storage: Identity IdentityOf (r:1 w:0)
	// Storage: Identity SuperOf (r:1 w:1)
	// Storage: Identity SubsOf (r:1 w:1)
	/// The range of component `s` is `[1, 99]`.
	fn add_sub(s: u32, ) -> Weight {
		(64_262_000 as Weight)
			// Standard Error: 2_000
			.saturating_add((166_000 as Weight).saturating_mul(s as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: Identity IdentityOf (r:1 w:0)
	// Storage: Identity SuperOf (r:1 w:1)
	/// The range of component `s` is `[1, 100]`.
	fn rename_sub(s: u32, ) -> Weight {
		(23_650_000 as Weight)
			// Standard Error: 1_000
			.saturating_add((68_000 as Weight).saturating_mul(s as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Identity IdentityOf (r:1 w:0)
	// Storage: Identity SuperOf (r:1 w:1)
	// Storage: Identity SubsOf (r:1 w:1)
	/// The range of component `s` is `[1, 100]`.
	fn remove_sub(s: u32, ) -> Weight {
		(65_161_000 as Weight)
			// Standard Error: 2_000
			.saturating_add((157_000 as Weight).saturating_mul(s as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: Identity SuperOf (r:1 w:1)
	// Storage: Identity SubsOf (r:1 w:1)
	/// The range of component `s` is `[1, 99]`.
	fn quit_sub(s: u32, ) -> Weight {
		(45_884_000 as Weight)
			// Standard Error: 2_000
			.saturating_add((152_000 as Weight).saturating_mul(s as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
}
