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
    traits::{AccountIdConversion, CheckedDiv},
    FixedU128,
};

pub use pallet::*;
use pallet_traits::*;

use orml_traits::{DataFeeder, DataProvider, DataProviderExtended};
use sp_std::prelude::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod orml_tests;

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
    use crate::helpers::{Coffer, OracleDeposit, Repeater};
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

        // Balance that parallel finance funds to pay repeaters
        #[pallet::constant]
        type Treasury: Get<BalanceOf<Self>>;
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
        /// Coffer balance low :cry:
        InsufficientCofferBalance,
        /// No Coffer found for the repeater
        CofferMissing,
        /// No rounds yet, but someone called the manager ?
        NoRoundsStartedYet,
        /// Staked Amount Is Less than Min Stake Amount
        StakedAmountIsLessThanMinStakeAmount,
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
        /// Set emergency price. \[asset_id, price_detail\]
        SetPrice(CurrencyId, Price),
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

    /// Rounds
    #[pallet::storage]
    #[pallet::getter(fn get_rounds)]
    pub type Round<T: Config> = StorageValue<_, u128>;

    #[pallet::storage]
    #[pallet::getter(fn get_manager)]
    pub type Manager<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Coffer>;

    /// Mapping from currency id to it's emergency price
    #[pallet::storage]
    #[pallet::getter(fn emergency_price)]
    pub type EmergencyPrice<T: Config> =
        StorageMap<_, Twox64Concat, CurrencyId, Price, OptionQuery>;

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Punish Slash !
        #[pallet::weight(T::WeightInfo::stake())]
        #[transactional]
        pub fn slash_staking_pool(
            origin: OriginFor<T>,
            asset: AssetIdOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            let now = T::UnixTime::now().as_secs();

            // Cannot slash a non repeater
            ensure!(
                Repeaters::<T>::contains_key(who.clone()),
                Error::<T>::InvalidStaker
            );

            ensure!(
                Manager::<T>::contains_key(&Self::account_id()),
                Error::<T>::NoRoundsStartedYet
            );

            StakingPool::<T>::mutate(
                who.clone(),
                asset,
                |oracle_stake_deposit| -> DispatchResultWithPostInfo {
                    let oracle_stake_deposit = oracle_stake_deposit
                        .as_mut()
                        .ok_or(Error::<T>::StakingAccountNotFound)?;

                    // Calculates time delta
                    let time_delta = now
                        .checked_sub(oracle_stake_deposit.timestamp)
                        .ok_or(Error::<T>::StakingAccountNotFound)?;

                    // Slash if the time diff is more than half an hour
                    // slash_amount = (OracleDeposit.total / minimum_staking_amount) * missed_unix_time_stamp) / 100
                    // If slash_amount >= OracleDeposit.total
                    //    Then -> OracleDeposit.total - slash_amount
                    // Else
                    //    OracleDeposit.total -> 0
                    //    Remove Repeater
                    if time_delta > T::MinSlashedTime::get() {
                        let slash_amount = oracle_stake_deposit
                            .total
                            .checked_div(T::MinStake::get())
                            .and_then(|r| r.checked_mul(time_delta as u128))
                            .and_then(|r| r.checked_sub(100u128))
                            .ok_or(ArithmeticError::Underflow)?;

                        if slash_amount >= oracle_stake_deposit.total {
                            oracle_stake_deposit.total = oracle_stake_deposit
                                .total
                                .checked_sub(slash_amount)
                                .ok_or(ArithmeticError::Underflow)?;

                            Self::deposit_event(Event::<T>::Slashed(who.clone()));

                            log::trace!(
                                target: "distributed-oracle::slash",
                                "Slashed Account {:?} slashed_amount {:?}",
                                &who.clone(),
                                &slash_amount,
                            );
                        } else {
                            oracle_stake_deposit.total = Zero::zero();
                            Repeaters::<T>::remove(who.clone());
                            Self::deposit_event(Event::<T>::SlashedandsRemoved(who.clone()));

                            log::trace!(
                                target: "distributed-oracle::slash",
                                "Account {:?} got slashed and removed due to unavailability of \
                                funds slashed_amount {:?}",
                                &who.clone(),
                                &slash_amount,
                            );
                        }
                    }
                    Ok(().into())
                },
            )
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

            // Rewards
            // repeater.balance / staker_time_stamp * 100
            Repeaters::<T>::mutate(who.clone(), |repeater| -> DispatchResultWithPostInfo {
                let repeater = repeater.as_mut().ok_or(Error::<T>::InvalidStaker)?;

                repeater.staked_balance = oracle_stake_deposit.total;

                let reward = repeater
                    .staked_balance
                    .checked_div(current_time_stamp as u128)
                    .and_then(|r| r.checked_div(100_000_000u128))
                    .ok_or(ArithmeticError::Underflow)?;

                repeater.reward = repeater
                    .reward
                    .checked_add(reward)
                    .ok_or(ArithmeticError::Underflow)?;

                Ok(().into())
            })?;

            StakingPool::<T>::insert(&who, &asset, oracle_stake_deposit);

            // manager has a coffer which stores balances and rounds
            // TODO: We might need to use mutate rather than inserting here
            let mut coffer = Self::get_manager(&Self::account_id()).unwrap_or_default();

            coffer.balance = coffer
                .balance
                .checked_add(amount)
                .ok_or(ArithmeticError::Underflow)?;

            coffer.blocks_in_round = coffer
                .blocks_in_round
                .checked_add(1u128)
                .ok_or(ArithmeticError::Underflow)?;

            Manager::<T>::insert(Self::account_id(), coffer);

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

                    // TODO: Handle this error properly
                    // Update the balances -> remove unstake amount from balances
                    Manager::<T>::mutate(Self::account_id(), |coffer| -> DispatchResult {
                        let coffer = coffer.as_mut().ok_or(Error::<T>::CofferMissing)?;

                        ensure!(
                            coffer.balance >= amount,
                            Error::<T>::InsufficientCofferBalance
                        );

                        // Deduct balance from unstaked amount
                        coffer.balance = coffer
                            .balance
                            .checked_sub(amount)
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

        // pub fn set_price_in_round() {
        //     //
        // }

        /// Set emergency price
        #[pallet::weight((<T as Config>::WeightInfo::set_price(), DispatchClass::Operational))]
        #[transactional]
        pub fn set_price(
            origin: OriginFor<T>,
            asset_id: CurrencyId,
            price: Price,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(
                Self::staking_pool(who, T::StakingCurrency::get())
                    .unwrap_or_default()
                    .total
                    > T::MinUnstake::get(),
                Error::<T>::StakedAmountIsLessThanMinStakeAmount
            );
            <Pallet<T> as EmergencyPriceFeeder<CurrencyId, Price>>::set_emergency_price(
                asset_id, price,
            );
            Ok(().into())
        }

        /// Reset emergency price
        #[pallet::weight((<T as Config>::WeightInfo::reset_price(), DispatchClass::Operational))]
        #[transactional]
        pub fn reset_price(
            origin: OriginFor<T>,
            asset_id: CurrencyId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(
                Self::staking_pool(who, T::StakingCurrency::get())
                    .unwrap_or_default()
                    .total
                    > T::MinUnstake::get(),
                Error::<T>::StakedAmountIsLessThanMinStakeAmount
            );
            <Pallet<T> as EmergencyPriceFeeder<CurrencyId, Price>>::reset_emergency_price(asset_id);
            Ok(().into())
        }
    }
}

impl<T: Config> Pallet<T> {
    pub fn account_id() -> AccountOf<T> {
        T::PalletId::get().into_account()
    }

    // get emergency price, the timestamp is zero
    fn get_emergency_price(asset_id: &CurrencyId) -> Option<PriceDetail> {
        Self::emergency_price(asset_id).and_then(|p| {
            let mantissa = Self::get_asset_mantissa(asset_id)?;
            log::trace!(
                target: "price::get_emergency_price",
                "mantissa: {:?}",
                mantissa
            );
            p.checked_div(&FixedU128::from_inner(mantissa))
                .map(|price| (price, 0))
        })
    }

    fn get_asset_mantissa(asset_id: &CurrencyId) -> Option<u128> {
        let decimal = T::Decimal::get_decimal(asset_id)?;
        10u128.checked_pow(decimal as u32)
    }
}

impl<T: Config> PriceFeeder for Pallet<T> {
    /// Returns the uniform format price and timestamp by asset id.
    /// Formula: `price = oracle_price * 10.pow(18 - asset_decimal)`
    /// We use `oracle_price.checked_div(&FixedU128::from_inner(mantissa))` represent that.
    /// This particular price makes it easy to calculate the asset value in other pallets,
    /// because we don't have to consider decimal for each asset.
    ///
    /// Timestamp is zero means the price is emergency price
    fn get_price(asset_id: &CurrencyId) -> Option<PriceDetail> {
        // if emergency price exists, return it, otherwise return latest price from oracle.
        Self::get_emergency_price(asset_id).or_else(|| {
            let mantissa = Self::get_asset_mantissa(asset_id)?;
            T::Source::get(asset_id).and_then(|p| {
                p.value
                    .checked_div(&FixedU128::from_inner(mantissa))
                    .map(|price| (price, p.timestamp))
            })
        })
    }
}

impl<T: Config> EmergencyPriceFeeder<CurrencyId, Price> for Pallet<T> {
    /// Set emergency price
    fn set_emergency_price(asset_id: CurrencyId, price: Price) {
        // set price direct
        EmergencyPrice::<T>::insert(asset_id, price);
        <Pallet<T>>::deposit_event(Event::SetPrice(asset_id, price));
    }

    /// Reset emergency price
    fn reset_emergency_price(asset_id: CurrencyId) {
        EmergencyPrice::<T>::remove(asset_id);
        <Pallet<T>>::deposit_event(Event::ResetPrice(asset_id));
    }
}

impl<T: Config> DataProviderExtended<CurrencyId, TimeStampedPrice> for Pallet<T> {
    fn get_no_op(asset_id: &CurrencyId) -> Option<TimeStampedPrice> {
        T::Source::get_no_op(asset_id)
    }

    fn get_all_values() -> Vec<(CurrencyId, Option<TimeStampedPrice>)> {
        T::Source::get_all_values()
    }
}
