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
    traits::{
        tokens::fungibles::{Inspect, Mutate, Transfer},
        ChangeMembers, Get, SortedMembers,
    },
    PalletId,
};
use frame_system::pallet_prelude::*;
use primitives::{Balance, CurrencyId};
use sp_runtime::traits::AccountIdConversion;

pub use pallet::*;

mod mock;
mod tests;

type AssetIdOf<T> =
    <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;

type BalanceOf<T> =
    <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

pub type ChainId = u8;
pub type ChainNonce = u64;
pub type TeleAccount = Vec<u8>;

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

        /// The bridge's pallet id, keep all teleported assets.
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// Assets for teleport/materialize assets to/from bridge pallet
        type Assets: Transfer<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
            + Inspect<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
            + Mutate<Self::AccountId, AssetId = CurrencyId, Balance = Balance>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(PhantomData<T>);

    /// Error for the Bridge Pallet
    #[pallet::error]
    pub enum Error<T> {
        /// Vote threshold not set
        VoteThresholdNotSet,
        /// The new threshold is invalid
        InvalidVoteThreshold,
        /// Origin has no permission to operate on the bridge
        OriginNoPermission,
        /// The chain_id is invalid, it cannot be a existed chain_id or this chain_id
        ChainIdAlreadyRegistered,
        /// The chain_id is not registed and the related operation will be invalid
        ChainIdNotRegistered,
        /// The currency_id is invalid, it cannot be a existed currency_id
        CurrencyIdAlreadyRegistered,
        /// The currency_id is not registed and the related operation will be invalid
        CurrencyIdNotRegistered,
    }

    /// Event for the Bridge Pallet
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Vote threshold has changed
        /// [new_threshold]
        VoteThresholdChanged(u32),

        /// New chain_id has been registered
        /// [new_chain_id]
        ChainIdRegistered(ChainId),

        /// Initialize a cross-chain transfer
        /// [dest_id, chain_nonce, currency_id, amount, recipient]
        Teleported(ChainId, ChainNonce, CurrencyId, BalanceOf<T>, TeleAccount),
        
        /// New currency_id has been registered
        /// [asset_id, currency_id]
        CurrencyIdRegistered(AssetIdOf<T>, CurrencyId),
    }

    #[pallet::type_value]
    pub fn DefaultVoteThreshold() -> u32 {
        1u32
    }
    #[pallet::storage]
    #[pallet::getter(fn vote_threshold)]
    pub type VoteThreshold<T: Config> = StorageValue<_, u32, ValueQuery, DefaultVoteThreshold>;

    #[pallet::storage]
    #[pallet::getter(fn chain_nonces)]
    pub type ChainNonces<T: Config> =
        StorageMap<_, Blake2_256, ChainId, ChainNonce, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn currency_ids)]
    pub type CurrencyIds<T: Config> = StorageMap<_, Twox64Concat, AssetIdOf<T>, CurrencyId, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn asset_ids)]
    pub type AssetIds<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, AssetIdOf<T>, ValueQuery>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Set the threshold required to reach multi-signature consensus
        #[pallet::weight(0)]
        pub fn set_threshold(origin: OriginFor<T>, threshold: u32) -> DispatchResult {
            Self::ensure_admin(origin)?;
            Self::set_vote_threshold(threshold)
        }

        #[pallet::weight(0)]
        pub fn register_chain(origin: OriginFor<T>, id: ChainId) -> DispatchResult {
            Self::ensure_admin(origin)?;
            
            // Registered chain_id cannot be this chain_id
            ensure!(id != T::ChainId::get(), Error::<T>::ChainIdAlreadyRegistered);
    
            // Registered chain_id cannot be a existed chain_id
            ensure!(
                !Self::chain_registered(id),
                Error::<T>::ChainIdAlreadyRegistered
            );
            
            // Register a new chain_id
            ChainNonces::<T>::insert(id, 0);
            Self::deposit_event(Event::ChainIdRegistered(id));
    
            Ok(())
        }
        
        #[pallet::weight(0)]
        pub fn register_currency(
            origin: OriginFor<T>,
            asset_id: AssetIdOf<T>,
            currency_id: CurrencyId,
        ) -> DispatchResultWithPostInfo {
            Self::ensure_admin(origin)?;
            ensure!(
                !CurrencyIds::<T>::contains_key(currency_id)
                    && !AssetIds::<T>::contains_key(asset_id),
                Error::<T>::CurrencyIdAlreadyRegistered,
            );
    
            CurrencyIds::<T>::insert(asset_id, currency_id);
            AssetIds::<T>::insert(currency_id, asset_id);

            Self::deposit_event(Event::CurrencyIdRegistered(asset_id, currency_id));
            Ok(().into())
        }

        /// Teleport the currency to specified recipient in the destination chain
        #[pallet::weight(0)]
        pub fn teleport(
            origin: OriginFor<T>,
            dest_id: ChainId,
            currency_id: CurrencyId,
            to: TeleAccount,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::ensure_chain_registered(dest_id)?;
            Self::ensure_currency_registered(currency_id)?;
            let asset_id = AssetIds::<T>::get(currency_id);
            
            T::Assets::transfer(asset_id, &who, &Self::account_id(), amount, true)?;

            Self::internal_teleport(dest_id, currency_id, to, amount)
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
    /// Provides an AccountId for the bridge pallet.
    /// Used for teleport/materialize account.
    pub fn account_id() -> T::AccountId {
        T::PalletId::get().into_account()
    }

    /// Checks if the origin is admin members
    fn ensure_admin(origin: T::Origin) -> DispatchResult {
        let who = ensure_signed(origin)?;
        ensure!(
            T::AdminMembers::contains(&who) || who == T::RootOperatorAccountId::get(),
            Error::<T>::OriginNoPermission
        );

        Ok(())
    }

    /// Checks if a chain is registered
    fn chain_registered(id: ChainId) -> bool {
        return ChainNonces::<T>::contains_key(id)
    }

    fn ensure_chain_registered(id: ChainId) -> DispatchResult {
        ensure!(
            Self::chain_registered(id),
            Error::<T>::ChainIdNotRegistered
        );

        Ok(())
    }

    /// Checks if a currency is registered
    fn currency_registered(currency_id: CurrencyId) -> bool {
        return AssetIds::<T>::contains_key(currency_id)
    }
    
    fn ensure_currency_registered(currency_id: CurrencyId) -> DispatchResult {
        ensure!(
            Self::currency_registered(currency_id),
            Error::<T>::CurrencyIdNotRegistered
        );

        Ok(())
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

    /// Increments the chain nonce for the specified chain_id
    fn bump_nonce(id: ChainId) -> ChainNonce {
        let nonce = Self::chain_nonces(id) + 1;
        ChainNonces::<T>::insert(id, nonce);
        nonce
    }

    /// Initiates a transfer of the currency
    fn internal_teleport(
        dest_id: ChainId,
        currency_id: CurrencyId,
        to: TeleAccount,
        amount: BalanceOf<T>,
    ) -> DispatchResult {          
        let nonce = Self::bump_nonce(dest_id);

        Self::deposit_event(Event::Teleported(dest_id, nonce, currency_id, amount, to));
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
