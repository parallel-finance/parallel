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

//! # Liquid staking pallet
//!
//! ## Overview
//!
//! This pallet manages the NPoS operations for relay chain asset.

// 1. update parachain era before relaychain era updated, a few blocks in advance,
// 2. calculate parachain status, whether bond/unbond/rebond,
// 3. invoke relaychain staking methods, bond/unbond/rebond,
// 4. invoke parachain method record relaychain resonse, whether bond/unbond successed or failed,
// 5. waiting relaychain era updated, get the reward amount.
// 6. record in parachain with blocknumber when real relaychain era updated.
// 7. record reward on parachain. update parachain exchange rate.
// 8. if step-4 successed, mint/deposit xToken out according to current exchangerate(after record reward),
#![cfg_attr(not(feature = "std"), no_std)]

mod protocol;
mod types;

use frame_support::{pallet_prelude::*, transactional, PalletId};
use frame_system::pallet_prelude::*;
use orml_traits::{MultiCurrency, MultiCurrencyExtended, XcmTransfer};
use sp_runtime::{
    traits::{AccountIdConversion, Zero},
    ArithmeticError, FixedPointNumber,
};
use sp_std::prelude::*;
use xcm::v0::{Junction, MultiLocation, NetworkId};

use primitives::{Amount, Balance, CurrencyId, EraIndex, Rate, Ratio};

use self::{protocol::*, types::*};

pub use pallet::*;

pub const MAX_UNSTAKE_CHUNKS: usize = 5;

