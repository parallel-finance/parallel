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
use sp_runtime::{
    traits::{AccountIdConversion, CheckedAdd, CheckedDiv, CheckedMul},
    FixedU128,
};

pub use pallet::*;
use pallet_traits::*;

use orml_traits::{DataFeeder, DataProvider, DataProviderExtended};
use sp_std::prelude::*;

#[cfg(test)]
mod mock;

// #[cfg(test)]
// mod orml_tests;

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
    use crate::helpers::{OracleDeposit, Repeater, RoundHolder, RoundManager};
    use sp_runtime::traits::Zero;
    use sp_runtime::ArithmeticError;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The data source, such as Oracle.
        type Source: DataProvider<CurrencyId, TimeStampedPrice>
            + DataProviderExtended<CurrencyId, TimeStampedPrice>
            + DataFeeder<CurrencyId, TimeStampedPrice, Self::AccountId>;

        /// The origin which may set prices feed to system.
        type FeederOrigin: EnsureOrigin<Self::Origin>;

        /// Decimal provider.
        type Decimal: DecimalProvider<CurrencyId>;

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

        #[pallet::constant]
        type MinSlashedTime: Get<u64>;

        // Balance that parallel finance funds to pay repeaters , prep populated value
        #[pallet::constant]
        type Treasury: Get<BalanceOf<Self>>;

        // Unix time gap between round
        #[pallet::constant]
        type RoundDuration: Get<u64>;

        #[pallet::constant]
        type RewardAmount: Get<u128>;

        #[pallet::constant]
        type SlashAmount: Get<u128>;
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
        /// Account Grounded for bad behavior unable to unstake
        UnableToStakeOnPunishment,
        /// No rounds yet, but someone called the manager ?
        NoRoundsStartedYet,
        /// Staked Amount Is Less than Min Stake Amount
        StakedAmountIsLessThanMinStakeAmount,
        /// PriceSubmittedAlready
        AccountAlreadySubmittedPrice,
        /// Current Price nort found
        CurrentRoundNotFound,
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
        /// Slashed
        Slashed(T::AccountId),
        /// Slashed and Removed
        SlashedandsRemoved(T::AccountId),
        /// Set emergency price Asset Price, Round number
        SetPrice(CurrencyId, Price, RoundNumber),
        /// Reset emergency price. \[asset_id\]
        ResetPrice(CurrencyId),
    }

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

    ///  Treasury Balance, pre-populate from pallet runtime constant
    #[pallet::storage]
    #[pallet::getter(fn get_treasury)]
    pub type OracleTreasury<T: Config> = StorageValue<_, BalanceOf<T>>;

    /// Rounds
    #[pallet::storage]
    #[pallet::getter(fn get_rounds)]
    pub type Round<T: Config> = StorageValue<_, u128>;

    #[pallet::storage]
    #[pallet::getter(fn get_round_manager)]
    pub type Manager<T: Config> = StorageValue<_, RoundManager<T>>;

    #[pallet::storage]
    #[pallet::getter(fn get_current_round)]
    pub type CurrentRound<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        CurrencyId,
        Blake2_128Concat,
        RoundNumber,
        RoundHolder<T>,
        OptionQuery,
    >;

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Populates the Oracle's Treasury from Pallet's Treasury Constant
        /// Should execute once
        #[pallet::weight(T::WeightInfo::stake())]
        #[transactional]
        pub fn populate_treasury(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            ensure_signed(origin)?;

            OracleTreasury::<T>::put(T::Treasury::get());

            Ok(().into())
        }

        /// Register Repeaters
        #[pallet::weight(T::WeightInfo::stake())]
        #[transactional]
        pub fn register_repeater(who: OriginFor<T>) -> DispatchResultWithPostInfo {
            let who = ensure_signed(who)?;

            ensure!(
                !Repeaters::<T>::contains_key(who.clone()),
                Error::<T>::RepeaterExists
            );

            // Initialize a repeater structure
            Repeaters::<T>::insert(
                who.clone(),
                Self::repeaters(who.clone()).unwrap_or_default(),
            );

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
            let current_time_stamp = T::UnixTime::now().as_secs();
            // Only repeaters can stake
            ensure!(
                Repeaters::<T>::contains_key(who.clone()),
                Error::<T>::InvalidStaker
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

            oracle_stake_deposit.timestamp = current_time_stamp;

            oracle_stake_deposit.blocks_in_round = oracle_stake_deposit
                .blocks_in_round
                .checked_add(1u128)
                .ok_or(ArithmeticError::Underflow)?;

            Repeaters::<T>::mutate(who.clone(), |repeater| -> DispatchResultWithPostInfo {
                let repeater = repeater.as_mut().ok_or(Error::<T>::InvalidStaker)?;

                repeater.staked_balance = oracle_stake_deposit.total;

                Ok(().into())
            })?;

            StakingPool::<T>::insert(&who, &asset, oracle_stake_deposit);

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
            // InvalidUnstaker
            ensure!(
                Repeaters::<T>::contains_key(who.clone()),
                Error::<T>::InvalidUnstaker
            );

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

        /// Set emergency price
        #[pallet::weight((<T as Config>::WeightInfo::set_price(), DispatchClass::Operational))]
        #[transactional]
        pub fn set_price_for_round(
            origin: OriginFor<T>,
            asset_id: CurrencyId,
            price: Price,
            round: u128,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let current_time_stamp = T::UnixTime::now().as_secs();

            ensure!(
                Self::staking_pool(who.clone(), T::StakingCurrency::get())
                    .unwrap_or_default()
                    .total
                    > T::MinUnstake::get(),
                Error::<T>::StakedAmountIsLessThanMinStakeAmount
            );

            let mut recent_round = Self::get_current_round(asset_id, round).unwrap_or_default();

            ensure!(
                !recent_round.submitters.contains_key(&who),
                Error::<T>::AccountAlreadySubmittedPrice
            );

            let mut round_manager = Manager::<T>::get().unwrap_or_default();

            round_manager
                .participated
                .insert(who.clone(), current_time_stamp);

            // TODO: Check slashing
            // * Time diff
            // * Reward good actors -> Tak from Treasury
            // * Slash bad actors -> give back to treasury
            // * Slash bad actors -> not participating for N amount of time -> give back to treasury
            // * Update tests

            // New round , no one has submitted any thing
            if recent_round.avg_price == Zero::zero() {
                round_manager
                    .people_to_reward
                    .insert(who.clone(), current_time_stamp);

                // submitters.insert(who.clone(), (price, current_time_stamp));
                recent_round
                    .submitters
                    .insert(who.clone(), (price, current_time_stamp));
                let round_holder = RoundHolder {
                    // The average price
                    avg_price: price,
                    // list of submitters -> <key: account_id, value: (submitted_price, timestamp)>
                    round_started_time: current_time_stamp,
                    submitters: recent_round.submitters,
                };

                // first round first time
                CurrentRound::<T>::insert(asset_id, round, round_holder);

                // ********************************************************************************
                Repeaters::<T>::mutate(who.clone(), |repeater| -> DispatchResultWithPostInfo {
                    let repeater = repeater.as_mut().ok_or(Error::<T>::InvalidStaker)?;

                    // Adds reward `1 units` to the total balance
                    repeater.staked_balance = repeater
                        .staked_balance
                        .checked_add(1u128)
                        .ok_or(ArithmeticError::Underflow)?;

                    // amount collected as rewards
                    repeater.reward = repeater
                        .reward
                        .checked_add(T::RewardAmount::get())
                        .ok_or(ArithmeticError::Underflow)?;

                    // Decrement Treasury Balance
                    let new_treasury_balance = OracleTreasury::<T>::get()
                        .unwrap_or_default()
                        .checked_sub(T::SlashAmount::get())
                        .ok_or(ArithmeticError::Underflow)?;

                    OracleTreasury::<T>::put(new_treasury_balance);

                    Ok(().into())
                })?;
                // ********************************************************************************
            } else {
                // Threshold price is +/- 50 of the current price
                let price_lower_limit = recent_round
                    .avg_price
                    .checked_div(&FixedU128::from_inner(2u128))
                    .ok_or(ArithmeticError::Underflow)?;

                let price_upper_limit = recent_round
                    .avg_price
                    .checked_div(&FixedU128::from_inner(2u128))
                    .and_then(|r| r.checked_mul(&FixedU128::from_inner(3u128)))
                    .ok_or(ArithmeticError::Underflow)?;

                if price >= price_lower_limit && price <= price_upper_limit {
                    recent_round
                        .submitters
                        .insert(who.clone(), (price, current_time_stamp));

                    let avg_price = recent_round
                        .avg_price
                        .checked_add(&price)
                        .and_then(|r| {
                            r.checked_div(&FixedU128::from_inner(
                                recent_round.submitters.len() as u128
                            ))
                        })
                        .ok_or(ArithmeticError::Underflow)?;

                    CurrentRound::<T>::mutate(asset_id, round, |rec| -> DispatchResult {
                        let mut rec = rec.as_mut().ok_or(Error::<T>::CurrentRoundNotFound)?;

                        rec.avg_price = avg_price;
                        rec.submitters = recent_round.submitters;

                        if current_time_stamp - rec.round_started_time <= T::RoundDuration::get() {
                            round_manager
                                .people_to_reward
                                .insert(who.clone(), current_time_stamp);

                            // Adds reward
                            // ********************************************************************************
                            Repeaters::<T>::mutate(who.clone(), |repeater| -> DispatchResult {
                                let repeater =
                                    repeater.as_mut().ok_or(Error::<T>::InvalidStaker)?;

                                // Adds reward `1 units` to the total balance
                                repeater.staked_balance = repeater
                                    .staked_balance
                                    .checked_add(1u128)
                                    .ok_or(ArithmeticError::Underflow)?;

                                // amount collected as rewards
                                repeater.reward = repeater
                                    .reward
                                    .checked_add(1u128)
                                    .ok_or(ArithmeticError::Underflow)?;

                                // Decrement Treasury to give rewards
                                let new_treasury_balance = OracleTreasury::<T>::get()
                                    .unwrap_or_default()
                                    .checked_sub(1u128)
                                    .ok_or(ArithmeticError::Underflow)?;

                                OracleTreasury::<T>::put(new_treasury_balance);

                                Ok(())
                            })?;
                            // ********************************************************************************

                            //
                        } else {
                            // Reward
                            round_manager
                                .people_to_slash
                                .insert(who.clone(), current_time_stamp);

                            // ********************************************************************************
                            Repeaters::<T>::mutate(who.clone(), |repeater| -> DispatchResult {
                                let repeater =
                                    repeater.as_mut().ok_or(Error::<T>::InvalidStaker)?;

                                // Adds reward `1 units` to the total balance
                                repeater.staked_balance = repeater
                                    .staked_balance
                                    .checked_sub(1u128)
                                    .ok_or(ArithmeticError::Underflow)?;

                                // amount collected as rewards
                                repeater.reward = repeater
                                    .reward
                                    .checked_sub(1u128)
                                    .ok_or(ArithmeticError::Underflow)?;

                                // Increment Treasury to give rewards
                                let new_treasury_balance = OracleTreasury::<T>::get()
                                    .unwrap_or_default()
                                    .checked_add(1u128)
                                    .ok_or(ArithmeticError::Underflow)?;

                                OracleTreasury::<T>::put(new_treasury_balance);

                                Ok(())
                            })?;
                        }

                        Ok(())
                    })?;
                } else {
                    round_manager
                        .people_to_slash
                        .insert(who.clone(), current_time_stamp);
                }
            }

            Manager::<T>::put(round_manager);
            Self::deposit_event(Event::SetPrice(asset_id, price, round));

            Ok(().into())
        }
    }
}

impl<T: Config> Pallet<T> {
    pub fn account_id() -> AccountOf<T> {
        T::PalletId::get().into_account()
    }
}
