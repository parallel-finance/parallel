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

//! # Prices pallet
//!
//! ## Overview
//!
//! This pallet provides the price from Oracle Module by implementing the
//! `PriceFeeder` trait. In case of emergency, the price can be set directly
//! by Oracle Collective.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{log, pallet_prelude::*, transactional, weights::DispatchClass};
use frame_system::pallet_prelude::*;
use orml_oracle::DataProviderExtended;
use orml_traits::DataProvider;
use primitives::*;
use sp_runtime::{
    traits::{CheckedDiv, CheckedMul},
    FixedU128,
};
use sp_std::vec::Vec;

pub use pallet::*;
use pallet_traits::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub mod weights;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    use weights::WeightInfo;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The data source, such as Oracle.
        type Source: DataProvider<CurrencyId, TimeStampedPrice>
            + DataProviderExtended<CurrencyId, TimeStampedPrice>;

        /// The origin which may set prices feed to system.
        type FeederOrigin: EnsureOrigin<Self::Origin>;

        /// Liquid currency & staking currency provider
        type LiquidStakingCurrenciesProvider: LiquidStakingCurrenciesProvider<CurrencyId>;

        /// The provider of the exchange rate between liquid currency and
        /// staking currency.
        type LiquidStakingExchangeRateProvider: ExchangeRateProvider;

        /// Decimal provider.
        type Decimal: DecimalProvider<CurrencyId>;

        /// Weight information
        type WeightInfo: WeightInfo;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Set emergency price. \[asset_id, price_detail\]
        SetPrice(CurrencyId, Price),
        /// Reset emergency price. \[asset_id\]
        ResetPrice(CurrencyId),
    }

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
        /// Set emergency price
        #[pallet::weight((<T as Config>::WeightInfo::set_price(), DispatchClass::Operational))]
        #[transactional]
        pub fn set_price(
            origin: OriginFor<T>,
            asset_id: CurrencyId,
            price: Price,
        ) -> DispatchResultWithPostInfo {
            T::FeederOrigin::ensure_origin(origin)?;
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
            T::FeederOrigin::ensure_origin(origin)?;
            <Pallet<T> as EmergencyPriceFeeder<CurrencyId, Price>>::reset_emergency_price(asset_id);
            Ok(().into())
        }
    }
}

impl<T: Config> Pallet<T> {
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
