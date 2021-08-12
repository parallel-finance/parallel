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
use sp_std::convert::TryInto;
use sp_std::prelude::*;
use xcm::v0::{Junction, MultiLocation, NetworkId};

use orml_traits::{MultiCurrency, MultiCurrencyExtended};

pub use pallet::*;
use primitives::{
    Amount, Balance, BlockNumber, CurrencyId, EraIndex, ExchangeRateProvider,
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

impl Default for ResponseStatus {
    fn default() -> Self {
        Self::Ready
    }
}

#[derive(Copy, Clone, Eq, Default, PartialEq, Encode, Decode, RuntimeDebug)]
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
        // type XcmTransfer: XcmTransfer<Self::AccountId, Balance, CurrencyId>;

        /// Approved agent list on relaychain
        // type Members: SortedMembers<Self::AccountId>;

        /// Base xcm weight to use for cross chain transfer
        // type BaseXcmWeight: Get<Weight>;

        /// The maximum size of Unstake
        // #[pallet::constant]
        type MaxUnstake: Get<u32>;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// ExchangeRate is invalid
        InvalidExchangeRate,
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
        ValueQuery,
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
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        #[transactional]
        pub fn trigger_new_era(
            origin: OriginFor<T>,
            era_index: EraIndex,
        ) -> DispatchResultWithPostInfo {
            let _who = ensure_signed(origin)?;
            let current_era = Self::current_era();
            PreviousEra::<T>::put(current_era);

            CurrentEra::<T>::put(era_index);
            Ok(().into())
        }

        //todo，record reward on each era, invoked by stake-client
        // StakingPool = StakingPool + reward amount
        // StakingPool/T::currency::total_issuance
        #[pallet::weight(10_000)]
        #[transactional]
        pub fn record_reward(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            Ok(().into())
        }

        //todo invoked by stake-client, considering insurrance pool
        // StakingPool = StakingPool - slash amount
        // StakingPool/T::currency::total_issuance
        #[pallet::weight(10_000)]
        #[transactional]
        pub fn record_slash(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            Ok(().into())
        }

        // bond/unbond/rebond/bond_extra may be merge into one
        #[pallet::weight(10_000)]
        #[transactional]
        pub fn record_bond_response(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            //todo we need to mint more xToken, and transfer to Self::account_id
            Ok(().into())
        }

        #[pallet::weight(10_000)]
        #[transactional]
        pub fn record_bond_extra_response(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            Ok(().into())
        }

        // no need do it now
        #[pallet::weight(10_000)]
        #[transactional]
        pub fn record_rebond_response(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            Ok(().into())
        }

        #[pallet::weight(10_000)]
        #[transactional]
        pub fn record_unbond_response(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            // todo we need to burn some xToken, and widthdraw from Self::account_id
            Ok(().into())
        }

        #[pallet::weight(10_000)]
        #[transactional]
        pub fn transfer_to_relaychain(
            origin: OriginFor<T>,
            amount: Balance,
        ) -> DispatchResultWithPostInfo {
            // todo xcm transfer
            // maybe multiple in one era
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
                |operation| -> DispatchResult {
                    ensure!(operation.status == ResponseStatus::Ready, "error");
                    operation.amount = amount;
                    operation.block_number = frame_system::Pallet::<T>::block_number();
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
        // ensure!(token == T::LiquidCurrency::get() || token == T::StakingCurrency::get(),"error");
        let mut withdrawable_stake_amount = 0u128;
        let mut withdrawable_unstake_amount = 0u128;
        let _ = MatchingQueue::<T>::iter_prefix(who).filter_map(|(era_index, user_buffer)| {
            let pool_buffer = MatchingPool::<T>::get(&era_index);
            pool_buffer.operation_type.and_then(|t| {
                let operation = StakingOperationHistory::<T>::get(&era_index, &t);
                if operation.status != ResponseStatus::Successed {
                    return None;
                }
                let current_era = Self::current_era();
                match t {
                    StakingOperationType::Bond => {
                        // if bond, need to wait 1 era or get the matching part
                        if era_index + T::StakingDuration::get() > current_era {
                            if !user_buffer.claimed_matching {
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
                                    // after matching mechanism，
                                    // for bond operation, user can get all unstake amount directly
                                    withdrawable_unstake_amount += user_buffer.total_unstake_amount
                                        - user_buffer.claimed_unstake_amount;
                                    // check_add, 考虑精度和溢出
                                    withdrawable_stake_amount += (user_buffer.total_stake_amount
                                        / pool_buffer.total_stake_amount)
                                        * pool_buffer.total_unstake_amount;

                                    //todo 修改存储

                                    Some(())
                                })
                            } else {
                                None
                            }
                        } else {
                            Some(())
                        }
                    }
                    StakingOperationType::Unbond => {
                        // if unbond, need to wait 28 eras
                        if era_index + T::BondingDuration::get() > current_era {
                            None
                        } else {
                            Some(())
                        }
                    }
                    StakingOperationType::Matching => {
                        // todo if matching, can claim all directly
                        Some(())
                    }
                    _ => None,
                }
            })
        });

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
