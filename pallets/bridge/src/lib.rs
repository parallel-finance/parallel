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

//! # Bridge pallet
//!
//! ## Overview
//!
//! The bridge pallet implement the transfer of tokens between `parallel` and `eth chains`
//! and the security of funds is secured by multiple signatures

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    pallet_prelude::*,
    traits::{ChangeMembers, Get, SortedMembers},
    PalletId,
};
use frame_system::pallet_prelude::*;

pub use pallet::*;
use sp_runtime::traits::AccountIdConversion;

mod mock;
mod tests;

pub type ChainId = u8;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// Origin used to administer the pallet
        type AdminMembers: SortedMembers<Self::AccountId>;

        /// Root origin that can be used to bypass admin permissions
        /// This will be removed later
        type RootOperatorAccountId: Get<Self::AccountId>;

        /// The identifier for this chain.
        /// This must be unique and must not collide with existing IDs within a set of bridged chains.
        #[pallet::constant]
        type ChainId: Get<ChainId>;

        /// The bridge's pallet id, keep all deposited assets.
        #[pallet::constant]
        type PalletId: Get<PalletId>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(PhantomData<T>);

    /// Error for the Assets Pallet
    #[pallet::error]
    pub enum Error<T> {
        /// Vote threshold not set
        VoteThresholdNotSet,
        /// The new threshold is invalid
        InvalidVoteThreshold,
        /// Origin has no permission to operate on the bridge
        OriginNoPermission,
    }

    /// Event for the Bridge Pallet
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T> {
        /// Vote threshold has changed
        /// [new_threshold]
        VoteThresholdChanged(u32),
    }

    #[pallet::type_value]
    pub fn DefaultVoteThreshold() -> u32 {
        3u32
    }
    #[pallet::storage]
    #[pallet::getter(fn vote_threshold)]
    pub type VoteThreshold<T: Config> = StorageValue<_, u32, ValueQuery, DefaultVoteThreshold>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(0)]
        pub fn set_threshold(origin: OriginFor<T>, threshold: u32) -> DispatchResult {
            Self::ensure_admin(origin)?;
            Self::set_vote_threshold(threshold)
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
        fn on_finalize(_n: T::BlockNumber) {
            // do nothing
        }
    }
}

impl<T: Config> Pallet<T> {
    pub fn ensure_admin(origin: T::Origin) -> DispatchResult {
        let who = ensure_signed(origin)?;
        ensure!(
            T::AdminMembers::contains(&who) || who == T::RootOperatorAccountId::get(),
            Error::<T>::OriginNoPermission
        );

        Ok(())
    }

    /// Provides an AccountId for the bridge pallet.
    /// Used for teleport/materialize account.
    pub fn account_id() -> T::AccountId {
        T::PalletId::get().into_account()
    }

    /// Set a new voting threshold
    pub fn set_vote_threshold(threshold: u32) -> DispatchResult {
        ensure!(
            threshold > 0 && threshold <= Self::get_members_count(),
            Error::<T>::InvalidVoteThreshold
        );

        VoteThreshold::<T>::put(threshold);
        Self::deposit_event(Event::VoteThresholdChanged(threshold));

        Ok(())
    }

    /// Get the count of members in the `AdminMembers`.
    pub fn get_members_count() -> u32 {
        T::AdminMembers::count() as u32
    }
}

impl<T: Config> ChangeMembers<T::AccountId> for Pallet<T> {
    fn change_members_sorted(
        _incoming: &[T::AccountId],
        _outgoing: &[T::AccountId],
        _new: &[T::AccountId],
    ) {
        // nothing
    }

    fn set_prime(_prime: Option<T::AccountId>) {
        // nothing
    }
}
