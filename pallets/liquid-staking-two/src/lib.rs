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

use frame_support::{
    pallet_prelude::*, traits::SortedMembers, transactional, BoundedVec, PalletId,
};
use frame_system::pallet_prelude::*;
use orml_traits::XcmTransfer;
use sp_runtime::{traits::AccountIdConversion, ArithmeticError, FixedPointNumber, RuntimeDebug};

use sp_std::prelude::*;
use xcm::v0::{Junction, MultiLocation, NetworkId};

use orml_traits::{MultiCurrency, MultiCurrencyExtended};

pub use pallet::*;
use primitives::{
    Amount, Balance, CurrencyId, EraIndex, ExchangeRateProvider,
    LiquidStakingProtocol, Rate, Ratio,
};

pub const MAX_UNSTAKE_CHUNKS: usize = 5;

#[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub enum StakingOperationType {
    Bond,
    Unbond,
    Rebond,
    Matching,
    TransferToRelaychain,
    RecordReward,
    RecordSlash,
}

#[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub enum ResponseStatus {
    Ready,
    Processing,
    Successed,
    Failed,
}

#[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub struct Operation<BlockNumber> {
    amount: Balance,
    block_number: BlockNumber,
    status: ResponseStatus,
}

#[derive(Copy, Clone, Eq, PartialEq, Default, Encode, Decode, RuntimeDebug)]
pub struct MatchingPoolBuffer {
    total_unstake_amount: Balance,
    total_stake_amount: Balance,
    operation_type: Option<StakingOperationType>,
}

