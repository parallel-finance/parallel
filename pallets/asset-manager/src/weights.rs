// Copyright 2021 Parallel Finance Developer.
// This file is part of Parallel Finance.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
// http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Autogenerated weights for pallet_asset_manager
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-02-22, STEPS: `32`, REPEAT: 64, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/parallel
// benchmark
// --chain=parallel
// --execution=wasm
// --wasm-execution=compiled
// --pallet
// pallet_asset_manager
// --extrinsic
// *
// --steps
// 32
// --repeat
// 64
// --raw
// --template=./.maintain/frame-weight-template.hbs
// --output
// /tmp/
// --record-proof

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{
	traits::Get,
	weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_asset_manager.
pub trait WeightInfo {
	fn register_asset() -> Weight;
	fn set_asset_units_per_second() -> Weight;
	fn change_existing_asset_type() -> Weight;
	fn remove_supported_asset() -> Weight;
	fn remove_existing_asset_type() -> Weight;
}

/// Weights for pallet_asset_manager using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	fn register_asset() -> Weight {
		(52_375_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(7 as Weight))
			.saturating_add(T::DbWeight::get().writes(6 as Weight))
	}
	fn set_asset_units_per_second() -> Weight {
		(31_655_000 as Weight) // Standard Error: 4_000
			.saturating_add(T::DbWeight::get().reads(6 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
	}
	fn change_existing_asset_type() -> Weight {
		(39_476_000 as Weight) // Standard Error: 4_000
			.saturating_add(T::DbWeight::get().reads(7 as Weight))
			.saturating_add(T::DbWeight::get().writes(8 as Weight))
	}
	fn remove_supported_asset() -> Weight {
		(24_269_000 as Weight) // Standard Error: 4_000
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
	}
	fn remove_existing_asset_type() -> Weight {
		(30_428_000 as Weight) // Standard Error: 3_000
			.saturating_add(T::DbWeight::get().reads(6 as Weight))
			.saturating_add(T::DbWeight::get().writes(6 as Weight))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	fn register_asset() -> Weight {
		(52_375_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(7 as Weight))
			.saturating_add(RocksDbWeight::get().writes(6 as Weight))
	}
	fn set_asset_units_per_second() -> Weight {
		(31_655_000 as Weight) // Standard Error: 4_000
			.saturating_add(RocksDbWeight::get().reads(6 as Weight))
			.saturating_add(RocksDbWeight::get().writes(4 as Weight))
	}
	fn change_existing_asset_type() -> Weight {
		(39_476_000 as Weight) // Standard Error: 4_000
			.saturating_add(RocksDbWeight::get().reads(7 as Weight))
			.saturating_add(RocksDbWeight::get().writes(8 as Weight))
	}
	fn remove_supported_asset() -> Weight {
		(24_269_000 as Weight) // Standard Error: 4_000
			.saturating_add(RocksDbWeight::get().reads(5 as Weight))
			.saturating_add(RocksDbWeight::get().writes(4 as Weight))
	}
	fn remove_existing_asset_type() -> Weight {
		(30_428_000 as Weight) // Standard Error: 3_000
			.saturating_add(RocksDbWeight::get().reads(6 as Weight))
			.saturating_add(RocksDbWeight::get().writes(6 as Weight))
	}
}
