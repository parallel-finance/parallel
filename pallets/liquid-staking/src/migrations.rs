// Copyright 2021-2022 Parallel Finance Developer.
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

pub mod v3 {
    use crate::{
        types::{MatchingLedger, ReservableAmount},
        BalanceOf, Config, MatchingPool, StorageVersion,
    };
    use frame_support::pallet_prelude::*;
    use frame_support::{
        generate_storage_alias, log,
        traits::{tokens::Balance as BalanceT, Get},
        weights::Weight,
    };

    generate_storage_alias!(LiquidStaking, MarketCap => Value<u128,ValueQuery>);

    #[derive(Copy, Clone, Eq, PartialEq, Default, Encode, Decode, RuntimeDebug, TypeInfo)]
    pub struct OldMatchingLedger<Balance: BalanceT> {
        /// The total stake amount in one era
        pub total_stake_amount: Balance,
        /// The total unstake amount in one era
        pub total_unstake_amount: Balance,
    }

    #[cfg(feature = "try-runtime")]
    pub fn pre_migrate<T: Config>() -> Result<(), &'static str> {
        generate_storage_alias!(LiquidStaking, MatchingPool => Value<OldMatchingLedger<u128>,ValueQuery>);
        let matching_ledger = MatchingPool::get();
        log::info!(
            "MatchingLedger total_stake_amount: {:?}, total_unstake_amount: {:?}",
            matching_ledger.total_stake_amount,
            matching_ledger.total_unstake_amount
        );

        log::info!("MarketCap.get()? {:?}", MarketCap::get());
        assert!(MarketCap::exists(), "MarketCap storage item not found!");
        Ok(())
    }

    pub fn migrate<T: Config>() -> Weight {
        if StorageVersion::<T>::get() == crate::Versions::V2 {
            log::info!("Migrating liquidStaking to Versions::V3",);
            // 1.Clear MarketCap, now use StakingLedgerCap
            MarketCap::kill();

            // 2.Update MatchingPool, MatchingLedger
            let r = MatchingPool::<T>::translate::<OldMatchingLedger<BalanceOf<T>>, _>(
                |matching_ledger| {
                    let new_matching_ledger = MatchingLedger {
                        total_stake_amount: ReservableAmount {
                            total: matching_ledger.unwrap_or_default().total_stake_amount,
                            reserved: Default::default(),
                        },
                        total_unstake_amount: ReservableAmount {
                            total: matching_ledger.unwrap_or_default().total_unstake_amount,
                            reserved: Default::default(),
                        },
                    };
                    Some(new_matching_ledger)
                },
            );
            log::info!("result:{:?}", r);

            StorageVersion::<T>::put(crate::Versions::V3);
            log::info!("👜 completed liquidStaking migration to Versions::V3",);

            T::BlockWeights::get().max_block
        } else {
            T::DbWeight::get().reads(1)
        }
    }

    #[cfg(feature = "try-runtime")]
    pub fn post_migrate<T: Config>() -> Result<(), &'static str> {
        frame_support::ensure!(
            StorageVersion::<T>::get() == crate::Versions::V3,
            "must upgrade to V3"
        );
        log::info!("MarketCap.get()? {:?}", MarketCap::get());
        assert!(!MarketCap::exists(), "MarketCap storage item found!");

        let matching_ledger = MatchingPool::<T>::get();
        log::info!("MatchingLedger");
        log::info!(
            "total_stake_amount.total: {:?}, total_unstake_amount.total: {:?}",
            matching_ledger.total_stake_amount.total,
            matching_ledger.total_unstake_amount.total
        );
        log::info!(
            "total_stake_amount.reserved: {:?}, total_unstake_amount.reserved: {:?}",
            matching_ledger.total_stake_amount.reserved,
            matching_ledger.total_unstake_amount.reserved
        );
        log::info!("👜 liquidStaking migration passes POST migrate checks ✅",);

        Ok(())
    }
}
