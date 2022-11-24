
//! Autogenerated weights for `pallet_bridge`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-05-30, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("kerria-dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/parallel
// benchmark
// pallet
// --chain=kerria-dev
// --execution=wasm
// --wasm-execution=compiled
// --pallet=pallet_bridge
// --extrinsic=*
// --steps=50
// --repeat=20
// --output=./runtime/kerria/src/weights/pallet_bridge.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_bridge`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_bridge::WeightInfo for WeightInfo<T> {
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: Bridge ChainNonces (r:1 w:1)
	// Storage: Bridge BridgeRegistry (r:0 w:1)
	fn register_chain() -> Weight {
		(44_238_000 as u64)
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: Bridge ChainNonces (r:1 w:1)
	// Storage: Bridge BridgeRegistry (r:0 w:1)
	fn unregister_chain() -> Weight {
		(43_179_000 as u64)
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: Bridge BridgeTokens (r:1 w:1)
	// Storage: Bridge AssetIds (r:1 w:1)
	fn register_bridge_token() -> Weight {
		(49_245_000 as u64)
			.saturating_add(T::DbWeight::get().reads(3 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: Bridge AssetIds (r:1 w:1)
	// Storage: Bridge BridgeTokens (r:0 w:1)
	fn unregister_bridge_token() -> Weight {
		(46_367_000 as u64)
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: Bridge AssetIds (r:1 w:0)
	// Storage: Bridge BridgeTokens (r:1 w:1)
	fn set_bridge_token_fee() -> Weight {
		(52_603_000 as u64)
			.saturating_add(T::DbWeight::get().reads(3 as u64))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: Bridge AssetIds (r:1 w:0)
	// Storage: Bridge BridgeTokens (r:1 w:1)
	fn set_bridge_token_status() -> Weight {
		(51_621_000 as u64)
			.saturating_add(T::DbWeight::get().reads(3 as u64))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: Bridge AssetIds (r:1 w:0)
	// Storage: Bridge BridgeTokens (r:1 w:1)
	fn set_bridge_token_cap() -> Weight {
		(52_616_000 as u64)
			.saturating_add(T::DbWeight::get().reads(3 as u64))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: Bridge AssetIds (r:1 w:0)
	// Storage: Bridge BridgeTokens (r:1 w:1)
	fn clean_cap_accumulated_value() -> Weight {
		(51_728_000 as u64)
			.saturating_add(T::DbWeight::get().reads(3 as u64))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: Bridge ChainNonces (r:1 w:1)
	// Storage: Bridge AssetIds (r:1 w:0)
	// Storage: Bridge BridgeTokens (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	fn teleport() -> Weight {
		(130_108_000 as u64)
			.saturating_add(T::DbWeight::get().reads(5 as u64))
			.saturating_add(T::DbWeight::get().writes(4 as u64))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: Bridge ChainNonces (r:1 w:0)
	// Storage: Bridge BridgeRegistry (r:1 w:1)
	// Storage: Bridge AssetIds (r:1 w:0)
	// Storage: Bridge BridgeTokens (r:1 w:1)
	// Storage: Bridge ProposalVotes (r:1 w:1)
	// Storage: Bridge VoteThreshold (r:1 w:0)
	// Storage: BridgeMembership Members (r:1 w:0)
	// Storage: System Account (r:2 w:1)
	// Storage: Assets Metadata (r:1 w:0)
	fn materialize() -> Weight {
		(211_212_000 as u64)
			.saturating_add(T::DbWeight::get().reads(11 as u64))
			.saturating_add(T::DbWeight::get().writes(5 as u64))
	}
}
