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
    ArithmeticError, FixedU128,
};
use std::collections::BTreeMap;

pub use pallet::*;
use pallet_traits::*;

use orml_traits::{DataFeeder, DataProvider, DataProviderExtended};
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

pub use weights::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use crate::helpers::{Repeater, RoundHolder, RoundManager};
    use sp_runtime::traits::Zero;
    use sp_runtime::ArithmeticError;
    use std::collections::BTreeMap;

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

        // Unix time gap between round this has to be twice the value of a round
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
        InvalidRepeater,
        /// Only a repeater can unstake,
        InvalidUnstaker,
        /// Slashing Failure
        SlashFailure,
        /// Treasury ran out of funds
        RewardFailureTreasuryRunningLow,
        /// Staked Amount Is Less than Min Stake Amount
        StakedAmountIsLessThanMinStakeAmount,
        /// PriceSubmittedAlready
        AccountAlreadySubmittedPrice,
        /// Current Price nort found
        CurrentRoundNotFound,
        /// Rewarding Account not found
        RewardingAccountNotFound,
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
        /// Reset Price
        ResetPrice(CurrencyId, u128),
    }

    /// Repeaters
    #[pallet::storage]
    #[pallet::getter(fn repeaters)]
    pub type Repeaters<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        AssetIdOf<T>,
        Repeater,
    >;

    ///  Treasury Balance, pre-populate from pallet runtime constant
    #[pallet::storage]
    #[pallet::getter(fn get_treasury)]
    pub type OracleTreasury<T: Config> = StorageValue<_, BalanceOf<T>>;

    /// Rounds
    #[pallet::storage]
    #[pallet::getter(fn get_rounds)]
    pub type Round<T: Config> = StorageValue<_, u128>;

    #[pallet::storage]
    #[pallet::getter(fn manager)]
    pub type Manager<T: Config> = StorageValue<_, RoundManager<T>>;

    /// Holds the average price per round
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
            log::trace!(
                target: "distributed-oracle::populate_treasury",
                "Treasury Populated with the amount :- {:?}",
                T::Treasury::get(),
            );

            Ok(().into())
        }

        /// Reset price per round for a give asset_id
        #[pallet::weight(T::WeightInfo::stake())]
        pub fn reset_prices(
            origin: OriginFor<T>,
            asset_id: AssetIdOf<T>,
            round: u128,
        ) -> DispatchResultWithPostInfo {
            let _ = ensure_signed(origin)?;

            CurrentRound::<T>::mutate(asset_id, round, |rec| -> DispatchResultWithPostInfo {
                let mut rec = rec.as_mut().ok_or(Error::<T>::CurrentRoundNotFound)?;

                rec.mean_price = FixedU128::from_inner(0u128);
                rec.agg_price = FixedU128::from_inner(0u128);
                rec.submitters = BTreeMap::new();
                rec.submitter_count = Zero::zero();

                Self::deposit_event(Event::<T>::ResetPrice(asset_id, round));

                log::trace!(
                    target: "distributed-oracle::reset_prices",
                    "Price reset for the round :- {:?} \n asset_id {:?}",
                    round,
                    asset_id
                );

                Ok(().into())
            })
        }

        /// Register Repeaters
        #[pallet::weight(T::WeightInfo::stake())]
        #[transactional]
        pub fn register_repeater(
            who: OriginFor<T>,
            asset_id: AssetIdOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(who)?;

            ensure!(
                !Repeaters::<T>::contains_key(who.clone(), asset_id),
                Error::<T>::RepeaterExists
            );

            // Initialize a repeater structure
            Repeaters::<T>::insert(who.clone(), asset_id, Repeater::default());

            Self::deposit_event(Event::<T>::RepeaterRegistered(who));

            Ok(().into())
        }

        /// Stake amounts
        #[pallet::weight(T::WeightInfo::stake())]
        #[transactional]
        pub fn stake(
            who: OriginFor<T>,
            asset_id: AssetIdOf<T>,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(who)?;

            if !Repeaters::<T>::contains_key(who.clone(), asset_id) {
                Repeaters::<T>::insert(
                    who.clone(),
                    asset_id,
                    Self::repeaters(who.clone(), asset_id).unwrap_or_default(),
                );
            }

            // Checks for the Asset type to stake
            ensure!(
                T::StakingCurrency::get() == asset_id,
                Error::<T>::InvalidStakingCurrency
            );

            // Check for the minimum amount to stake
            ensure!(
                amount >= T::MinStake::get(),
                Error::<T>::InsufficientStakeAmount
            );

            Repeaters::<T>::mutate(
                who.clone(),
                asset_id,
                |repeater| -> DispatchResultWithPostInfo {
                    let repeater = repeater.as_mut().ok_or(Error::<T>::InvalidRepeater)?;

                    repeater.staked_balance = repeater
                        .staked_balance
                        .checked_add(amount)
                        .ok_or(ArithmeticError::Underflow)?;

                    Ok(().into())
                },
            )?;

            Self::deposit_event(Event::<T>::Staked(who, asset_id, amount));

            log::trace!(
                target: "distributed-oracle::stake",
                "stake_amount: {:?}",
                &amount,
            );

            Ok(().into())
        }

        /// Unstake
        #[pallet::weight(T::WeightInfo::unstake())]
        pub fn unstake(
            origin: OriginFor<T>,
            asset_id: AssetIdOf<T>,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            // InvalidUnstaker
            ensure!(
                Repeaters::<T>::contains_key(who.clone(), asset_id),
                Error::<T>::InvalidUnstaker
            );

            ensure!(
                amount > T::MinUnstake::get(),
                Error::<T>::InsufficientUnStakeAmount
            );

            Repeaters::<T>::mutate(
                who.clone(),
                asset_id,
                |repeater| -> DispatchResultWithPostInfo {
                    let repeater = repeater.as_mut().ok_or(Error::<T>::InvalidRepeater)?;

                    ensure!(
                        repeater.staked_balance >= amount,
                        Error::<T>::UnstakeAmoutExceedsStakedBalance
                    );

                    if repeater.staked_balance == amount {
                        Repeaters::<T>::remove(&who, &asset_id);

                        log::trace!(
                            target: "distributed-oracle::unstake",
                            "Repeater with Account: {:?}, removed with 0 balance",
                            &who,
                        );

                        Self::deposit_event(Event::<T>::StakeAccountRemoved(who.clone(), asset_id));
                    } else {
                        repeater.staked_balance = repeater
                            .staked_balance
                            .checked_sub(amount)
                            .ok_or(ArithmeticError::Underflow)?;
                    }

                    Self::deposit_event(Event::<T>::Unstaked(who.clone(), asset_id, amount));

                    Ok(().into())
                },
            )
        }

        /// Set emergency price
        #[pallet::weight((<T as Config>::WeightInfo::set_price(), DispatchClass::Operational))]
        #[transactional]
        pub fn set_price_for_round(
            origin: OriginFor<T>,
            asset_id: AssetIdOf<T>,
            price: Price,
            round: u128,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            let current_time_stamp = T::UnixTime::now().as_secs();

            let mut recent_round = Self::get_current_round(asset_id, round).unwrap_or_default();

            ensure!(
                !recent_round.submitters.contains_key(&who),
                Error::<T>::AccountAlreadySubmittedPrice
            );

            ensure!(
                Self::repeaters(who.clone(), T::StakingCurrency::get())
                    .unwrap_or_default()
                    .staked_balance
                    > T::MinUnstake::get(),
                Error::<T>::InvalidRepeater
            );

            let prev_round = Self::get_current_round(asset_id, round - 1).unwrap_or_default();

            let mut round_agg_price = recent_round.agg_price;
            let mut round_mean_price = recent_round.mean_price;
            // New round, brings the values from previous round
            if recent_round.agg_price == Zero::zero() {
                round_agg_price = prev_round.agg_price;
                round_mean_price = prev_round.mean_price;
            }

            let mut round_manager = Manager::<T>::get().unwrap_or_default();

            if !round_manager.participated.contains_key(&who) {
                round_manager.participated.insert(who.clone(), round);
            }

            if !round_manager.people_to_reward.contains_key(&who) {
                round_manager.people_to_reward.insert(who.clone(), round);
            }
            Manager::<T>::put(round_manager.clone());

            // Begins a  new round
            // if its Zero, its the beginning
            if round_agg_price == Zero::zero() {
                recent_round
                    .submitters
                    .insert(who.clone(), (price, current_time_stamp));

                CurrentRound::<T>::insert(
                    asset_id,
                    round,
                    RoundHolder {
                        agg_price: price,
                        mean_price: price,
                        round_started_time: current_time_stamp,
                        submitters: recent_round.submitters,
                        submitter_count: 1u32,
                    },
                );

                if round > 1 {
                    if prev_round.submitters.contains_key(&who) {
                        Self::do_reward(
                            who,
                            asset_id,
                            T::RewardAmount::get(),
                            round_manager.people_to_reward.clone(),
                        )
                        .unwrap();
                    } else {
                        let participated_round = round_manager.participated.get(&who).unwrap();
                        let round_gap = round - *participated_round;
                        if round_gap > 0 {
                            round_manager.people_to_slash.insert(who.clone(), round);
                            Self::do_slash(who.clone(), asset_id, T::RewardAmount::get()).unwrap();
                        }
                    }
                    Manager::<T>::put(round_manager.clone());
                }
            } else {
                // Threshold price is +/- 50 of the current price
                let price_lower_limit = round_mean_price
                    .checked_div(&FixedU128::from(2u128))
                    .ok_or(ArithmeticError::Underflow)?;

                let price_upper_limit = round_mean_price
                    .checked_div(&FixedU128::from(2u128))
                    .and_then(|r| r.checked_mul(&FixedU128::from(3u128)))
                    .ok_or(ArithmeticError::Underflow)?;

                if price >= price_lower_limit && price <= price_upper_limit {
                    recent_round
                        .submitters
                        .insert(who.clone(), (price, current_time_stamp));

                    recent_round.submitter_count = recent_round
                        .submitter_count
                        .checked_add(1u32)
                        .ok_or(ArithmeticError::Underflow)?;

                    let agg_price = recent_round
                        .agg_price
                        .checked_add(&price)
                        .ok_or(ArithmeticError::Underflow)?;

                    let mean_price = agg_price
                        .clone()
                        .checked_div(&FixedU128::from(recent_round.submitter_count as u128))
                        .ok_or(ArithmeticError::Underflow)?;

                    if !CurrentRound::<T>::contains_key(asset_id, round) {
                        CurrentRound::<T>::insert(
                            asset_id,
                            round,
                            RoundHolder {
                                agg_price,
                                mean_price,
                                round_started_time: current_time_stamp,
                                submitters: recent_round.submitters,
                                submitter_count: recent_round.submitter_count,
                            },
                        );

                        if round > 1 {
                            if prev_round.submitters.contains_key(&who) {
                                Self::do_reward(
                                    who,
                                    asset_id,
                                    T::RewardAmount::get(),
                                    round_manager.people_to_reward.clone(),
                                )
                                .unwrap();
                            } else {
                                let participant_round =
                                    round_manager.participated.get(&who).unwrap();
                                let round_gap = round - *participant_round;

                                if round_gap > 0 {
                                    Self::do_slash(who.clone(), asset_id, T::RewardAmount::get())
                                        .unwrap();
                                }
                            }
                            Manager::<T>::put(round_manager.clone());
                        }
                    } else {
                        CurrentRound::<T>::mutate(asset_id, round, |rec| -> DispatchResult {
                            let mut rec = rec.as_mut().ok_or(Error::<T>::CurrentRoundNotFound)?;

                            rec.agg_price = agg_price;
                            rec.mean_price = mean_price;
                            rec.submitters = recent_round.submitters;
                            rec.submitter_count = recent_round.submitter_count;

                            if round > 1 {
                                if prev_round.submitters.contains_key(&who) {
                                    Self::do_reward(
                                        who.clone(),
                                        asset_id,
                                        T::RewardAmount::get(),
                                        round_manager.people_to_reward.clone(),
                                    )
                                    .unwrap();
                                } else {
                                    let participant_round =
                                        round_manager.participated.get(&who).unwrap();
                                    let round_gap = round - *participant_round;
                                    if round_gap > 0 && !prev_round.submitters.contains_key(&who) {
                                        Self::do_slash(
                                            who.clone(),
                                            asset_id,
                                            T::RewardAmount::get(),
                                        )
                                        .unwrap();
                                    }
                                }
                                Manager::<T>::put(round_manager.clone());
                            }
                            Ok(())
                        })?;
                    }
                } else {
                    round_manager.people_to_slash.insert(who.clone(), round);
                    Self::do_slash(who, asset_id, T::RewardAmount::get()).unwrap();
                }
            }

            Self::deposit_event(Event::SetPrice(asset_id, price, round));

            Ok(().into())
        }
    }
}

