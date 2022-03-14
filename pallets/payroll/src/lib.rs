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

use frame_support::{
    log,
    pallet_prelude::*,
    traits::{
        tokens::fungibles::{Inspect, Mutate, Transfer},
        UnixTime,
    },
    transactional,
    weights::DispatchClass,
    PalletId,
};
use codec::{Encode, Decode};
use sp_std::{fmt::Debug, prelude::*};
use frame_system::pallet_prelude::*;
use primitives::*;
use sp_runtime::{
    traits::{
        AccountIdConversion, CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, One, StaticLookup,
        Zero,
    },
    ArithmeticError, FixedPointNumber, FixedU128,
};

pub use pallet::*;

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Kitty<Hash, Balance> {
    id: Hash,
    dna: Hash,
    price: Balance,
    gen: u64,
}

//#[cfg(test)]
//mod mock;
//#[cfg(test)]
//mod tests;

pub mod weights;

type AssetIdOf<T> =
    <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;
type BalanceOf<T> =
    <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

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

        /// The payroll module id, keep all collaterals of CDPs.
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// Unix time
        type UnixTime: UnixTime;

        /// Weight information
        type WeightInfo: WeightInfo;

    }

    #[pallet::error]
    pub enum Error<T> {
        StreamToOrigin,
        DepositIsZero,
        StartBeforeBlockTime,
        StopBeforeStart,
        NotTheStreamer,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Creates a payment stream. \[stream_id, sender, recipient, deposit, currency_id, start_time, stop_time\]
        CreateStream(
            StreamId,
            T::AccountId,
            T::AccountId,
            BalanceOf<T>,
            AssetIdOf<T>,
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
        pub fn create_stream(
            origin: OriginFor<T>,
            recipient: T::AccountId,
            deposit: BalanceOf<T>,
            currency: AssetIdOf<T>,
            rate_per_sec: BalanceOf<T>,
            start_time: Timestamp,
            stop_time: Timestamp,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            ensure!(recipient != sender, Error::<T>::StreamToOrigin);
            ensure!(deposit > 0, Error::<T>::DepositIsZero);
            ensure!(
                start_time >= T::UnixTime::now().as_secs(),
                Error::<T>::StartBeforeBlockTime
            );
            ensure!(stop_time > start_time, Error::<T>::StopBeforeStart);
            // insert stream to the Streams
            let stream: Stream = (
                deposit,      // remaining balance same value for now due to initialization
                deposit,      // deposit
                currency,     // currency id
                rate_per_sec, // rate per second
                recipient,    // recipient
                sender,       // sender
                start_time,   // start_time
                stop_time,    // stop_time
            );

            Streams::<T>::insert(NextStreamId::get(), stream);
            Ok(().into())
        }

        /// Cancel an existing stream
        #[pallet::weight((<T as Config>::WeightInfo::cancel_stream(), DispatchClass::Operational))]
        #[transactional]
        pub fn cancel_stream(
            origin: OriginFor<T>,
            stream_id: StreamId,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            // check sender is stream sender
            let stream = Streams::<T>::get(stream_id);
            ensure!(sender == stream.5.into_account(), Error::<T>::NotTheStreamer);
            // send funds back to sender
            T::Assets::transfer(stream.2, &Self::account_id(), &sender, stream.0, false)?;
            Streams::<T>::remove(stream_id);
            Ok(().into())
        }

        /// Withdraw from an existing stream
        #[pallet::weight((<T as Config>::WeightInfo::withdraw_from_stream(), DispatchClass::Operational))]
        #[transactional]
        pub fn withdraw_from_stream(
            origin: OriginFor<T>,
            stream_id: StreamId,
            amount: Amount,
        ) -> DispatchResultWithPostInfo {
            Ok(().into())
        }
    }
}

impl<T: Config> Pallet<T> {
    pub fn account_id() -> T::AccountId {
        T::PalletId::get().into_account()
    }
}
