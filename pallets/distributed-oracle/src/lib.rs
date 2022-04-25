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

//! # Distributed Oracle pallet
//!
//! ## Overview
//!

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
    pallet_prelude::*,
    traits::{
        tokens::fungibles::{Inspect, Mutate, Transfer},
        UnixTime,
    },
    transactional,
    // weights::DispatchClass,
    PalletId,
};
use frame_system::pallet_prelude::*;
use primitives::*;
use scale_info::TypeInfo;
use sp_runtime::traits::AccountIdConversion;
use sp_std::prelude::*;

pub use pallet::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

// pub mod weights;

// type AssetIdOf<T> =
//     <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;
// type BalanceOf<T> =
//     <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance;
type AccountOf<T> = <T as frame_system::Config>::AccountId;

pub type RelayerId = u128;

// Struct for Relayer
#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct Relayer<T: Config> {
    // Owner
    owner: AccountOf<T>,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    // use weights::WeightInfo;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// Assets for deposit/withdraw collateral assets to/from loans module
        type Assets: Transfer<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
            + Inspect<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
            + Mutate<Self::AccountId, AssetId = CurrencyId, Balance = Balance>;

        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// Unix time
        type UnixTime: UnixTime;

        // /// Weight information
        // type WeightInfo: WeightInfo;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// TODO - need errors
        /// Some error
        MyCustomError,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T: Config> {
        /// New relayer initated
        NewRelayer(RelayerId),
    }

    /// Global storage for relayers
    #[pallet::storage]
    #[pallet::getter(fn get_relayer)]
    pub type Relayers<T: Config> = StorageMap<_, Twox64Concat, RelayerId, Relayer<T>>;

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// TODO - need functions
        #[pallet::weight(1000)]
        #[transactional]
        pub fn create_something(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let _sender = ensure_signed(origin)?;
            if 1 == 0 {
                unimplemented!();
            }
            Ok(().into())
        }
    }
}

impl<T: Config> Pallet<T> {
    pub fn account_id() -> AccountOf<T> {
        T::PalletId::get().into_account()
    }
}
