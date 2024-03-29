
//! Autogenerated weights for `pallet_crowdloans`
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
// --pallet=pallet_crowdloans
// --extrinsic=*
// --steps=50
// --repeat=20
// --output=./runtime/parallel/src/weights/pallet_crowdloans.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_crowdloans`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_crowdloans::WeightInfo for WeightInfo<T> {
	// Storage: Crowdloans CTokensRegistry (r:1 w:1)
	// Storage: Crowdloans Vaults (r:1 w:1)
	// Storage: Crowdloans LeasesRegistry (r:1 w:1)
	// Storage: ParachainSystem ValidationData (r:1 w:0)
	// Storage: Crowdloans NextTrieIndex (r:1 w:1)
	fn create_vault() -> Weight {
		// Minimum execution time: 62_770 nanoseconds.
		Weight::from_ref_time(63_378_000)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	// Storage: Crowdloans LeasesRegistry (r:1 w:0)
	// Storage: Crowdloans Vaults (r:1 w:1)
	// Storage: ParachainSystem ValidationData (r:1 w:0)
	fn update_vault() -> Weight {
		// Minimum execution time: 54_852 nanoseconds.
		Weight::from_ref_time(55_972_000)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Crowdloans LeasesRegistry (r:1 w:0)
	// Storage: Crowdloans Vaults (r:1 w:1)
	// Storage: ParachainSystem ValidationData (r:1 w:0)
	// Storage: Crowdloans IsVrf (r:1 w:0)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:3 w:3)
	// Storage: System Account (r:1 w:1)
	// Storage: XcmHelper XcmWeightFee (r:1 w:0)
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: PolkadotXcm QueryCounter (r:1 w:1)
	// Storage: PolkadotXcm SupportedVersion (r:1 w:0)
	// Storage: PolkadotXcm VersionDiscoveryQueue (r:1 w:1)
	// Storage: PolkadotXcm SafeXcmVersion (r:1 w:0)
	// Storage: ParachainSystem HostConfiguration (r:1 w:0)
	// Storage: ParachainSystem PendingUpwardMessages (r:1 w:1)
	// Storage: Crowdloans XcmRequests (r:0 w:1)
	// Storage: PolkadotXcm Queries (r:0 w:1)
	// Storage: unknown [0xd861ea1ebf4800d4b89f4ff787ad79ee96d9a708c85b57da7eb8f9ddeda61291] (r:1 w:1)
	fn contribute() -> Weight {
		// Minimum execution time: 218_502 nanoseconds.
		Weight::from_ref_time(221_513_000)
			.saturating_add(T::DbWeight::get().reads(18))
			.saturating_add(T::DbWeight::get().writes(12))
	}
	// Storage: Crowdloans LeasesRegistry (r:1 w:0)
	// Storage: Crowdloans Vaults (r:1 w:1)
	fn open() -> Weight {
		// Minimum execution time: 51_531 nanoseconds.
		Weight::from_ref_time(51_773_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Crowdloans LeasesRegistry (r:1 w:0)
	// Storage: Crowdloans Vaults (r:1 w:1)
	fn close() -> Weight {
		// Minimum execution time: 51_297 nanoseconds.
		Weight::from_ref_time(51_909_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Crowdloans IsVrf (r:0 w:1)
	fn set_vrf() -> Weight {
		// Minimum execution time: 24_747 nanoseconds.
		Weight::from_ref_time(25_284_000)
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Crowdloans ProxyAddress (r:0 w:1)
	fn update_proxy() -> Weight {
		// Minimum execution time: 25_828 nanoseconds.
		Weight::from_ref_time(26_394_000)
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Crowdloans LeasesBonus (r:0 w:1)
	fn update_leases_bonus() -> Weight {
		// Minimum execution time: 29_404 nanoseconds.
		Weight::from_ref_time(30_444_000)
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Crowdloans LeasesRegistry (r:1 w:0)
	// Storage: Crowdloans Vaults (r:1 w:1)
	fn reopen() -> Weight {
		// Minimum execution time: 51_106 nanoseconds.
		Weight::from_ref_time(52_159_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Crowdloans LeasesRegistry (r:1 w:0)
	// Storage: Crowdloans Vaults (r:1 w:1)
	fn auction_succeeded() -> Weight {
		// Minimum execution time: 51_462 nanoseconds.
		Weight::from_ref_time(51_901_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Crowdloans LeasesRegistry (r:1 w:0)
	// Storage: Crowdloans Vaults (r:1 w:1)
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: XcmHelper XcmWeightFee (r:1 w:0)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:1 w:1)
	// Storage: PolkadotXcm QueryCounter (r:1 w:1)
	// Storage: PolkadotXcm SupportedVersion (r:1 w:0)
	// Storage: PolkadotXcm VersionDiscoveryQueue (r:1 w:1)
	// Storage: PolkadotXcm SafeXcmVersion (r:1 w:0)
	// Storage: ParachainSystem HostConfiguration (r:1 w:0)
	// Storage: ParachainSystem PendingUpwardMessages (r:1 w:1)
	// Storage: Crowdloans XcmRequests (r:0 w:1)
	// Storage: PolkadotXcm Queries (r:0 w:1)
	fn auction_failed() -> Weight {
		// Minimum execution time: 160_245 nanoseconds.
		Weight::from_ref_time(161_811_000)
			.saturating_add(T::DbWeight::get().reads(12))
			.saturating_add(T::DbWeight::get().writes(8))
	}
	// Storage: Crowdloans CTokensRegistry (r:1 w:0)
	// Storage: Crowdloans Vaults (r:1 w:0)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:1 w:1)
	// Storage: Crowdloans LeasesBonus (r:1 w:0)
	// Storage: Assets Metadata (r:1 w:0)
	// Storage: unknown [0xd861ea1ebf4800d4b89f4ff787ad79ee96d9a708c85b57da7eb8f9ddeda61291] (r:1 w:1)
	fn claim() -> Weight {
		// Minimum execution time: 109_882 nanoseconds.
		Weight::from_ref_time(111_106_000)
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	// Storage: Crowdloans Vaults (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:1 w:1)
	// Storage: unknown [0xd861ea1ebf4800d4b89f4ff787ad79ee96d9a708c85b57da7eb8f9ddeda61291] (r:1 w:1)
	fn withdraw() -> Weight {
		// Minimum execution time: 96_239 nanoseconds.
		Weight::from_ref_time(97_297_000)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	// Storage: Crowdloans CTokensRegistry (r:1 w:0)
	// Storage: Crowdloans Vaults (r:1 w:1)
	// Storage: Assets Asset (r:2 w:2)
	// Storage: Assets Account (r:2 w:2)
	fn redeem() -> Weight {
		// Minimum execution time: 135_121 nanoseconds.
		Weight::from_ref_time(136_154_000)
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(5))
	}
	// Storage: Crowdloans LeasesRegistry (r:1 w:0)
	// Storage: Crowdloans Vaults (r:1 w:1)
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: XcmHelper XcmWeightFee (r:1 w:0)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:1 w:1)
	// Storage: PolkadotXcm QueryCounter (r:1 w:1)
	// Storage: PolkadotXcm SupportedVersion (r:1 w:0)
	// Storage: PolkadotXcm VersionDiscoveryQueue (r:1 w:1)
	// Storage: PolkadotXcm SafeXcmVersion (r:1 w:0)
	// Storage: ParachainSystem HostConfiguration (r:1 w:0)
	// Storage: ParachainSystem PendingUpwardMessages (r:1 w:1)
	// Storage: Crowdloans XcmRequests (r:0 w:1)
	// Storage: PolkadotXcm Queries (r:0 w:1)
	fn slot_expired() -> Weight {
		// Minimum execution time: 160_119 nanoseconds.
		Weight::from_ref_time(162_491_000)
			.saturating_add(T::DbWeight::get().reads(12))
			.saturating_add(T::DbWeight::get().writes(8))
	}
	// Storage: Crowdloans LeasesRegistry (r:1 w:0)
	// Storage: Crowdloans Vaults (r:1 w:1)
	// Storage: Crowdloans IsVrf (r:1 w:0)
	// Storage: XcmHelper XcmWeightFee (r:1 w:0)
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:1 w:1)
	// Storage: PolkadotXcm QueryCounter (r:1 w:1)
	// Storage: PolkadotXcm SupportedVersion (r:1 w:0)
	// Storage: PolkadotXcm VersionDiscoveryQueue (r:1 w:1)
	// Storage: PolkadotXcm SafeXcmVersion (r:1 w:0)
	// Storage: ParachainSystem HostConfiguration (r:1 w:0)
	// Storage: ParachainSystem PendingUpwardMessages (r:1 w:1)
	// Storage: Crowdloans XcmRequests (r:0 w:1)
	// Storage: PolkadotXcm Queries (r:0 w:1)
	// Storage: unknown [0x] (r:1 w:0)
	// Storage: unknown [0xd861ea1ebf4800d4b89f4ff787ad79ee96d9a708c85b57da7eb8f9ddeda61291] (r:2 w:2)
	fn migrate_pending() -> Weight {
		// Minimum execution time: 225_744 nanoseconds.
		Weight::from_ref_time(228_866_000)
			.saturating_add(T::DbWeight::get().reads(16))
			.saturating_add(T::DbWeight::get().writes(10))
	}
	// Storage: Crowdloans XcmRequests (r:1 w:1)
	// Storage: Crowdloans Vaults (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	// Storage: unknown [0xd861ea1ebf4800d4b89f4ff787ad79ee96d9a708c85b57da7eb8f9ddeda61291] (r:2 w:2)
	fn notification_received() -> Weight {
		// Minimum execution time: 138_229 nanoseconds.
		Weight::from_ref_time(139_934_000)
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(7))
	}
	// Storage: Crowdloans Vaults (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:1 w:1)
	// Storage: unknown [0x] (r:3 w:0)
	// Storage: unknown [0xd861ea1ebf4800d4b89f4ff787ad79ee96d9a708c85b57da7eb8f9ddeda61291] (r:1 w:1)
	fn refund() -> Weight {
		// Minimum execution time: 157_685 nanoseconds.
		Weight::from_ref_time(172_971_000)
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	// Storage: Crowdloans Vaults (r:1 w:1)
	// Storage: Crowdloans LeasesRegistry (r:1 w:1)
	// Storage: unknown [0x] (r:3 w:0)
	fn dissolve_vault() -> Weight {
		// Minimum execution time: 115_318 nanoseconds.
		Weight::from_ref_time(116_935_000)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: Crowdloans Vaults (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:2 w:2)
	// Storage: System Account (r:1 w:1)
	// Storage: unknown [0xd861ea1ebf4800d4b89f4ff787ad79ee96d9a708c85b57da7eb8f9ddeda61291] (r:1 w:1)
	fn refund_for() -> Weight {
		// Minimum execution time: 131_633 nanoseconds.
		Weight::from_ref_time(132_824_000)
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(6))
	}
}
