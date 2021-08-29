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

//! Autogenerated weights for pallet_amm
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2021-08-29, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("parallel"), DB CACHE: 128

// Executed Command:
// ./target/release/parallel
// benchmark
// --chain=parallel
// --execution=wasm
// --wasm-execution=compiled
// --pallet=pallet-amm
// --extrinsic=*
// --steps=50
// --repeat=20
// --heap-pages=4096
// --template=./.maintain/frame-weight-template.hbs
// --output=./pallets/amm/src/weights.rs

#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::all)]

use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_amm.
pub trait WeightInfo {
    fn add_liquidity_non_existing_pool() -> Weight;
    fn add_liquidity_existing_pool() -> Weight;
    fn remove_liquidity() -> Weight;
}

/// Weights for pallet_amm using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn add_liquidity_non_existing_pool() -> Weight {
        (98_000_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(6 as Weight))
            .saturating_add(T::DbWeight::get().writes(7 as Weight))
    }
    fn add_liquidity_existing_pool() -> Weight {
        (84_000_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(6 as Weight))
            .saturating_add(T::DbWeight::get().writes(6 as Weight))
    }
    fn remove_liquidity() -> Weight {
        (83_000_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(7 as Weight))
            .saturating_add(T::DbWeight::get().writes(6 as Weight))
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    fn add_liquidity_non_existing_pool() -> Weight {
        (98_000_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(6 as Weight))
            .saturating_add(RocksDbWeight::get().writes(7 as Weight))
    }
    fn add_liquidity_existing_pool() -> Weight {
        (84_000_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(6 as Weight))
            .saturating_add(RocksDbWeight::get().writes(6 as Weight))
    }
    fn remove_liquidity() -> Weight {
        (83_000_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(7 as Weight))
            .saturating_add(RocksDbWeight::get().writes(6 as Weight))
    }
}
