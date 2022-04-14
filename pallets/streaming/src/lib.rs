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

//! # Stream pallet
//!
//! ## Overview
//!
//! This pallet is part of DAOFi modules that provides payroll streaming management

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
    pallet_prelude::*,
    traits::{
        tokens::fungibles::{Inspect, Mutate, Transfer},
        UnixTime,
    },
    transactional,
    weights::DispatchClass,
    PalletId,
};
use frame_system::pallet_prelude::*;
use primitives::*;
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{AccountIdConversion, Zero},
    ArithmeticError, DispatchError,
};
use sp_std::prelude::*;

pub use pallet::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub mod weights;

type AssetIdOf<T> =
    <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;
type BalanceOf<T> =
    <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance;
type AccountOf<T> = <T as frame_system::Config>::AccountId;

// Struct for Stream
#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct Stream<T: Config> {
    // Remaining Balance
    remaining_balance: BalanceOf<T>,
    // Deposit
    deposit: BalanceOf<T>,
    // Currency Id
    currency_id: AssetIdOf<T>,
    // Rate Per Second
    rate_per_sec: BalanceOf<T>,
    // Recipient
    recipient: AccountOf<T>,
    // Sender
    sender: AccountOf<T>,
    // Start Time
    start_time: Timestamp,
    // Stop Time
    stop_time: Timestamp,
}

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

        /// The streaming module id, keep all collaterals of CDPs.
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// Unix time
        type UnixTime: UnixTime;

        /// Weight information
        type WeightInfo: WeightInfo;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Sender as specified themselves as the recipient
        RecipientIsAlsoSender,
        /// Insufficient deposit size
        DepositIsZero,
        /// Start time is before current block time
        StartBeforeBlockTime,
        /// Stop time is before start time
        StopBeforeStart,
        /// Caller is not the streamer
        NotTheStreamer,
        /// Caller is not the recipient
        NotTheRecipient,
        /// Amount exceeds balance
        InsufficientBalance,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Creates a payment stream. \[stream_id, sender, recipient, deposit, currency_id, start_time, stop_time\]
        StreamCreated(
            StreamId,
            AccountOf<T>,
            AccountOf<T>,
            BalanceOf<T>,
            AssetIdOf<T>,
            Timestamp,
            Timestamp,
        ),
        /// Withdraw payment from stream. \[stream_id, recipient, amount\]
        StreamWithdrawn(StreamId, AccountOf<T>, AssetIdOf<T>, BalanceOf<T>),
        /// Cancel an existing stream. \[stream_id, sender, recipient, sender_balance, recipient_balance]
        StreamCanceled(
            StreamId,
            AccountOf<T>,
            AccountOf<T>,
            AssetIdOf<T>,
            BalanceOf<T>,
            BalanceOf<T>,
        ),
    }

    /// Next Stream Id
    #[pallet::storage]
    #[pallet::getter(fn next_stream)]
    pub type NextStreamId<T: Config> = StorageValue<_, StreamId, ValueQuery>;

    /// Global storage for streams
    #[pallet::storage]
    #[pallet::getter(fn get_stream)]
    pub type Streams<T: Config> = StorageMap<_, Twox64Concat, StreamId, Stream<T>>;

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
            recipient: AccountOf<T>,
            deposit: BalanceOf<T>,
            currency_id: AssetIdOf<T>,
            start_time: Timestamp,
            stop_time: Timestamp,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            ensure!(recipient != sender, Error::<T>::RecipientIsAlsoSender);
            ensure!(!deposit.is_zero(), Error::<T>::DepositIsZero);
            ensure!(
                start_time >= T::UnixTime::now().as_secs(),
                Error::<T>::StartBeforeBlockTime
            );
            ensure!(stop_time > start_time, Error::<T>::StopBeforeStart);

            // get rate per sec
            let duration = stop_time
                .checked_sub(start_time)
                .ok_or(ArithmeticError::Underflow)?;
            let rate_per_sec = deposit
                .checked_div(duration as u128)
                .ok_or(ArithmeticError::DivisionByZero)?;
            // insert stream to the Streams
            let stream: Stream<T> = Stream {
                remaining_balance: deposit, // remaining balance same value for now due to initialization
                deposit,                    // deposit
                currency_id,                // currency id
                rate_per_sec,               // rate per second
                recipient: recipient.clone(), // recipient
                sender: sender.clone(),     // sender
                start_time,                 // start_time
                stop_time,                  // stop_time
            };
            let stream_id = NextStreamId::<T>::get();
            // Insert stream to runtime
            Streams::<T>::insert(stream_id, stream);
            // Increment stream id
            NextStreamId::<T>::set(stream_id + 1);
            // transfer deposit from sender to global EOA
            T::Assets::transfer(currency_id, &sender, &Self::account_id(), deposit, false)?;
            Self::deposit_event(Event::<T>::StreamCreated(
                stream_id,
                sender,
                recipient,
                deposit,
                currency_id,
                start_time,
                stop_time,
            ));
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
            let stream = Streams::<T>::get(stream_id).ok_or(DispatchError::CannotLookup)?;
            ensure!(sender == stream.sender, Error::<T>::NotTheStreamer);
            // get sender and recipient balance at result
            let sender_balance = Self::balance_of(&stream, &sender)?;
            let recipient_balance = Self::balance_of(&stream, &stream.recipient)?;
            // send funds back to sender and recipient with balance function
            T::Assets::transfer(
                stream.currency_id,
                &Self::account_id(),
                &stream.recipient,
                recipient_balance,
                false,
            )?;
            T::Assets::transfer(
                stream.currency_id,
                &Self::account_id(),
                &sender,
                sender_balance,
                false,
            )?;
            // remove stream
            Streams::<T>::remove(stream_id);
            Self::deposit_event(Event::<T>::StreamCanceled(
                stream_id,
                sender,
                stream.recipient,
                stream.currency_id,
                sender_balance,
                recipient_balance,
            ));
            Ok(().into())
        }

        /// Withdraw from an existing stream
        #[pallet::weight((<T as Config>::WeightInfo::withdraw_from_stream(), DispatchClass::Operational))]
        #[transactional]
        pub fn withdraw_from_stream(
            origin: OriginFor<T>,
            stream_id: StreamId,
            amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            // check sender is stream recipient
            let mut stream = Streams::<T>::get(stream_id).ok_or(DispatchError::CannotLookup)?;
            ensure!(sender == stream.recipient, Error::<T>::NotTheRecipient);
            // Check balance
            let balance = Self::balance_of(&stream, &stream.recipient)?;
            ensure!(balance >= amount, Error::<T>::InsufficientBalance);
            stream.remaining_balance = stream
                .remaining_balance
                .checked_sub(amount)
                .ok_or(ArithmeticError::Underflow)?;
            // Check if balance is zero, then remove
            if stream.remaining_balance.is_zero() {
                // remove
                Streams::<T>::remove(stream_id);
            } else {
                // insert new streaming
                Streams::<T>::insert(stream_id, stream.clone());
            }
            // withdraw deposit from stream
            T::Assets::transfer(
                stream.currency_id,
                &Self::account_id(),
                &sender,
                amount,
                false,
            )?;
            Self::deposit_event(Event::<T>::StreamWithdrawn(
                stream_id,
                stream.recipient,
                stream.currency_id,
                amount,
            ));
            Ok(().into())
        }
    }
}