impl<T: Config> Pallet<T> {
    pub fn account_id() -> AccountOf<T> {
        T::PalletId::get().into_account()
    }

    pub fn do_reward(
        who: AccountOf<T>,
        asset_id: AssetIdOf<T>,
        reward_amount: BalanceOf<T>,
        mut people_to_reward: BTreeMap<AccountOf<T>, u128>,
    ) -> Result<(), DispatchError> {
        // Remove rewarded accounts
        let mut round_manager = Manager::<T>::get().ok_or(Error::<T>::RewardingAccountNotFound)?;
        people_to_reward.remove(&who);
        round_manager.people_to_reward = people_to_reward;
        Manager::<T>::put(round_manager);
        Repeaters::<T>::mutate(who.clone(), asset_id, |repeater| -> DispatchResult {
            let repeater = repeater.as_mut().ok_or(Error::<T>::InvalidRepeater)?;

            // Adds rewards to the staked amount, accumulating
            repeater.staked_balance = repeater
                .staked_balance
                .checked_add(reward_amount)
                .ok_or(ArithmeticError::Underflow)?;

            // accumulate reward balance
            repeater.reward = repeater
                .reward
                .checked_add(reward_amount)
                .ok_or(ArithmeticError::Underflow)?;

            let new_treasury_balance = OracleTreasury::<T>::get()
                .unwrap_or_default()
                .checked_sub(reward_amount)
                .ok_or(ArithmeticError::Underflow)?;

            OracleTreasury::<T>::put(new_treasury_balance);

            Ok(())
        })
    }

