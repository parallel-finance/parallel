
//! Autogenerated weights for `pallet_liquid_staking`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-10-20, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `ip-172-88-3-164`, CPU: `Intel(R) Xeon(R) Platinum 8124M CPU @ 3.00GHz`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("heiko-dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/parallel
// benchmark
// pallet
// --chain=heiko-dev
// --execution=wasm
// --wasm-execution=compiled
// --pallet=pallet_liquid_staking
// --extrinsic=*
// --steps=50
// --repeat=20
// --output=./runtime/heiko/src/weights/pallet_liquid_staking.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_liquid_staking`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_liquid_staking::WeightInfo for WeightInfo<T> {
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
		(191_939_000 as u64)
			.saturating_add(T::DbWeight::get().reads(16 as u64))
			.saturating_add(T::DbWeight::get().writes(10 as u64))
	}
	// Storage: LiquidStaking ExchangeRate (r:1 w:0)
	// Storage: LiquidStaking Unlockings (r:1 w:1)
	// Storage: LiquidStaking CurrentEra (r:1 w:0)
	// Storage: Assets Metadata (r:1 w:0)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:1 w:1)
	// Storage: LiquidStaking MatchingPool (r:1 w:1)
	fn unstake() -> Weight {
		(84_517_000 as u64)
			.saturating_add(T::DbWeight::get().reads(7 as u64))
			.saturating_add(T::DbWeight::get().writes(4 as u64))
	}
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
		(137_884_000 as u64)
			.saturating_add(T::DbWeight::get().reads(13 as u64))
			.saturating_add(T::DbWeight::get().writes(8 as u64))
	}
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
		(126_180_000 as u64)
			.saturating_add(T::DbWeight::get().reads(11 as u64))
			.saturating_add(T::DbWeight::get().writes(7 as u64))
	}
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
		(140_667_000 as u64)
			.saturating_add(T::DbWeight::get().reads(13 as u64))
			.saturating_add(T::DbWeight::get().writes(8 as u64))
	}
	// Storage: LiquidStaking StakingLedgers (r:1 w:1)
	// Storage: LiquidStaking IsUpdated (r:1 w:1)
	// Storage: LiquidStaking XcmRequests (r:1 w:0)
	fn force_set_staking_ledger() -> Weight {
		(51_741_000 as u64)
			.saturating_add(T::DbWeight::get().reads(3 as u64))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
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
		(131_196_000 as u64)
			.saturating_add(T::DbWeight::get().reads(12 as u64))
			.saturating_add(T::DbWeight::get().writes(8 as u64))
	}
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
	fn rebond() -> Weight {
		(130_325_000 as u64)
			.saturating_add(T::DbWeight::get().reads(12 as u64))
			.saturating_add(T::DbWeight::get().writes(8 as u64))
	}
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
		(132_311_000 as u64)
			.saturating_add(T::DbWeight::get().reads(12 as u64))
			.saturating_add(T::DbWeight::get().writes(7 as u64))
	}
	// Storage: LiquidStaking ReserveFactor (r:1 w:1)
	fn update_reserve_factor() -> Weight {
		(26_745_000 as u64)
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: LiquidStaking CommissionRate (r:0 w:1)
	fn update_commission_rate() -> Weight {
		(23_681_000 as u64)
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: LiquidStaking StakingLedgerCap (r:1 w:1)
	fn update_staking_ledger_cap() -> Weight {
		(26_846_000 as u64)
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: LiquidStaking XcmRequests (r:1 w:1)
	// Storage: LiquidStaking StakingLedgers (r:1 w:1)
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: LiquidStaking MatchingPool (r:1 w:1)
	// Storage: Assets Metadata (r:1 w:0)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:1 w:1)
	fn notification_received() -> Weight {
		(94_750_000 as u64)
			.saturating_add(T::DbWeight::get().reads(7 as u64))
			.saturating_add(T::DbWeight::get().writes(5 as u64))
	}
	// Storage: LiquidStaking CurrentEra (r:1 w:0)
	// Storage: LiquidStaking Unlockings (r:1 w:1)
	// Storage: Assets Metadata (r:1 w:0)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:2 w:2)
	// Storage: LiquidStaking TotalReserves (r:1 w:0)
	// Storage: LiquidStaking MatchingPool (r:1 w:0)
	fn claim_for() -> Weight {
		(109_068_000 as u64)
			.saturating_add(T::DbWeight::get().reads(8 as u64))
			.saturating_add(T::DbWeight::get().writes(4 as u64))
	}
	// Storage: LiquidStaking EraStartBlock (r:0 w:1)
	fn force_set_era_start_block() -> Weight {
		(8_488_000 as u64)
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: LiquidStaking CurrentEra (r:0 w:1)
	// Storage: LiquidStaking IsMatched (r:0 w:1)
	fn force_set_current_era() -> Weight {
		(9_685_000 as u64)
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
	// Storage: ParachainSystem ValidationData (r:1 w:0)
	// Storage: LiquidStaking IsMatched (r:1 w:0)
	// Storage: LiquidStaking EraStartBlock (r:1 w:0)
	fn on_initialize() -> Weight {
		(12_624_000 as u64)
			.saturating_add(T::DbWeight::get().reads(3 as u64))
	}
	// Storage: LiquidStaking StakingLedgers (r:10 w:0)
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
	// Storage: LiquidStaking CurrentEra (r:1 w:1)
	// Storage: ParachainSystem ValidationData (r:1 w:0)
	// Storage: Assets Metadata (r:1 w:0)
	// Storage: LiquidStaking ExchangeRate (r:1 w:1)
	// Storage: LiquidStaking EraStartBlock (r:0 w:1)
	// Storage: LiquidStaking IsMatched (r:0 w:1)
	// Storage: LiquidStaking XcmRequests (r:0 w:2)
	// Storage: PolkadotXcm Queries (r:0 w:2)
	fn force_advance_era() -> Weight {
		(358_801_000 as u64)
			.saturating_add(T::DbWeight::get().reads(28 as u64))
			.saturating_add(T::DbWeight::get().writes(14 as u64))
	}
	// Storage: LiquidStaking StakingLedgers (r:10 w:0)
	// Storage: LiquidStaking MatchingPool (r:1 w:1)
	// Storage: LiquidStaking StakingLedgerCap (r:1 w:0)
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: XcmHelper XcmWeightFee (r:2 w:0)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:1 w:1)
	// Storage: PolkadotXcm QueryCounter (r:1 w:1)
	// Storage: PolkadotXcm SupportedVersion (r:1 w:0)
	// Storage: PolkadotXcm VersionDiscoveryQueue (r:1 w:1)
	// Storage: PolkadotXcm SafeXcmVersion (r:1 w:0)
	// Storage: ParachainSystem HostConfiguration (r:1 w:0)
	// Storage: ParachainSystem PendingUpwardMessages (r:1 w:1)
	// Storage: LiquidStaking CurrentEra (r:1 w:0)
	// Storage: LiquidStaking IsMatched (r:0 w:1)
	// Storage: LiquidStaking XcmRequests (r:0 w:2)
	// Storage: PolkadotXcm Queries (r:0 w:2)
	fn force_matching() -> Weight {
		(304_997_000 as u64)
			.saturating_add(T::DbWeight::get().reads(24 as u64))
			.saturating_add(T::DbWeight::get().writes(11 as u64))
	}
	// Storage: LiquidStaking TotalReserves (r:1 w:1)
	// Storage: Assets Metadata (r:1 w:0)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:2 w:2)
	fn reduce_reserves() -> Weight {
		(85_536_000 as u64)
			.saturating_add(T::DbWeight::get().reads(5 as u64))
			.saturating_add(T::DbWeight::get().writes(4 as u64))
	}
	// Storage: LiquidStaking FastUnstakeRequests (r:1 w:1)
	// Storage: Assets Metadata (r:1 w:0)
	// Storage: Assets Asset (r:1 w:0)
	// Storage: Assets Account (r:1 w:0)
	fn cancel_unstake() -> Weight {
		(52_267_000 as u64)
			.saturating_add(T::DbWeight::get().reads(4 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: LiquidStaking FastUnstakeRequests (r:1 w:1)
	// Storage: Assets Metadata (r:2 w:0)
	// Storage: Assets Asset (r:2 w:2)
	// Storage: Assets Account (r:4 w:4)
	// Storage: LiquidStaking MatchingPool (r:1 w:1)
	// Storage: LiquidStaking ExchangeRate (r:1 w:0)
	// Storage: System Account (r:2 w:2)
	/// The range of component `n` is `[1, 50]`.
	fn fast_match_unstake(n: u32, ) -> Weight {
		(62_681_000 as u64)
			// Standard Error: 40_000
			.saturating_add((128_519_000 as u64).saturating_mul(n as u64))
			.saturating_add(T::DbWeight::get().reads(9 as u64))
			.saturating_add(T::DbWeight::get().reads((4 as u64).saturating_mul(n as u64)))
			.saturating_add(T::DbWeight::get().writes(6 as u64))
			.saturating_add(T::DbWeight::get().writes((4 as u64).saturating_mul(n as u64)))
	}
}
