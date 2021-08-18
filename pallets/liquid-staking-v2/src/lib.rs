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
        pallet_prelude::*,
        traits::{Get, IsType},
        transactional, Blake2_128Concat, PalletId, Twox64Concat,
    };
    use frame_system::{ensure_signed, pallet_prelude::OriginFor};
    use orml_traits::{MultiCurrency, MultiCurrencyExtended};
    use sp_runtime::{traits::AccountIdConversion, ArithmeticError, FixedPointNumber};

    use primitives::{Amount, Balance, CurrencyId, EraIndex, Rate};

    use crate::{
        types::{PoolLedger, StakingSettlementKind, UserLedger},
        weights::WeightInfo,
    };

    pub(crate) type BalanceOf<T> =
        <<T as Config>::Currency as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// Liquid/Staked asset currency.
        type Currency: MultiCurrencyExtended<
            Self::AccountId,
            CurrencyId = CurrencyId,
            Balance = Balance,
            Amount = Amount,
        >;
        /// The pallet id of liquid staking, keeps all the staking assets.
        type PalletId: Get<PalletId>;
        /// The liquid voucher currency id.
        type LiquidCurrency: Get<CurrencyId>;
        type WeightInfo: WeightInfo;
    }

    /// Store total stake amount and total unstake amount during current era,
    /// And will update when trigger new era, calculate whether bond/unbond/rebond,
    /// Will be remove when corresponding era has been finished success.
    #[pallet::storage]
    #[pallet::getter(fn matching_pool_by_era)]
    pub type MatchingPoolByEra<T: Config> =
        StorageMap<_, Blake2_128Concat, EraIndex, PoolLedger<BalanceOf<T>>, ValueQuery>;

    /// Store single user's stake and unstake request during each era,
    ///
    /// For stake request, after all xtoken have been mint, this may take 6 hours in Kusama, and one day in Polkadot
    /// The data can be removed after user claim their xToken
    /// the corresponding era's operation in `StakingOperationHistory` should also be successed.
    ///
    /// For unstake request, xtoken need to be burn, and get token back,
    /// the data can be removed after the lock period passed and user claim.
    /// this may take 7 days in Kusama, and 28 days in Polkadot.
    /// the corresponding era's operation in `StakingOperationHistory` should also be successed.
    #[pallet::storage]
    #[pallet::getter(fn matching_queue_by_user)]
    pub type MatchingPoolByUser<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        EraIndex,
        UserLedger<BalanceOf<T>>,
        ValueQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Rewards/Slashes recorded
        StakingSettlementRecorded(StakingSettlementKind, BalanceOf<T>),
        /// The xtoken gets unstaked successfully
        Unstaked(T::AccountId, BalanceOf<T>, BalanceOf<T>),
        /// Era index was updated.
        EraIndexUpdated(EraIndex, EraIndex),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Reward/Slash has been recorded.
        StakingSettlementAlreadyRecorded,
        /// Exchange rate is invalid.
        InvalidExchangeRate,
        /// Era has been pushed before.
        EraAlreadyPushed,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Set era index. Usually happend when era advanced in relaychain.
        #[pallet::weight(<T as Config>::WeightInfo::set_era_index())]
        #[transactional]
        pub fn trigger_new_era(
            origin: OriginFor<T>,
            era_index: EraIndex,
        ) -> DispatchResultWithPostInfo {
            let _who = ensure_signed(origin)?;
            let current_era = Self::current_era();
            ensure!(current_era < era_index, Error::<T>::EraAlreadyPushed);

            PreviousEra::<T>::put(current_era);
            CurrentEra::<T>::put(era_index);
            Self::deposit_event(Event::<T>::EraIndexUpdated(current_era, era_index));
            Ok(().into())
        }

        /// Handle staking settlement at the end of an era, such as getting reward or been slashed in relaychain.
        #[pallet::weight(<T as Config>::WeightInfo::record_rewards())]
        #[transactional]
        pub fn record_staking_settlement(
            origin: OriginFor<T>,
            era_index: EraIndex,
            #[pallet::compact] amount: BalanceOf<T>,
            kind: StakingSettlementKind,
        ) -> DispatchResultWithPostInfo {
            // TODO(wangyafei): Check if approved.
            let _who = ensure_signed(origin)?;
            Self::ensure_settlement_not_recorded(era_index, kind)?;
            Self::update_staking_pool(kind, amount)?;

            StakingSettlementRecords::<T>::insert(era_index, kind, amount);
            Self::deposit_event(Event::<T>::StakingSettlementRecorded(kind, amount));
            Ok(().into())
        }

        #[pallet::weight(10_000)]
        #[transactional]
        pub fn unstake(
            origin: OriginFor<T>,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            T::Currency::transfer(T::LiquidCurrency::get(), &who, &Self::account_id(), amount)?;

            let exchange_rate = ExchangeRate::<T>::get();
            let asset_amount = exchange_rate
                .checked_mul_int(amount)
                .ok_or(Error::<T>::InvalidExchangeRate)?;

            MatchingPoolByUser::<T>::try_mutate(
                &who,
                &Self::current_era(),
                |user_ledger| -> DispatchResult {
                    user_ledger.total_unstake_amount = user_ledger
                        .total_unstake_amount
                        .checked_add(asset_amount)
                        .ok_or(ArithmeticError::Overflow)?;

                    Ok(())
                },
            )?;

            MatchingPoolByEra::<T>::try_mutate(
                &Self::current_era(),
                |pool_ledger| -> DispatchResult {
                    pool_ledger.total_unstake_amount = pool_ledger
                        .total_unstake_amount
                        .checked_add(asset_amount)
                        .ok_or(ArithmeticError::Overflow)?;

                    Ok(())
                },
            )?;

            Self::deposit_event(Event::<T>::Unstaked(who, amount, asset_amount));
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
    pub type StakingSettlementRecords<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        EraIndex,
        Twox64Concat,
        StakingSettlementKind,
        BalanceOf<T>,
    >;

    /// Last updated era index.
    #[pallet::storage]
    #[pallet::getter(fn previous_era)]
    pub type PreviousEra<T: Config> = StorageValue<_, EraIndex, ValueQuery>;

    /// Current era_index.
    #[pallet::storage]
    #[pallet::getter(fn current_era)]
    pub type CurrentEra<T: Config> = StorageValue<_, EraIndex, ValueQuery>;

    impl<T: Config> Pallet<T> {
        pub fn account_id() -> T::AccountId {
            T::PalletId::get().into_account()
        }

        #[inline]
        fn ensure_settlement_not_recorded(
            era_index: EraIndex,
            kind: StakingSettlementKind,
        ) -> DispatchResult {
            ensure!(
                !StakingSettlementRecords::<T>::contains_key(era_index, kind),
                Error::<T>::StakingSettlementAlreadyRecorded
            );
            Ok(())
        }

        /// Increase/Decrease staked asset in staking pool, and synchronized the exchange rate.
        fn update_staking_pool(
            kind: StakingSettlementKind,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            match kind {
                StakingSettlementKind::Reward => {
                    StakingPool::<T>::try_mutate(|p| -> DispatchResult {
                        *p = p.checked_add(amount).ok_or(ArithmeticError::Overflow)?;
                        Ok(())
                    })
                }
                StakingSettlementKind::Slash => {
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
