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

//! # Asset Registry pallet
//!
//! ## Overview
//! This pallet should be in charge of registering assets.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

pub use pallet::*;

use frame_support::{dispatch::DispatchResult, pallet_prelude::*, transactional};
use primitives::ForeignAssetId;
use sp_std::{boxed::Box, convert::TryInto};

use xcm::{v1::MultiLocation, VersionedMultiLocation};

#[frame_support::pallet]
pub mod pallet {
    use frame_system::pallet_prelude::OriginFor;

    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The origin which can register asset
        type RegisterOrigin: EnsureOrigin<<Self as frame_system::Config>::Origin>;

        /// The origin which can update asset
        type UpdateOrigin: EnsureOrigin<<Self as frame_system::Config>::Origin>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Asset Has Been Registered
        AssetRegistered(ForeignAssetId),
        /// Asset Has Been Updated
        AssetUpdated,
    }

    #[pallet::storage]
    #[pallet::getter(fn asset_multi_locations)]
    pub type AssetMultiLocations<T: Config> =
        StorageMap<_, Twox64Concat, ForeignAssetId, MultiLocation, OptionQuery>;

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::error]
    pub enum Error<T> {
        /// Asset already exists
        AssetAlreadyExists,
        /// Bad Location
        BadLocation,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Update xcm fees amount to be used in xcm.Withdraw message
        #[pallet::weight(10_000)]
        #[transactional]
        pub fn register_asset(
            origin: OriginFor<T>,
            asset_id: ForeignAssetId,
            location: Box<VersionedMultiLocation>,
        ) -> DispatchResult {
            T::RegisterOrigin::ensure_origin(origin)?;

            let location: MultiLocation = (*location)
                .try_into()
                .map_err(|()| Error::<T>::BadLocation)?;

            Self::do_register_asset(asset_id, location)?;
            Self::deposit_event(Event::<T>::AssetRegistered(asset_id));
            Ok(())
        }
    }
}

impl<T: Config> Pallet<T> {
    fn do_register_asset(
        asset_id: ForeignAssetId,
        location: MultiLocation,
    ) -> Result<(), DispatchError> {
        AssetMultiLocations::<T>::try_mutate(asset_id, |maybe_location| -> DispatchResult {
            ensure!(maybe_location.is_none(), Error::<T>::AssetAlreadyExists);
            *maybe_location = Some(location);
            Ok(())
        })?;

        Ok(())
    }
}
