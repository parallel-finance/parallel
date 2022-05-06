
//! Autogenerated weights for `pallet_liquid_staking`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-05-04, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("kerria-dev"), DB CACHE: 1024

// Executed Command:
// target/release/parallel
// benchmark
// pallet
// --chain=kerria-dev
// --execution=wasm
// --wasm-execution=compiled
// --pallet=pallet_liquid_staking
// --extrinsic=*
// --steps=50
// --repeat=20
// --output=./runtime/kerria/src/weights/pallet_liquid_staking.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_liquid_staking`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_liquid_staking::WeightInfo for WeightInfo<T> {
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: LiquidStaking ReserveFactor (r:1 w:0)
	// Storage: Assets Metadata (r:2 w:0)
	// Storage: Assets Asset (r:2 w:2)
	// Storage: Assets Account (r:4 w:4)
	// Storage: System Account (r:2 w:2)
	// Storage: LiquidStaking ExchangeRate (r:1 w:0)
	// Storage: LiquidStaking StakingLedgers (r:1 w:0)
	// Storage: LiquidStaking StakingLedgerCap (r:1 w:0)
	// Storage: LiquidStaking MatchingPool (r:1 w:1)
	// Storage: LiquidStaking TotalReserves (r:1 w:1)
	fn stake() -> Weight {
		(102_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(17 as Weight))
			.saturating_add(T::DbWeight::get().writes(11 as Weight))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: LiquidStaking ExchangeRate (r:1 w:0)
	// Storage: LiquidStaking Unlockings (r:1 w:1)
	// Storage: LiquidStaking CurrentEra (r:1 w:0)
	// Storage: Assets Metadata (r:1 w:0)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:1 w:1)
	// Storage: LiquidStaking MatchingPool (r:1 w:1)
	fn unstake() -> Weight {
		(43_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(8 as Weight))
			.saturating_add(T::DbWeight::get().writes(5 as Weight))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: LiquidStaking StakingLedgers (r:1 w:0)
	// Storage: LiquidStaking StakingLedgerCap (r:1 w:0)
	// Storage: LiquidStaking MatchingPool (r:1 w:1)
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
	// Storage: LiquidStaking XcmRequests (r:0 w:1)
	// Storage: PolkadotXcm Queries (r:0 w:1)
	fn bond() -> Weight {
		(76_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(14 as Weight))
			.saturating_add(T::DbWeight::get().writes(9 as Weight))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: LiquidStaking StakingLedgers (r:1 w:0)
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
	// Storage: LiquidStaking XcmRequests (r:0 w:1)
	// Storage: PolkadotXcm Queries (r:0 w:1)
	fn nominate() -> Weight {
		(64_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(12 as Weight))
			.saturating_add(T::DbWeight::get().writes(8 as Weight))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: LiquidStaking StakingLedgers (r:1 w:0)
	// Storage: LiquidStaking StakingLedgerCap (r:1 w:0)
	// Storage: LiquidStaking MatchingPool (r:1 w:1)
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
	// Storage: LiquidStaking XcmRequests (r:0 w:1)
	// Storage: PolkadotXcm Queries (r:0 w:1)
	fn bond_extra() -> Weight {
		(78_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(14 as Weight))
			.saturating_add(T::DbWeight::get().writes(9 as Weight))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: LiquidStaking StakingLedgers (r:1 w:1)
	// Storage: LiquidStaking IsUpdated (r:1 w:1)
	// Storage: LiquidStaking XcmRequests (r:1 w:0)
	fn force_set_staking_ledger() -> Weight {
		(25_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: LiquidStaking StakingLedgers (r:1 w:0)
	// Storage: LiquidStaking MatchingPool (r:1 w:1)
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
	// Storage: LiquidStaking XcmRequests (r:0 w:1)
	// Storage: PolkadotXcm Queries (r:0 w:1)
	fn unbond() -> Weight {
		(68_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(13 as Weight))
			.saturating_add(T::DbWeight::get().writes(9 as Weight))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: LiquidStaking StakingLedgers (r:1 w:0)
	// Storage: LiquidStaking StakingLedgerCap (r:1 w:0)
	// Storage: LiquidStaking MatchingPool (r:1 w:1)
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
	// Storage: LiquidStaking XcmRequests (r:0 w:1)
	// Storage: PolkadotXcm Queries (r:0 w:1)
	fn rebond() -> Weight {
		(69_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(14 as Weight))
			.saturating_add(T::DbWeight::get().writes(9 as Weight))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: LiquidStaking CurrentEra (r:1 w:0)
	// Storage: LiquidStaking StakingLedgers (r:1 w:0)
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
	// Storage: LiquidStaking XcmRequests (r:0 w:1)
	// Storage: PolkadotXcm Queries (r:0 w:1)
	fn withdraw_unbonded() -> Weight {
		(67_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(13 as Weight))
			.saturating_add(T::DbWeight::get().writes(8 as Weight))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: LiquidStaking ReserveFactor (r:1 w:1)
	fn update_reserve_factor() -> Weight {
		(14_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: LiquidStaking StakingLedgerCap (r:1 w:1)
	fn update_staking_ledger_cap() -> Weight {
		(14_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: LiquidStaking XcmRequests (r:1 w:1)
	// Storage: LiquidStaking StakingLedgers (r:1 w:1)
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: LiquidStaking MatchingPool (r:1 w:1)
	// Storage: Assets Metadata (r:1 w:0)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:1 w:1)
	fn notification_received() -> Weight {
		(48_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(8 as Weight))
			.saturating_add(T::DbWeight::get().writes(6 as Weight))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: LiquidStaking CurrentEra (r:1 w:0)
	// Storage: LiquidStaking Unlockings (r:1 w:1)
	// Storage: Assets Metadata (r:1 w:0)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:2 w:2)
	// Storage: LiquidStaking TotalReserves (r:1 w:0)
	// Storage: LiquidStaking MatchingPool (r:1 w:0)
	fn claim_for() -> Weight {
		(57_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(9 as Weight))
			.saturating_add(T::DbWeight::get().writes(5 as Weight))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: LiquidStaking EraStartBlock (r:0 w:1)
	fn force_set_era_start_block() -> Weight {
		(3_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: LiquidStaking CurrentEra (r:0 w:1)
	fn force_set_current_era() -> Weight {
		(3_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: ParachainSystem ValidationData (r:1 w:0)
	// Storage: LiquidStaking EraStartBlock (r:1 w:0)
	fn on_initialize() -> Weight {
		(4_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: ParachainSystem ValidationData (r:1 w:0)
	// Storage: LiquidStaking CurrentEra (r:1 w:1)
	// Storage: LiquidStaking StakingLedgers (r:3 w:0)
	// Storage: LiquidStaking MatchingPool (r:1 w:1)
	// Storage: LiquidStaking StakingLedgerCap (r:1 w:0)
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: XcmHelper XcmWeightFee (r:2 w:0)
	// Storage: Assets Asset (r:2 w:1)
	// Storage: Assets Account (r:1 w:1)
	// Storage: PolkadotXcm QueryCounter (r:1 w:1)
	// Storage: PolkadotXcm SupportedVersion (r:1 w:0)
	// Storage: PolkadotXcm VersionDiscoveryQueue (r:1 w:1)
	// Storage: PolkadotXcm SafeXcmVersion (r:1 w:0)
	// Storage: ParachainSystem HostConfiguration (r:1 w:0)
	// Storage: ParachainSystem PendingUpwardMessages (r:1 w:1)
	// Storage: Assets Metadata (r:1 w:0)
	// Storage: LiquidStaking ExchangeRate (r:1 w:1)
	// Storage: LiquidStaking EraStartBlock (r:0 w:1)
	// Storage: LiquidStaking XcmRequests (r:0 w:2)
	// Storage: PolkadotXcm Queries (r:0 w:2)
	fn force_advance_era() -> Weight {
		(221_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(22 as Weight))
			.saturating_add(T::DbWeight::get().writes(14 as Weight))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: LiquidStaking StakingLedgers (r:3 w:0)
	// Storage: LiquidStaking MatchingPool (r:1 w:1)
	// Storage: LiquidStaking StakingLedgerCap (r:1 w:0)
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: XcmHelper XcmWeightFee (r:2 w:0)
	// Storage: Assets Asset (r:2 w:1)
	// Storage: Assets Account (r:1 w:1)
	// Storage: PolkadotXcm QueryCounter (r:1 w:1)
	// Storage: PolkadotXcm SupportedVersion (r:1 w:0)
	// Storage: PolkadotXcm VersionDiscoveryQueue (r:1 w:1)
	// Storage: PolkadotXcm SafeXcmVersion (r:1 w:0)
	// Storage: ParachainSystem HostConfiguration (r:1 w:0)
	// Storage: ParachainSystem PendingUpwardMessages (r:1 w:1)
	// Storage: LiquidStaking CurrentEra (r:1 w:0)
	// Storage: Assets Metadata (r:1 w:0)
	// Storage: LiquidStaking ExchangeRate (r:1 w:1)
	// Storage: LiquidStaking XcmRequests (r:0 w:2)
	// Storage: PolkadotXcm Queries (r:0 w:2)
	fn force_matching() -> Weight {
		(187_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(21 as Weight))
			.saturating_add(T::DbWeight::get().writes(12 as Weight))
	}

	fn reduce_reserves() -> Weight {
		(84_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(7 as Weight))
			.saturating_add(T::DbWeight::get().writes(5 as Weight))
	}
}
