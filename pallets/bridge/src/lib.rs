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
        /// Relayer threshold not set
        ThresholdNotSet,
        /// The new threshold is invalid
        InvalidThreshold,
        /// Origin has no permission to operate on the bridge
        OriginNoPermission,
    }

    /// Event for the Bridge Pallet
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T> {
        /// Vote threshold has changed
        /// [new_threshold]
        RelayerThresholdChanged(u32),
    }

    #[pallet::type_value]
    pub fn DefaultRelayerThreshold() -> u32 {
        3u32
    }
    #[pallet::storage]
    #[pallet::getter(fn relayer_threshold)]
    pub type RelayerThreshold<T: Config> =
        StorageValue<_, u32, ValueQuery, DefaultRelayerThreshold>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(0)]
        pub fn set_threshold(origin: OriginFor<T>, threshold: u32) -> DispatchResult {
            Self::ensure_admin(origin)?;
            Self::set_relayer_threshold(threshold)
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
        fn on_finalize(_n: T::BlockNumber) {
            let threshold = Self::relayer_threshold();
            if threshold != DefaultRelayerThreshold::get() {
                Self::deposit_event(Event::RelayerThresholdChanged(threshold));
            }
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

    /// Set a new voting threshold
    pub fn set_relayer_threshold(threshold: u32) -> DispatchResult {
        ensure!(threshold > 0, Error::<T>::InvalidThreshold);

        RelayerThreshold::<T>::put(threshold);
        Self::deposit_event(Event::RelayerThresholdChanged(threshold));

        Ok(())
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
