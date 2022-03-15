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

use codec::{Decode, Encode, MaxEncodedLen};
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
use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;

#[cfg(feature = "std")]
use frame_support::serde::{Deserialize, Serialize};
use primitives::*;
use sp_runtime::{
    traits::{
        AccountIdConversion, CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, One, StaticLookup,
        Zero,
    },
    ArithmeticError, FixedPointNumber, FixedU128, DispatchError
};
use sp_std::{fmt::Debug, prelude::*};

pub use pallet::*;

//#[cfg(test)]
//mod mock;
//#[cfg(test)]
//mod tests;

pub mod weights;

type AssetIdOf<T> =
    <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;
type BalanceOf<T> =
    <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance;
type AccountOf<T> = <T as frame_system::Config>::AccountId;

// Struct for Payroll stream
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
        NotTheRecipient,
        ExceedsBalance,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Creates a payment stream. \[stream_id, sender, recipient, deposit, currency_id, start_time, stop_time\]
        CreateStream(
            StreamId,
            AccountOf<T>,
            AccountOf<T>,
            BalanceOf<T>,
            AssetIdOf<T>,
            Timestamp,
            Timestamp,
        ),
        /// Withdraw payment from stream. \[stream_id, recipient, amount\]
        WithdrawFromStream(StreamId, AccountOf<T>, AssetIdOf<T>, BalanceOf<T>),
        /// Cancel an existing stream. \[stream_id, sender, recipient, sender_balance, recipient_balance]
        CancelStream(
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
            let stream: Stream<T> = Stream {
                remaining_balance: deposit, // remaining balance same value for now due to initialization
                deposit,                    // deposit
                currency_id,                // currency id
                rate_per_sec,               // rate per second
                recipient: recipient.clone(),                  // recipient
                sender: sender.clone(),                     // sender
                start_time,                 // start_time
                stop_time,                  // stop_time
            };
            let stream_id = NextStreamId::<T>::get();
            // Insert stream to runtime
            Streams::<T>::insert(stream_id.clone(), stream);
            // Increment stream id
            NextStreamId::<T>::set(stream_id + 1);
            Self::deposit_event(Event::<T>::CreateStream(stream_id, sender, recipient, deposit, currency_id, start_time, stop_time));
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
            let stream = Streams::<T>::get(stream_id.clone()).ok_or(DispatchError::CannotLookup)?;
            ensure!(
                sender == stream.sender,
                Error::<T>::NotTheStreamer
            );
            // get sender and recipient balance at result
            let sender_balance =  Self::balance_of(stream.clone(), &sender)?;
            let recipient_balance = Self::balance_of(stream.clone(), &stream.recipient)?;
            // send funds back to sender and recipient with balance function
            T::Assets::transfer(stream.currency_id.clone(), &Self::account_id(), &sender, sender_balance.clone(), false)?;
            T::Assets::transfer(stream.currency_id.clone(), &Self::account_id(), &sender, recipient_balance.clone(), false)?;
            // remove stream
            Streams::<T>::remove(stream_id);
            Self::deposit_event(Event::<T>::CancelStream(stream_id, sender, stream.recipient, stream.currency_id, sender_balance, recipient_balance));
            Ok(().into())
        }

        /// Withdraw from an existing stream
        #[pallet::weight((<T as Config>::WeightInfo::withdraw_from_stream(), DispatchClass::Operational))]
        #[transactional]
        pub fn withdraw_from_stream(
            origin: OriginFor<T>,
            stream_id: StreamId,
            currency_id: AssetIdOf<T>,
            amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            // check sender is stream recipient
            let mut stream = Streams::<T>::get(stream_id).ok_or(DispatchError::CannotLookup)?;
            ensure!(
                sender == stream.recipient,
                Error::<T>::NotTheRecipient
            );
            // Check balance
            let balance = Self::balance_of(stream.clone(), &stream.recipient)?;
            ensure!(balance >= amount, Error::<T>::ExceedsBalance);
            stream.remaining_balance = stream.remaining_balance.checked_sub(amount).ok_or(ArithmeticError::Underflow)?;
            // Check if balance is zero, then remove
            if stream.remaining_balance == 0 {
                // remove
                Streams::<T>::remove(stream_id);
            }
            // withdraw deposit from stream
            T::Assets::transfer(stream.currency_id, &Self::account_id(), &sender, amount, false)?;
            Self::deposit_event(Event::<T>::WithdrawFromStream(stream_id, stream.recipient, stream.currency_id, amount));
            Ok(().into())
        }
    }
}

impl<T: Config> Pallet<T> {
    pub fn account_id() -> AccountOf<T> {
        T::PalletId::get().into_account()
    }

    // Measure balance of payroll with rate per sec
    pub fn balance_of(stream: Stream<T>, who: &AccountOf<T>) -> Result<BalanceOf<T>, DispatchError> {
        let now = T::UnixTime::now().as_secs();
        let delta = if now < stream.start_time {
            BalanceOf::<T>::zero()
        } else if now < stream.stop_time {
            now.checked_sub(stream.start_time).ok_or(ArithmeticError::Underflow)? as u128
        } else {
            stream.stop_time.checked_sub(stream.start_time).ok_or(ArithmeticError::Underflow)? as u128
        };
        
        /*
         * If the stream `balance` does not equal `deposit`, it means there have been withdrawals.
         * We have to subtract the total amount withdrawn from the amount of money that has been
         * streamed until now.
         */
        let recipient_balance = if stream.deposit > stream.remaining_balance {
            let withdrawl_amount = stream.deposit.checked_sub(stream.remaining_balance).ok_or(ArithmeticError::Underflow)?;
            let recipient_balance = delta.checked_mul(stream.rate_per_sec).ok_or(ArithmeticError::Overflow)?;
            recipient_balance.checked_sub(withdrawl_amount).ok_or(ArithmeticError::Underflow)?
        } else {
            delta.checked_mul(stream.rate_per_sec).ok_or(ArithmeticError::Overflow)?
        };

        if *who == stream.recipient {
            return Ok(recipient_balance);
        }
        if *who == stream.sender {
            let _recipient_balance = &recipient_balance;
            let sender_balance = stream.remaining_balance.checked_sub(*_recipient_balance).ok_or(ArithmeticError::Underflow)?;
            return Ok(sender_balance);
        }
        Ok(0)
    }
}