    pub fn do_slash(
        who: AccountOf<T>,
        asset_id: AssetIdOf<T>,
        slash_amount: BalanceOf<T>,
    ) -> Result<(), DispatchError> {
        // Check if it has to remove
        let mut round_manager = Manager::<T>::get().ok_or(Error::<T>::RewardingAccountNotFound)?;
        if round_manager.people_to_reward.contains_key(&who) {
            Self::do_reward(
                who.clone(),
                asset_id,
                slash_amount,
                round_manager.people_to_reward.clone(),
            )
            .unwrap();
        }
        round_manager.people_to_slash.remove(&who);
        Manager::<T>::put(round_manager);

        Repeaters::<T>::mutate(who.clone(), asset_id, |repeater| -> DispatchResult {
            let repeater = repeater.as_mut().ok_or(Error::<T>::InvalidRepeater)?;

            if repeater.staked_balance != 0 {
                repeater.staked_balance = repeater
                    .staked_balance
                    .checked_sub(slash_amount)
                    .ok_or(ArithmeticError::Underflow)?;

                if repeater.reward > 0 {
                    repeater.reward = repeater
                        .reward
                        .checked_sub(slash_amount)
                        .ok_or(ArithmeticError::Underflow)?;
                }
            } else {
                Repeaters::<T>::remove(who.clone(), asset_id);
            }

            let new_treasury_balance = OracleTreasury::<T>::get()
                .unwrap_or_default()
                .checked_add(slash_amount)
                .ok_or(ArithmeticError::Underflow)?;

            OracleTreasury::<T>::put(new_treasury_balance);

            Ok(())
        })
    }
}
