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

//! # Add vaults for batch 1 winning projects

use super::*;
/// Add vaults for batch 1 winning projects
pub fn migrate<T: Config>() -> frame_support::weights::Weight {
    // if StorageVersion::<T>::get() == crate::Releases::V7_0_0 {
    // 	crate::log!(info, "migrating staking to Releases::V8_0_0");

    // 	let migrated = T::SortedListProvider::unsafe_regenerate(
    // 		Nominators::<T>::iter().map(|(id, _)| id),
    // 		Pallet::<T>::weight_of_fn(),
    // 	);
    // 	debug_assert_eq!(T::SortedListProvider::sanity_check(), Ok(()));

    // 	StorageVersion::<T>::put(crate::Releases::V8_0_0);
    // 	crate::log!(
    // 		info,
    // 		"👜 completed staking migration to Releases::V8_0_0 with {} voters migrated",
    // 		migrated,
    // 	);

    // 	T::BlockWeights::get().max_block
    // } else {
    // 	T::DbWeight::get().reads(1)
    // }
    //TODO: https://github.com/parallel-finance/parallel/issues/1086#issuecomment-1004561593
    // Only works for polkadot network

    // paraId, ctoken, raised,cap,end_block,trie_index,lease_start,lease_end
    // FIXME: contributed is not the raised
    let batch = vec![
        // Acala
        (
            2000,
            4000,
            "325,159,802,323,576,263",
            "500_000_000_000_000_000",
            "8179200",
            0,
            6,
            13,
        ),
        // Clover
        (
            2002,
            4000,
            "97,524,874,268,038,525",
            "500,000,000,000,000,000",
            "8,179,200",
            1,
            6,
            13,
        ),
        // Moonbeam
        (
            2004,
            4000,
            "357,599,313,927,924,796",
            "1,000,000,000,000,000,000",
            "8,179,199",
            2,
            6,
            13,
        ),
        // Astar
        (
            2006,
            4000,
            "103,335,520,433,166,970",
            "350,000,010,000,000,000",
            "8,179,200",
            3,
            6,
            13,
        ),
        // Parallel
        (
            2012,
            4000,
            "107,515,186,195,417,478",
            "400,000,000,000,000,000",
            "8,179,200",
            4,
            6,
            13,
        ),
        // Efinity
        (
            2021,
            4001,
            "76,953,774,505,455,550",
            "500,000,000,000,000,000",
            "9,388,800",
            5,
            7,
            14,
        ),
    ];
    0
}