/// The single user's stake/unsatke amount in each era
#[derive(Copy, Clone, Eq, PartialEq, Default, Encode, Decode, RuntimeDebug)]
pub struct MatchingUserBuffer {
    /// The token amount that user unstake during this era, will be calculated
    /// by exchangerate and xToken amount
    total_unstake_amount: Balance,
    /// The token amount that user stake during this era, this amounut is equal
    /// to what the user input.
    total_stake_amount: Balance,
    /// The token amount that user have alreay claimed before the lock period,
    /// this will happen because, in matching pool total_unstake_amount and
    /// total_stake_amount can match each other
    claimed_unstake_amount: Balance,
    /// The token amount that user have alreay claimed before the lock period,
    claimed_stake_amount: Balance,
    /// To confirm that before lock period, user can only claim once because of
    /// the matching.
    claimed_matching: bool,
}

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

        /// The origin which can withdraw staking assets.
        // type WithdrawOrigin: EnsureOrigin<Self::Origin>;

        /// XCM transfer
        type XcmTransfer: XcmTransfer<Self::AccountId, Balance, CurrencyId>;

        /// Approved agent list on relaychain
        // type Members: SortedMembers<Self::AccountId>;

        /// Base xcm weight to use for cross chain transfer
        type BaseXcmWeight: Get<Weight>;

        /// The maximum size of Unstake
        // #[pallet::constant]
        type MaxUnstake: Get<u32>;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// ExchangeRate is invalid
        InvalidExchangeRate,
        /// Agent is not approved.
        IllegalAgent,
        /// Operation is not ready for processing
        OperationNotReady,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// The assets get staked successfully
        Staked(T::AccountId, Balance),
        /// The xtoken gets unstaked successfully
        Unstaked(T::AccountId, Balance, Balance),
        /// The withdraw request is successful
        Claimed(T::AccountId, Balance),
        /// The rewards are recorded
        RewardsRecorded(T::AccountId, Balance),
        /// The slash is recorded
        SlashRecorded(T::AccountId, Balance),

        DepositEventToRelaychain(T::AccountId, EraIndex, StakingOperationType, Balance),
        /// Bond operation in relaychain was successed.
        BondSucceed(EraIndex),
        /// Unbond operation in relaychain was successed.
        UnbondSucceed(EraIndex),
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
    pub type InsurancePool<T: Config> = StorageValue<_, Balance, ValueQuery>;

    /// The total amount of staking pool.
    #[pallet::storage]
    #[pallet::getter(fn staking_pool)]
    pub type StakingPool<T: Config> = StorageValue<_, Balance, ValueQuery>;

    /// Store total stake amount and total unstake amount during current era,
    /// And will update when trigger new era, calculate whether bond/unbond/rebond,
    /// Will be remove when corresponding era has been finished success.
    #[pallet::storage]
    #[pallet::getter(fn matching_pool)]
    pub type MatchingPool<T: Config> =
        StorageMap<_, Blake2_128Concat, EraIndex, MatchingPoolBuffer, ValueQuery>;

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
    #[pallet::getter(fn matching_queue)]
    pub type MatchingQueue<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        EraIndex,
        MatchingUserBuffer,
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
        Operation<T::BlockNumber>,
    >;

    // #[pallet::genesis_config]
    // pub struct GenesisConfig {
    //     pub exchange_rate: Rate,
    //     pub reserve_factor: Ratio,
    // }

    // #[cfg(feature = "std")]
    // impl Default for GenesisConfig {
    //     fn default() -> Self {
    //         Self {
    //             exchange_rate: Rate::default(),
    //             reserve_factor: Ratio::default(),
    //         }
    //     }
    // }

    // #[pallet::genesis_build]
    // impl<T: Config> GenesisBuild<T> for GenesisConfig {
    //     fn build(&self) {
    //         ExchangeRate::<T>::put(self.exchange_rate);
    //         ReserveFactor::<T>::put(self.reserve_factor);
    //     }
    // }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

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
            // T::WithdrawOrigin::ensure_origin(origin)?;
            let _who = ensure_signed(origin)?;
            let current_era = Self::current_era();
            PreviousEra::<T>::put(current_era);
            CurrentEra::<T>::put(era_index);
            Ok(().into())
        }

        //todo，record reward on each era, invoked by stake-client
        // StakingPool = StakingPool + reward amount
        // StakingPool/T::currency::total_issuance(liquidcurrency)
        #[pallet::weight(10_000)]
        #[transactional]
        pub fn record_reward(
            origin: OriginFor<T>,
            agent: T::AccountId,
            era_index: EraIndex,
            #[pallet::compact] amount: Balance,
        ) -> DispatchResultWithPostInfo {
            // T::WithdrawOrigin::ensure_origin(origin)?;
            let _who = ensure_signed(origin)?;
            // ensure!(T::Members::contains(&agent), Error::<T>::IllegalAgent);
            StakingPool::<T>::try_mutate(|m| -> DispatchResult {
                *m = m.checked_add(amount).ok_or(ArithmeticError::Overflow)?;
                Ok(())
            })?;

            let exchange_rate = Rate::checked_from_rational(
                StakingPool::<T>::get(),
                T::Currency::total_issuance(T::LiquidCurrency::get()),
            )
            .ok_or(Error::<T>::InvalidExchangeRate)?;
            ExchangeRate::<T>::put(exchange_rate);

            StakingOperationHistory::<T>::insert(
                era_index,
                StakingOperationType::RecordReward,
                Operation::<_> {
                    amount,
                    block_number: frame_system::Pallet::<T>::block_number(),
                    status: ResponseStatus::Successed,
                },
            );

            Self::deposit_event(Event::<T>::RewardsRecorded(agent, amount));
            Ok(().into())
        }

        /// Record slash event from relaychain.
        ///
        /// Can only invoked by stake client. Decrease asset amount in `StakingPool` and change
        /// exchange rate.
        #[pallet::weight(10_000)]
        #[transactional]
        pub fn record_slash(
            origin: OriginFor<T>,
            agent: T::AccountId,
            era_index: EraIndex,
            #[pallet::compact] amount: Balance,
        ) -> DispatchResultWithPostInfo {
            // T::WithdrawOrigin::ensure_origin(origin)?;
            let _who = ensure_signed(origin)?;
            // ensure!(T::Members::contains(&agent), Error::<T>::IllegalAgent);
            StakingPool::<T>::try_mutate(|m| -> DispatchResult {
                *m = m.checked_sub(amount).ok_or(ArithmeticError::Underflow)?;
                Ok(())
            })?;

            let exchange_rate = Rate::checked_from_rational(
                StakingPool::<T>::get(),
                T::Currency::total_issuance(T::LiquidCurrency::get()),
            )
            .ok_or(Error::<T>::InvalidExchangeRate)?;
            ExchangeRate::<T>::put(exchange_rate);

            StakingOperationHistory::<T>::insert(
                era_index,
                StakingOperationType::RecordSlash,
                Operation::<_> {
                    amount,
                    block_number: frame_system::Pallet::<T>::block_number(),
                    status: ResponseStatus::Successed,
                },
            );

            Self::deposit_event(Event::<T>::SlashRecorded(agent, amount));
            Ok(().into())
        }

        // bond/unbond/rebond/bond_extra may be merge into one
        #[pallet::weight(10_000)]
        #[transactional]
        pub fn record_bond_response(
            origin: OriginFor<T>,
            era_index: Option<EraIndex>,
        ) -> DispatchResultWithPostInfo {
            let _who = ensure_signed(origin)?;
            let era_index = era_index.unwrap_or(Self::current_era());

            let op = StakingOperationHistory::<T>::try_mutate(
                era_index,
                StakingOperationType::Bond,
                |op| -> Result<Operation<_>, DispatchError> {
                    let next_op = op
                        .clone()
                        .filter(|op| op.status == ResponseStatus::Ready)
                        .map(|op| Operation {
                            status: ResponseStatus::Successed,
                            ..op
                        })
                        .ok_or(Error::<T>::OperationNotReady)?;
                    *op = Some(next_op.clone());
                    Ok(next_op)
                },
            )?;
            T::Currency::deposit(T::LiquidCurrency::get(), &Self::account_id(), op.amount)?;
            Self::deposit_event(Event::<T>::BondSucceed(era_index));
            Ok(().into())
        }

        /// Invoked when unbonding extrinsic finished. Burn previously transfered xksm if
        /// successed.
        #[pallet::weight(10_000)]
        #[transactional]
        pub fn record_unbond_response(
            origin: OriginFor<T>,
            era_index: Option<EraIndex>,
        ) -> DispatchResultWithPostInfo {
            let _who = ensure_signed(origin)?;
            let era_index = era_index.unwrap_or(Self::current_era());

            let op = StakingOperationHistory::<T>::try_mutate(
                &era_index,
                StakingOperationType::Unbond,
                |op| -> Result<Operation<_>, DispatchError> {
                    let next_op = op
                        .clone()
                        .filter(|op| op.status == ResponseStatus::Ready)
                        .map(|op| Operation {
                            status: ResponseStatus::Successed,
                            ..op
                        })
                        .ok_or(Error::<T>::OperationNotReady)?;
                    *op = Some(next_op.clone());
                    Ok(next_op)
                },
            )?;
            T::Currency::withdraw(T::LiquidCurrency::get(), &Self::account_id(), op.amount)?;
            Self::deposit_event(Event::<T>::UnbondSucceed(era_index));
            Ok(().into())
        }

        #[pallet::weight(10_000)]
        #[transactional]
        pub fn transfer_to_relaychain(
            origin: OriginFor<T>,
            amount: Balance,
            agent: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let _who = ensure_signed(origin)?;
            // TODO(Alan WANG): Check agent is approved.
            T::XcmTransfer::transfer(
                Self::account_id(),
                T::StakingCurrency::get(),
                amount,
                MultiLocation::X2(
                    Junction::Parent,
                    Junction::AccountId32 {
                        network: NetworkId::Any,
                        id: agent.into(),
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
            //todo ensure that this method is called after update era index.
            let who = ensure_signed(origin)?;
            // match stake and unstake,
            let previous_era = Self::previous_era();
            let pool_buffer = MatchingPool::<T>::get(&previous_era);
            let pool_stake_amount = pool_buffer.total_stake_amount;
            let pool_unstake_amount = pool_buffer.total_unstake_amount;
            let (operation_type, amount) = if pool_stake_amount > pool_unstake_amount {
                (
                    StakingOperationType::Bond,
                    pool_stake_amount - pool_unstake_amount,
                )
            } else if pool_stake_amount < pool_unstake_amount {
                (
                    StakingOperationType::Unbond,
                    pool_unstake_amount - pool_stake_amount,
                )
            } else {
                (StakingOperationType::Matching, 0)
            };
            StakingOperationHistory::<T>::try_mutate(
                &previous_era,
                &operation_type,
                |o| -> DispatchResult {
                    ensure!(*o == None, "error");
                    o.as_mut().and_then(|operation| {
                        operation.status = ResponseStatus::Ready;
                        operation.amount = amount;
                        operation.block_number = frame_system::Pallet::<T>::block_number();
                        Some(())
                    });

                    Ok(())
                },
            )?;
            MatchingPool::<T>::try_mutate(&previous_era, |pool_buffer| -> DispatchResult {
                pool_buffer.operation_type = Some(operation_type);
                Ok(())
            })?;
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
            #[pallet::compact] amount: Balance,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            <Self as LiquidStakingProtocol<T::AccountId>>::stake(&who, amount)?;
            Ok(().into())
        }

        #[pallet::weight(10_000)]
        #[transactional]
        pub fn unstake(
            origin: OriginFor<T>,
            #[pallet::compact] amount: Balance,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            <Self as LiquidStakingProtocol<T::AccountId>>::unstake(&who, amount)?;
            Ok(().into())
        }

        #[pallet::weight(10_000)]
        #[transactional]
        pub fn claim(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            <Self as LiquidStakingProtocol<T::AccountId>>::claim(&who)?;
            Ok(().into())
        }
    }
}

impl<T: Config> LiquidStakingProtocol<T::AccountId> for Pallet<T> {
    // After confirmed bond on relaychain,
    // after update exchangerate (record_reward),
    // and then mint/deposit xKSM.
    fn stake(who: &T::AccountId, amount: Balance) -> DispatchResultWithPostInfo {
        //todo reserve, insurance pool
        T::Currency::transfer(T::StakingCurrency::get(), who, &Self::account_id(), amount)?;

        MatchingQueue::<T>::try_mutate(
            who,
            &Self::current_era(),
            |user_buffer| -> DispatchResult {
                user_buffer.total_stake_amount = user_buffer
                    .total_stake_amount
                    .checked_add(amount)
                    .ok_or(ArithmeticError::Overflow)?;

                Ok(())
            },
        )?;

        MatchingPool::<T>::try_mutate(&Self::current_era(), |pool_buffer| -> DispatchResult {
            pool_buffer.total_stake_amount = pool_buffer
                .total_stake_amount
                .checked_add(amount)
                .ok_or(ArithmeticError::Overflow)?;

            Ok(())
        })?;
        Self::deposit_event(Event::Staked(who.clone(), amount));
        Ok(().into())
    }

    // After confirmed unbond on relaychain,
    // and then burn/withdraw xKSM.
    // before update exchangerate (record_reward)
    fn unstake(who: &T::AccountId, amount: Balance) -> DispatchResultWithPostInfo {
        // can not burn directly because we have match mechanism
        T::Currency::transfer(T::LiquidCurrency::get(), who, &Self::account_id(), amount)?;

        let exchange_rate = ExchangeRate::<T>::get();
        let asset_amount = exchange_rate
            .checked_mul_int(amount)
            .ok_or(Error::<T>::InvalidExchangeRate)?;

        MatchingQueue::<T>::try_mutate(
            who,
            &Self::current_era(),
            |user_buffer| -> DispatchResult {
                user_buffer.total_unstake_amount = user_buffer
                    .total_unstake_amount
                    .checked_add(asset_amount)
                    .ok_or(ArithmeticError::Overflow)?;

                Ok(())
            },
        )?;

        MatchingPool::<T>::try_mutate(&Self::current_era(), |pool_buffer| -> DispatchResult {
            pool_buffer.total_unstake_amount = pool_buffer
                .total_unstake_amount
                .checked_add(asset_amount)
                .ok_or(ArithmeticError::Overflow)?;

            Ok(())
        })?;
        Self::deposit_event(Event::Unstaked(who.clone(), amount, asset_amount));
        Ok(().into())
    }

    fn claim(who: &T::AccountId) -> DispatchResultWithPostInfo {
        let mut withdrawable_stake_amount = 0u128;
        let mut withdrawable_unstake_amount = 0u128;
        let mut remove = vec![];
        MatchingQueue::<T>::iter_prefix(who).for_each(|(era_index, user_buffer)| {
            let mut claim_unstake_each_era = 0u128;
            let mut claim_stake_each_era = 0u128;
            let pool_buffer = MatchingPool::<T>::get(&era_index);
            pool_buffer.operation_type.and_then(|t| {
                let operation = StakingOperationHistory::<T>::get(&era_index, &t)?;
                if operation.status != ResponseStatus::Successed {
                    return None;
                }
                let current_era = Self::current_era();
                match t {
                    StakingOperationType::Bond => {
                        // if bond, need to wait 1 era or get the matching part instantly
                        if era_index + T::StakingDuration::get() > current_era {
                            if !user_buffer.claimed_matching {
                                // get the matching part only
                                MatchingQueue::<T>::try_mutate(
                                    who,
                                    &era_index,
                                    |b| -> DispatchResult {
                                        b.claimed_matching = true;
                                        Ok(())
                                    },
                                )
                                .ok()
                                .and_then(|_| {
                                    // after matching mechanism，for bond operation, user who unstake can get all amount directly
                                    claim_unstake_each_era = user_buffer.total_unstake_amount;
                                    claim_stake_each_era = Rate::saturating_from_rational(
                                        user_buffer.total_stake_amount,
                                        pool_buffer.total_stake_amount,
                                    )
                                    .saturating_mul_int(pool_buffer.total_unstake_amount);
                                    Some(())
                                });
                            }
                        } else {
                            // why users who unstake can get KSM back directly?
                            // because this era is about bond operation,
                            // so all user who unstake in this era, no need wait 28 eras.
                            claim_unstake_each_era = user_buffer
                                .total_unstake_amount
                                .saturating_sub(user_buffer.claimed_unstake_amount);

                            // after waiting 1 era, users who stake get the left part
                            claim_stake_each_era = user_buffer
                                .total_stake_amount
                                .saturating_sub(user_buffer.claimed_stake_amount);
                        }
                    }
                    StakingOperationType::Unbond => {
                        // if unbond, need to wait 28 eras
                        if era_index + T::BondingDuration::get() > current_era {
                            if !user_buffer.claimed_matching {
                                // get the matching part only
                                MatchingQueue::<T>::try_mutate(
                                    who,
                                    &era_index,
                                    |b| -> DispatchResult {
                                        b.claimed_matching = true;
                                        Ok(())
                                    },
                                )
                                .ok()
                                .and_then(|_| {
                                    // after matching mechanism，for unbond operation, user who stake can get all amount directly
                                    claim_stake_each_era = user_buffer.total_stake_amount;
                                    claim_unstake_each_era = Rate::saturating_from_rational(
                                        user_buffer.total_unstake_amount,
                                        pool_buffer.total_unstake_amount,
                                    )
                                    .saturating_mul_int(pool_buffer.total_stake_amount);
                                    Some(())
                                });
                            }
                        } else {
                            // after waiting 28 eras, users who unstake get the left part
                            claim_unstake_each_era = user_buffer
                                .total_unstake_amount
                                .saturating_sub(user_buffer.claimed_unstake_amount);

                            // in case users who stake but forget to claim in 28 eras.
                            claim_stake_each_era = user_buffer
                                .total_stake_amount
                                .saturating_sub(user_buffer.claimed_stake_amount);
                        }
                    }
                    StakingOperationType::Matching => {
                        //if matching, can claim all directly
                        claim_unstake_each_era = user_buffer
                            .total_unstake_amount
                            .saturating_sub(user_buffer.claimed_unstake_amount);

                        claim_stake_each_era = user_buffer
                            .total_stake_amount
                            .saturating_sub(user_buffer.claimed_stake_amount);
                    }
                    _ => (),
                };

                MatchingQueue::<T>::try_mutate(who, &era_index, |b| -> DispatchResult {
                    b.claimed_unstake_amount = b
                        .claimed_unstake_amount
                        .saturating_add(claim_unstake_each_era);
                    b.claimed_stake_amount =
                        b.claimed_stake_amount.saturating_add(claim_stake_each_era);

                    if b.total_stake_amount == b.claimed_stake_amount
                        && b.total_unstake_amount == b.claimed_unstake_amount
                    {
                        // user have already claimed all he can claim in this era, remove it from MatchingQueue
                        remove.push(era_index);
                    }
                    Ok(())
                })
                .ok()
                .and_then(|_| {
                    withdrawable_unstake_amount =
                        withdrawable_unstake_amount.saturating_add(claim_unstake_each_era);
                    withdrawable_stake_amount =
                        withdrawable_stake_amount.saturating_add(claim_stake_each_era);
                    Some(())
                })
            });
        });

        // remove finished records from MatchingQueue
        if remove.len() > 0 {
            remove.iter().for_each(|era_index| {
                MatchingQueue::<T>::remove(who, era_index);
            });
        }

        // transfer xKSM from palletId to who
        if withdrawable_stake_amount > 0 {
            let xtoken_amount = ExchangeRate::<T>::get()
                .reciprocal()
                .and_then(|r| r.checked_mul_int(withdrawable_stake_amount))
                .ok_or(Error::<T>::InvalidExchangeRate)?;
            T::Currency::transfer(
                T::LiquidCurrency::get(),
                &Self::account_id(),
                who,
                xtoken_amount,
            )?;
        }

        // transfer KSM from palletId to who
        if withdrawable_unstake_amount > 0 {
            T::Currency::transfer(
                T::StakingCurrency::get(),
                &Self::account_id(),
                who,
                withdrawable_unstake_amount,
            )?;
        }

        Ok(().into())
    }
}

impl<T: Config> Pallet<T> {
    pub fn account_id() -> T::AccountId {
        T::PalletId::get().into_account()
    }
}

impl<T: Config> ExchangeRateProvider for Pallet<T> {
    fn get_exchange_rate() -> Rate {
        ExchangeRate::<T>::get()
    }
}
