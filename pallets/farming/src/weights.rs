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

//! Autogenerated weights for pallet_farming
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-02-23, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("vanilla-dev"), DB CACHE: 1024

// Executed Command:
// target/release/parallel
// benchmark
// --chain=vanilla-dev
// --execution=wasm
// --wasm-execution=compiled
// --pallet=pallet-farming
// --extrinsic=*
// --steps=50
// --repeat=20
// --heap-pages=4096
// --template=./.maintain/frame-weight-template.hbs
// --output=./pallets/farming/src/weights.rs

#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::all)]

use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_farming.
pub trait WeightInfo {	fn create() -> Weight;	fn set_pool_status() -> Weight;	fn set_pool_lock_duration() -> Weight;	fn deposit() -> Weight;	fn withdraw() -> Weight;	fn redeem() -> Weight;	fn claim() -> Weight;	fn dispatch_reward() -> Weight;}

/// Weights for pallet_farming using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {	fn create() -> Weight {
		(12_000_000 as Weight)			.saturating_add(T::DbWeight::get().reads(1 as Weight))			.saturating_add(T::DbWeight::get().writes(1 as Weight))	}	fn set_pool_status() -> Weight {
		(5_000_000 as Weight)			.saturating_add(T::DbWeight::get().reads(1 as Weight))			.saturating_add(T::DbWeight::get().writes(1 as Weight))	}	fn set_pool_lock_duration() -> Weight {
		(4_000_000 as Weight)			.saturating_add(T::DbWeight::get().reads(1 as Weight))			.saturating_add(T::DbWeight::get().writes(1 as Weight))	}	fn deposit() -> Weight {
		(47_000_000 as Weight)			.saturating_add(T::DbWeight::get().reads(3 as Weight))			.saturating_add(T::DbWeight::get().writes(3 as Weight))	}	fn withdraw() -> Weight {
		(26_000_000 as Weight)			.saturating_add(T::DbWeight::get().reads(2 as Weight))			.saturating_add(T::DbWeight::get().writes(2 as Weight))	}	fn redeem() -> Weight {
		(35_000_000 as Weight)			.saturating_add(T::DbWeight::get().reads(3 as Weight))			.saturating_add(T::DbWeight::get().writes(2 as Weight))	}	fn claim() -> Weight {
		(38_000_000 as Weight)			.saturating_add(T::DbWeight::get().reads(3 as Weight))			.saturating_add(T::DbWeight::get().writes(3 as Weight))	}	fn dispatch_reward() -> Weight {
		(40_000_000 as Weight)			.saturating_add(T::DbWeight::get().reads(2 as Weight))			.saturating_add(T::DbWeight::get().writes(2 as Weight))	}}

// For backwards compatibility and tests
impl WeightInfo for () {	fn create() -> Weight {
		(12_000_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(1 as Weight))			.saturating_add(RocksDbWeight::get().writes(1 as Weight))	}	fn set_pool_status() -> Weight {
		(5_000_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(1 as Weight))			.saturating_add(RocksDbWeight::get().writes(1 as Weight))	}	fn set_pool_lock_duration() -> Weight {
		(4_000_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(1 as Weight))			.saturating_add(RocksDbWeight::get().writes(1 as Weight))	}	fn deposit() -> Weight {
		(47_000_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(3 as Weight))			.saturating_add(RocksDbWeight::get().writes(3 as Weight))	}	fn withdraw() -> Weight {
		(26_000_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(2 as Weight))			.saturating_add(RocksDbWeight::get().writes(2 as Weight))	}	fn redeem() -> Weight {
		(35_000_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(3 as Weight))			.saturating_add(RocksDbWeight::get().writes(2 as Weight))	}	fn claim() -> Weight {
		(38_000_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(3 as Weight))			.saturating_add(RocksDbWeight::get().writes(3 as Weight))	}	fn dispatch_reward() -> Weight {
		(40_000_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(2 as Weight))			.saturating_add(RocksDbWeight::get().writes(2 as Weight))	}}
