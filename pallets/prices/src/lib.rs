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
use orml_traits::{DataFeeder, DataProvider, DataProviderExtended};
use primitives::*;
use sp_runtime::{
    traits::{CheckedDiv, CheckedMul},
    FixedU128,
};
use sp_std::vec::Vec;

pub use pallet::*;
pub use pallet_traits::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub mod weights;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    use frame_support::traits::fungibles::{Inspect, Mutate, Transfer};
    use weights::WeightInfo;

    pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
    pub(crate) type AssetIdOf<T> =
        <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;
    pub(crate) type BalanceOf<T> =
        <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

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
        type LiquidStakingExchangeRateProvider: ExchangeRateProvider<CurrencyId>;

        /// VaultTokenCurrenciesFilter
        type VaultTokenCurrenciesFilter: VaultTokenCurrenciesFilter<CurrencyId>;

        /// The provider of the exchange rate between vault_token currency and
        /// relay currency.
        type VaultTokenExchangeRateProvider: VaultTokenExchangeRateProvider<CurrencyId>;

        /// The provider of Loans rate for vault_token
        type VaultLoansRateProvider: LoansRateProvider<CurrencyId>;

        /// Specify all the AMMs we are routing between
        type AMM: AMM<AccountIdOf<Self>, AssetIdOf<Self>, BalanceOf<Self>, Self::BlockNumber>;

        /// Currency type for deposit/withdraw assets to/from amm route
        /// module
        type Assets: Transfer<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
            + Inspect<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
            + Mutate<Self::AccountId, AssetId = CurrencyId, Balance = Balance>;

        /// Relay currency
        #[pallet::constant]
        type RelayCurrency: Get<CurrencyId>;

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

    fn normalize_detail_price(price: TimeStampedPrice, mantissa: u128) -> Option<PriceDetail> {
        price
            .value
            .checked_div(&FixedU128::from_inner(mantissa))
            .map(|value| (value, price.timestamp))
    }

    fn get_staking_asset_price(
        asset_id: &CurrencyId,
        mantissa: u128,
        base_price: TimeStampedPrice,
    ) -> Option<(Price, TimeStampedPrice)> {
        if Some(asset_id.clone()) == T::LiquidStakingCurrenciesProvider::get_liquid_currency() {
            return base_price
                .value
                .checked_div(&FixedU128::from_inner(mantissa))
                .and_then(|staking_currency_price| {
                    staking_currency_price.checked_mul(
                        &T::LiquidStakingExchangeRateProvider::get_exchange_rate(&asset_id)
                            .unwrap_or_default(),
                    )
                })
                .map(|price| (price, base_price));
        }
        None
    }

    fn get_staking_asset_detail_price(
        asset_id: &CurrencyId,
        mantissa: u128,
        base_price: TimeStampedPrice,
    ) -> Option<PriceDetail> {
        Self::get_staking_asset_price(asset_id, mantissa, base_price)
            .map(|(price, base_price)| (price, base_price.timestamp))
    }

    fn get_staking_asset_no_op_price(
        asset_id: &CurrencyId,
        mantissa: u128,
        base_price: TimeStampedPrice,
    ) -> Option<TimeStampedPrice> {
        Self::get_staking_asset_price(asset_id, mantissa, base_price).map(|(price, base_price)| {
            TimeStampedPrice {
                value: price,
                timestamp: base_price.timestamp,
            }
        })
    }

    fn get_vault_asset_price(
        asset_id: &CurrencyId,
        mantissa: u128,
        base_price: TimeStampedPrice,
    ) -> Option<(Price, TimeStampedPrice)> {
        if T::VaultTokenCurrenciesFilter::contains(asset_id) {
            return T::VaultLoansRateProvider::get_full_interest_rate(asset_id).and_then(
                |implied_yield_rate| {
                    base_price
                        .value
                        .checked_div(&FixedU128::from_inner(mantissa))
                        .and_then(|relay_currency_price| {
                            T::VaultTokenExchangeRateProvider::get_exchange_rate(
                                asset_id,
                                implied_yield_rate,
                            )
                            .and_then(|rate| relay_currency_price.checked_mul(&rate))
                        })
                        .map(|price| (price, base_price))
                },
            );
        }
        None
    }

    fn get_vault_asset_detail_price(
        asset_id: &CurrencyId,
        mantissa: u128,
        base_price: TimeStampedPrice,
    ) -> Option<PriceDetail> {
        Self::get_vault_asset_price(asset_id, mantissa, base_price)
            .map(|(price, base_price)| (price, base_price.timestamp))
    }

    fn get_vault_asset_no_op_price(
        asset_id: &CurrencyId,
        mantissa: u128,
        base_price: TimeStampedPrice,
    ) -> Option<TimeStampedPrice> {
        Self::get_vault_asset_price(asset_id, mantissa, base_price).map(|(price, base_price)| {
            TimeStampedPrice {
                value: price,
                timestamp: base_price.timestamp,
            }
        })
    }

    fn get_lp_vault_asset_price(
        asset_id: &CurrencyId,
        _mantissa: u128,
        _base_price: TimeStampedPrice,
    ) -> Option<(Price, TimeStampedPrice)> {
        if let Some((base_asset, _quota_asset, _pool)) =
            T::AMM::get_pool_by_lp_asset(asset_id.clone())
        {
            if T::VaultTokenCurrenciesFilter::contains(&base_asset) {
                //todo impl
            }
        }
        None
    }

    fn get_lp_vault_asset_detail_price(
        asset_id: &CurrencyId,
        mantissa: u128,
        base_price: TimeStampedPrice,
    ) -> Option<PriceDetail> {
        Self::get_lp_vault_asset_price(asset_id, mantissa, base_price)
            .map(|(price, time_stamped_price)| (price, time_stamped_price.timestamp))
    }

    fn get_lp_vault_asset_no_op_price(
        asset_id: &CurrencyId,
        mantissa: u128,
        base_price: TimeStampedPrice,
    ) -> Option<TimeStampedPrice> {
        Self::get_lp_vault_asset_price(asset_id, mantissa, base_price).map(|(price, base_price)| {
            TimeStampedPrice {
                value: price,
                timestamp: base_price.timestamp,
            }
        })
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
        // if emergency price exists, return it
        Self::get_emergency_price(asset_id).or_else(|| {
            let mantissa = Self::get_asset_mantissa(asset_id)?;
            if let Some(base_price) = T::Source::get(&T::RelayCurrency::get()) {
                // then check staking asset
                return Self::get_staking_asset_detail_price(asset_id, mantissa, base_price)
                    .or_else(|| {
                        // then check vault asset
                        Self::get_vault_asset_detail_price(asset_id, mantissa, base_price).or_else(
                            || {
                                // then check lp_vault asset
                                Self::get_lp_vault_asset_detail_price(
                                    asset_id, mantissa, base_price,
                                )
                                .or_else(|| {
                                    // fall through to oracle
                                    T::Source::get(asset_id).and_then(|price| {
                                        Self::normalize_detail_price(price, mantissa)
                                    })
                                })
                            },
                        )
                    });
            }
            return T::Source::get(asset_id)
                .and_then(|price| Self::normalize_detail_price(price, mantissa));
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
        let mantissa = Self::get_asset_mantissa(asset_id)?;
        if let Some(base_price) = T::Source::get_no_op(&T::RelayCurrency::get()) {
            // then check staking asset
            return Self::get_staking_asset_no_op_price(asset_id, mantissa, base_price).or_else(
                || {
                    // then check vault asset
                    Self::get_vault_asset_no_op_price(asset_id, mantissa, base_price).or_else(
                        || {
                            // then check lp_vault asset
                            Self::get_lp_vault_asset_no_op_price(asset_id, mantissa, base_price)
                                .or_else(|| {
                                    // fall through to oracle
                                    T::Source::get_no_op(asset_id)
                                })
                        },
                    )
                },
            );
        }
        return T::Source::get_no_op(asset_id);
    }

    fn get_all_values() -> Vec<(CurrencyId, Option<TimeStampedPrice>)> {
        T::Source::get_all_values()
    }
}
