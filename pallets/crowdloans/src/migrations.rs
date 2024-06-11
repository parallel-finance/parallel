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

    pub fn pre_migrate<T: Config>() -> Result<(), &'static str> {
        frame_support::ensure!(
            StorageVersion::<T>::get() == Releases::V0_0_0,
            "must be V0_0_0"
        );
        frame_support::ensure!(NextTrieIndex::<T>::get() == 10, "must be 10");
        Ok(())
    }

    /// Add vaults for batch 1 winning projects
    pub fn migrate<T: Config>() -> frame_support::weights::Weight {
        if StorageVersion::<T>::get() == Releases::V0_0_0 {
            log::info!(
                target: "crowdloans::migrate",
                "migrating crowdloan to Releases::V1_0_0"
            );
            let next_trie_index: u32 = NextTrieIndex::<T>::get();
            // paraId, ctoken, contributed, cap, end_block, trie_index, lease_start, lease_end
            let batch: Vec<(u32, u32, u128, u128, u32, u32, u32, u32)> = vec![
                // Parallel Heiko
                // 57,307,000,000,000,000
                (
                    2085,
                    100150022,
                    57_307_000_000_000_000,
                    400_000_000_000_000_000,
                    9676800,
                    next_trie_index,
                    15,
                    22,
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
                LeasesRegistry::<T>::insert(ParaId::from(para_id), (lease_start, lease_end));
            }
            NextTrieIndex::<T>::put(next_trie_index + 1);
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

    pub fn post_migrate<T: Config>() -> Result<(), &'static str> {
        frame_support::ensure!(
            StorageVersion::<T>::get() == Releases::V1_0_0,
            "must be V1_0_0"
        );
        frame_support::ensure!(NextTrieIndex::<T>::get() == 11, "must be 11");
        log::info!("👜 crowdloan migration passes POST migrate checks ✅",);

        Ok(())
    }
}

pub mod v2 {
    use super::*;
    use frame_support::{log, traits::Get};
    use primitives::ParaId;
    use sp_runtime::traits::Zero;
    use sp_std::{vec, vec::Vec};
    use types::*;

    pub fn pre_migrate<T: Config>() -> Result<(), &'static str> {
        frame_support::ensure!(
            StorageVersion::<T>::get() == Releases::V1_0_0,
            "must be V1_0_0"
        );
        frame_support::ensure!(NextTrieIndex::<T>::get() == 9, "must be 9");
        Ok(())
    }

    pub fn migrate<T: Config>() -> frame_support::weights::Weight {
        if StorageVersion::<T>::get() == Releases::V1_0_0 {
            log::info!(
                target: "crowdloans::migrate",
                "migrating crowdloan to Releases::V2_0_0"
            );
            // paraId, ctoken, contributed, cap, end_block, trie_index, lease_start, lease_end
            let batch: Vec<(u32, u32, u128, u128, u32, u32, u32, u32)> = vec![
                // Nodle
                // 17574741856000000
                (
                    2026,
                    200070014,
                    17_574_741_856_000_000,
                    250_000_000_000_000_000,
                    9_388_800,
                    9,
                    7,
                    14,
                ),
                // Interlay
                // 4898587497400000
                (
                    2032,
                    200070014,
                    4898587497400000,
                    500_000_000_000_000_000,
                    9_388_800,
                    10,
                    7,
                    14,
                ),
                // Equilibrium
                // 484572193000000
                (
                    2011,
                    200070014,
                    4898587497400000,
                    30_000_000_000_000_000,
                    9671800,
                    11,
                    7,
                    14,
                ),
                // Phala Network
                // 3071389985000000
                (
                    2035,
                    200070014,
                    3071389985000000,
                    30_000_000_000_000_000,
                    9671800,
                    12,
                    7,
                    14,
                ),
                // Unique Network
                // 2769629790000000
                (
                    2037,
                    200080015,
                    2769629790000000,
                    150_000_000_000_000_000,
                    10881401,
                    13,
                    8,
                    15,
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
                LeasesRegistry::<T>::insert(ParaId::from(para_id), (lease_start, lease_end));
            }
            NextTrieIndex::<T>::put(14);
            StorageVersion::<T>::put(Releases::V2_0_0);
            log::info!(
                target: "crowdloans::migrate",
                "completed crowdloans migration to Releases::V2_0_0"
            );
            <T as frame_system::Config>::DbWeight::get().writes(length * 3 + 1u64)
        } else {
            T::DbWeight::get().reads(1)
        }
    }

    pub fn post_migrate<T: Config>() -> Result<(), &'static str> {
        frame_support::ensure!(
            StorageVersion::<T>::get() == Releases::V2_0_0,
            "must be V2_0_0"
        );
        frame_support::ensure!(NextTrieIndex::<T>::get() == 14, "must be 14");
        log::info!("👜 crowdloan migration passes POST migrate checks ✅",);

        Ok(())
    }
}

pub mod v3 {
    use super::*;
    use frame_support::{log, traits::Get};
    use primitives::ParaId;
    use sp_runtime::traits::Zero;
    use sp_std::{vec, vec::Vec};
    use types::*;

