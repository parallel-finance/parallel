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
use frame_support::traits::Get;
use primitives::ParaId;
use sp_runtime::traits::Zero;
use sp_std::{vec, vec::Vec};
use types::*;
/// Add vaults for batch 1 winning projects
pub fn migrate<T: Config>() -> frame_support::weights::Weight {
    // paraId, ctoken, raised, cap, end_block, trie_index, lease_start, lease_end
    // FIXME: contributed is not the raised
    let batch: Vec<(u32, u32, u128, u128, u32, u32, u32, u32)> = vec![
        // Acala
        (
            2000,
            4000,
            325_159_802_323_576_263,
            500_000_000_000_000_000,
            8179200,
            0,
            6,
            13,
        ),
        // Clover
        (
            2002,
            4000,
            97_524_874_268_038_525,
            500_000_000_000_000_000,
            8179200,
            1,
            6,
            13,
        ),
        // Moonbeam
        (
            2004,
            4000,
            357_599_313_927_924_796,
            1_000_000_000_000_000_000,
            8179199,
            2,
            6,
            13,
        ),
        // Astar
        (
            2006,
            4000,
            103_335_520_433_166_970,
            350_000_010_000_000_000,
            8179200,
            3,
            6,
            13,
        ),
        // Parallel
        (
            2012,
            4000,
            107_515_186_195_417_478,
            400_000_000_000_000_000,
            8179200,
            4,
            6,
            13,
        ),
        // Efinity
        (
            2021,
            4001,
            76_953_774_505_455_550,
            500_000_000_000_000_000,
            9388800,
            5,
            7,
            14,
        ),
    ];
    let length = batch.len() as u64;
    for (para_id, ctoken, raised, cap, end_block, trie_index, lease_start, lease_end) in
        batch.into_iter()
    {
        let vault = Vault::<T> {
            ctoken,
            phase: VaultPhase::Succeeded,
            contributed: raised,
            pending: Zero::zero(),
            flying: Zero::zero(),
            contribution_strategy: ContributionStrategy::XCM,
            cap,
            end_block: end_block.into(),
            trie_index,
            lease_start,
            lease_end,
        };

        Vaults::<T>::insert((&ParaId::from(para_id), &lease_start, &lease_end), vault);
        CTokensRegistry::<T>::insert((&lease_start, &lease_end), ctoken);
        LeasesRegistry::<T>::insert(&ParaId::from(para_id), (lease_start, lease_end));
    }
    NextTrieIndex::<T>::put(6);
    <T as frame_system::Config>::DbWeight::get().writes(length * 3 + 1u64)
}