impl<T: Config> Pallet<T> {
    pub fn account_id() -> AccountOf<T> {
        T::PalletId::get().into_account()
    }

    pub fn delta_of(stream: &Stream<T>) -> Result<u64, DispatchError> {
        let now = T::UnixTime::now().as_secs();
        if now <= stream.start_time {
            Ok(Zero::zero())
        } else if now < stream.stop_time {
            now.checked_sub(stream.start_time)
                .ok_or(DispatchError::Arithmetic(ArithmeticError::Underflow))
        } else {
            stream
                .stop_time
                .checked_sub(stream.start_time)
                .ok_or(DispatchError::Arithmetic(ArithmeticError::Underflow))
        }
    }

    // Measure balance of streaming with rate per sec
    pub fn balance_of(
        stream: &Stream<T>,
        who: &AccountOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        let delta = Self::delta_of(stream)? as BalanceOf<T>;

        /*
         * If the stream `balance` does not equal `deposit`, it means there have been withdrawals.
         * We have to subtract the total amount withdrawn from the amount of money that has been
         * streamed until now.
         */
        let recipient_balance = if stream.deposit > stream.remaining_balance {
            let withdrawal_amount = stream
                .deposit
                .checked_sub(stream.remaining_balance)
                .ok_or(ArithmeticError::Underflow)?;
            let recipient_balance = delta
                .checked_mul(stream.rate_per_sec)
                .ok_or(ArithmeticError::Overflow)?;
            recipient_balance
                .checked_sub(withdrawal_amount)
                .ok_or(ArithmeticError::Underflow)?
        } else {
            delta
                .checked_mul(stream.rate_per_sec)
                .ok_or(ArithmeticError::Overflow)?
        };

        if *who == stream.recipient {
            if delta == (stream.stop_time - stream.start_time).into() {
                Ok(stream.remaining_balance)
            } else {
                Ok(recipient_balance)
            }
        } else if *who == stream.sender {
            let _recipient_balance = &recipient_balance;
            let sender_balance = stream
                .remaining_balance
                .checked_sub(*_recipient_balance)
                .ok_or(ArithmeticError::Underflow)?;
            Ok(sender_balance)
        } else {
            Ok(Zero::zero())
        }
    }
}
