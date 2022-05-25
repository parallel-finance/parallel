// This file is part of Parallel Finance.

// Copyright (C) 2022 Parallel Finance Developer.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Autogenerated weights for pallet_liquid_staking
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
// --pallet=pallet-liquid-staking
// --extrinsic=*
// --steps=50
// --repeat=20
// --heap-pages=4096
// --template=./.maintain/frame-weight-template.hbs
// --output=./pallets/liquid-staking/src/weights.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_liquid_staking.
pub trait WeightInfo {
	fn stake() -> Weight;
	fn unstake() -> Weight;
	fn bond() -> Weight;
	fn nominate() -> Weight;
	fn bond_extra() -> Weight;
	fn force_set_staking_ledger() -> Weight;
	fn unbond() -> Weight;
	fn rebond() -> Weight;
	fn withdraw_unbonded() -> Weight;
	fn update_reserve_factor() -> Weight;
	fn update_staking_ledger_cap() -> Weight;
	fn notification_received() -> Weight;
	fn claim_for() -> Weight;
	fn force_set_era_start_block() -> Weight;
	fn force_set_current_era() -> Weight;
	fn on_initialize() -> Weight;
	fn force_advance_era() -> Weight;
	fn force_matching() -> Weight;
	fn reduce_reserves() -> Weight;
	fn cancel_unstake() -> Weight;
}

/// Weights for pallet_liquid_staking using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
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
		(101_000_000 as Weight)
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
		(44_000_000 as Weight)
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
		(66_000_000 as Weight)
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
		(61_000_000 as Weight)
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
		(69_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(14 as Weight))
			.saturating_add(T::DbWeight::get().writes(9 as Weight))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: LiquidStaking StakingLedgers (r:1 w:1)
	// Storage: LiquidStaking IsUpdated (r:1 w:1)
	// Storage: LiquidStaking XcmRequests (r:1 w:0)
	fn force_set_staking_ledger() -> Weight {
		(22_000_000 as Weight)
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
		(64_000_000 as Weight)
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
		(68_000_000 as Weight)
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
		(66_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(13 as Weight))
			.saturating_add(T::DbWeight::get().writes(8 as Weight))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: LiquidStaking ReserveFactor (r:1 w:1)
	fn update_reserve_factor() -> Weight {
		(13_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: LiquidStaking StakingLedgerCap (r:1 w:1)
	fn update_staking_ledger_cap() -> Weight {
		(13_000_000 as Weight)
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
		(47_000_000 as Weight)
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
		(56_000_000 as Weight)
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
		(3_000_000 as Weight)
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
		(160_000_000 as Weight)
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
		(153_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(21 as Weight))
			.saturating_add(T::DbWeight::get().writes(12 as Weight))
	}

	fn reduce_reserves() -> Weight {
		(56_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(6 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}

	fn cancel_unstake() -> Weight {
		(109_589_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(8 as Weight))
			.saturating_add(T::DbWeight::get().writes(5 as Weight))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
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
		(101_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(17 as Weight))
			.saturating_add(RocksDbWeight::get().writes(11 as Weight))
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
		(44_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(8 as Weight))
			.saturating_add(RocksDbWeight::get().writes(5 as Weight))
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
		(66_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(14 as Weight))
			.saturating_add(RocksDbWeight::get().writes(9 as Weight))
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
		(61_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(12 as Weight))
			.saturating_add(RocksDbWeight::get().writes(8 as Weight))
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
		(69_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(14 as Weight))
			.saturating_add(RocksDbWeight::get().writes(9 as Weight))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: LiquidStaking StakingLedgers (r:1 w:1)
	// Storage: LiquidStaking IsUpdated (r:1 w:1)
	// Storage: LiquidStaking XcmRequests (r:1 w:0)
	fn force_set_staking_ledger() -> Weight {
		(22_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(4 as Weight))
			.saturating_add(RocksDbWeight::get().writes(3 as Weight))
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
		(64_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(13 as Weight))
			.saturating_add(RocksDbWeight::get().writes(9 as Weight))
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
		(68_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(14 as Weight))
			.saturating_add(RocksDbWeight::get().writes(9 as Weight))
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
		(66_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(13 as Weight))
			.saturating_add(RocksDbWeight::get().writes(8 as Weight))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: LiquidStaking ReserveFactor (r:1 w:1)
	fn update_reserve_factor() -> Weight {
		(13_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(2 as Weight))
			.saturating_add(RocksDbWeight::get().writes(2 as Weight))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: LiquidStaking StakingLedgerCap (r:1 w:1)
	fn update_staking_ledger_cap() -> Weight {
		(13_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(2 as Weight))
			.saturating_add(RocksDbWeight::get().writes(2 as Weight))
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
		(47_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(8 as Weight))
			.saturating_add(RocksDbWeight::get().writes(6 as Weight))
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
		(56_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(9 as Weight))
			.saturating_add(RocksDbWeight::get().writes(5 as Weight))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: LiquidStaking EraStartBlock (r:0 w:1)
	fn force_set_era_start_block() -> Weight {
		(3_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes(2 as Weight))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: LiquidStaking CurrentEra (r:0 w:1)
	fn force_set_current_era() -> Weight {
		(3_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes(2 as Weight))
	}
	// Storage: ParachainSystem ValidationData (r:1 w:0)
	// Storage: LiquidStaking EraStartBlock (r:1 w:0)
	fn on_initialize() -> Weight {
		(3_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(2 as Weight))
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
		(160_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(22 as Weight))
			.saturating_add(RocksDbWeight::get().writes(14 as Weight))
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
		(153_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(21 as Weight))
			.saturating_add(RocksDbWeight::get().writes(12 as Weight))
	}

	fn reduce_reserves() -> Weight {
		(56_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(6 as Weight))
			.saturating_add(RocksDbWeight::get().writes(5 as Weight))
	}

	fn cancel_unstake() -> Weight {
		(109_589_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(8 as Weight))
			.saturating_add(RocksDbWeight::get().writes(5 as Weight))
	}
}
