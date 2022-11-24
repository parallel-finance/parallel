
//! Autogenerated weights for `pallet_crowdloans`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-10-20, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
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
		(61_017_000 as u64)
			.saturating_add(T::DbWeight::get().reads(5 as u64))
			.saturating_add(T::DbWeight::get().writes(4 as u64))
	}
	// Storage: Crowdloans LeasesRegistry (r:1 w:0)
	// Storage: Crowdloans Vaults (r:1 w:1)
	// Storage: ParachainSystem ValidationData (r:1 w:0)
	fn update_vault() -> Weight {
		(49_011_000 as u64)
			.saturating_add(T::DbWeight::get().reads(3 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
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
		(211_135_000 as u64)
			.saturating_add(T::DbWeight::get().reads(18 as u64))
			.saturating_add(T::DbWeight::get().writes(12 as u64))
	}
	// Storage: Crowdloans LeasesRegistry (r:1 w:0)
	// Storage: Crowdloans Vaults (r:1 w:1)
	fn open() -> Weight {
		(46_258_000 as u64)
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Crowdloans LeasesRegistry (r:1 w:0)
	// Storage: Crowdloans Vaults (r:1 w:1)
	fn close() -> Weight {
		(46_173_000 as u64)
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Crowdloans IsVrf (r:0 w:1)
	fn set_vrf() -> Weight {
		(24_057_000 as u64)
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Crowdloans ProxyAddress (r:0 w:1)
	fn update_proxy() -> Weight {
		(25_325_000 as u64)
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Crowdloans LeasesBonus (r:0 w:1)
	fn update_leases_bonus() -> Weight {
		(28_426_000 as u64)
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Crowdloans LeasesRegistry (r:1 w:0)
	// Storage: Crowdloans Vaults (r:1 w:1)
	fn reopen() -> Weight {
		(46_601_000 as u64)
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Crowdloans LeasesRegistry (r:1 w:0)
	// Storage: Crowdloans Vaults (r:1 w:1)
	fn auction_succeeded() -> Weight {
		(46_541_000 as u64)
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
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
		(143_462_000 as u64)
			.saturating_add(T::DbWeight::get().reads(12 as u64))
			.saturating_add(T::DbWeight::get().writes(8 as u64))
	}
	// Storage: Crowdloans CTokensRegistry (r:1 w:0)
	// Storage: Crowdloans Vaults (r:1 w:0)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:1 w:1)
	// Storage: Crowdloans LeasesBonus (r:1 w:0)
	// Storage: Assets Metadata (r:1 w:0)
	// Storage: unknown [0xd861ea1ebf4800d4b89f4ff787ad79ee96d9a708c85b57da7eb8f9ddeda61291] (r:1 w:1)
	fn claim() -> Weight {
		(103_245_000 as u64)
			.saturating_add(T::DbWeight::get().reads(7 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
	// Storage: Crowdloans Vaults (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:1 w:1)
	// Storage: unknown [0xd861ea1ebf4800d4b89f4ff787ad79ee96d9a708c85b57da7eb8f9ddeda61291] (r:1 w:1)
	fn withdraw() -> Weight {
		(86_342_000 as u64)
			.saturating_add(T::DbWeight::get().reads(4 as u64))
			.saturating_add(T::DbWeight::get().writes(4 as u64))
	}
	// Storage: Crowdloans CTokensRegistry (r:1 w:0)
	// Storage: Crowdloans Vaults (r:1 w:1)
	// Storage: Assets Asset (r:2 w:2)
	// Storage: Assets Account (r:2 w:2)
	fn redeem() -> Weight {
		(121_438_000 as u64)
			.saturating_add(T::DbWeight::get().reads(6 as u64))
			.saturating_add(T::DbWeight::get().writes(5 as u64))
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
		(143_350_000 as u64)
			.saturating_add(T::DbWeight::get().reads(12 as u64))
			.saturating_add(T::DbWeight::get().writes(8 as u64))
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
		(221_947_000 as u64)
			.saturating_add(T::DbWeight::get().reads(16 as u64))
			.saturating_add(T::DbWeight::get().writes(10 as u64))
	}
	// Storage: Crowdloans XcmRequests (r:1 w:1)
	// Storage: Crowdloans Vaults (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	// Storage: unknown [0xd861ea1ebf4800d4b89f4ff787ad79ee96d9a708c85b57da7eb8f9ddeda61291] (r:2 w:2)
	fn notification_received() -> Weight {
		(130_032_000 as u64)
			.saturating_add(T::DbWeight::get().reads(7 as u64))
			.saturating_add(T::DbWeight::get().writes(7 as u64))
	}
	// Storage: Crowdloans Vaults (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:1 w:1)
	// Storage: unknown [0x] (r:3 w:0)
	// Storage: unknown [0xd861ea1ebf4800d4b89f4ff787ad79ee96d9a708c85b57da7eb8f9ddeda61291] (r:1 w:1)
	fn refund() -> Weight {
		(169_835_000 as u64)
			.saturating_add(T::DbWeight::get().reads(7 as u64))
			.saturating_add(T::DbWeight::get().writes(4 as u64))
	}
	// Storage: Crowdloans Vaults (r:1 w:1)
	// Storage: Crowdloans LeasesRegistry (r:1 w:1)
	// Storage: unknown [0x] (r:3 w:0)
	fn dissolve_vault() -> Weight {
		(111_735_000 as u64)
			.saturating_add(T::DbWeight::get().reads(5 as u64))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
	// Storage: Crowdloans Vaults (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:2 w:2)
	// Storage: System Account (r:1 w:1)
	// Storage: unknown [0xd861ea1ebf4800d4b89f4ff787ad79ee96d9a708c85b57da7eb8f9ddeda61291] (r:1 w:1)
	fn refund_for() -> Weight {
		(122_452_000 as u64)
			.saturating_add(T::DbWeight::get().reads(6 as u64))
			.saturating_add(T::DbWeight::get().writes(6 as u64))
	}
}
