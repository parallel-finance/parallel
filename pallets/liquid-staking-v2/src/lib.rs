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
pub mod weights;

pub use pallet::*;
pub use weights::WeightInfo;

use codec::{Decode, Encode, FullCodec};
use frame_support::{
    ensure,
    pallet_prelude::*,
    traits::{Get, IsType},
    transactional, Blake2_128Concat, PalletId, Twox64Concat,
};
use frame_system::{ensure_signed, pallet_prelude::OriginFor};
use orml_traits::{MultiCurrency, MultiCurrencyExtended};
// use orml_traits::XcmTransfer;
use sp_runtime::{
    traits::{AccountIdConversion, AtLeast32BitUnsigned},
    ArithmeticError, DispatchResult, FixedPointNumber, RuntimeDebug,
};
// use xcm::v0::MultiLocation;

use primitives::{Amount, Balance, CurrencyId, EraIndex, Rate};

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug)]
pub enum StakingSettlementKind {
    Reward,
    Slash,
}

#[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub enum StakingRelayOperation<Balance> {
    Bond(Balance),
    Unbond(Balance),
    NoOp,
}

#[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub struct Operation<BlockNumber, Balance> {
    pub amount: Balance,
    pub block_number: BlockNumber,
    pub status: ResponseStatus,
}

#[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub enum ResponseStatus {
    Waiting,
    Processing,
    Succeeded,
    Failed,
}

/// The matching pool's total stake & unstake amount in one era
#[derive(Copy, Clone, Eq, PartialEq, Default, Encode, Decode, RuntimeDebug)]
pub struct PoolLedger<Balance> {
    /// The matching pool's total unstake amount
    /// **NOTE** will be calculated by: exchangeRate * xToken amount
    pub total_unstake_amount: Balance,
    /// The matching pool's total stake amount
    pub total_stake_amount: Balance,
}

impl<Balance> PoolLedger<Balance>
where
    Balance: AtLeast32BitUnsigned + FullCodec + Copy + MaybeSerializeDeserialize + Default,
{
    pub fn operation_before_new_era(&self) -> StakingRelayOperation<Balance> {
        if self.total_stake_amount > self.total_unstake_amount {
            return StakingRelayOperation::Bond(
                self.total_stake_amount - self.total_unstake_amount,
            );
        }
        if self.total_unstake_amount > self.total_stake_amount {
            return StakingRelayOperation::Unbond(
                self.total_unstake_amount - self.total_stake_amount,
            );
        }
        StakingRelayOperation::NoOp
    }
}

/// The single user's stake & unstake amount in one era
#[derive(Copy, Clone, Eq, PartialEq, Default, Encode, Decode, RuntimeDebug)]
pub struct UserLedger<Balance> {
    /// The token amount that user unstaked
    /// **NOTE** will be calculated by: exchangeRate * xToken amount
    pub total_unstake_amount: Balance,
    /// The token amount that user staked, this amount is equal
    /// to what the user input.
    pub total_stake_amount: Balance,
}

#[frame_support::pallet]
mod pallet {
    use super::*;

    pub(crate) type BalanceOf<T> =
        <<T as Config>::Currency as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// Currency type used for staking and liquid assets
        type Currency: MultiCurrencyExtended<
            Self::AccountId,
            CurrencyId = CurrencyId,
            Balance = Balance,
            Amount = Amount,
        >;

        /// Currency used for liquid voucher
        #[pallet::constant]
        type LiquidCurrency: Get<CurrencyId>;

        /// Currency used for staking
        #[pallet::constant]
        type StakingCurrency: Get<CurrencyId>;

        /// The pallet id of liquid staking, keeps all the staking assets.
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// Number of eras that staked funds must remain bonded for.
        #[pallet::constant]
        type BondingDuration: Get<EraIndex>;

        // /// The agent account for where the staking currencies are sent to.
        // #[pallet::constant]
        // type RelayAgentAccountLocation: Get<MultiLocation>;

