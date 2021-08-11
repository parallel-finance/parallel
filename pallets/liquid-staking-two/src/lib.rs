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

#[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub enum StakingOperationType {
    Bond,
    BondExtra,
    Unbond,
    Rebond,
    TransferToRelaychain,
    RecordReward,
    RecordSlash,
}

#[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub enum Phase {
    Started,
    OnNewEra,
    RecordReward,
    EmitEventToRelaychain,
    RecordStakingOperation,
    Finished,
}

impl Default for Phase {
    fn default() -> Self {
        Self::Finished
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub enum ResponseStatus {
    Ready,
    Processing,
    Successed,
    Failed,
}

#[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub struct Operation {
    amount: Balance,
    status: ResponseStatus,
    block_number: BlockNumber,
}

#[derive(Copy, Clone, Eq, PartialEq, Default, Encode, Decode, RuntimeDebug)]
pub struct MatchingBuffer<AccountId> {
    stability_pool: Option<AccountId>,
    total_unstake_amount: Balance,
    total_stake_amount: Balance,
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

        /// The origin which can withdraw staking assets.
        type WithdrawOrigin: EnsureOrigin<Self::Origin>;

        /// XCM transfer
        type XcmTransfer: XcmTransfer<Self::AccountId, Balance, CurrencyId>;

        /// Approved agent list on relaychain
        type Members: SortedMembers<Self::AccountId>;

        /// Base xcm weight to use for cross chain transfer
        type BaseXcmWeight: Get<Weight>;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// ExchangeRate is invalid
        InvalidExchangeRate,
        /// Operation is not allowed
        OperationNotAllowed,
        /// Caller is not approved.
        IllegalAgent,
        /// Slash or reword event is recored.
        EventRecorded,
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
        /// Era advanced.
        EraIndexUpdated(EraIndex),
        /// A staking operation reponse recorded.
        ResponseRecorded(StakingOperationType, Balance, ResponseStatus, BlockNumber),
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
    /// this is all about one user, stability pool
    /// And will update when trigger new era.
    #[pallet::storage]
    #[pallet::getter(fn matching_pool)]
    pub type MatchingPool<T: Config> =
        StorageMap<_, Twox64Concat, EraIndex, MatchingBuffer<T::AccountId>, ValueQuery>;

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
    pub type StakingOperationHistory<T: Config> =
        StorageDoubleMap<_, Twox64Concat, EraIndex, Twox64Concat, StakingOperationType, Operation>;

    /// Store current phase during each era
    #[pallet::storage]
    #[pallet::getter(fn current_phase)]
    pub type CurrentPhase<T: Config> = StorageValue<_, Phase, ValueQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig {
        pub exchange_rate: Rate,
        pub reserve_factor: Ratio,
    }

