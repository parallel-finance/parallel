
//! Autogenerated weights for `pallet_proxy`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-03-22, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `ip-172-88-3-164`, CPU: `Intel(R) Xeon(R) Platinum 8124M CPU @ 3.00GHz`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("heiko-dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/parallel
// benchmark
// pallet
// --chain=heiko-dev
// --execution=wasm
// --wasm-execution=compiled
// --pallet=pallet_proxy
// --extrinsic=*
// --steps=50
// --repeat=20
// --output=./runtime/heiko/src/weights/pallet_proxy.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_proxy`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_proxy::WeightInfo for WeightInfo<T> {
	// Storage: Proxy Proxies (r:1 w:0)
	/// The range of component `p` is `[1, 31]`.
	fn proxy(p: u32, ) -> Weight {
		// Minimum execution time: 37_867 nanoseconds.
		Weight::from_ref_time(39_039_729)
			// Standard Error: 2_828
			.saturating_add(Weight::from_ref_time(71_633).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(1))
	}
	// Storage: Proxy Proxies (r:1 w:0)
	// Storage: Proxy Announcements (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	/// The range of component `a` is `[0, 31]`.
	/// The range of component `p` is `[1, 31]`.
	fn proxy_announced(a: u32, p: u32, ) -> Weight {
		// Minimum execution time: 71_163 nanoseconds.
		Weight::from_ref_time(71_235_398)
			// Standard Error: 3_495
			.saturating_add(Weight::from_ref_time(292_086).saturating_mul(a.into()))
			// Standard Error: 3_611
			.saturating_add(Weight::from_ref_time(65_758).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: Proxy Announcements (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	/// The range of component `a` is `[0, 31]`.
	/// The range of component `p` is `[1, 31]`.
	fn remove_announcement(a: u32, p: u32, ) -> Weight {
		// Minimum execution time: 48_082 nanoseconds.
		Weight::from_ref_time(49_296_315)
			// Standard Error: 2_896
			.saturating_add(Weight::from_ref_time(273_812).saturating_mul(a.into()))
			// Standard Error: 2_992
			.saturating_add(Weight::from_ref_time(24_439).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: Proxy Announcements (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	/// The range of component `a` is `[0, 31]`.
	/// The range of component `p` is `[1, 31]`.
	fn reject_announcement(a: u32, p: u32, ) -> Weight {
		// Minimum execution time: 47_574 nanoseconds.
		Weight::from_ref_time(49_469_693)
			// Standard Error: 2_717
			.saturating_add(Weight::from_ref_time(276_199).saturating_mul(a.into()))
			// Standard Error: 2_807
			.saturating_add(Weight::from_ref_time(15_563).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: Proxy Proxies (r:1 w:0)
	// Storage: Proxy Announcements (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	/// The range of component `a` is `[0, 31]`.
	/// The range of component `p` is `[1, 31]`.
	fn announce(a: u32, p: u32, ) -> Weight {
		// Minimum execution time: 62_808 nanoseconds.
		Weight::from_ref_time(64_172_715)
			// Standard Error: 2_992
			.saturating_add(Weight::from_ref_time(274_401).saturating_mul(a.into()))
			// Standard Error: 3_091
			.saturating_add(Weight::from_ref_time(60_932).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: Proxy Proxies (r:1 w:1)
	/// The range of component `p` is `[1, 31]`.
	fn add_proxy(p: u32, ) -> Weight {
		// Minimum execution time: 52_687 nanoseconds.
		Weight::from_ref_time(53_825_711)
			// Standard Error: 1_958
			.saturating_add(Weight::from_ref_time(108_589).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Proxy Proxies (r:1 w:1)
	/// The range of component `p` is `[1, 31]`.
	fn remove_proxy(p: u32, ) -> Weight {
		// Minimum execution time: 51_832 nanoseconds.
		Weight::from_ref_time(53_802_141)
			// Standard Error: 2_836
			.saturating_add(Weight::from_ref_time(137_309).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Proxy Proxies (r:1 w:1)
	/// The range of component `p` is `[1, 31]`.
	fn remove_proxies(p: u32, ) -> Weight {
		// Minimum execution time: 44_785 nanoseconds.
		Weight::from_ref_time(45_845_601)
			// Standard Error: 2_012
			.saturating_add(Weight::from_ref_time(71_595).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: unknown [0x3a65787472696e7369635f696e646578] (r:1 w:0)
	// Storage: Proxy Proxies (r:1 w:1)
	/// The range of component `p` is `[1, 31]`.
	fn create_pure(p: u32, ) -> Weight {
		// Minimum execution time: 59_299 nanoseconds.
		Weight::from_ref_time(60_578_890)
			// Standard Error: 2_027
			.saturating_add(Weight::from_ref_time(43_923).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Proxy Proxies (r:1 w:1)
	/// The range of component `p` is `[0, 30]`.
	fn kill_pure(p: u32, ) -> Weight {
		// Minimum execution time: 47_091 nanoseconds.
		Weight::from_ref_time(48_712_686)
			// Standard Error: 2_013
			.saturating_add(Weight::from_ref_time(75_825).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}