pub(crate) type BalanceOf<T> =
    <<T as Config>::Currency as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {

    use super::*;

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

        /// Currency used for staking
        #[pallet::constant]
        type StakingCurrency: Get<CurrencyId>;

        /// Currency used for liquid voucher
        #[pallet::constant]
        type LiquidCurrency: Get<CurrencyId>;

        /// The pallet id of liquid staking, keeps all the staking assets.
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// Number of eras that user can get xToken back after stake on parachain.
        #[pallet::constant]
        type StakingDuration: Get<EraIndex>;

        /// Number of eras that staked funds must remain bonded for.
        #[pallet::constant]
        type BondingDuration: Get<EraIndex>;

        /// XCM transfer
        type XcmTransfer: XcmTransfer<Self::AccountId, BalanceOf<Self>, CurrencyId>;

        /// Base xcm weight to use for cross chain transfer
        type BaseXcmWeight: Get<Weight>;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// ExchangeRate is invalid
        InvalidExchangeRate,
        /// Agent is not approved.
        IllegalAgent,
        /// Operation is not ready for processing
        OperationNotReady,
        /// Operation has been performed,
        OperationAlreadyPerformed,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// The assets get staked successfully
        Staked(T::AccountId, BalanceOf<T>),
        /// The xtoken gets unstaked successfully
        Unstaked(T::AccountId, BalanceOf<T>, BalanceOf<T>),
        /// The withdraw request is successful
        Claimed(T::AccountId),
        /// The rewards are recorded
        RewardsRecorded(BalanceOf<T>),
        /// The slash is recorded
        SlashRecorded(BalanceOf<T>),
        DepositEventToRelaychain(T::AccountId, EraIndex, StakingOperationType, BalanceOf<T>),
        /// Bond operation in relaychain was successed.
        BondSucceeded(EraIndex),
        /// Unbond operation in relaychain was successed.
        UnbondSucceeded(EraIndex),
        /// Era index was updated.
        EraUpdated(EraIndex),
    }

    /// The exchange rate converts staking native token to voucher.
    #[pallet::storage]
    #[pallet::getter(fn exchange_rate)]
    pub type ExchangeRate<T: Config> = StorageValue<_, Rate, ValueQuery>;

    /// Fraction of staking currency currently set aside for insurance pool
    #[pallet::storage]
    #[pallet::getter(fn reserve_factor)]
    pub type ReserveFactor<T: Config> = StorageValue<_, Ratio, ValueQuery>;

    /// The total amount of insurance pool.
    #[pallet::storage]
    #[pallet::getter(fn insurance_pool)]
    pub type InsurancePool<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    /// The total amount of staking pool.
    #[pallet::storage]
    #[pallet::getter(fn staking_pool)]
    pub type StakingPool<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

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
    pub type MatchingQueueByUser<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        EraIndex,
        UserLedger<BalanceOf<T>>,
        ValueQuery,
    >;

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
        StakingOperationType,
        Operation<T::BlockNumber, BalanceOf<T>>,
    >;

    #[pallet::call]
    impl<T: Config> Pallet<T>
    where
        [u8; 32]: From<<T as frame_system::Config>::AccountId>,
    {
        #[pallet::weight(10_000)]
        #[transactional]
        pub fn trigger_new_era(
            origin: OriginFor<T>,
            era_index: EraIndex,
        ) -> DispatchResultWithPostInfo {
            //TODO(Alan WANG): Check if approved.
            let _who = ensure_signed(origin)?;
            let current_era = Self::current_era();
            PreviousEra::<T>::put(current_era);
            CurrentEra::<T>::put(era_index);
            Self::deposit_event(Event::<T>::EraUpdated(era_index));
            Ok(().into())
        }

        /// Record reward on each era, invoked by stake-client
        /// StakingPool = StakingPool + reward amount
        /// StakingPool/T::currency::total_issuance(liquidcurrency)
        #[pallet::weight(10_000)]
        #[transactional]
        pub fn record_rewards(
            origin: OriginFor<T>,
            era_index: EraIndex,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let _who = ensure_signed(origin)?;
            Self::ensure_op_not_exists(era_index, StakingOperationType::RecordRewards)?;
            Self::increase_staked_asset(amount)?;

            StakingOperationHistory::<T>::insert(
                era_index,
                StakingOperationType::RecordRewards,
                Operation {
                    amount,
                    block_number: frame_system::Pallet::<T>::block_number(),
                    status: ResponseStatus::Succeeded,
                },
            );

            Self::deposit_event(Event::<T>::RewardsRecorded(amount));
            Ok(().into())
        }

        /// Record slash event from relaychain.
        ///
        /// Can only invoked by stake client. Decrease asset amount in `StakingPool` and change
        /// exchange rate.
        #[pallet::weight(10_000)]
        #[transactional]
        pub fn record_slashes(
            origin: OriginFor<T>,
            era_index: EraIndex,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let _who = ensure_signed(origin)?;
            Self::ensure_op_not_exists(era_index, StakingOperationType::RecordSlashes)?;
            Self::decrease_staked_asset(amount)?;

            StakingOperationHistory::<T>::insert(
                era_index,
                StakingOperationType::RecordSlashes,
                Operation {
                    amount,
                    block_number: frame_system::Pallet::<T>::block_number(),
                    status: ResponseStatus::Succeeded,
                },
            );

            Self::deposit_event(Event::<T>::SlashRecorded(amount));
            Ok(().into())
        }

        /// Invoked when bonding extrinsic finished. Mint xksm if
        /// succeeded.
        #[pallet::weight(10_000)]
        #[transactional]
        pub fn record_bond_response(
            origin: OriginFor<T>,
            era_index: EraIndex,
        ) -> DispatchResultWithPostInfo {
            let _who = ensure_signed(origin)?;
            let amount = Self::try_mark_op_succeeded(era_index, StakingOperationType::Bond)?;
            T::Currency::deposit(T::LiquidCurrency::get(), &Self::account_id(), amount)?;
            Self::deposit_event(Event::<T>::BondSucceeded(era_index));
            Ok(().into())
        }

        /// Invoked when unbonding extrinsic finished. Burn previously transfered xksm if
        /// succeeded.
        #[pallet::weight(10_000)]
        #[transactional]
        pub fn record_unbond_response(
            origin: OriginFor<T>,
            era_index: EraIndex,
        ) -> DispatchResultWithPostInfo {
            let _who = ensure_signed(origin)?;
            let amount = Self::try_mark_op_succeeded(era_index, StakingOperationType::Unbond)?;
            T::Currency::withdraw(T::LiquidCurrency::get(), &Self::account_id(), amount)?;
            Self::deposit_event(Event::<T>::UnbondSucceeded(era_index));
            Ok(().into())
        }

        #[pallet::weight(10_000)]
        #[transactional]
        pub fn transfer_to_relaychain(
            origin: OriginFor<T>,
            amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let _who = ensure_signed(origin)?;
            // TODO (Alan WANG): Check agent is approved.
            T::XcmTransfer::transfer(
                Self::account_id(),
                T::StakingCurrency::get(),
                amount,
                MultiLocation::X2(
                    Junction::Parent,
                    Junction::AccountId32 {
                        network: NetworkId::Any,
                        id: Self::account_id().into(),
                    },
                ),
                T::BaseXcmWeight::get(),
            )?;
            Ok(().into())
        }

        // this method must be called after update the era index, so we use previous era to calculate
        #[pallet::weight(10_000)]
        #[transactional]
        pub fn deposit_event_to_relaychain(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            // TODO ensure that this method is called after update era index.
            let who = ensure_signed(origin)?;
            // match stake and unstake,
            let previous_era = Self::previous_era();
            let pool_ledger_per_era = MatchingPoolByEra::<T>::get(&previous_era);

            let (operation_type, amount) = pool_ledger_per_era.op_after_new_era();
            Self::ensure_op_not_exists(previous_era, operation_type)?;
            StakingOperationHistory::<T>::insert(
                previous_era,
                operation_type,
                Operation {
                    status: ResponseStatus::Pending,
                    amount,
                    block_number: frame_system::Pallet::<T>::block_number(),
                },
            );
            MatchingPoolByEra::<T>::mutate(&previous_era, |pool_ledger_per_era| {
                pool_ledger_per_era.operation_type = Some(operation_type);
            });
            // Deposit event, Offchain stake-client need to listen this
            Self::deposit_event(Event::DepositEventToRelaychain(
                who,
                previous_era,
                operation_type,
                amount,
            ));
            Ok(().into())
        }

        #[pallet::weight(10_000)]
        #[transactional]
        pub fn stake(
            origin: OriginFor<T>,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            <Self as LiquidStakingProtocol<T::AccountId, BalanceOf<T>>>::stake(&who, amount)?;
            Self::deposit_event(Event::<T>::Staked(who, amount));
            Ok(().into())
        }

        #[pallet::weight(10_000)]
        #[transactional]
        pub fn unstake(
            origin: OriginFor<T>,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let asset_amount =
                <Self as LiquidStakingProtocol<T::AccountId, BalanceOf<T>>>::unstake(&who, amount)?;
            Self::deposit_event(Event::<T>::Unstaked(who, amount, asset_amount));
            Ok(().into())
        }

        #[pallet::weight(10_000)]
        #[transactional]
        pub fn claim(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            <Self as LiquidStakingProtocol<T::AccountId, BalanceOf<T>>>::claim(&who)?;
            Self::deposit_event(Event::<T>::Claimed(who));
            Ok(().into())
        }
    }
}