    #[cfg(feature = "std")]
    impl Default for GenesisConfig {
        fn default() -> Self {
            Self {
                exchange_rate: Rate::default(),
                reserve_factor: Ratio::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig {
        fn build(&self) {
            ExchangeRate::<T>::put(self.exchange_rate);
            ReserveFactor::<T>::put(self.reserve_factor);
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        #[transactional]
        pub fn trigger_new_era(
            origin: OriginFor<T>,
            agent: T::AccountId,
            era_index: EraIndex,
        ) -> DispatchResultWithPostInfo {
            T::WithdrawOrigin::ensure_origin(origin)?;
            ensure!(T::Members::contains(&agent), Error::<T>::IllegalAgent);
            CurrentEra::<T>::put(era_index);
            Self::deposit_event(Event::<T>::EraIndexUpdated(era_index));
            Ok(().into())
        }

        /// Invoked by offchain stake client. Declare reward received.
        ///
        /// `origin` should be approved account.(not implement yet).
        ///
        /// Q: What's the agent?
        /// Q: Should update exchange_rate?
        #[pallet::weight(10_000)]
        #[transactional]
        pub fn record_reward(
            origin: OriginFor<T>,
            agent: T::AccountId,
            #[pallet::compact] amount: Balance,
            block_number: BlockNumber,
        ) -> DispatchResultWithPostInfo {
            T::WithdrawOrigin::ensure_origin(origin)?;
            ensure!(T::Members::contains(&agent), Error::<T>::IllegalAgent);
            ensure!(
                !StakingOperationHistory::<T>::contains_key(
                    Self::current_era(),
                    StakingOperationType::RecordReward
                ),
                Error::<T>::EventRecorded
            );

            StakingPool::<T>::try_mutate(|m| -> DispatchResult {
                *m = m.checked_add(amount).ok_or(ArithmeticError::Overflow)?;
                Ok(())
            })?;

            let exchange_rate = Rate::checked_from_rational(
                StakingPool::<T>::get(),
                T::Currency::free_balance(T::LiquidCurrency::get(), &agent),
            )
            .ok_or(Error::<T>::InvalidExchangeRate)?;

            ExchangeRate::<T>::put(exchange_rate);

            StakingOperationHistory::<T>::insert(
                Self::current_era(),
                StakingOperationType::RecordReward,
                Operation {
                    status: ResponseStatus::Successed,
                    amount,
                    block_number,
                },
            );
            Self::deposit_event(Event::<T>::RewardsRecorded(agent, amount));
            Ok(().into())
        }

        /// Invoked by offchain stake client. Declare reward received.
        ///
        /// `origin` should be approved account.(not implement yet).
        ///
        /// Q: Should update exchange_rate?
        #[pallet::weight(10_000)]
        #[transactional]
        pub fn record_slash(
            origin: OriginFor<T>,
            agent: T::AccountId,
            #[pallet::compact] amount: Balance,
            block_number: BlockNumber,
        ) -> DispatchResultWithPostInfo {
            T::WithdrawOrigin::ensure_origin(origin)?;
            ensure!(T::Members::contains(&agent), Error::<T>::IllegalAgent);
            ensure!(
                !StakingOperationHistory::<T>::contains_key(
                    Self::current_era(),
                    StakingOperationType::RecordSlash
                ),
                Error::<T>::EventRecorded
            );
            StakingPool::<T>::try_mutate(|m| -> DispatchResult {
                *m = m.checked_sub(amount).ok_or(ArithmeticError::Underflow)?;
                Ok(())
            })?;

            let exchange_rate = Rate::checked_from_rational(
                StakingPool::<T>::get(),
                T::Currency::free_balance(T::LiquidCurrency::get(), &agent),
            )
            .ok_or(Error::<T>::InvalidExchangeRate)?;

            ExchangeRate::<T>::put(exchange_rate);

            StakingOperationHistory::<T>::insert(
                Self::current_era(),
                StakingOperationType::RecordSlash,
                Operation {
                    status: ResponseStatus::Successed,
                    amount,
                    block_number,
                },
            );
            Self::deposit_event(Event::<T>::SlashRecorded(agent, amount));
            Ok(().into())
        }

        /// Invoked by offchain stake client.
        ///
        /// Record bond operation and corresponding data.
        #[pallet::weight(10_000)]
        #[transactional]
        pub fn record_staking_operation_response(
            origin: OriginFor<T>,
            bond_type: StakingOperationType,
            operation: Operation,
        ) -> DispatchResultWithPostInfo {
            T::WithdrawOrigin::ensure_origin(origin)?;
            match bond_type {
                StakingOperationType::Bond
                | StakingOperationType::Unbond
                | StakingOperationType::Rebond
                | StakingOperationType::BondExtra => {
                    StakingOperationHistory::<T>::insert(Self::current_era(), bond_type, operation);
                    Ok(())
                }
                _ => Err(Error::<T>::OperationNotAllowed),
            }?;
            Self::deposit_event(Event::<T>::ResponseRecorded(
                bond_type,
                operation.amount,
                operation.status,
                operation.block_number,
            ));
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

        // todo below three method should be remove while stablity pool is ready
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
        pub fn claim(
            origin: OriginFor<T>,
            #[pallet::compact] amount: Balance,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            <Self as LiquidStakingProtocol<T::AccountId>>::claim(&who)?;
            Ok(().into())
        }
    }
}

impl<T: Config> LiquidStakingProtocol<T::AccountId> for Pallet<T> {
    // After confirmed bond on relaychain, and then mint/deposit xKSM.
    // before record_reward.
    fn stake(who: &T::AccountId, amount: Balance) -> DispatchResultWithPostInfo {
        T::Currency::transfer(T::StakingCurrency::get(), who, &Self::account_id(), amount)?;
        MatchingPool::<T>::try_mutate(&Self::current_era(), |matching_buffer| -> DispatchResult {
            let stability_pool = matching_buffer.stability_pool.clone().or(Some(who.clone()));
            let total_stake_amount = matching_buffer
                .total_stake_amount
                .checked_add(amount)
                .ok_or(ArithmeticError::Overflow)?;
            *matching_buffer = MatchingBuffer {
                stability_pool,
                total_stake_amount,
                total_unstake_amount: matching_buffer.total_unstake_amount,
            };
            Ok(())
        })?;
        Self::deposit_event(Event::Staked(who.clone(), amount));
        Ok(().into())
    }

    // After confirmed unbond on relaychain, and then burn/withdraw xKSM.
    fn unstake(who: &T::AccountId, amount: Balance) -> DispatchResultWithPostInfo {
        T::Currency::transfer(T::LiquidCurrency::get(), who, &Self::account_id(), amount)?;

        let exchange_rate = ExchangeRate::<T>::get();
        let asset_amount = exchange_rate
            .checked_mul_int(amount)
            .ok_or(Error::<T>::InvalidExchangeRate)?;

        MatchingPool::<T>::try_mutate(&Self::current_era(), |matching_buffer| -> DispatchResult {
            let stability_pool = matching_buffer.stability_pool.clone().or(Some(who.clone()));
            let total_unstake_amount = matching_buffer
                .total_unstake_amount
                .checked_add(asset_amount)
                .ok_or(ArithmeticError::Overflow)?;
            *matching_buffer = MatchingBuffer {
                stability_pool,
                total_stake_amount: matching_buffer.total_stake_amount,
                total_unstake_amount,
            };
            Ok(())
        })?;
        Self::deposit_event(Event::Unstaked(who.clone(), amount, asset_amount));
        Ok(().into())
    }

    fn claim(who: &T::AccountId) -> DispatchResultWithPostInfo {
        // let block_number = frame_system::Pallet::<T>::block_number();
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
