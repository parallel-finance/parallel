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
use primitives::*;
use sp_runtime::{
    traits::{AccountIdConversion, One, Zero},
    ArithmeticError,
};
use sp_std::prelude::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

mod benchmarking;

mod types;

pub use pallet::*;

pub mod weights;
pub use weights::WeightInfo;

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

        /// The max count of streams for an account
        #[pallet::constant]
        type MaxStreamsCount: Get<u32>;

        /// The max count of streams that has been cancelled or completed for an account
        #[pallet::constant]
        type MaxFinishedStreamsCount: Get<u32>;

        /// The Unix time
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
        StartTimeBeforeCurrentTime,
        /// End time is before start time
        EndTimeBeforeStartTime,
        /// The duration calculated is too short or too long
        InvalidDuration,
        /// The rate per second calculated is zero
        InvalidRatePerSecond,
        /// The stream id is not found
        InvalidStreamId,
        /// Caller is not the stream sender
        NotTheSender,
        /// Caller is not the stream recipient
        NotTheRecipient,
        /// Amount exceeds balance
        InsufficientStreamBalance,
        /// Excess max streams count
        ExcessMaxStreamsCount,
        /// Stream was cancelled or completed
        StreamHasFinished,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Creates a payment stream.
        /// \[stream_id, sender, recipient, deposit, asset_id, start_time, end_time\]
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
    #[pallet::getter(fn streams)]
    pub type Streams<T: Config> = StorageMap<_, Blake2_128Concat, StreamId, Stream<T>, OptionQuery>;

    /// Streams holds by each account
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
        /// - `end_time`: the time when the stream will end
        #[pallet::weight(<T as Config>::WeightInfo::create())]
        #[transactional]
        pub fn create(
            origin: OriginFor<T>,
            recipient: AccountOf<T>,
            deposit: BalanceOf<T>,
            asset_id: AssetIdOf<T>,
            start_time: Timestamp,
            end_time: Timestamp,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            ensure!(sender != recipient, Error::<T>::RecipientIsAlsoSender);

            let minimum_deposit = MinimumDeposits::<T>::get(asset_id);
            ensure!(
                deposit >= minimum_deposit.unwrap_or(1u128),
                Error::<T>::DepositLowerThanMinimum
            );

            let duration = Self::ensure_valid_duration(start_time, end_time)?;
            let rate_per_sec = deposit
                .checked_div(duration as u128)
                .ok_or(Error::<T>::InvalidRatePerSecond)?;
            ensure!(!rate_per_sec.is_zero(), Error::<T>::InvalidRatePerSecond);

            // Transfer deposit asset from sender to global EOA
            T::Assets::transfer(asset_id, &sender, &Self::account_id(), deposit, false)?;

            // The remaining balance will be the same value as the deposit due to initialization
            let stream: Stream<T> = Stream::new(
                deposit,
                asset_id,
                rate_per_sec,
                sender.clone(),
                recipient.clone(),
                start_time,
                end_time,
            );

            let stream_id = NextStreamId::<T>::get();
            // Increment next stream id and store the new created stream
            NextStreamId::<T>::set(
                stream_id
                    .checked_add(One::one())
                    .ok_or(ArithmeticError::Overflow)?,
            );
            Streams::<T>::insert(stream_id, stream);

            // Remove the outdated and finished streams
            Self::update_finished_stream_library(&sender)?;
            Self::update_finished_stream_library(&recipient)?;
            // Add the stream_id to stream_library for both the sender and receiver.
            Self::try_push_stream_library(&sender, StreamKind::Send, stream_id)?;
            Self::try_push_stream_library(&recipient, StreamKind::Receive, stream_id)?;

            Self::deposit_event(Event::<T>::StreamCreated(
                stream_id, sender, recipient, deposit, asset_id, start_time, end_time,
            ));
            Ok(().into())
        }

        /// Cancel a existed stream and return back the deposit to sender and recipient
        ///
        /// Can only be called by the sender
        ///
        /// - `stream_id`: the stream id which will be canceled
        #[pallet::weight(T::WeightInfo::cancel())]
        #[transactional]
        pub fn cancel(origin: OriginFor<T>, stream_id: StreamId) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            let mut stream = Streams::<T>::get(stream_id).ok_or(Error::<T>::InvalidStreamId)?;
            ensure!(stream.is_sender(&sender), Error::<T>::NotTheSender);

            // calculate the balance to return
            let sender_balance = stream.sender_balance()?;
            let recipient_balance = stream.recipient_balance()?;

            // return funds back to sender and recipient
            T::Assets::transfer(
                stream.asset_id,
                &Self::account_id(),
                &sender,
                sender_balance,
                false,
            )?;
            T::Assets::transfer(
                stream.asset_id,
                &Self::account_id(),
                &stream.recipient,
                recipient_balance,
                false,
            )?;

            // Will keep remaining_balance in the stream
            stream.status = StreamStatus::Cancelled;
            Self::try_push_stream_library(&stream.sender, StreamKind::Finish, stream_id)?;
            Self::try_push_stream_library(&stream.recipient, StreamKind::Finish, stream_id)?;

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
        #[pallet::weight(T::WeightInfo::withdraw())]
        #[transactional]
        pub fn withdraw(
            origin: OriginFor<T>,
            stream_id: StreamId,
            amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let recipient = ensure_signed(origin)?;

            let mut stream = Streams::<T>::get(stream_id).ok_or(Error::<T>::InvalidStreamId)?;
            ensure!(!stream.has_finished(), Error::<T>::StreamHasFinished);
            ensure!(stream.is_recipient(&recipient), Error::<T>::NotTheRecipient);
            let recipient_balance = stream.recipient_balance()?;
            ensure!(
                recipient_balance >= amount,
                Error::<T>::InsufficientStreamBalance
            );

            stream.try_deduct(amount)?;
            stream.try_complete()?;
            if stream.has_finished() {
                Self::try_push_stream_library(&stream.sender, StreamKind::Finish, stream_id)?;
                Self::try_push_stream_library(&recipient, StreamKind::Finish, stream_id)?;
            }
            Streams::<T>::insert(stream_id, stream.clone());

            // Withdraw deposit from stream
            T::Assets::transfer(
                stream.asset_id,
                &Self::account_id(),
                &recipient,
                amount,
                false,
            )?;
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

    pub fn ensure_valid_duration(
        start_time: Timestamp,
        end_time: Timestamp,
    ) -> Result<Timestamp, DispatchError> {
        ensure!(
            start_time >= T::UnixTime::now().as_secs(),
            Error::<T>::StartTimeBeforeCurrentTime
        );
        ensure!(end_time > start_time, Error::<T>::EndTimeBeforeStartTime);

        let duration = end_time
            .checked_sub(start_time)
            .ok_or(Error::<T>::InvalidDuration)?;

        Ok(duration)
    }

    pub fn update_finished_stream_library(account: &AccountOf<T>) -> DispatchResult {
        let checked_pop =
            |registry: &mut Option<BoundedVec<StreamId, T::MaxStreamsCount>>| -> DispatchResult {
                let mut r = registry.take().unwrap_or_default();
                r.as_mut().sort_unstable_by(|a, b| b.cmp(a));

                let len = r.len() as u32;
                match len {
                    _x if len >= T::MaxFinishedStreamsCount::get() => {
                        if let Some(stream_id) = r.pop() {
                            if Streams::<T>::get(stream_id).is_some() {
                                Streams::<T>::remove(stream_id);
                            }

                            Self::try_remove_stream_library(account, StreamKind::Send, stream_id)?;
                            Self::try_remove_stream_library(
                                account,
                                StreamKind::Receive,
                                stream_id,
                            )?;
                        }
                    }
                    _ => {}
                }

                *registry = Some(r);
                Ok(())
            };

        StreamLibrary::<T>::try_mutate(account, &StreamKind::Finish, checked_pop)?;

        Ok(())
    }

    pub fn try_push_stream_library(
        account: &AccountOf<T>,
        kind: StreamKind,
        stream_id: StreamId,
    ) -> DispatchResult {
        let checked_push =
            |registry: &mut Option<BoundedVec<StreamId, T::MaxStreamsCount>>| -> DispatchResult {
                let mut r = registry.take().unwrap_or_default();
                r.try_push(stream_id)
                    .map_err(|_| Error::<T>::ExcessMaxStreamsCount)?;
                *registry = Some(r);
                Ok(())
            };

        StreamLibrary::<T>::try_mutate(account, &kind, checked_push)?;
        Ok(())
    }

    pub fn try_remove_stream_library(
        account: &AccountOf<T>,
        kind: StreamKind,
        stream_id: StreamId,
    ) -> DispatchResult {
        let checked_remove =
            |registry: &mut Option<BoundedVec<StreamId, T::MaxStreamsCount>>| -> DispatchResult {
                let mut r = registry.take().unwrap_or_default();
                if let Ok(index) = r.binary_search(&stream_id) {
                    r.remove(index);
                }
                *registry = Some(r);
                Ok(())
            };

        StreamLibrary::<T>::try_mutate(account, &kind, checked_remove)?;
        Ok(())
    }
}
