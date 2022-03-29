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

//! Autogenerated weights for pallet_bridge
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-03-29, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("kerria-dev"), DB CACHE: 1024

// Executed Command:
// target/release/parallel
// benchmark
// --chain=kerria-dev
// --execution=wasm
// --wasm-execution=compiled
// --pallet=pallet-bridge
// --extrinsic=*
// --steps=50
// --repeat=20
// --heap-pages=4096
// --template=./.maintain/frame-weight-template.hbs
// --output=./pallets/bridge/src/weights.rs

#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::all)]

use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_bridge.
pub trait WeightInfo {	fn register_chain() -> Weight;	fn unregister_chain() -> Weight;	fn register_bridge_token() -> Weight;	fn unregister_bridge_token() -> Weight;	fn set_bridge_token_fee() -> Weight;	fn set_bridge_token_status() -> Weight;	fn set_bridge_token_cap() -> Weight;	fn clean_cap_accumulated_value() -> Weight;	fn teleport() -> Weight;	fn materialize() -> Weight;}

/// Weights for pallet_bridge using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {	fn register_chain() -> Weight {
		(24_411_000 as Weight)			.saturating_add(T::DbWeight::get().reads(1 as Weight))			.saturating_add(T::DbWeight::get().writes(2 as Weight))	}	fn unregister_chain() -> Weight {
		(25_191_000 as Weight)			.saturating_add(T::DbWeight::get().reads(1 as Weight))			.saturating_add(T::DbWeight::get().writes(2 as Weight))	}	fn register_bridge_token() -> Weight {
		(28_960_000 as Weight)			.saturating_add(T::DbWeight::get().reads(2 as Weight))			.saturating_add(T::DbWeight::get().writes(2 as Weight))	}	fn unregister_bridge_token() -> Weight {
		(27_210_000 as Weight)			.saturating_add(T::DbWeight::get().reads(1 as Weight))			.saturating_add(T::DbWeight::get().writes(2 as Weight))	}	fn set_bridge_token_fee() -> Weight {
		(30_901_000 as Weight)			.saturating_add(T::DbWeight::get().reads(2 as Weight))			.saturating_add(T::DbWeight::get().writes(1 as Weight))	}	fn set_bridge_token_status() -> Weight {
		(30_761_000 as Weight)			.saturating_add(T::DbWeight::get().reads(2 as Weight))			.saturating_add(T::DbWeight::get().writes(1 as Weight))	}	fn set_bridge_token_cap() -> Weight {
		(30_650_000 as Weight)			.saturating_add(T::DbWeight::get().reads(2 as Weight))			.saturating_add(T::DbWeight::get().writes(1 as Weight))	}	fn clean_cap_accumulated_value() -> Weight {
		(30_420_000 as Weight)			.saturating_add(T::DbWeight::get().reads(2 as Weight))			.saturating_add(T::DbWeight::get().writes(1 as Weight))	}	fn teleport() -> Weight {
		(94_592_000 as Weight)			.saturating_add(T::DbWeight::get().reads(4 as Weight))			.saturating_add(T::DbWeight::get().writes(3 as Weight))	}	fn materialize() -> Weight {
		(153_754_000 as Weight)			.saturating_add(T::DbWeight::get().reads(10 as Weight))			.saturating_add(T::DbWeight::get().writes(4 as Weight))	}}

// For backwards compatibility and tests
impl WeightInfo for () {	fn register_chain() -> Weight {
		(24_411_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(1 as Weight))			.saturating_add(RocksDbWeight::get().writes(2 as Weight))	}	fn unregister_chain() -> Weight {
		(25_191_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(1 as Weight))			.saturating_add(RocksDbWeight::get().writes(2 as Weight))	}	fn register_bridge_token() -> Weight {
		(28_960_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(2 as Weight))			.saturating_add(RocksDbWeight::get().writes(2 as Weight))	}	fn unregister_bridge_token() -> Weight {
		(27_210_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(1 as Weight))			.saturating_add(RocksDbWeight::get().writes(2 as Weight))	}	fn set_bridge_token_fee() -> Weight {
		(30_901_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(2 as Weight))			.saturating_add(RocksDbWeight::get().writes(1 as Weight))	}	fn set_bridge_token_status() -> Weight {
		(30_761_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(2 as Weight))			.saturating_add(RocksDbWeight::get().writes(1 as Weight))	}	fn set_bridge_token_cap() -> Weight {
		(30_650_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(2 as Weight))			.saturating_add(RocksDbWeight::get().writes(1 as Weight))	}	fn clean_cap_accumulated_value() -> Weight {
		(30_420_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(2 as Weight))			.saturating_add(RocksDbWeight::get().writes(1 as Weight))	}	fn teleport() -> Weight {
		(94_592_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(4 as Weight))			.saturating_add(RocksDbWeight::get().writes(3 as Weight))	}	fn materialize() -> Weight {
		(153_754_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(10 as Weight))			.saturating_add(RocksDbWeight::get().writes(4 as Weight))	}}
