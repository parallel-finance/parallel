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

//! # Liquid staking pallet v2
//!
//! ## Overview
//!
//! This pallet manages the NPoS operations for relay chain asset.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
pub mod types;
pub mod weights;

pub use self::pallet::*;

#[frame_support::pallet]
mod pallet {
    use frame_support::{
        dispatch::{DispatchResult, DispatchResultWithPostInfo},
        ensure,
        pallet_prelude::{StorageDoubleMap, StorageValue, ValueQuery},
        traits::{Get, IsType},
        transactional, Twox64Concat,
    };
    use frame_system::{ensure_signed, pallet_prelude::OriginFor};
    use orml_traits::{MultiCurrency, MultiCurrencyExtended};
    use sp_runtime::{ArithmeticError, FixedPointNumber};

    use primitives::{Amount, Balance, CurrencyId, EraIndex, Rate};

    use crate::types::StakeingSettlementKind;
    use crate::weights::WeightInfo;

    pub(crate) type BalanceOf<T> =
        <<T as Config>::Currency as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Currency: MultiCurrencyExtended<
            Self::AccountId,
            CurrencyId = CurrencyId,
            Balance = Balance,
            Amount = Amount,
        >;
        type LiquidCurrency: Get<CurrencyId>;
        type WeightInfo: WeightInfo;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        StakeingSettlementRecorded(StakeingSettlementKind, BalanceOf<T>),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Reward/Slash has been recorded.
        StakeingSettlementAlreadyRecorded,
        /// Exchange rate is invalid.
        InvalidExchangeRate,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(<T as Config>::WeightInfo::record_rewards())]
        #[transactional]
        pub fn record_staking_settlement(
            origin: OriginFor<T>,
            era_index: EraIndex,
            #[pallet::compact] amount: BalanceOf<T>,
            kind: StakeingSettlementKind,
        ) -> DispatchResultWithPostInfo {
            // TODO(wangyafei): Check if approved.
            let _who = ensure_signed(origin)?;
            Self::ensure_settlement_not_recorded(era_index, kind)?;
            Self::update_staking_pool(kind, amount)?;

            StakeingSettlementRecords::<T>::insert(era_index, kind, amount);
            Self::deposit_event(Event::<T>::StakeingSettlementRecorded(kind, amount));
            Ok(().into())
        }
    }

    /// The exchange rate between relaychain native asset and the voucher.
    #[pallet::storage]
    #[pallet::getter(fn exchange_rate)]
    pub type ExchangeRate<T: Config> = StorageValue<_, Rate, ValueQuery>;

    /// Total amount of staked assets in relaycahin.
    #[pallet::storage]
    #[pallet::getter(fn staking_pool)]
    pub type StakingPool<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    /// Records reward or slash during each era.
    #[pallet::storage]
    #[pallet::getter(fn reward_records)]
    pub type StakeingSettlementRecords<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        EraIndex,
        Twox64Concat,
        StakeingSettlementKind,
        BalanceOf<T>,
    >;

    impl<T: Config> Pallet<T> {
        #[inline]
        pub(crate) fn ensure_settlement_not_recorded(
            era_index: EraIndex,
            kind: StakeingSettlementKind,
        ) -> DispatchResult {
            ensure!(
                !StakeingSettlementRecords::<T>::contains_key(era_index, kind),
                Error::<T>::StakeingSettlementAlreadyRecorded
            );
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        pub(crate) fn update_staking_pool(
            kind: StakeingSettlementKind,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            match kind {
                StakeingSettlementKind::Reward => {
                    StakingPool::<T>::try_mutate(|p| -> DispatchResult {
                        *p = p.checked_add(amount).ok_or(ArithmeticError::Overflow)?;
                        Ok(())
                    })
                }
                StakeingSettlementKind::Slash => {
                    StakingPool::<T>::try_mutate(|p| -> DispatchResult {
                        *p = p.checked_sub(amount).ok_or(ArithmeticError::Underflow)?;
                        Ok(())
                    })
                }
            }?;

            // Update exchange rate.
            let exchange_rate = Rate::checked_from_rational(
                StakingPool::<T>::get(),
                T::Currency::total_issuance(T::LiquidCurrency::get()),
            )
            .ok_or(Error::<T>::InvalidExchangeRate)?;
            ExchangeRate::<T>::put(exchange_rate);
            Ok(())
        }
    }
}
