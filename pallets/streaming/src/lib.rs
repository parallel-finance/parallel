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

use crate::types::{Stream, StreamKind};
use frame_support::{
    pallet_prelude::*,
    traits::{
        tokens::fungibles::{Inspect, Mutate, Transfer},
        UnixTime,
    },
    transactional, PalletId,
};
use frame_system::pallet_prelude::*;
use pallet_traits::Streaming;
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

        /// Currency id of the native token
        #[pallet::constant]
        type NativeCurrencyId: Get<AssetIdOf<Self>>;

        /// The essential balance for an existed account
        #[pallet::constant]
        type NativeExistentialDeposit: Get<Balance>;

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
        /// Asset is not supported to create stream
        InvalidAssetId,
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
        /// Stream cannot be cancelled
        CannotBeCancelled,
        /// Amount exceeds balance
        InsufficientStreamBalance,
        /// Excess max streams count
        ExcessMaxStreamsCount,
        /// Stream not started
        NotStarted,
        /// Stream was cancelled or completed
        HasFinished,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Creates a payment stream.
        /// \[stream_id, sender, recipient, deposit, asset_id, start_time, end_time, cancellable\]
        StreamCreated(
            StreamId,
            AccountOf<T>,
            AccountOf<T>,
            BalanceOf<T>,
            AssetIdOf<T>,
            Timestamp,
            Timestamp,
            bool,
        ),
        /// Withdraw payment from stream.
        /// \[stream_id, recipient, asset_id, amount\]
        StreamWithdrawn(StreamId, AccountOf<T>, AssetIdOf<T>, BalanceOf<T>),
        /// Cancel an existing stream.
        /// \[stream_id, sender, recipient, asset_id, sender_balance, recipient_balance]
        StreamCancelled(
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
            cancellable: bool,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            let stream_id = Self::do_create(
                sender.clone(),
                recipient.clone(),
                deposit,
                asset_id,
                start_time,
                end_time,
                cancellable,
            )?;
            // Add the stream_id to stream_library for both the sender and receiver.
            Self::try_push_stream_library(&sender, stream_id, StreamKind::Send)?;
            Self::try_push_stream_library(&recipient, stream_id, StreamKind::Receive)?;
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
            ensure!(!stream.has_finished(), Error::<T>::HasFinished);
            ensure!(stream.cancellable, Error::<T>::CannotBeCancelled);

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

            stream.try_cancel(sender_balance)?;
            Streams::<T>::insert(stream_id, stream.clone());

            Self::try_push_stream_library(&stream.sender, stream_id, StreamKind::Finish)?;
            Self::try_push_stream_library(&stream.recipient, stream_id, StreamKind::Finish)?;
            Self::update_finished_stream_library(&stream.sender, &stream.recipient)?;

            Self::deposit_event(Event::<T>::StreamCancelled(
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
            ensure!(stream.is_recipient(&recipient), Error::<T>::NotTheRecipient);
            ensure!(!stream.has_finished(), Error::<T>::HasFinished);
            ensure!(stream.has_started()?, Error::<T>::NotStarted);
            let recipient_balance = stream.recipient_balance()?;
            ensure!(
                recipient_balance >= amount,
                Error::<T>::InsufficientStreamBalance
            );

            let mut amount = amount;
            if stream.asset_id == T::NativeCurrencyId::get()
                && amount.saturating_add(T::NativeExistentialDeposit::get())
                    >= stream.remaining_balance
            {
                amount = stream.remaining_balance
            }

            stream.try_deduct(amount)?;
            stream.try_complete()?;
            Streams::<T>::insert(stream_id, stream.clone());
            if stream.has_finished() {
                Self::try_push_stream_library(&stream.sender, stream_id, StreamKind::Finish)?;
                Self::try_push_stream_library(&recipient, stream_id, StreamKind::Finish)?;
                Self::update_finished_stream_library(&stream.sender, &recipient)?;
            }

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
        T::PalletId::get().into_account_truncating()
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

    pub fn update_finished_stream_library(
        sender: &AccountOf<T>,
        recipient: &AccountOf<T>,
    ) -> DispatchResult {
        let checked_pop =
            |registry: &mut Option<BoundedVec<StreamId, T::MaxStreamsCount>>| -> DispatchResult {
                let mut r = registry.take().unwrap_or_default();

                if r.len() as u32 <= T::MaxFinishedStreamsCount::get() {
                    *registry = Some(r);
                    return Ok(());
                }

                // This is safe because we have ensured that the length is greater than the limit(at least 0)
                if let Some(stream_id) = r.pop() {
                    let stream = Streams::<T>::get(stream_id).ok_or(Error::<T>::InvalidStreamId)?;
                    // Remove all related storage
                    Self::try_remove_stream_library(&stream.sender, stream_id, None)?;
                    Self::try_remove_stream_library(&stream.recipient, stream_id, None)?;
                    Streams::<T>::remove(stream_id);
                }

                *registry = Some(r);
                Ok(())
            };

        StreamLibrary::<T>::try_mutate(sender, &StreamKind::Finish, checked_pop)?;
        StreamLibrary::<T>::try_mutate(recipient, &StreamKind::Finish, checked_pop)?;

        Ok(())
    }

    pub fn try_push_stream_library(
        account: &AccountOf<T>,
        stream_id: StreamId,
        kind: StreamKind,
    ) -> DispatchResult {
        let checked_push =
            |registry: &mut Option<BoundedVec<StreamId, T::MaxStreamsCount>>| -> DispatchResult {
                let mut r = registry.take().unwrap_or_default();
                if !r.to_vec().iter().any(|&x| x == stream_id) {
                    r.try_push(stream_id)
                        .map_err(|_| Error::<T>::ExcessMaxStreamsCount)?;
                }

                r.as_mut().sort_unstable_by(|a, b| b.cmp(a));
                *registry = Some(r);
                Ok(())
            };

        StreamLibrary::<T>::try_mutate(account, &kind, checked_push)?;
        Ok(())
    }

    pub fn try_remove_stream_library(
        account: &AccountOf<T>,
        stream_id: StreamId,
        kind: Option<StreamKind>,
    ) -> DispatchResult {
        let checked_remove =
            |registry: &mut Option<BoundedVec<StreamId, T::MaxStreamsCount>>| -> DispatchResult {
                let mut r = registry.take().unwrap_or_default();
                if let Some(index) = r.to_vec().iter().position(|&x| x == stream_id) {
                    r.remove(index);
                }else {
                    // TODO: replace registry.take() as as_mut()
                    return Ok(())
                }

                r.as_mut().sort_unstable_by(|a, b| b.cmp(a));
                *registry = Some(r);
                Ok(())
            };

        if let Some(k) = kind {
            StreamLibrary::<T>::try_mutate(account, &k, checked_remove)?;
        } else {
            StreamLibrary::<T>::try_mutate(account, StreamKind::Send, checked_remove)?;
            StreamLibrary::<T>::try_mutate(account, StreamKind::Receive, checked_remove)?;
            StreamLibrary::<T>::try_mutate(account, StreamKind::Finish, checked_remove)?;
        }

        Ok(())
    }

    pub fn do_create(
        sender: AccountOf<T>,
        recipient: AccountOf<T>,
        deposit: BalanceOf<T>,
        asset_id: AssetIdOf<T>,
        start_time: Timestamp,
        end_time: Timestamp,
        cancellable: bool,
    ) -> Result<StreamId, DispatchError> {
        ensure!(sender != recipient, Error::<T>::RecipientIsAlsoSender);

        let minimum_deposit = Self::minimum_deposit(asset_id).ok_or(Error::<T>::InvalidAssetId)?;
        ensure!(
            deposit >= minimum_deposit,
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
            cancellable,
        );

        let stream_id = NextStreamId::<T>::get();
        // Increment next stream id and store the new created stream
        NextStreamId::<T>::set(
            stream_id
                .checked_add(One::one())
                .ok_or(ArithmeticError::Overflow)?,
        );
        Streams::<T>::insert(stream_id, stream);

        // Remove the outdated and finished streams, should do update after push
        Self::update_finished_stream_library(&sender, &recipient)?;

        Self::deposit_event(Event::<T>::StreamCreated(
            stream_id, sender, recipient, deposit, asset_id, start_time, end_time, true,
        ));
        Ok(stream_id)
    }
}

impl<T: Config> Streaming<AccountIdOf<T>, AssetIdOf<T>, BalanceOf<T>> for Pallet<T> {
    fn create(
        sender: AccountOf<T>,
        recipient: AccountOf<T>,
        deposit: BalanceOf<T>,
        asset_id: AssetIdOf<T>,
        start_time: Timestamp,
        end_time: Timestamp,
        cancellable: bool,
    ) -> Result<(), DispatchError> {
        let stream_id = Self::do_create(
            sender,
            recipient.clone(),
            deposit,
            asset_id,
            start_time,
            end_time,
            cancellable,
        )?;
        // Add the stream_id to stream_library for both the sender and receiver.
        // Self::try_push_stream_library(&sender, stream_id, StreamKind::Send)?;
        // Self::try_push_stream_library(&recipient, stream_id, StreamKind::Receive)?;
        Ok(())
    }
}