impl<T: Config> Pallet<T> {
    pub fn account_id() -> T::AccountId {
        T::PalletId::get().into_account()
    }

    fn accumulate_claim_by_era(
        who: &T::AccountId,
        era_index: EraIndex,
        user_ledger_per_era: UserLedger<BalanceOf<T>>,
        withdrawable_unstake_amount: &mut u128,
        withdrawable_stake_amount: &mut u128,
        remove_record_from_user_queue: &mut Vec<EraIndex>,
    ) {
        let pool_ledger_per_era = MatchingPoolByEra::<T>::get(&era_index);
        let success_operation = pool_ledger_per_era.operation_type.and_then(|ty| {
            StakingOperationHistory::<T>::get(&era_index, &ty)
                .filter(|op| op.status == ResponseStatus::Succeeded)
                .map(|_| ty)
        });

        if success_operation.is_none() {
            return;
        }

        let current_era = Self::current_era();
        // This unwrap is safe because we checked it's not none
        let (claim_unstake_each_era, claim_stake_each_era) = match success_operation.unwrap() {
            StakingOperationType::Bond => Self::claim_in_bond_operation(
                who,
                era_index,
                current_era,
                &user_ledger_per_era,
                &pool_ledger_per_era,
            ),
            StakingOperationType::Unbond => Self::claim_in_unbond_operation(
                who,
                era_index,
                current_era,
                &user_ledger_per_era,
                &pool_ledger_per_era,
            ),
            StakingOperationType::Matching => user_ledger_per_era.remaining_withdrawal_limit(), //if matching, can claim all directly
            _ => (Zero::zero(), Zero::zero()),
        };

        MatchingQueueByUser::<T>::mutate(who, &era_index, |b| {
            b.claimed_unstake_amount = b
                .claimed_unstake_amount
                .saturating_add(claim_unstake_each_era);
            b.claimed_stake_amount = b.claimed_stake_amount.saturating_add(claim_stake_each_era);

            if b.total_stake_amount == b.claimed_stake_amount
                && b.total_unstake_amount == b.claimed_unstake_amount
            {
                // user have already claimed all he can claim in this era, remove it from MatchingQueue
                remove_record_from_user_queue.push(era_index);
            }
        });

        *withdrawable_unstake_amount =
            withdrawable_unstake_amount.saturating_add(claim_unstake_each_era);
        *withdrawable_stake_amount = withdrawable_stake_amount.saturating_add(claim_stake_each_era);
    }