    pub fn pre_migrate<T: Config>() -> Result<Vec<u8>, &'static str> {
        frame_support::ensure!(
            StorageVersion::<T>::get() == Releases::V2_0_0,
            "must be V2_0_0"
        );
        frame_support::ensure!(NextTrieIndex::<T>::get() == 24, "must be 24");
        Ok(Vec::new())
    }

    pub fn migrate<T: Config>() -> frame_support::weights::Weight {
        if StorageVersion::<T>::get() == Releases::V2_0_0 {
            log::info!(
                target: "crowdloans::migrate",
                "migrating crowdloan to Releases::V3_0_0"
            );
            // paraId, ctoken, contributed, cap, end_block, trie_index, lease_start, lease_end
            let batch: Vec<(u32, u32, u128, u128, u32, u32, u32, u32)> = vec![
                // 2040,8-15,5058525762400000,150000000000000000,10881401
                // Polkadex
                // 5058525762400000
                (
                    2040,
                    200080015,
                    5058525762400000,
                    150_000_000_000_000_000,
                    10881401,
                    25,
                    8,
                    15,
                ),
                // 2013,8-15,1891535952620463,150000000000000000,10881401
                // Litentry
                // 1891535952620463
                (
                    2013,
                    200080015,
                    1891535952620463,
                    150_000_000_000_000_000,
                    10881401,
                    26,
                    8,
                    15,
                ),
                // 2030,8-15,222383369000000,150000000000000000,10881401
                // Bifrost
                // 222383369000000
                (
                    2030,
                    200080015,
                    222383369000000,
                    150_000_000_000_000_000,
                    10881401,
                    27,
                    8,
                    15,
                ),
                // 2027,8-15,574831000000000,150000000000000000,10881401
                // Coinversation
                // 574831000000000
                (
                    2027,
                    200080015,
                    574831000000000,
                    150_000_000_000_000_000,
                    10881401,
                    28,
                    8,
                    15,
                ),
                // 2043,8-15,2599910262000000,150000000000000000,10881401
                // NeuroWeb
                // 2599910262000000
                (
                    2043,
                    200080015,
                    2599910262000000,
                    150_000_000_000_000_000,
                    10881401,
                    29,
                    8,
                    15,
                ),
                // 2007,8-15,320506980000000,150000000000000000,10881401
                // Kapex
                // 320506980000000
                (
                    2007,
                    200080015,
                    320506980000000,
                    150_000_000_000_000_000,
                    10881401,
                    30,
                    8,
                    15,
                ),
            ];
            let length = batch.len() as u64;
            for (para_id, ctoken, raised, cap, end_block, trie_index, lease_start, lease_end) in
                batch.into_iter()
            {
                let vault = Vault::<T> {
                    ctoken,
                    phase: VaultPhase::Expired,
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
                LeasesRegistry::<T>::insert(ParaId::from(para_id), (lease_start, lease_end));
            }
            NextTrieIndex::<T>::put(31);
            StorageVersion::<T>::put(Releases::V3_0_0);
            log::info!(
                target: "crowdloans::migrate",
                "completed crowdloans migration to Releases::V3_0_0"
            );
            <T as frame_system::Config>::DbWeight::get().writes(length * 3 + 1u64)
        } else {
            T::DbWeight::get().reads(1)
        }
    }

    pub fn post_migrate<T: Config>() -> Result<(), &'static str> {
        frame_support::ensure!(
            StorageVersion::<T>::get() == Releases::V3_0_0,
            "must be V3_0_0"
        );
        frame_support::ensure!(NextTrieIndex::<T>::get() == 31, "must be 31");
        log::info!("👜 crowdloan migration passes POST migrate checks ✅",);

        Ok(())
    }
}

pub mod v4 {
    use super::*;
    use frame_support::{log, traits::Get};
    use primitives::ParaId;
    use sp_std::{vec, vec::Vec};
    use types::*;

    pub fn pre_migrate<T: Config>() -> Result<Vec<u8>, &'static str> {
        frame_support::ensure!(
            StorageVersion::<T>::get() == Releases::V3_0_0,
            "must be V3_0_0"
        );
        frame_support::ensure!(NextTrieIndex::<T>::get() == 31, "must be 31");
        Ok(Vec::new())
    }

    pub fn migrate<T: Config>() -> frame_support::weights::Weight {
        if StorageVersion::<T>::get() == Releases::V3_0_0 {
            log::info!(
                target: "crowdloans::migrate",
                "migrating crowdloan storage"
            );
            // paraId, ctoken, contributed, cap, end_block, trie_index, lease_start, lease_end
            let batch: Vec<(u32, u32, u128, u128, u32, u32, u32, u32)> = vec![
                // 2040,8-15,3000000000000000,150000000000000000,10881401
                (
                    2040,
                    200080015,
                    3000000000000000,
                    150_000_000_000_000_000,
                    10881401,
                    25,
                    8,
                    15,
                ),
            ];
            let length = batch.len() as u64;
            for (para_id, _, raised, _, _, _, lease_start, lease_end) in batch.into_iter() {
                match Vaults::<T>::get((&ParaId::from(para_id), &lease_start, &lease_end)) {
                    Some(vault) if vault.phase == VaultPhase::Expired => {
                        Vaults::<T>::insert(
                            (&ParaId::from(para_id), &lease_start, &lease_end),
                            Vault {
                                contributed: raised,
                                ..vault
                            },
                        );
                    }
                    Some(_) => {
                        log::error!("Vault for para_id {} is not in Expired phase", para_id);
                    }
                    None => {
                        log::error!(
                            "No vault found for para_id {} ({}, {})",
                            para_id,
                            lease_start,
                            lease_end
                        );
                    }
                }
            }
            log::info!(
                target: "crowdloans::migrate",
                "completed crowdloans storage migration"
            );
            <T as frame_system::Config>::DbWeight::get().writes(length * 3 + 1u64)
        } else {
            T::DbWeight::get().reads(1)
        }
    }

    pub fn post_migrate<T: Config>() -> Result<(), &'static str> {
        frame_support::ensure!(
            StorageVersion::<T>::get() == Releases::V3_0_0,
            "must be V3_0_0"
        );
        frame_support::ensure!(NextTrieIndex::<T>::get() == 31, "must be 31");
        log::info!("👜 crowdloan migration passes POST migrate checks ✅",);

        Ok(())
    }
}
