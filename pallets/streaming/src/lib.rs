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

//! # Streaming pallet
//!
//! ## Overview
//!
//! This pallet is part of DAOFi modules that provides payroll streaming management

#![cfg_attr(not(feature = "std"), no_std)]

use crate::types::{Stream, StreamKind, StreamStatus};
use frame_support::{
    pallet_prelude::*,
    traits::{
        tokens::fungibles::{Inspect, Mutate, Transfer},
        UnixTime,
    },
    transactional, PalletId,
};
use frame_system::pallet_prelude::*;
pub use pallet::*;
use primitives::*;
use sp_runtime::{
    traits::{AccountIdConversion, One, Zero},
    ArithmeticError, DispatchError,
};
use sp_std::prelude::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

mod benchmarking;

mod types;

pub mod weights;

type AssetIdOf<T> =
    <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;
type BalanceOf<T> =
    <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance;
type AccountOf<T> = <T as frame_system::Config>::AccountId;

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

        /// Specify max count of streams for an account
        #[pallet::constant]
        type MaxStreamsCount: Get<u32>;

        /// Unix time
        type UnixTime: UnixTime;

        /// The origin which can update minimum_deposit
        type UpdateOrigin: EnsureOrigin<<Self as frame_system::Config>::Origin>;

        /// Weight information
        type WeightInfo: WeightInfo;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Sender as specified themselves as the recipient
        RecipientIsAlsoSender,
        /// Insufficient deposit size
        DepositLowerThanMinimum,
        /// Start time is before current block time
        StartBeforeBlockTime,
        /// Stop time is before start time
        StopBeforeStart,
        /// Caller is not the streamer
        NotTheStreamer,
        /// Caller is not the recipient
        NotTheRecipient,
        /// Amount exceeds balance
        LowRemainingBalance,
        /// Excess max streams count
        ExcessMaxStreamsCount,
        /// Stream was completed
        StreamCompleted,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Creates a payment stream.
        /// \[stream_id, sender, recipient, deposit, asset_id, start_time, stop_time\]
        StreamCreated(
            StreamId,
            AccountOf<T>,
            AccountOf<T>,
            BalanceOf<T>,
            AssetIdOf<T>,
            Timestamp,
            Timestamp,
        ),
        /// Withdraw payment from stream.
        /// \[stream_id, recipient, asset_id, amount\]
        StreamWithdrawn(StreamId, AccountOf<T>, AssetIdOf<T>, BalanceOf<T>),
        /// Cancel an existing stream.
        /// \[stream_id, sender, recipient, sender_balance, recipient_balance]
        StreamCanceled(
            StreamId,
            AccountOf<T>,
            AccountOf<T>,
            AssetIdOf<T>,
            BalanceOf<T>,
            BalanceOf<T>,
        ),
        /// Set minimum deposit for creating a stream
        /// \[asset_id, minimum_deposit\]
        MinimumDepositSet(AssetIdOf<T>, BalanceOf<T>),
    }

    /// Next Stream Id
    #[pallet::storage]
    #[pallet::getter(fn next_stream)]
    pub type NextStreamId<T: Config> = StorageValue<_, StreamId, ValueQuery>;

    /// Global storage for streams
    #[pallet::storage]
    #[pallet::getter(fn get_stream)]
    pub type Streams<T: Config> = StorageMap<_, Blake2_128Concat, StreamId, Stream<T>, OptionQuery>;

    /// Streaming holds by account
    /// account_id => stream_kind => stream_id
    #[pallet::storage]
    #[pallet::getter(fn stream_library)]
    pub type StreamLibrary<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        StreamKind,
        BoundedVec<StreamId, T::MaxStreamsCount>,
        OptionQuery,
    >;

    /// Minimum deposit for each asset
    #[pallet::storage]
    #[pallet::getter(fn minimum_deposit)]
    pub type MinimumDeposits<T: Config> = StorageMap<_, Twox64Concat, AssetIdOf<T>, BalanceOf<T>>;

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a new stream between sender and recipient
        ///
        /// Transfer assets to another account in the form of stream
        /// Any supported assets in parallel/heiko can be used to stream.
        /// The sender's assets will be locked into palletId
        /// Will transfer to recipient as the stream progresses
        ///
        /// - `recipient`: the receiving address
        /// - `deposit`: the amount sender will deposit to create the stream
        /// - `asset_id`: asset should be able to lookup.
        /// - `start_time`: the time when the stream will start
        /// - `stop_time`: the time when the stream will end
        #[pallet::weight(<T as Config>::WeightInfo::create_stream())]
        #[transactional]
        pub fn create_stream(
            origin: OriginFor<T>,
            recipient: AccountOf<T>,
            deposit: BalanceOf<T>,
            asset_id: AssetIdOf<T>,
            start_time: Timestamp,
            stop_time: Timestamp,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            ensure!(recipient != sender, Error::<T>::RecipientIsAlsoSender);
            ensure!(
                start_time >= T::UnixTime::now().as_secs(),
                Error::<T>::StartBeforeBlockTime
            );
            ensure!(stop_time > start_time, Error::<T>::StopBeforeStart);

            let minimum_deposit = MinimumDeposits::<T>::get(asset_id);
            ensure!(
                deposit >= minimum_deposit.unwrap_or(1u128),
                Error::<T>::DepositLowerThanMinimum
            );

            let duration = stop_time
                .checked_sub(start_time)
                .ok_or(ArithmeticError::Underflow)?;
            let rate_per_sec = deposit
                .checked_div(duration as u128)
                .ok_or(ArithmeticError::DivisionByZero)?;

            // Transfer deposit asset from sender to global EOA
            T::Assets::transfer(asset_id, &sender, &Self::account_id(), deposit, false)?;

            // The remaining balance will be the same value as the deposit due to initialization
            let stream: Stream<T> = Stream {
                remaining_balance: deposit,
                deposit,
                asset_id,
                rate_per_sec,
                recipient: recipient.clone(),
                sender: sender.clone(),
                start_time,
                stop_time,
                status: StreamStatus::Ongoing,
            };

            let stream_id = NextStreamId::<T>::get();
            // Increment next stream id and store the new created stream
            NextStreamId::<T>::set(
                stream_id
                    .checked_add(One::one())
                    .ok_or(ArithmeticError::Overflow)?,
            );
            Streams::<T>::insert(stream_id, stream);

            // Add the stream to stream_library for both the sender and receiver.
            let checked_push =
                |r: &mut Option<BoundedVec<StreamId, T::MaxStreamsCount>>| -> DispatchResult {
                    let mut registry = r.take().unwrap_or_default();
                    registry
                        .try_push(stream_id)
                        .map_err(|_| Error::<T>::ExcessMaxStreamsCount)?;
                    *r = Some(registry);
                    Ok(())
                };
            StreamLibrary::<T>::try_mutate(&sender, &StreamKind::Send, checked_push)?;
            StreamLibrary::<T>::try_mutate(&recipient, &StreamKind::Receive, checked_push)?;

            Self::deposit_event(Event::<T>::StreamCreated(
                stream_id, sender, recipient, deposit, asset_id, start_time, stop_time,
            ));
            Ok(().into())
        }

        /// Cancel a existed stream and return the deposit to sender
        ///
        /// Can only be called by the sender
        ///
        /// - `stream_id`: the stream id which will be canceled
        #[pallet::weight(T::WeightInfo::cancel_stream())]
        #[transactional]
        pub fn cancel_stream(
            origin: OriginFor<T>,
            stream_id: StreamId,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            let mut stream = Streams::<T>::get(stream_id).ok_or(DispatchError::CannotLookup)?;
            ensure!(sender == stream.sender, Error::<T>::NotTheStreamer);

            // calculate the balance to return
            let sender_balance = Self::balance_of(&stream, &sender)?;
            let recipient_balance = Self::balance_of(&stream, &stream.recipient)?;

            // return funds back to sender and recipient
            T::Assets::transfer(
                stream.asset_id,
                &Self::account_id(),
                &stream.recipient,
                recipient_balance,
                false,
            )?;
            T::Assets::transfer(
                stream.asset_id,
                &Self::account_id(),
                &sender,
                sender_balance,
                false,
            )?;

            stream.status = StreamStatus::Completed;
            Streams::<T>::insert(stream_id, stream.clone());

            Self::deposit_event(Event::<T>::StreamCanceled(
                stream_id,
                sender,
                stream.recipient,
                stream.asset_id,
                sender_balance,
                recipient_balance,
            ));

            Ok(().into())
        }

        /// Withdraw from a existed stream
        ///
        /// Can be called by the sender or the recipient
        ///
        /// - `stream_id`: the stream id which will be withdraw from
        /// ` `amount`: the amount of asset to withdraw
        #[pallet::weight(T::WeightInfo::withdraw_from_stream())]
        #[transactional]
        pub fn withdraw_from_stream(
            origin: OriginFor<T>,
            stream_id: StreamId,
            amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            let mut stream = Streams::<T>::get(stream_id).ok_or(DispatchError::CannotLookup)?;
            ensure!(
                stream.status != StreamStatus::Completed,
                Error::<T>::StreamCompleted
            );
            ensure!(sender == stream.recipient, Error::<T>::NotTheRecipient);

            let balance = Self::balance_of(&stream, &stream.recipient)?;
            ensure!(balance >= amount, Error::<T>::LowRemainingBalance);
            stream.try_deduct(amount)?;
            stream.try_complete()?;
            Streams::<T>::insert(stream_id, stream.clone());

            // withdraw deposit from stream
            T::Assets::transfer(stream.asset_id, &Self::account_id(), &sender, amount, false)?;
            Self::deposit_event(Event::<T>::StreamWithdrawn(
                stream_id,
                stream.recipient,
                stream.asset_id,
                amount,
            ));

            Ok(().into())
        }

        /// Set the minimum deposit for a stream
        ///
        /// Can only be called by the UpdateOrigin
        ///
        /// - `asset_id`: the stream id which will be set the minimum deposit
        /// - `minimum_deposit`: the minimum deposit for a stream
        #[pallet::weight(T::WeightInfo::set_minimum_deposit())]
        #[transactional]
        pub fn set_minimum_deposit(
            origin: OriginFor<T>,
            asset_id: AssetIdOf<T>,
            minimum_deposit: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            T::UpdateOrigin::ensure_origin(origin)?;
            MinimumDeposits::<T>::insert(asset_id, minimum_deposit);

            Self::deposit_event(Event::<T>::MinimumDepositSet(asset_id, minimum_deposit));
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
