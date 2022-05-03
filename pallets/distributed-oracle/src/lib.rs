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
    traits::{AccountIdConversion, CheckedDiv, CheckedMul},
    FixedU128,
};

pub use pallet::*;
use pallet_traits::*;

use orml_traits::{DataFeeder, DataProvider, DataProviderExtended};
use sp_std::prelude::*;

#[cfg(test)]
mod mock;

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
    use crate::helpers::{OracleDeposit, Relayer, Repeater};
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

        /// Liquid currency & staking currency provider
        type LiquidStakingCurrenciesProvider: LiquidStakingCurrenciesProvider<CurrencyId>;

        /// The provider of the exchange rate between liquid currency and
        /// staking currency.
        type LiquidStakingExchangeRateProvider: ExchangeRateProvider;

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
        /// Set emergency price. \[asset_id, price_detail\]
        SetPrice(CurrencyId, Price),
        /// Reset emergency price. \[asset_id\]
        ResetPrice(CurrencyId),
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
        /// Stake amounts
        #[pallet::weight(T::WeightInfo::stake())]
        #[transactional]
        pub fn stake(
            who: OriginFor<T>,
            asset: AssetIdOf<T>,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(who)?;

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
        pub fn set_price(
            origin: OriginFor<T>,
            asset_id: CurrencyId,
            price: Price,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(
                Self::staking_pool(who.clone(), T::StakingCurrency::get())
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
                Self::staking_pool(who.clone(), T::StakingCurrency::get())
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
            match T::LiquidStakingCurrenciesProvider::get_staking_currency()
                .zip(T::LiquidStakingCurrenciesProvider::get_liquid_currency())
            {
                Some((staking_currency, liquid_currency)) if asset_id == &liquid_currency => {
                    T::Source::get(&staking_currency).and_then(|p| {
                        p.value
                            .checked_div(&FixedU128::from_inner(mantissa))
                            .and_then(|staking_currency_price| {
                                staking_currency_price.checked_mul(
                                    &T::LiquidStakingExchangeRateProvider::get_exchange_rate(),
                                )
                            })
                            .map(|price| (price, p.timestamp))
                    })
                }
                _ => T::Source::get(asset_id).and_then(|p| {
                    p.value
                        .checked_div(&FixedU128::from_inner(mantissa))
                        .map(|price| (price, p.timestamp))
                }),
            }
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
        match T::LiquidStakingCurrenciesProvider::get_staking_currency()
            .zip(T::LiquidStakingCurrenciesProvider::get_liquid_currency())
        {
            Some((staking_currency, liquid_currency)) if &liquid_currency == asset_id => {
                T::Source::get_no_op(&staking_currency).and_then(|p| {
                    p.value
                        .checked_mul(&T::LiquidStakingExchangeRateProvider::get_exchange_rate())
                        .map(|price| TimeStampedPrice {
                            value: price,
                            timestamp: p.timestamp,
                        })
                })
            }
            _ => T::Source::get_no_op(asset_id),
        }
    }

    fn get_all_values() -> Vec<(CurrencyId, Option<TimeStampedPrice>)> {
        T::Source::get_all_values()
    }
}
