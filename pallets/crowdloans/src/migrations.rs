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

pub mod v1 {
    use super::*;
    use frame_support::{log, traits::Get};
    use primitives::ParaId;
    use sp_runtime::traits::Zero;
    use sp_std::{vec, vec::Vec};
    use types::*;
    /// Add vaults for batch 1 winning projects
    pub fn migrate<T: Config>() -> frame_support::weights::Weight {
        if StorageVersion::<T>::get() == Releases::V0_0_0 {
            log::info!(
                target: "crowdloans::migrate",
                "migrating crowdloan to Releases::V1_0_0"
            );
            // paraId, ctoken, contributed, cap, end_block, trie_index, lease_start, lease_end
            let batch: Vec<(u32, u32, u128, u128, u32, u32, u32, u32)> = vec![
                // Acala
                // 1,441,645.1500372255
                (
                    2000,
                    200060013,
                    14_416_451_500_372_255,
                    500_000_000_000_000_000,
                    8179200,
                    0,
                    6,
                    13,
                ),
                // Clover
                // 3,952,961.0099297280
                (
                    2002,
                    200060013,
                    39_529_610_099_297_280,
                    500_000_000_000_000_000,
                    8179200,
                    1,
                    6,
                    13,
                ),
                // Moonbeam
                // 3,470,561.7504208070
                (
                    2004,
                    200060013,
                    34_705_617_504_208_070,
                    1_000_000_000_000_000_000,
                    8179199,
                    2,
                    6,
                    13,
                ),
                // Astar
                // 1,790,762.0716266251
                (
                    2006,
                    200060013,
                    17_907_620_716_266_251,
                    350_000_010_000_000_000,
                    8179200,
                    3,
                    6,
                    13,
                ),
                // Parallel
                (
                    2012,
                    200060013,
                    85_381_150_820_717_022,
                    400_000_000_000_000_000,
                    8179200,
                    4,
                    6,
                    13,
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
            NextTrieIndex::<T>::put(5);
            StorageVersion::<T>::put(Releases::V1_0_0);
            log::info!(
                target: "crowdloans::migrate",
                "completed crowdloans migration to Releases::V1_0_0"
            );
            <T as frame_system::Config>::DbWeight::get().writes(length * 3 + 1u64)
        } else {
            T::DbWeight::get().reads(1)
        }
    }
}
