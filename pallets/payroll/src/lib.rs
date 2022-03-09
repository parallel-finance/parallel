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

//! # Payroll pallet
//!
//! ## Overview
//!
//! This pallet is part of DAOFi modules that provides payroll management

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{log, pallet_prelude::*, transactional, weights::DispatchClass};
use frame_system::pallet_prelude::*;
use orml_oracle::DataProviderExtended;
use orml_traits::DataProvider;
use primitives::*;
use sp_runtime::{
    traits::{CheckedDiv, CheckedMul},
    FixedU128,
};
use sp_std::vec::Vec;

pub use pallet::*;

//#[cfg(test)]
//mod mock;
//#[cfg(test)]
//mod tests;

pub mod weights;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    use weights::WeightInfo;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// Assets for deposit/withdraw collateral assets to/from loans module
        type Assets: Transfer<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
            + Inspect<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
            + Mutate<Self::AccountId, AssetId = CurrencyId, Balance = Balance>;

        /// Decimal provider.
        type Decimal: DecimalProvider<CurrencyId>;

        /// The loan's module id, keep all collaterals of CDPs.
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// Unix time
        type UnixTime: UnixTime;

        /// Weight information
        type WeightInfo: WeightInfo;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Creates a payment stream. \[stream_id, sender, recipient, deposit, currency_id, start_time, stop_time\]
        CreateStream(
            StreamId,
            AccountId,
            AccountId,
            Amount,
            CurrencyId,
            Timestamp,
            Timestamp,
        ),
        /// Withdraw payment from stream. \[stream_id, recipient, amount\]
        WithdrawFromStream(StreamId, AccountId, Amount),
        /// Cancel an existing stream. \[stream_id, sender, recipient, sender_balance, recipient_balance]
        CancelStream(StreamId, AccountId, AccountId, Amount, Amount),
    }

    /// Next Stream Id
    #[pallet::storage]
    #[pallet::getter(fn next_stream)]
    pub type NextStreamId<T: Config> = StorageValue<_, StreamId, ValueQuery>;

    /// Global storage for streams
    #[pallet::storage]
    #[pallet::getter(fn get_stream)]
    pub type Streams<T: Config> = StorageMap<_, Blake2_128Concat, StreamId, Stream, ValueQuery>;

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a stream
        #[pallet::weight((<T as Config>::WeightInfo::create_stream(), DispatchClass::Operational))]
        #[transactional]
        pub fn create_stream(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            T::FeederOrigin::ensure_origin(origin)?;
            Ok(().into())
        }

        /// Cancel an existing stream
        #[pallet::weight((<T as Config>::WeightInfo::cancel_stream(), DispatchClass::Operational))]
        #[transactional]
        pub fn cancel_stream(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            T::FeederOrigin::ensure_origin(origin)?;
            Ok(().into())
        }

        /// Withdraw from an existing stream
        #[pallet::weight((<T as Config>::WeightInfo::withdraw_from_stream(), DispatchClass::Operational))]
        #[transactional]
        pub fn withdraw_from_stream(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            T::FeederOrigin::ensure_origin(origin)?;
            Ok(().into())
        }
    }
}

impl<T: Config> Pallet<T> {}
