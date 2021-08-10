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

impl Default for ResponseStatus {
    fn default() -> Self {
        Self::Ready
    }
}

#[derive(Copy, Clone, Eq, Default, PartialEq, Encode, Decode, RuntimeDebug)]
pub struct Operation {
    amount: Balance,
    block_number: BlockNumber,
    status: ResponseStatus,
}

#[derive(Copy, Clone, Eq, PartialEq, Default, Encode, Decode, RuntimeDebug)]
pub struct MatchingBuffer {
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
        StorageMap<_, Blake2_128Concat, EraIndex, MatchingBuffer, ValueQuery>;

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
    #[pallet::getter(fn stake_queue)]
    pub type MatchingQueue<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        EraIndex,
        MatchingBuffer,
        ValueQuery,
    >;

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
        Operation,
        ValueQuery,
    >;

    /// Store current phase during each era
    #[pallet::storage]
    #[pallet::getter(fn current_phase)]
    pub type CurrentPhase<T: Config> = StorageValue<_, Phase, ValueQuery>;

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
            Ok(().into())
        }

        //todo，record reward on each era, invoked by stake-client
        #[pallet::weight(10_000)]
        #[transactional]
        pub fn record_reward(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            Ok(().into())
        }

        //todo invoked by stake-client, considering insurrance pool
        #[pallet::weight(10_000)]
        #[transactional]
        pub fn record_slash(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            Ok(().into())
        }

        // bond/unbond/rebond/bond_extra may be merge into one
        #[pallet::weight(10_000)]
        #[transactional]
        pub fn record_bond_response(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            Ok(().into())
        }

        #[pallet::weight(10_000)]
        #[transactional]
        pub fn record_bond_extra_response(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            Ok(().into())
        }

        #[pallet::weight(10_000)]
        #[transactional]
        pub fn record_rebond_response(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            Ok(().into())
        }

        #[pallet::weight(10_000)]
        #[transactional]
        pub fn record_unbond_response(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
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
        pub fn claim_token(
            origin: OriginFor<T>,
            #[pallet::compact] amount: Balance,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            <Self as LiquidStakingProtocol<T::AccountId>>::claim(&who)?;
            Ok(().into())
        }

        // pub fn claim_xtoken(){

        // }
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
