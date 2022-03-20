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

use super::*;

pub mod v2 {
    use super::*;
    use crate::{Config, StorageVersion, Weight};
    use frame_support::{log, traits::Get};

    #[cfg(feature = "try-runtime")]
    pub fn pre_migrate<T: Config>() -> Result<(), &'static str> {
        frame_support::ensure!(
            StorageVersion::<T>::get() == crate::Versions::V1,
            "must upgrade linearly"
        );
        log::info!("👜 loans borrow-limit migration passes PRE migrate checks ✅",);
        Ok(())
    }

    /// Migration to sorted [`SortedListProvider`].
    pub fn migrate<T: Config>() -> Weight {
        if StorageVersion::<T>::get() == crate::Versions::V1 {
            log::info!("migrating loans to Versions::V2",);

            Markets::<T>::translate::<Market<BalanceOf<T>>, _>(|_key, market| {
                Some(Market {
                    borrow_limit: 1_000_000_000_000_000u128,
                    ..market
                })
            });

            StorageVersion::<T>::put(crate::Versions::V2);
            log::info!("👜 completed loans migration to Versions::V2",);

            T::BlockWeights::get().max_block
        } else {
            T::DbWeight::get().reads(1)
        }
    }

    #[cfg(feature = "try-runtime")]
    pub fn post_migrate<T: Config>() -> Result<(), &'static str> {
        frame_support::ensure!(
            StorageVersion::<T>::get() == crate::Versions::V2,
            "must upgrade to V2"
        );
        log::info!("👜 loans borrow-limit migration passes POST migrate checks ✅",);
        Ok(())
    }
}
