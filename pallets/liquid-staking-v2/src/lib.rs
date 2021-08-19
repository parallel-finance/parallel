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
        transactional, PalletId, Twox64Concat,
    };
    use frame_system::{
        ensure_signed,
        pallet_prelude::{BlockNumberFor, OriginFor},
    };
    use orml_traits::{MultiCurrency, MultiCurrencyExtended};
    use sp_runtime::{traits::AccountIdConversion, ArithmeticError, FixedPointNumber};

    use primitives::{Amount, Balance, CurrencyId, EraIndex, Rate};

    use crate::types::{
        MatchingLedger, Operation, ResponseStatus, StakingSettlementKind, UnstakeMisc,
    };
    use crate::weights::WeightInfo;

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

        /// Offchain bridge accout who manages staking currency in relaychain.
        type BridgeOrigin: EnsureOrigin<Self::Origin>;

        /// The staking currency id.
        #[pallet::constant]
        type StakingCurrency: Get<CurrencyId>;

        /// The liquid voucher currency id.
        #[pallet::constant]
        type LiquidCurrency: Get<CurrencyId>;

        /// The pallet id of liquid staking, keeps all the staking assets.
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        type WeightInfo: WeightInfo;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    #[pallet::metadata(T::AccountId = "Account", BalanceOf<T> = "Balance")]
    pub enum Event<T: Config> {
        /// The assets get staked successfully
        Staked(T::AccountId, BalanceOf<T>),
        /// The derivative get unstaked successfully
        Unstaked(T::AccountId, BalanceOf<T>, BalanceOf<T>),
        /// Reward/Slash has been recorded.
        StakeingSettlementRecorded(StakingSettlementKind, BalanceOf<T>),
        /// Era index updated.
        ///
        /// Constructed by (last_era_index, current_era_index).
        EraIndexUpdated(EraIndex, EraIndex),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Reward/Slash has been recorded.
        StakeingSettlementAlreadyRecorded,
        /// Exchange rate is invalid.
        InvalidExchangeRate,
        /// Era has been pushed before.
        EraAlreadyPushed,
        /// Operation wasn't submitted to relaychain or has been processed.
        OperationNotReady,
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

    /// Records relay operations during each era.
    #[pallet::storage]
    pub type UnbondingOperationHistory<T: Config> =
        StorageMap<_, Twox64Concat, EraIndex, Operation<BlockNumberFor<T>, BalanceOf<T>>>;

    /// Record all the pending unstaking requests.
    /// Key is the owner of assets.
    #[pallet::storage]
    #[pallet::getter(fn account_unstake)]
    pub type AccountUnstake<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        T::AccountId,
        Blake2_128Concat,
        EraIndex,
        UnstakeMisc<BalanceOf<T>>,
        ValueQuery,
    >;

    /// Store total stake amount and unstake amount in each era,
    /// And will update when stake/unstake occurred.
    #[pallet::storage]
    #[pallet::getter(fn era_matching_pool)]
    pub type EraMatchingPool<T: Config> =
        StorageMap<_, Blake2_128Concat, EraIndex, MatchingLedger<BalanceOf<T>>, ValueQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig {
        pub exchange_rate: Rate,
    }

    #[cfg(feature = "std")]
    impl Default for GenesisConfig {
        fn default() -> Self {
            Self {
                exchange_rate: Rate::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig {
        fn build(&self) {
            ExchangeRate::<T>::put(self.exchange_rate);
        }
    }

    #[cfg(feature = "std")]
    impl GenesisConfig {
        /// Direct implementation of `GenesisBuild::build_storage`.
        ///
        /// Kept in order not to break dependency.
        pub fn build_storage<T: Config>(&self) -> Result<sp_runtime::Storage, String> {
            <Self as GenesisBuild<T>>::build_storage(self)
        }

        /// Direct implementation of `GenesisBuild::assimilate_storage`.
        ///
        /// Kept in order not to break dependency.
        pub fn assimilate_storage<T: Config>(
            &self,
            storage: &mut sp_runtime::Storage,
        ) -> Result<(), String> {
            <Self as GenesisBuild<T>>::assimilate_storage(self, storage)
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Put assets under staking, the native assets will be transferred to the account
        /// owned by the pallet, user receive derivative in return, such derivative can be
        /// further used as collateral for lending.
        ///
        /// - `amount`: the amount of staking assets
        #[pallet::weight(T::WeightInfo::stake())]
        #[transactional]
        pub fn stake(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            let exchange_rate = ExchangeRate::<T>::get();
            let liquid_amount = exchange_rate
                .reciprocal()
                .and_then(|r| r.checked_mul_int(amount))
                .ok_or(Error::<T>::InvalidExchangeRate)?;
            T::Currency::transfer(T::StakingCurrency::get(), &who, &Self::account_id(), amount)?;
            T::Currency::deposit(T::LiquidCurrency::get(), &who, liquid_amount)?;
            StakingPool::<T>::try_mutate(|b| -> DispatchResult {
                *b = b.checked_add(amount).ok_or(ArithmeticError::Overflow)?;
                Ok(())
            })?;

            EraMatchingPool::<T>::try_mutate(
                Self::current_era(),
                |matching_ledger| -> DispatchResult {
                    let new_stake_amount = matching_ledger
                        .total_stake_amount
                        .checked_add(amount)
                        .ok_or(ArithmeticError::Overflow)?;
                    matching_ledger.total_stake_amount = new_stake_amount;
                    Ok(())
                },
            )?;

            Self::deposit_event(Event::Staked(who, amount));
            Ok(().into())
        }

        /// Unstake by exchange derivative for assets, the assets will not be avaliable immediately.
        /// Instead, the request is recorded and pending for the nomination accounts in relay
        /// chain to do the `unbond` operation.
        ///
        /// - `amount`: the amount of derivative
        #[pallet::weight(T::WeightInfo::unstake())]
        #[transactional]
        pub fn unstake(
            origin: OriginFor<T>,
            liquid_amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            let exchange_rate = ExchangeRate::<T>::get();
            let asset_amount = exchange_rate
                .checked_mul_int(liquid_amount)
                .ok_or(Error::<T>::InvalidExchangeRate)?;

            // During the same era, accumulate unstake amount for each account
            AccountUnstake::<T>::try_mutate(
                &who,
                Self::current_era(),
                |unstake_misc| -> DispatchResult {
                    let new_pending_amount = unstake_misc
                        .total_amount
                        .checked_add(asset_amount)
                        .ok_or(ArithmeticError::Overflow)?;
                    unstake_misc.total_amount = new_pending_amount;
                    Ok(())
                },
            )?;
            EraMatchingPool::<T>::try_mutate(
                Self::current_era(),
                |matching_ledger| -> DispatchResult {
                    let new_unstake_amount = matching_ledger
                        .total_unstake_amount
                        .checked_add(asset_amount)
                        .ok_or(ArithmeticError::Overflow)?;
                    matching_ledger.total_unstake_amount = new_unstake_amount;
                    Ok(())
                },
            )?;

            T::Currency::withdraw(T::LiquidCurrency::get(), &who, liquid_amount)?;
            StakingPool::<T>::try_mutate(|b| -> DispatchResult {
                *b = b
                    .checked_sub(asset_amount)
                    .ok_or(ArithmeticError::Underflow)?;
                Ok(())
            })?;

            Self::deposit_event(Event::Unstaked(who, liquid_amount, asset_amount));
            Ok(().into())
        }

        /// Set era index. Usually happend when era advanced in relaychain.
        #[pallet::weight(<T as Config>::WeightInfo::trigger_new_era())]
        #[transactional]
        pub fn trigger_new_era(
            origin: OriginFor<T>,
            era_index: EraIndex,
        ) -> DispatchResultWithPostInfo {
            T::BridgeOrigin::ensure_origin(origin)?;
            let current_era_index = Self::current_era();
            ensure!(current_era_index < era_index, Error::<T>::EraAlreadyPushed,);

            PreviousEra::<T>::put(current_era_index);
            CurrentEra::<T>::put(era_index);

            Self::deposit_event(Event::<T>::EraIndexUpdated(current_era_index, era_index));
            Ok(().into())
        }

        /// Handle staking settlement at the end of an era, such as getting reward or been slashed in relaychain.
        #[pallet::weight(<T as Config>::WeightInfo::record_staking_settlement())]
        #[transactional]
        pub fn record_staking_settlement(
            origin: OriginFor<T>,
            era_index: EraIndex,
            #[pallet::compact] amount: BalanceOf<T>,
            kind: StakingSettlementKind,
        ) -> DispatchResultWithPostInfo {
            T::BridgeOrigin::ensure_origin(origin)?;
            Self::ensure_settlement_not_recorded(era_index, kind)?;
            Self::update_staking_pool(kind, amount)?;

            StakingSettlementRecords::<T>::insert(era_index, kind, amount);
            Self::deposit_event(Event::<T>::StakeingSettlementRecorded(kind, amount));
            Ok(().into())
        }

        /// Handle `withdrawal_unbond` response.
        ///
        /// It's invoked when an unbond operation succeeded in relaychain and reported by
        /// stake-client.
        #[pallet::weight(<T as Config>::WeightInfo::record_withdrawal_unbond_response())]
        #[transactional]
        pub fn record_withdrawal_unbond_response(
            origin: OriginFor<T>,
            era_index: EraIndex,
        ) -> DispatchResultWithPostInfo {
            T::BridgeOrigin::ensure_origin(origin)?;
            // try to mark operation succeeded.
            UnbondingOperationHistory::<T>::try_mutate(era_index, |op| -> DispatchResult {
                let next_op = op
                    .filter(|op| op.status == ResponseStatus::Pending)
                    .map(|op| Operation {
                        status: ResponseStatus::Succeeded,
                        ..op
                    })
                    .ok_or(Error::<T>::OperationNotReady)?;
                *op = Some(next_op);
                Ok(())
            })?;
            Ok(().into())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Ensure settlement not recorded for this `era_index`.
        #[inline]
        pub(crate) fn ensure_settlement_not_recorded(
            era_index: EraIndex,
            kind: StakingSettlementKind,
        ) -> DispatchResult {
            ensure!(
                !StakingSettlementRecords::<T>::contains_key(era_index, kind),
                Error::<T>::StakeingSettlementAlreadyRecorded
            );
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Increase/Decrease staked asset in staking pool, and synchronized the exchange rate.
        pub(crate) fn update_staking_pool(
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

        pub fn account_id() -> T::AccountId {
            T::PalletId::get().into_account()
        }
    }
}
