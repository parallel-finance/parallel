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

//! # Distributed Oracle pallet
//!
//! ## Overview
//!

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
    // weights::DispatchClass,
    PalletId,
};
use frame_system::pallet_prelude::*;
pub use pallet::*;
// use pallet_timestamp::{self as timestamp};
use primitives::*;
use scale_info::TypeInfo;
use sp_runtime::traits::AccountIdConversion;
use sp_std::prelude::*;

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

pub type RelayerId = u128;

// Struct for Relayer
#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct Relayer<T: Config> {
    // Owner
    owner: AccountOf<T>,
}

pub use weights::WeightInfo;

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct OracleDeposit<T: Config> {
    /// The stash account whose balance is actually locked and at stake.
    /// Stake Asset
    pub asset: AssetIdOf<T>,
    /// The total amount of the stash's balance that we are currently accounting for.
    /// It's just `active` plus all the `unlocking` balances.
    #[codec(compact)]
    pub total: Balance,

    /// Stake Added Unix Time
    pub timestamp: Timestamp,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// Assets for deposit/withdraw collateral assets to/from loans module
        type Assets: Transfer<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
            + Inspect<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
            + Mutate<Self::AccountId, AssetId = CurrencyId, Balance = Balance>;

        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// Unix time
        type UnixTime: UnixTime;

        /// Weight information
        type WeightInfo: WeightInfo;

        /// Minimum stake amount
        #[pallet::constant]
        type MinStake: Get<BalanceOf<Self>>;

        /// Minimum unstake amount
        #[pallet::constant]
        type MinUnstake: Get<BalanceOf<Self>>;

        #[pallet::constant]
        type MinHoldTime: Get<BalanceOf<Self>>;

        /// Allowed staking currency
        #[pallet::constant]
        type StakingCurrency: Get<AssetIdOf<Self>>;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Insufficient Staking Amount
        InsufficientStakeAmount,
        /// Insufficient Staking Amount
        InsufficientUnStakeAmount,
        /// Invalid Staking Currency
        InvalidStakingCurrency,

        /// Stake added successfully
        AddedStake,

        /// Stake removed successfully
        RemovedStake,

        /// Error removing stake insufficient balance
        ErrorRemovingStakeInsufficientBalance,

        /// Staking Account not found
        StakingAccountNotFound,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T: Config> {
        /// The assets get staked successfully
        Staked(T::AccountId, BalanceOf<T>),
        /// The derivative get unstaked successfully
        Unstaked(T::AccountId, BalanceOf<T>),
    }

    /// Global storage for relayers
    #[pallet::storage]
    #[pallet::getter(fn get_relayer)]
    pub type Relayers<T: Config> = StorageMap<_, Twox64Concat, RelayerId, Relayer<T>>;

    /// Platform's staking ledgers
    #[pallet::storage]
    #[pallet::getter(fn staking_ledger)]
    pub type StakingPool<T: Config> =
        StorageMap<_, Blake2_128Concat, DerivativeIndex, OracleDeposit<T>>;

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// TODO - need functions
        #[pallet::weight(1000)]
        #[transactional]
        pub fn create_something(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let _sender = ensure_signed(origin)?;
            if 1 == 0 {
                unimplemented!();
            }
            Ok(().into())
        }

        /// Stake amounts
        #[pallet::weight(T::WeightInfo::stake())]
        pub fn stake(
            who: OriginFor<T>,
            asset: AssetIdOf<T>,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(who)?;
            // Check for token type
            // Check for amount
            ensure!(
                amount >= T::MinStake::get(),
                Error::<T>::InsufficientStakeAmount
            );
            // Transfer
            // Add Stake to the store
            T::Assets::transfer(
                T::StakingCurrency::get(),
                &who,
                &Self::account_id(),
                amount,
                false,
            )?;

            let _oracle_deposit = OracleDeposit::<T> {
                asset,
                total: amount,
                timestamp: T::UnixTime::now().as_secs(),
            };
            // If Account has amount add the new stake amount or else insert a new record
            // StakingPool::<T>::insert()
            // // Emit a message
            // Self::deposit_event(Event::<T>::Staked(who, amount));

            log::trace!(
                target: "distributed-oracle::stake",
                "stake_amount: {:?}",
                &amount,
            );

            Ok(().into())
        }

        /// Unstake amounts
        #[pallet::weight(T::WeightInfo::unstake())]
        pub fn unstake(
            origin: OriginFor<T>,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(
                amount < T::MinStake::get(),
                Error::<T>::InsufficientUnStakeAmount
            );
            // InsufficientUnStakeAmount
            // CHeck for Minimum Balance
            // Check for Token
            // Check for Time duration

            // Check if a staking account exists or throw an error
            // else update storage substract the value

            T::Assets::burn_from(T::StakingCurrency::get(), &who, amount)?;
            Self::deposit_event(Event::<T>::Unstaked(who, amount));

            log::trace!(
                target: "distributed-oracle::unstake",
                "unstake_amount: {:?}",
                &amount,
            );

            Ok(().into())
        }
    }
}

impl<T: Config> Pallet<T> {
    pub fn account_id() -> AccountOf<T> {
        T::PalletId::get().into_account()
    }
}
