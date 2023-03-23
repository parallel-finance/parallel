
//! Autogenerated weights for `pallet_liquid_staking`
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
// --pallet=pallet_liquid_staking
// --extrinsic=*
// --steps=50
// --repeat=20
// --output=./runtime/parallel/src/weights/pallet_liquid_staking.rs

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
		// Minimum execution time: 193_241 nanoseconds.
		Weight::from_ref_time(195_328_000)
			.saturating_add(T::DbWeight::get().reads(16))
			.saturating_add(T::DbWeight::get().writes(10))
	}
	// Storage: LiquidStaking ExchangeRate (r:1 w:0)
	// Storage: LiquidStaking Unlockings (r:1 w:1)
	// Storage: LiquidStaking CurrentEra (r:1 w:0)
	// Storage: Assets Metadata (r:1 w:0)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:1 w:1)
	// Storage: LiquidStaking MatchingPool (r:1 w:1)
	fn unstake() -> Weight {
		// Minimum execution time: 91_306 nanoseconds.
		Weight::from_ref_time(92_093_000)
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(4))
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
		// Minimum execution time: 148_258 nanoseconds.
		Weight::from_ref_time(150_590_000)
			.saturating_add(T::DbWeight::get().reads(13))
			.saturating_add(T::DbWeight::get().writes(8))
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
		// Minimum execution time: 142_306 nanoseconds.
		Weight::from_ref_time(143_796_000)
			.saturating_add(T::DbWeight::get().reads(11))
			.saturating_add(T::DbWeight::get().writes(7))
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
		// Minimum execution time: 155_036 nanoseconds.
		Weight::from_ref_time(157_369_000)
			.saturating_add(T::DbWeight::get().reads(13))
			.saturating_add(T::DbWeight::get().writes(8))
	}
	// Storage: LiquidStaking StakingLedgers (r:1 w:1)
	// Storage: LiquidStaking IsUpdated (r:1 w:1)
	// Storage: LiquidStaking XcmRequests (r:1 w:0)
	fn force_set_staking_ledger() -> Weight {
		// Minimum execution time: 53_018 nanoseconds.
		Weight::from_ref_time(54_300_000)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
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
		// Minimum execution time: 147_196 nanoseconds.
		Weight::from_ref_time(149_071_000)
			.saturating_add(T::DbWeight::get().reads(12))
			.saturating_add(T::DbWeight::get().writes(8))
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
		// Minimum execution time: 147_793 nanoseconds.
		Weight::from_ref_time(149_351_000)
			.saturating_add(T::DbWeight::get().reads(12))
			.saturating_add(T::DbWeight::get().writes(8))
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
		// Minimum execution time: 151_907 nanoseconds.
		Weight::from_ref_time(154_112_000)
			.saturating_add(T::DbWeight::get().reads(12))
			.saturating_add(T::DbWeight::get().writes(7))
	}
	// Storage: LiquidStaking ReserveFactor (r:1 w:1)
	fn update_reserve_factor() -> Weight {
		// Minimum execution time: 30_041 nanoseconds.
		Weight::from_ref_time(30_559_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: LiquidStaking CommissionRate (r:0 w:1)
	fn update_commission_rate() -> Weight {
		// Minimum execution time: 25_259 nanoseconds.
		Weight::from_ref_time(25_664_000)
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: LiquidStaking Incentive (r:0 w:1)
	fn update_incentive() -> Weight {
		// Minimum execution time: 25_110 nanoseconds.
		Weight::from_ref_time(25_596_000)
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: LiquidStaking StakingLedgerCap (r:1 w:1)
	fn update_staking_ledger_cap() -> Weight {
		// Minimum execution time: 28_073 nanoseconds.
		Weight::from_ref_time(28_890_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: LiquidStaking XcmRequests (r:1 w:1)
	// Storage: LiquidStaking StakingLedgers (r:1 w:1)
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: LiquidStaking MatchingPool (r:1 w:1)
	// Storage: Assets Metadata (r:1 w:0)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:1 w:1)
	fn notification_received() -> Weight {
		// Minimum execution time: 106_248 nanoseconds.
		Weight::from_ref_time(107_701_000)
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(5))
	}
	// Storage: LiquidStaking CurrentEra (r:1 w:0)
	// Storage: LiquidStaking Unlockings (r:1 w:1)
	// Storage: Assets Metadata (r:1 w:0)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:2 w:2)
	// Storage: LiquidStaking TotalReserves (r:1 w:0)
	// Storage: LiquidStaking MatchingPool (r:1 w:0)
	fn claim_for() -> Weight {
		// Minimum execution time: 117_611 nanoseconds.
		Weight::from_ref_time(118_897_000)
			.saturating_add(T::DbWeight::get().reads(8))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	// Storage: LiquidStaking EraStartBlock (r:0 w:1)
	fn force_set_era_start_block() -> Weight {
		// Minimum execution time: 9_698 nanoseconds.
		Weight::from_ref_time(9_944_000)
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: LiquidStaking CurrentEra (r:0 w:1)
	// Storage: LiquidStaking IsMatched (r:0 w:1)
	fn force_set_current_era() -> Weight {
		// Minimum execution time: 10_585 nanoseconds.
		Weight::from_ref_time(10_952_000)
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: ParachainSystem ValidationData (r:1 w:0)
	// Storage: LiquidStaking IsMatched (r:1 w:0)
	// Storage: LiquidStaking EraStartBlock (r:1 w:0)
	fn on_initialize() -> Weight {
		// Minimum execution time: 14_040 nanoseconds.
		Weight::from_ref_time(14_300_000)
			.saturating_add(T::DbWeight::get().reads(3))
	}
	// Storage: LiquidStaking StakingLedgers (r:7 w:0)
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
		// Minimum execution time: 337_220 nanoseconds.
		Weight::from_ref_time(342_165_000)
			.saturating_add(T::DbWeight::get().reads(25))
			.saturating_add(T::DbWeight::get().writes(14))
	}
	// Storage: LiquidStaking StakingLedgers (r:7 w:0)
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
		// Minimum execution time: 291_627 nanoseconds.
		Weight::from_ref_time(294_665_000)
			.saturating_add(T::DbWeight::get().reads(21))
			.saturating_add(T::DbWeight::get().writes(11))
	}
	// Storage: LiquidStaking TotalReserves (r:1 w:1)
	// Storage: Assets Metadata (r:1 w:0)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:2 w:2)
	fn reduce_reserves() -> Weight {
		// Minimum execution time: 93_224 nanoseconds.
		Weight::from_ref_time(94_455_000)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	// Storage: LiquidStaking FastUnstakeRequests (r:1 w:1)
	// Storage: Assets Metadata (r:1 w:0)
	// Storage: Assets Asset (r:1 w:0)
	// Storage: Assets Account (r:1 w:0)
	fn cancel_unstake() -> Weight {
		// Minimum execution time: 58_979 nanoseconds.
		Weight::from_ref_time(59_815_000)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(1))
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
		// Minimum execution time: 206_262 nanoseconds.
		Weight::from_ref_time(81_150_569)
			// Standard Error: 32_545
			.saturating_add(Weight::from_ref_time(130_436_493).saturating_mul(n.into()))
			.saturating_add(T::DbWeight::get().reads(9))
			.saturating_add(T::DbWeight::get().reads((4_u64).saturating_mul(n.into())))
			.saturating_add(T::DbWeight::get().writes(6))
			.saturating_add(T::DbWeight::get().writes((4_u64).saturating_mul(n.into())))
	}
}