        // /// Base xcm weight to use for cross chain transfer
        // #[pallet::constant]
        // type BaseXcmWeight: Get<Weight>;

        // /// XCM transfer
        // type XcmTransfer: XcmTransfer<Self::AccountId, BalanceOf<Self>, CurrencyId>;

        /// Weight info
        type WeightInfo: WeightInfo;
    }

    /// Previous era index on Parachain.
    ///
    /// Considering that in case stake client cannot update era index one by one.
    #[pallet::storage]
    #[pallet::getter(fn previous_era)]
    pub type PreviousEra<T: Config> = StorageValue<_, EraIndex, ValueQuery>;

    /// Current era index on Relaychain.
    ///
    /// CurrentEra: EraIndex
    #[pallet::storage]
    #[pallet::getter(fn current_era)]
    pub type CurrentEra<T: Config> = StorageValue<_, EraIndex, ValueQuery>;

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

    /// Store operations and corresponding status during each era
    ///
    /// The operation include: bond/bond_extra/unbond/rebond/on_new_era/transfer_to_relaychain/record_reward/record_slash
    /// The status include: start/processing/successed/failed
    #[pallet::storage]
    #[pallet::getter(fn staking_operation_history)]
    pub type StakingOperationHistory<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        EraIndex,
        Blake2_128Concat,
        StakingRelayOperation<BalanceOf<T>>,
        Operation<T::BlockNumber, BalanceOf<T>>,
    >;

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

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// The xtoken gets unstaked successfully
        Unstaked(T::AccountId, BalanceOf<T>, BalanceOf<T>),
        /// Rewards/Slashes have been recorded
        StakingSettlementRecorded(StakingSettlementKind, BalanceOf<T>),
        /// Era index was updated.
        EraUpdated(EraIndex),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Reward/Slash has been recorded.
        StakingSettlementAlreadyRecorded,
        /// Exchange rate is invalid.
        InvalidExchangeRate,
        /// New era is invalid
        InvalidNewEra,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
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
        pub fn trigger_new_era(origin: OriginFor<T>, era_index: EraIndex) -> DispatchResult {
            // TODO: Check if approved.
            ensure_signed(origin)?;
            let current_era = Self::current_era();
            if era_index <= current_era || era_index - current_era != 1 {
                return Err(Error::<T>::InvalidNewEra.into());
            }
            let pool_ledger = MatchingPoolByEra::<T>::get(&current_era);
            let staking_operation = pool_ledger.operation_before_new_era();

            match staking_operation {
                StakingRelayOperation::Bond(amount) => {
                    // Self::transfer_to_relaychain(amount, T::RelayAgentAccountLocation::get())?;
                    Self::record_staking_operation(staking_operation, amount);
                }
                StakingRelayOperation::Unbond(amount) => {
                    Self::record_staking_operation(staking_operation, amount);
                }
                _ => {}
            };

            PreviousEra::<T>::put(current_era);
            CurrentEra::<T>::put(era_index);

            Self::deposit_event(Event::<T>::EraUpdated(era_index));
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

    impl<T: Config> Pallet<T> {
        pub fn account_id() -> T::AccountId {
            T::PalletId::get().into_account()
        }

        #[inline]
        fn record_staking_operation(op: StakingRelayOperation<BalanceOf<T>>, amount: BalanceOf<T>) {
            StakingOperationHistory::<T>::insert(
                Self::current_era(),
                op,
                Operation {
                    status: ResponseStatus::Waiting,
                    amount,
                    block_number: frame_system::Pallet::<T>::block_number(),
                },
            );
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

        // fn transfer_to_relaychain(amount: BalanceOf<T>, dest: MultiLocation) -> DispatchResult {
        // T::XcmTransfer::transfer(
        //     Self::account_id(),
        //     T::StakingCurrency::get(),
        //     amount,
        //     dest,
        //     T::BaseXcmWeight::get(),
        // )?;
        //     Ok(().into())
        // }

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