    // if bond, need to wait 1 era or get the matching part instantly
    // users who unstake can get KSM back directly, because this era is about bond operation,
    // so all user who unstake in this era, no need wait 28 eras.
    // after waiting 1 era, users who stake get the left part
    fn claim_in_bond_operation(
        who: &T::AccountId,
        claim_era: EraIndex,
        current_era: EraIndex,
        user_ledger_per_era: &UserLedger<BalanceOf<T>>,
        pool_ledger_per_era: &PoolLedger<BalanceOf<T>>,
    ) -> WithdrawalAmount<BalanceOf<T>> {
        if claim_era + T::StakingDuration::get() <= current_era {
            return user_ledger_per_era.remaining_withdrawal_limit();
        }

        if user_ledger_per_era.claimed_matching {
            return (Zero::zero(), Zero::zero());
        }
        MatchingQueueByUser::<T>::mutate(who, claim_era, |b| {
            b.claimed_matching = true;
        });

        user_ledger_per_era.instant_withdrawal_by_bond(pool_ledger_per_era)
    }

    // if unbond, normally need to wait 28 eras
    // after waiting 28 eras, users who unstake and get the left part
    // considering users who stake and forget to claim within 28 eras.
    fn claim_in_unbond_operation(
        who: &T::AccountId,
        claim_era: EraIndex,
        current_era: EraIndex,
        user_ledger_per_era: &UserLedger<BalanceOf<T>>,
        pool_ledger_per_era: &PoolLedger<BalanceOf<T>>,
    ) -> WithdrawalAmount<BalanceOf<T>> {
        if claim_era + T::BondingDuration::get() <= current_era {
            return user_ledger_per_era.remaining_withdrawal_limit();
        }

        if user_ledger_per_era.claimed_matching {
            return (Zero::zero(), Zero::zero());
        }

        MatchingQueueByUser::<T>::mutate(who, claim_era, |b| {
            b.claimed_matching = true;
        });

        user_ledger_per_era.instant_withdrawal_by_unbond(pool_ledger_per_era)
    }
}

impl<T: Config> Pallet<T> {
    /// Increase staked asset and update exchange_rate later.
    #[inline]
    fn increase_staked_asset(amount: BalanceOf<T>) -> DispatchResult {
        StakingPool::<T>::try_mutate(|m| -> DispatchResult {
            *m = m.checked_add(amount).ok_or(ArithmeticError::Overflow)?;
            Ok(())
        })?;
        Self::update_exchange_rate()
    }

    /// Decrease staked asset and update exchange_rate later.
    #[inline]
    fn decrease_staked_asset(amount: BalanceOf<T>) -> DispatchResult {
        StakingPool::<T>::try_mutate(|m| -> DispatchResult {
            *m = m.checked_sub(amount).ok_or(ArithmeticError::Underflow)?;
            Ok(())
        })?;
        Self::update_exchange_rate()
    }

    #[inline]
    fn update_exchange_rate() -> DispatchResult {
        let exchange_rate = Rate::checked_from_rational(
            StakingPool::<T>::get(),
            T::Currency::total_issuance(T::LiquidCurrency::get()),
        )
        .ok_or(Error::<T>::InvalidExchangeRate)?;
        ExchangeRate::<T>::put(exchange_rate);
        Ok(())
    }

    #[inline]
    fn try_mark_op_succeeded(
        era_index: EraIndex,
        op_type: StakingOperationType,
    ) -> Result<BalanceOf<T>, DispatchError> {
        StakingOperationHistory::<T>::try_mutate(
            era_index,
            op_type,
            |op| -> Result<BalanceOf<T>, DispatchError> {
                let (next_op, amount) = op
                    .filter(|op| op.status == ResponseStatus::Pending)
                    .map(|op| {
                        (
                            Operation {
                                status: ResponseStatus::Succeeded,
                                ..op
                            },
                            op.amount,
                        )
                    })
                    .ok_or(Error::<T>::OperationNotReady)?;
                *op = Some(next_op);
                Ok(amount)
            },
        )
    }

    #[inline]
    fn ensure_op_not_exists(era_index: EraIndex, op_type: StakingOperationType) -> DispatchResult {
        ensure!(
            StakingOperationHistory::<T>::get(era_index, op_type).is_none(),
            Error::<T>::OperationAlreadyPerformed,
        );
        Ok(())
    }
}
