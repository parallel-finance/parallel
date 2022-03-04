// This file is part of Substrate.

// Copyright (C) 2021 Parity Technologies (UK) Ltd.
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

//! Autogenerated weights for pallet_crowdloans
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-01-26, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("vanilla-dev"), DB CACHE: 128

// Executed Command:
// ./target/release/parallel
// benchmark
// --chain=vanilla-dev
// --execution=wasm
// --wasm-execution=compiled
// --pallet=pallet-crowdloans
// --extrinsic=*
// --steps=50
// --repeat=20
// --heap-pages=4096
// --template=./.maintain/frame-weight-template.hbs
// --output=./pallets/crowdloans/src/weights.rs

#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::all)]

use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_crowdloans.
pub trait WeightInfo {
    fn create_vault() -> Weight;
    fn update_vault() -> Weight;
    fn contribute() -> Weight;
    fn open() -> Weight;
    fn close() -> Weight;
    fn set_vrf() -> Weight;
    fn reopen() -> Weight;
    fn auction_succeeded() -> Weight;
    fn auction_failed() -> Weight;
    fn claim() -> Weight;
    fn withdraw() -> Weight;
    fn redeem() -> Weight;
    fn slot_expired() -> Weight;
    fn migrate_pending() -> Weight;
    fn notification_received() -> Weight;
    fn refund() -> Weight;
    fn dissolve_vault() -> Weight;
}

/// Weights for pallet_crowdloans using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn create_vault() -> Weight {
        (92_638_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(5 as Weight))
            .saturating_add(T::DbWeight::get().writes(4 as Weight))
    }
    fn update_vault() -> Weight {
        (70_861_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn contribute() -> Weight {
        (403_418_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(19 as Weight))
            .saturating_add(T::DbWeight::get().writes(12 as Weight))
    }
    fn open() -> Weight {
        (60_225_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn close() -> Weight {
        (60_271_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn set_vrf() -> Weight {
        (44_341_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn reopen() -> Weight {
        (59_564_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn auction_succeeded() -> Weight {
        (59_688_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn auction_failed() -> Weight {
        (220_237_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(13 as Weight))
            .saturating_add(T::DbWeight::get().writes(8 as Weight))
    }
    fn claim() -> Weight {
        (148_938_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(5 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    fn withdraw() -> Weight {
        (139_012_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(4 as Weight))
    }
    fn redeem() -> Weight {
        (181_479_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(6 as Weight))
            .saturating_add(T::DbWeight::get().writes(5 as Weight))
    }
    fn slot_expired() -> Weight {
        (229_075_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(13 as Weight))
            .saturating_add(T::DbWeight::get().writes(8 as Weight))
    }
    fn migrate_pending() -> Weight {
        (548_013_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(17 as Weight))
            .saturating_add(T::DbWeight::get().writes(10 as Weight))
    }
    fn notification_received() -> Weight {
        (222_858_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(7 as Weight))
            .saturating_add(T::DbWeight::get().writes(7 as Weight))
    }
    fn refund() -> Weight {
        (685_505_000 as Weight).saturating_add(T::DbWeight::get().reads(4 as Weight))
    }
    fn dissolve_vault() -> Weight {
        (2_343_892_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(5 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    fn create_vault() -> Weight {
        (92_638_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(5 as Weight))
            .saturating_add(RocksDbWeight::get().writes(4 as Weight))
    }
    fn update_vault() -> Weight {
        (70_861_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(3 as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    fn contribute() -> Weight {
        (403_418_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(19 as Weight))
            .saturating_add(RocksDbWeight::get().writes(12 as Weight))
    }
    fn open() -> Weight {
        (60_225_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(2 as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    fn close() -> Weight {
        (60_271_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(2 as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    fn set_vrf() -> Weight {
        (44_341_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(1 as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    fn reopen() -> Weight {
        (59_564_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(2 as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    fn auction_succeeded() -> Weight {
        (59_688_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(2 as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    fn auction_failed() -> Weight {
        (220_237_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(13 as Weight))
            .saturating_add(RocksDbWeight::get().writes(8 as Weight))
    }
    fn claim() -> Weight {
        (148_938_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(5 as Weight))
            .saturating_add(RocksDbWeight::get().writes(3 as Weight))
    }
    fn withdraw() -> Weight {
        (139_012_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(4 as Weight))
            .saturating_add(RocksDbWeight::get().writes(4 as Weight))
    }
    fn redeem() -> Weight {
        (181_479_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(6 as Weight))
            .saturating_add(RocksDbWeight::get().writes(5 as Weight))
    }
    fn slot_expired() -> Weight {
        (229_075_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(13 as Weight))
            .saturating_add(RocksDbWeight::get().writes(8 as Weight))
    }
    fn migrate_pending() -> Weight {
        (548_013_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(17 as Weight))
            .saturating_add(RocksDbWeight::get().writes(10 as Weight))
    }
    fn notification_received() -> Weight {
        (222_858_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(7 as Weight))
            .saturating_add(RocksDbWeight::get().writes(7 as Weight))
    }
    fn refund() -> Weight {
        (685_505_000 as Weight).saturating_add(RocksDbWeight::get().reads(4 as Weight))
    }
    fn dissolve_vault() -> Weight {
        (2_343_892_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(5 as Weight))
            .saturating_add(RocksDbWeight::get().writes(2 as Weight))
    }
}
