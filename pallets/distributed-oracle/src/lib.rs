// Copyright 2022 Parallel Finance Developer.
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

use frame_support::{
    log,
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
use sp_runtime::traits::AccountIdConversion;
use sp_std::prelude::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod helpers;

pub mod weights;

type AssetIdOf<T> =
    <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;
type BalanceOf<T> =
    <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance;
type AccountOf<T> = <T as frame_system::Config>::AccountId;

pub type RelayerId = u128;

pub use weights::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use crate::helpers::{Coffer, OracleDeposit, Relayer, Repeater};
    use sp_runtime::ArithmeticError;
    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

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

        /// Unstake Amount Exceeds Balance
        UnstakeAmoutExceedsStakedBalance,

        /// Tries to register an existing repeater
        RepeaterExists,

        /// Only a repeater can stake
        InvalidStaker,

        /// Only a repeater can unstake,
        InvalidUnstaker,

        /// Account Grounded for bad behavior unabel to unstake
        UnableToStakeOnPunishment,

        /// Coffer balance low :cry:
        InsufficientCofferBalance,

        /// No Coffer found for the repeater
        CofferMissing,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T: Config> {
        /// The assets get staked successfully
        Staked(T::AccountId, AssetIdOf<T>, BalanceOf<T>),
        /// The derivative get unstaked successfully
        Unstaked(T::AccountId, AssetIdOf<T>, BalanceOf<T>),
        /// Stake Account  Removed
        StakeAccountRemoved(T::AccountId, AssetIdOf<T>),
        /// Register Repeater
        RepeaterRegistered(T::AccountId),
    }

    /// Global storage for relayers
    #[pallet::storage]
    #[pallet::getter(fn get_relayer)]
    pub type Relayers<T: Config> = StorageMap<_, Twox64Concat, RelayerId, Relayer<T>>;

    /// Platform's staking pool
    /// An Account can stake multiple assets
    #[pallet::storage]
    #[pallet::getter(fn staking_pool)]
    pub type StakingPool<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        AssetIdOf<T>,
        OracleDeposit,
    >;

    /// Repeaters
    #[pallet::storage]
    #[pallet::getter(fn repeaters)]
    pub type Repeaters<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Repeater>;

    #[pallet::storage]
    #[pallet::getter(fn get_manager)]
    pub type Manager<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Coffer>;

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Punish Slash !
        #[pallet::weight(T::WeightInfo::stake())]
        #[transactional]
        pub fn slash(who: OriginFor<T>, _asset: AssetIdOf<T>) -> DispatchResultWithPostInfo {
            let _who = ensure_signed(who)?;

            /*
            * Checks the number of rounds
            * Gets the minimum amount for slashing
            * Slashing_amount =
            If (OracleDeposit.total / minimum_staking_amount) * missed_rounds > OracleDeposit.total
                Then -> OracleDeposit.total - (OracleDeposit.total / minimum_staking_amount) * missed_rounds
            Else
                OracleDeposit.total -> 0
                Remove Repeater
            */
            // TODO: Implement
            Ok(().into())
        }

        /// Register Repeaters
        #[pallet::weight(T::WeightInfo::stake())]
        #[transactional]
        pub fn register_repeater(who: OriginFor<T>) -> DispatchResultWithPostInfo {
            let who = ensure_signed(who)?;

            let repeater = Self::repeaters(who.clone()).unwrap_or_default();

            ensure!(
                !Repeaters::<T>::contains_key(who.clone()),
                Error::<T>::RepeaterExists
            );

            Repeaters::<T>::insert(who.clone(), repeater);
            Self::deposit_event(Event::<T>::RepeaterRegistered(who));

            Ok(().into())
        }

        /// Stake amounts
        #[pallet::weight(T::WeightInfo::stake())]
        #[transactional]
        pub fn stake(
            who: OriginFor<T>,
            asset: AssetIdOf<T>,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(who)?;

            // Check if a repeater or else throw an error only a repeater can stake
            ensure!(
                Repeaters::<T>::contains_key(who.clone()),
                Error::<T>::RepeaterExists
            );

            // Checks for the Asset type to stake
            ensure!(
                T::StakingCurrency::get() == asset,
                Error::<T>::InvalidStakingCurrency
            );

            // Check for the minimum amount to stake
            ensure!(
                amount >= T::MinStake::get(),
                Error::<T>::InsufficientStakeAmount
            );

            let mut oracle_stake_deposit =
                Self::staking_pool(who.clone(), asset).unwrap_or_default();

            // Accumulate
            oracle_stake_deposit.total = oracle_stake_deposit
                .total
                .checked_add(amount)
                .ok_or(ArithmeticError::Underflow)?;

            oracle_stake_deposit.timestamp = T::UnixTime::now().as_secs();

            StakingPool::<T>::insert(&who, &asset, oracle_stake_deposit);

            let mut coffer = Self::get_manager(who.clone()).unwrap_or_default();

            coffer.balance = coffer
                .balance
                .checked_add(amount)
                .ok_or(ArithmeticError::Underflow)?;

            coffer.blocks_in_round = coffer
                .blocks_in_round
                .checked_add(1u128)
                .ok_or(ArithmeticError::Underflow)?;

            Manager::<T>::insert(who.clone(), coffer);

            Self::deposit_event(Event::<T>::Staked(who, asset, amount));

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
            asset: AssetIdOf<T>,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            // ***************************************
            // Check if its in the repeaters list ifnot throw an error
            // Checks for the Asset type to stake
            ensure!(
                T::StakingCurrency::get() == asset,
                Error::<T>::InvalidStakingCurrency
            );

            // TODO: Not Required? Only support full unstake?
            ensure!(
                amount > T::MinUnstake::get(),
                Error::<T>::InsufficientUnStakeAmount
            );

            StakingPool::<T>::mutate(
                who.clone(),
                asset,
                |oracle_stake_deposit| -> DispatchResultWithPostInfo {
                    let oracle_stake_deposit = oracle_stake_deposit
                        .as_mut()
                        .ok_or(Error::<T>::StakingAccountNotFound)?;

                    ensure!(
                        oracle_stake_deposit.total >= amount,
                        Error::<T>::UnstakeAmoutExceedsStakedBalance
                    );

                    if oracle_stake_deposit.total == amount {
                        StakingPool::<T>::remove(&who, &asset);

                        log::trace!(
                            target: "distributed-oracle::unstake",
                            "Account: {:?}, removed with 0 balance",
                            &who,
                        );

                        Self::deposit_event(Event::<T>::StakeAccountRemoved(who.clone(), asset));
                    }

                    oracle_stake_deposit.total = oracle_stake_deposit
                        .total
                        .checked_sub(amount)
                        .ok_or(ArithmeticError::Underflow)?;

                    oracle_stake_deposit.timestamp = T::UnixTime::now().as_secs();

                    // TODO: Handle this error properly
                    // Update the balances
                    Manager::<T>::mutate(who.clone(), |coffer| -> DispatchResult {
                        let coffer = coffer.as_mut().ok_or(Error::<T>::CofferMissing)?;

                        ensure!(
                            coffer.balance >= amount,
                            Error::<T>::InsufficientCofferBalance
                        );

                        coffer.balance = coffer
                            .balance
                            .checked_sub(amount)
                            .ok_or(ArithmeticError::Underflow)?;

                        coffer.blocks_in_round = coffer
                            .blocks_in_round
                            .checked_add(1u128)
                            .ok_or(ArithmeticError::Underflow)?;

                        Ok(())
                    })?;

                    Self::deposit_event(Event::<T>::Unstaked(who.clone(), asset, amount));

                    log::trace!(
                        target: "distributed-oracle::unstake",
                        "unstake_amount: {:?}, remaining balance: {:?}, time_stamp {:?}",
                        &amount,
                        &oracle_stake_deposit.total,
                        oracle_stake_deposit.timestamp,
                    );

                    Ok(().into())
                },
            )
        }
    }
}

impl<T: Config> Pallet<T> {
    pub fn account_id() -> AccountOf<T> {
        T::PalletId::get().into_account()
    }
}
