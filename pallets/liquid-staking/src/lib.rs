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

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub mod types;
pub mod weights;

use primitives::{ExchangeRateProvider, LiquidStakingCurrenciesProvider, Rate};

pub use pallet::*;

macro_rules! switch_relay {
    ({ $( $code:tt )* }) => {
        if T::RelayNetwork::get() == NetworkId::Polkadot {
            use crate::types::PolkadotCall as RelaychainCall;

            $( $code )*
        } else if T::RelayNetwork::get() == NetworkId::Kusama {
            use crate::types::KusamaCall as RelaychainCall;

            $( $code )*
        } else if T::RelayNetwork::get() == NetworkId::Named("westend".into()) {
            use crate::types::WestendCall as RelaychainCall;

            $( $code )*
        } else {
            unreachable!()
        }
    }
}

#[frame_support::pallet]
pub mod pallet {
    use cumulus_primitives_core::ParaId;
    use frame_support::{
        dispatch::{DispatchResult, DispatchResultWithPostInfo},
        ensure,
        pallet_prelude::*,
        traits::{
            fungibles::{Inspect, Mutate, Transfer},
            Get, IsType,
        },
        transactional,
        weights::Weight,
        BoundedVec, PalletId,
    };
    use frame_system::{
        ensure_signed,
        pallet_prelude::{BlockNumberFor, OriginFor},
    };
    use sp_runtime::{
        traits::{
            AccountIdConversion, AtLeast32BitUnsigned, CheckedAdd, CheckedSub, Saturating,
            StaticLookup, Zero,
        },
        ArithmeticError, FixedPointNumber, FixedPointOperand,
    };
    use sp_std::vec;
    use sp_std::{boxed::Box, vec::Vec};
    use xcm::{latest::prelude::*, DoubleEncoded};

    use primitives::{DerivativeProvider, Rate, Ratio};

    use crate::{types::*, weights::WeightInfo};

    pub type AssetIdOf<T> =
        <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;
    pub type BalanceOf<T> =
        <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// Assets for deposit/withdraw assets to/from pallet account
        type Assets: Transfer<Self::AccountId> + Mutate<Self::AccountId>;

        /// The origin which can do operation on relaychain using parachain's sovereign account
        type RelayOrigin: EnsureOrigin<Self::Origin>;

        /// The origin which can update liquid currency, staking currency
        type UpdateOrigin: EnsureOrigin<Self::Origin>;

        /// The pallet id of liquid staking, keeps all the staking assets
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// XCM message sender
        type XcmSender: SendXcm;

        /// Basic xcm transaction weight per message
        #[pallet::constant]
        type BaseXcmWeight: Get<Weight>;

        /// Returns the parachain ID we are running with.
        #[pallet::constant]
        type SelfParaId: Get<ParaId>;

        /// Account derivative index
        #[pallet::constant]
        type DerivativeIndex: Get<u16>;

        /// Account derivative functionality provider
        type DerivativeProvider: DerivativeProvider<Self::AccountId>;

        /// Unstake queue capacity
        #[pallet::constant]
        type UnstakeQueueCapacity: Get<u32>;

        /// Basis of period
        #[pallet::constant]
        type PeriodBasis: Get<BlockNumberFor<Self>>;

        /// Max rewards per era
        #[pallet::constant]
        type MaxRewardsPerEra: Get<BalanceOf<Self>>;

        /// Max slashes per era
        #[pallet::constant]
        type MaxSlashesPerEra: Get<BalanceOf<Self>>;

        /// Minimum stake amount
        #[pallet::constant]
        type MinStakeAmount: Get<BalanceOf<Self>>;

        /// Minimum unstake amount
        #[pallet::constant]
        type MinUnstakeAmount: Get<BalanceOf<Self>>;

        /// Relay network
        #[pallet::constant]
        type RelayNetwork: Get<NetworkId>;

        /// Weight information
        type WeightInfo: WeightInfo;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    #[pallet::metadata(T::AccountId = "AccountId", BalanceOf<T> = "Balance")]
    pub enum Event<T: Config> {
        /// The assets get staked successfully
        Staked(T::AccountId, BalanceOf<T>),
        /// The derivative get unstaked successfully
        Unstaked(T::AccountId, BalanceOf<T>, BalanceOf<T>),
        /// Rewards/Slashes has been recorded.
        StakingSettlementRecorded(StakingSettlementKind, BalanceOf<T>),
        /// Request to perform bond/rebond/unbond in relay chain
        ///
        /// Send `(bond_amount, rebond_amount, unbond_amount)` as args.
        StakingOpRequest(BalanceOf<T>, BalanceOf<T>, BalanceOf<T>),
        /// Period terminated.
        ///
        /// Emit when a period is finished which is defined by `PeriodBasis`. While current block
        /// height is accurately multiple of the basis, the event would be deposited during finalization of
        /// the block.
        PeriodTerminated,
        /// Sent staking.bond call to relaychain
        BondCallSent(T::AccountId, BalanceOf<T>, RewardDestination<T::AccountId>),
        /// Sent staking.bond_extra call to relaychain
        BondExtraCallSent(BalanceOf<T>),
        /// Sent staking.unbond call to relaychain
        UnbondCallSent(BalanceOf<T>),
        /// Sent staking.rebond call to relaychain
        RebondCallSent(BalanceOf<T>),
        /// Sent staking.withdraw_unbonded call to relaychain
        WithdrawUnbondedCallSent(u32),
        /// Send staking.nominate call to relaychain
        NominateCallSent(Vec<T::AccountId>),
        /// Compensation for extrinsics in relaychain was set to new value
        XcmFeesCompensationUpdated(BalanceOf<T>),
        /// Capacity of staking pool was set to new value
        StakingPoolCapacityUpdated(BalanceOf<T>),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Exchange rate is invalid.
        InvalidExchangeRate,
        /// Era has been pushed before.
        EraAlreadyPushed,
        /// Stake amount is too small
        StakeAmountTooSmall,
        /// Unstake amount is too small
        UnstakeAmountTooSmall,
        /// Operation wasn't submitted to relaychain or has been processed.
        OperationNotReady,
        /// Failed to send staking.bond call
        BondCallFailed,
        /// Failed to send staking.bond_extra call
        BondExtraCallFailed,
        /// Failed to send staking.unbond call
        UnbondCallFailed,
        /// Failed to send staking.rebond call
        RebondCallFailed,
        /// Failed to send staking.withdraw_unbonded call
        WithdrawUnbondedCallFailed,
        /// Failed to send staking.nominate call
        NominateCallFailed,
        /// Liquid currency hasn't been set
        LiquidCurrencyNotSet,
        /// Staking currency hasn't been set
        StakingCurrencyNotSet,
        /// Exceeded unstake queue's capacity
        ExceededUnstakeQueueCapacity,
        /// Exceeded max rewards per era
        ExceededMaxRewardsPerEra,
        /// Exceeded max slashes per era
        ExceededMaxSlashesPerEra,
        /// Exceeded staking pool's capacity
        ExceededStakingPoolCapacity,
    }

    /// The exchange rate between relaychain native asset and the voucher.
    #[pallet::storage]
    #[pallet::getter(fn exchange_rate)]
    pub type ExchangeRate<T: Config> = StorageValue<_, Rate, ValueQuery>;

    /// Total amount of staked assets on relaycahin.
    #[pallet::storage]
    #[pallet::getter(fn staking_pool)]
    pub type StakingPool<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    /// Total amount of charged assets to be used as xcm fees.
    #[pallet::storage]
    #[pallet::getter(fn insurance_pool)]
    pub type InsurancePool<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    /// Fraction of reward currently set aside for reserves.
    #[pallet::storage]
    #[pallet::getter(fn reserve_factor)]
    pub type ReserveFactor<T: Config> = StorageValue<_, Ratio, ValueQuery>;

    /// Store total stake amount and unstake amount in each era,
    /// And will update when stake/unstake occurred.
    #[pallet::storage]
    #[pallet::getter(fn matching_pool)]
    pub type MatchingPool<T: Config> = StorageValue<_, MatchingLedger<BalanceOf<T>>, ValueQuery>;

    /// Manage which we should pay off to.
    ///
    /// Insert a new record while user can't be paid instantly in unstaking operation.
    #[pallet::storage]
    #[pallet::getter(fn unstake_queue)]
    pub type UnstakeQueue<T: Config> = StorageValue<
        _,
        BoundedVec<(T::AccountId, BalanceOf<T>), T::UnstakeQueueCapacity>,
        ValueQuery,
    >;

    /// Liquid currency asset id
    #[pallet::storage]
    pub type LiquidCurrency<T: Config> = StorageValue<_, AssetIdOf<T>, OptionQuery>;

    /// Staking currency asset id
    #[pallet::storage]
    pub type StakingCurrency<T: Config> = StorageValue<_, AssetIdOf<T>, OptionQuery>;

    /// Relaychain xcm fees compensation
    #[pallet::storage]
    #[pallet::getter(fn xcm_fees_compensation)]
    pub type XcmFeesCompensation<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    /// Staking pool capacity
    #[pallet::storage]
    #[pallet::getter(fn staking_pool_capacity)]
    pub type StakingPoolCapacity<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

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
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T>
    where
        [u8; 32]: From<<T as frame_system::Config>::AccountId>,
        u128: From<
            <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance,
        >,
        BalanceOf<T>: FixedPointOperand,
        AssetIdOf<T>: AtLeast32BitUnsigned,
    {
        /// Try to pay off over the `UnstakeQueue` while blockchain is on idle.
        ///
        /// It breaks when:
        ///     - Pallet's balance is insufficiant.
        ///     - Queue is empty.
        ///     - `remaining_weight` is less than one pop_queue needed.
        fn on_idle(_n: BlockNumberFor<T>, mut remaining_weight: Weight) -> Weight {
            // TODO should use T::WeightInfo::on_idle instead
            // on_idle shouldn't run out of all remaining_weight normally
            let base_weight = T::WeightInfo::pop_queue();
            let staking_currency = Self::staking_currency();

            // Return if staking_currency haven't been set.
            if staking_currency.is_err() {
                return remaining_weight;
            }

            let staking_currency = staking_currency.expect("It must be ok; qed");

            loop {
                // Check weight is enough
                if remaining_weight < base_weight {
                    break;
                }

                if Self::unstake_queue().is_empty() {
                    break;
                }

                // Get the front of the queue.
                let (who, amount) = &Self::unstake_queue()[0];
                let account_id = Self::account_id();

                // InsurancePool should not be embazzled.
                let free_balance =
                    T::Assets::reducible_balance(staking_currency, &account_id, false)
                        .saturating_sub(Self::insurance_pool());
                if free_balance < *amount {
                    return remaining_weight;
                }

                if T::Assets::transfer(staking_currency, &account_id, who, *amount, false).is_err()
                {
                    // break if we cannot afford this
                    break;
                }

                // substract weight of this action if succeed.
                remaining_weight -= base_weight;

                // remove unstake request from queue
                Self::pop_unstake_task()
            }

            remaining_weight
        }

        fn on_finalize(n: BlockNumberFor<T>) {
            let basis = T::PeriodBasis::get();

            // check if current period end.
            if !(n % basis).is_zero() {
                return;
            }

            // check if there are staking to be settled.
            if Self::matching_pool().is_empty() {
                return;
            }

            Self::deposit_event(Event::<T>::PeriodTerminated);
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T>
    where
        [u8; 32]: From<<T as frame_system::Config>::AccountId>,
        u128: From<
            <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance,
        >,
        BalanceOf<T>: FixedPointOperand,
        AssetIdOf<T>: AtLeast32BitUnsigned,
    {
        /// Put assets under staking, the native assets will be transferred to the account
        /// owned by the pallet, user receive derivative in return, such derivative can be
        /// further used as collateral for lending.
        ///
        /// - `amount`: the amount of staking assets
        #[pallet::weight(<T as Config>::WeightInfo::stake())]
        #[transactional]
        pub fn stake(
            origin: OriginFor<T>,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            ensure!(
                amount > T::MinStakeAmount::get(),
                Error::<T>::StakeAmountTooSmall
            );

            T::Assets::transfer(
                Self::staking_currency()?,
                &who,
                &Self::account_id(),
                amount,
                false,
            )?;

            // Calculate staking fee and add it to insurance pool
            let fees = Self::reserve_factor().mul_floor(amount);
            InsurancePool::<T>::try_mutate(|b| -> DispatchResult {
                *b = b.checked_add(&fees).ok_or(ArithmeticError::Overflow)?;
                Ok(())
            })?;

            // Amount that we should mint to user
            let amount = amount
                .checked_sub(&fees)
                .ok_or(ArithmeticError::Underflow)?;
            let liquid_amount = Self::exchange_rate()
                .reciprocal()
                .and_then(|r| r.checked_mul_int(amount))
                .ok_or(Error::<T>::InvalidExchangeRate)?;
            T::Assets::mint_into(Self::liquid_currency()?, &who, liquid_amount)?;

            StakingPool::<T>::try_mutate(|b| -> DispatchResult {
                let new_amount = b.checked_add(&amount).ok_or(ArithmeticError::Overflow)?;
                ensure!(
                    new_amount < StakingPoolCapacity::<T>::get(),
                    Error::<T>::ExceededStakingPoolCapacity
                );
                *b = new_amount;
                Ok(())
            })?;

            MatchingPool::<T>::try_mutate(|p| -> DispatchResult {
                p.total_stake_amount = p
                    .total_stake_amount
                    .checked_add(&amount)
                    .ok_or(ArithmeticError::Overflow)?;
                Ok(())
            })?;

            Self::deposit_event(Event::<T>::Staked(who, amount));
            Ok(().into())
        }

        /// Unstake by exchange derivative for assets, the assets will not be avaliable immediately.
        /// Instead, the request is recorded and pending for the nomination accounts in relay
        /// chain to do the `unbond` operation.
        ///
        /// - `amount`: the amount of derivative
        #[pallet::weight(<T as Config>::WeightInfo::unstake())]
        #[transactional]
        pub fn unstake(
            origin: OriginFor<T>,
            #[pallet::compact] liquid_amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            ensure!(
                liquid_amount > T::MinUnstakeAmount::get(),
                Error::<T>::UnstakeAmountTooSmall
            );

            let exchange_rate = ExchangeRate::<T>::get();
            let asset_amount = exchange_rate
                .checked_mul_int(liquid_amount)
                .ok_or(Error::<T>::InvalidExchangeRate)?;

            if T::Assets::transfer(
                Self::staking_currency()?,
                &Self::account_id(),
                &who,
                asset_amount,
                false,
            )
            .is_err()
            {
                Self::push_unstake_task(&who, asset_amount)?;
            }

            T::Assets::burn_from(Self::liquid_currency()?, &who, liquid_amount)?;
            StakingPool::<T>::try_mutate(|b| -> DispatchResult {
                *b = b
                    .checked_sub(&asset_amount)
                    .ok_or(ArithmeticError::Underflow)?;
                Ok(())
            })?;

            MatchingPool::<T>::try_mutate(|p| -> DispatchResult {
                p.total_unstake_amount = p
                    .total_unstake_amount
                    .checked_add(&asset_amount)
                    .ok_or(ArithmeticError::Overflow)?;
                Ok(())
            })?;

            Self::deposit_event(Event::<T>::Unstaked(who, liquid_amount, asset_amount));
            Ok(().into())
        }

        /// Handle staking settlement at the end of an era
        /// such as getting reward or been slashed in relaychain.
        #[pallet::weight(<T as Config>::WeightInfo::record_staking_settlement())]
        #[transactional]
        pub fn record_staking_settlement(
            origin: OriginFor<T>,
            #[pallet::compact] amount: BalanceOf<T>,
            kind: StakingSettlementKind,
        ) -> DispatchResultWithPostInfo {
            T::RelayOrigin::ensure_origin(origin)?;

            Self::update_staking_pool(kind, amount)?;

            Self::deposit_event(Event::<T>::StakingSettlementRecorded(kind, amount));
            Ok(().into())
        }

        #[pallet::weight(<T as Config>::WeightInfo::update_xcm_fees_compensation())]
        #[transactional]
        pub fn update_xcm_fees_compensation(
            origin: OriginFor<T>,
            fee: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            T::RelayOrigin::ensure_origin(origin)?;
            XcmFeesCompensation::<T>::mutate(|v| *v = fee);
            Self::deposit_event(Event::<T>::XcmFeesCompensationUpdated(fee));
            Ok(().into())
        }

        #[pallet::weight(<T as Config>::WeightInfo::update_staking_pool_capacity())]
        #[transactional]
        pub fn update_staking_pool_capacity(
            origin: OriginFor<T>,
            capacity: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            T::RelayOrigin::ensure_origin(origin)?;
            StakingPoolCapacity::<T>::mutate(|v| *v = capacity);
            Self::deposit_event(Event::<T>::StakingPoolCapacityUpdated(capacity));
            Ok(().into())
        }

        /// Do settlement for matching pool.
        ///
        /// Calculate the imbalance of current state and send corresponding operations to
        /// relay-chain.
        #[pallet::weight(<T as Config>::WeightInfo::settlement())]
        #[transactional]
        pub fn settlement(
            origin: OriginFor<T>,
            bond_extra: bool,
            #[pallet::compact] unbonding_amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            T::RelayOrigin::ensure_origin(origin)?;

            let (bond_amount, rebond_amount, unbond_amount) =
                MatchingPool::<T>::take().matching(unbonding_amount);
            let staking_currency = Self::staking_currency()?;
            let account_id = Self::account_id();
            let xcm_fees_compensation = Self::xcm_fees_compensation();

            if !Self::xcm_fees_compensation().is_zero() {
                T::Assets::burn_from(staking_currency, &account_id, xcm_fees_compensation)?;

                InsurancePool::<T>::try_mutate(|b| -> DispatchResult {
                    *b = b
                        .checked_sub(&Self::xcm_fees_compensation())
                        .ok_or(ArithmeticError::Underflow)?;
                    Ok(())
                })?;
            }

            if !bond_amount.is_zero() {
                T::Assets::burn_from(staking_currency, &account_id, bond_amount)?;

                if !bond_extra {
                    Self::bond_internal(bond_amount, RewardDestination::Staked)?;
                } else {
                    Self::bond_extra_internal(bond_amount)?;
                }
            }

            if !unbond_amount.is_zero() {
                Self::unbond_internal(unbond_amount)?;
            }

            if !rebond_amount.is_zero() {
                Self::rebond_internal(rebond_amount)?;
            }

            Self::deposit_event(Event::<T>::StakingOpRequest(
                bond_amount,
                rebond_amount,
                unbond_amount,
            ));

            Ok(().into())
        }

        /// Bond on relaychain via xcm.transact
        #[pallet::weight(<T as Config>::WeightInfo::bond())]
        #[transactional]
        pub fn bond(
            origin: OriginFor<T>,
            value: BalanceOf<T>,
            payee: RewardDestination<T::AccountId>,
        ) -> DispatchResult {
            T::RelayOrigin::ensure_origin(origin)?;
            Self::bond_internal(value, payee)?;
            Ok(())
        }

        /// Bond_extra on relaychain via xcm.transact
        #[pallet::weight(<T as Config>::WeightInfo::bond_extra())]
        #[transactional]
        pub fn bond_extra(origin: OriginFor<T>, value: BalanceOf<T>) -> DispatchResult {
            T::RelayOrigin::ensure_origin(origin)?;
            Self::bond_extra_internal(value)?;
            Ok(())
        }

        /// unbond on relaychain via xcm.transact
        #[pallet::weight(<T as Config>::WeightInfo::unbond())]
        #[transactional]
        pub fn unbond(origin: OriginFor<T>, value: BalanceOf<T>) -> DispatchResult {
            T::RelayOrigin::ensure_origin(origin)?;
            Self::unbond_internal(value)?;
            Ok(())
        }

        /// rebond on relaychain via xcm.transact
        #[pallet::weight(<T as Config>::WeightInfo::rebond())]
        #[transactional]
        pub fn rebond(origin: OriginFor<T>, value: BalanceOf<T>) -> DispatchResult {
            T::RelayOrigin::ensure_origin(origin)?;
            Self::rebond_internal(value)?;
            Ok(())
        }

        /// withdraw unbonded on relaychain via xcm.transact
        #[pallet::weight(<T as Config>::WeightInfo::withdraw_unbonded())]
        #[transactional]
        pub fn withdraw_unbonded(
            origin: OriginFor<T>,
            num_slashing_spans: u32,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            T::RelayOrigin::ensure_origin(origin)?;
            Self::withdraw_unbonded_internal(num_slashing_spans, amount)?;
            Ok(())
        }

        /// Nominate on relaychain via xcm.transact
        #[pallet::weight(<T as Config>::WeightInfo::nominate())]
        #[transactional]
        pub fn nominate(origin: OriginFor<T>, targets: Vec<T::AccountId>) -> DispatchResult {
            T::RelayOrigin::ensure_origin(origin)?;

            let targets_source = targets
                .clone()
                .into_iter()
                .map(T::Lookup::unlookup)
                .collect();

            switch_relay!({
                let call = RelaychainCall::Utility(Box::new(UtilityCall::AsDerivative(
                    UtilityAsDerivativeCall {
                        index: T::DerivativeIndex::get(),
                        call: RelaychainCall::Staking::<T>(StakingCall::Nominate(
                            StakingNominateCall {
                                targets: targets_source,
                            },
                        )),
                    },
                )));

                let msg = Self::ump_transact(call.encode().into());

                match T::XcmSender::send_xcm(MultiLocation::parent(), msg) {
                    Ok(()) => {
                        Self::deposit_event(Event::<T>::NominateCallSent(targets));
                    }
                    Err(_e) => {
                        return Err(Error::<T>::NominateCallFailed.into());
                    }
                }
            });

            Ok(())
        }

        /// set liquid currency via governance
        #[pallet::weight(<T as Config>::WeightInfo::set_liquid_currency())]
        #[transactional]
        pub fn set_liquid_currency(origin: OriginFor<T>, asset_id: AssetIdOf<T>) -> DispatchResult {
            T::UpdateOrigin::ensure_origin(origin)?;
            LiquidCurrency::<T>::put(asset_id);
            Ok(())
        }

        /// set staking currency via governance
        #[pallet::weight(<T as Config>::WeightInfo::set_staking_currency())]
        #[transactional]
        pub fn set_staking_currency(
            origin: OriginFor<T>,
            asset_id: AssetIdOf<T>,
        ) -> DispatchResult {
            T::UpdateOrigin::ensure_origin(origin)?;
            StakingCurrency::<T>::put(asset_id);
            Ok(())
        }
    }

    impl<T: Config> Pallet<T>
    where
        [u8; 32]: From<<T as frame_system::Config>::AccountId>,
        u128: From<
            <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance,
        >,
        BalanceOf<T>: FixedPointOperand,
        AssetIdOf<T>: AtLeast32BitUnsigned,
    {
        /// Staking pool account
        pub fn account_id() -> T::AccountId {
            T::PalletId::get().into_account()
        }

        /// Parachain sovereign account
        pub fn para_account_id() -> T::AccountId {
            T::SelfParaId::get().into_account()
        }

        pub fn staking_currency() -> Result<AssetIdOf<T>, DispatchError> {
            StakingCurrency::<T>::get()
                .ok_or(Error::<T>::StakingCurrencyNotSet)
                .map_err(Into::into)
        }

        pub fn liquid_currency() -> Result<AssetIdOf<T>, DispatchError> {
            LiquidCurrency::<T>::get()
                .ok_or(Error::<T>::LiquidCurrencyNotSet)
                .map_err(Into::into)
        }

        /// Derivative parachain account
        pub fn derivative_para_account_id() -> T::AccountId {
            T::DerivativeProvider::derivative_account_id(
                Self::para_account_id(),
                T::DerivativeIndex::get(),
            )
        }

        /// Increase / decrease staked asset in staking pool, and synchronized the exchange rate.
        fn update_staking_pool(
            kind: StakingSettlementKind,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            use StakingSettlementKind::*;
            match kind {
                Reward => {
                    ensure!(
                        amount <= T::MaxRewardsPerEra::get(),
                        Error::<T>::ExceededMaxRewardsPerEra
                    );
                    StakingPool::<T>::try_mutate(|p| -> DispatchResult {
                        *p = p.checked_add(&amount).ok_or(ArithmeticError::Overflow)?;
                        Ok(())
                    })
                }
                Slash => {
                    ensure!(
                        amount <= T::MaxSlashesPerEra::get(),
                        Error::<T>::ExceededMaxSlashesPerEra
                    );
                    StakingPool::<T>::try_mutate(|p| -> DispatchResult {
                        *p = p.checked_sub(&amount).ok_or(ArithmeticError::Underflow)?;
                        Ok(())
                    })
                }
            }?;

            // Update exchange rate.
            let exchange_rate = Rate::checked_from_rational(
                StakingPool::<T>::get(),
                T::Assets::total_issuance(Self::liquid_currency()?),
            )
            .ok_or(Error::<T>::InvalidExchangeRate)?;
            ExchangeRate::<T>::put(exchange_rate);

            Ok(())
        }

        fn bond_internal(
            value: BalanceOf<T>,
            payee: RewardDestination<T::AccountId>,
        ) -> DispatchResult {
            let stash = Self::derivative_para_account_id();
            let controller = stash.clone();

            switch_relay!({
                let call =
                    RelaychainCall::Utility(Box::new(UtilityCall::BatchAll(UtilityBatchAllCall {
                        calls: vec![
                            RelaychainCall::Balances(BalancesCall::TransferKeepAlive(
                                BalancesTransferKeepAliveCall {
                                    dest: T::Lookup::unlookup(stash),
                                    value,
                                },
                            )),
                            RelaychainCall::Utility(Box::new(UtilityCall::AsDerivative(
                                UtilityAsDerivativeCall {
                                    index: T::DerivativeIndex::get(),
                                    call: RelaychainCall::Staking::<T>(StakingCall::Bond(
                                        StakingBondCall {
                                            controller: T::Lookup::unlookup(controller.clone()),
                                            value,
                                            payee: payee.clone(),
                                        },
                                    )),
                                },
                            ))),
                        ],
                    })));

                let msg = Self::ump_transact(call.encode().into());

                match T::XcmSender::send_xcm(MultiLocation::parent(), msg) {
                    Ok(()) => {
                        Self::deposit_event(Event::<T>::BondCallSent(controller, value, payee));
                    }
                    Err(_e) => {
                        return Err(Error::<T>::BondCallFailed.into());
                    }
                }
            });

            Ok(())
        }

        fn bond_extra_internal(value: BalanceOf<T>) -> DispatchResult {
            let stash = T::Lookup::unlookup(Self::derivative_para_account_id());

            switch_relay!({
                let call =
                    RelaychainCall::Utility(Box::new(UtilityCall::BatchAll(UtilityBatchAllCall {
                        calls: vec![
                            RelaychainCall::Balances(BalancesCall::TransferKeepAlive(
                                BalancesTransferKeepAliveCall { dest: stash, value },
                            )),
                            RelaychainCall::Utility(Box::new(UtilityCall::AsDerivative(
                                UtilityAsDerivativeCall {
                                    index: T::DerivativeIndex::get(),
                                    call: RelaychainCall::Staking::<T>(StakingCall::BondExtra(
                                        StakingBondExtraCall { value },
                                    )),
                                },
                            ))),
                        ],
                    })));

                let msg = Self::ump_transact(call.encode().into());

                match T::XcmSender::send_xcm(MultiLocation::parent(), msg) {
                    Ok(()) => {
                        Self::deposit_event(Event::<T>::BondExtraCallSent(value));
                    }
                    Err(_e) => {
                        return Err(Error::<T>::BondExtraCallFailed.into());
                    }
                }
            });

            Ok(())
        }

        fn unbond_internal(value: BalanceOf<T>) -> DispatchResult {
            switch_relay!({
                let call = RelaychainCall::Utility(Box::new(UtilityCall::AsDerivative(
                    UtilityAsDerivativeCall {
                        index: T::DerivativeIndex::get(),
                        call: RelaychainCall::Staking::<T>(StakingCall::Unbond(
                            StakingUnbondCall { value },
                        )),
                    },
                )));

                let msg = Self::ump_transact(call.encode().into());

                match T::XcmSender::send_xcm(MultiLocation::parent(), msg) {
                    Ok(()) => {
                        Self::deposit_event(Event::<T>::UnbondCallSent(value));
                    }
                    Err(_e) => {
                        return Err(Error::<T>::UnbondCallFailed.into());
                    }
                }
            });

            Ok(())
        }

        fn rebond_internal(value: BalanceOf<T>) -> DispatchResult {
            switch_relay!({
                let call = RelaychainCall::Utility(Box::new(UtilityCall::AsDerivative(
                    UtilityAsDerivativeCall {
                        index: T::DerivativeIndex::get(),
                        call: RelaychainCall::Staking::<T>(StakingCall::Rebond(
                            StakingRebondCall { value },
                        )),
                    },
                )));

                let msg = Self::ump_transact(call.encode().into());

                match T::XcmSender::send_xcm(MultiLocation::parent(), msg) {
                    Ok(()) => {
                        Self::deposit_event(Event::<T>::RebondCallSent(value));
                    }
                    Err(_e) => {
                        return Err(Error::<T>::RebondCallFailed.into());
                    }
                }
            });

            Ok(())
        }

        fn withdraw_unbonded_internal(
            num_slashing_spans: u32,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            switch_relay!({
                let call =
                    RelaychainCall::Utility(Box::new(UtilityCall::BatchAll(UtilityBatchAllCall {
                        calls: vec![
                            RelaychainCall::Utility(Box::new(UtilityCall::AsDerivative(
                                UtilityAsDerivativeCall {
                                    index: T::DerivativeIndex::get(),
                                    call: RelaychainCall::Staking::<T>(
                                        StakingCall::WithdrawUnbonded(
                                            StakingWithdrawUnbondedCall { num_slashing_spans },
                                        ),
                                    ),
                                },
                            ))),
                            RelaychainCall::Utility(Box::new(UtilityCall::AsDerivative(
                                UtilityAsDerivativeCall {
                                    index: T::DerivativeIndex::get(),
                                    call: RelaychainCall::Balances::<T>(BalancesCall::TransferAll(
                                        BalancesTransferAllCall {
                                            dest: T::Lookup::unlookup(Self::para_account_id()),
                                            keep_alive: true,
                                        },
                                    )),
                                },
                            ))),
                        ],
                    })));

                let msg = Self::ump_transact(call.encode().into());

                match T::XcmSender::send_xcm(MultiLocation::parent(), msg) {
                    Ok(()) => {
                        Self::deposit_event(Event::<T>::WithdrawUnbondedCallSent(
                            num_slashing_spans,
                        ));
                    }
                    Err(_e) => {
                        return Err(Error::<T>::WithdrawUnbondedCallFailed.into());
                    }
                }
            });

            T::Assets::mint_into(Self::staking_currency()?, &Self::account_id(), amount)?;

            Ok(())
        }

        /// push an unstake task into queue.
        #[inline]
        fn push_unstake_task(who: &T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
            UnstakeQueue::<T>::try_mutate(|q| -> DispatchResult {
                q.try_push((who.clone(), amount))
                    .map_err(|_| Error::<T>::ExceededUnstakeQueueCapacity)?;
                Ok(())
            })
        }

        /// Pop an unstake task from queue.
        #[inline]
        fn pop_unstake_task() {
            UnstakeQueue::<T>::mutate(|v| v.remove(0));
        }

        fn ump_transact(call: DoubleEncoded<()>) -> Xcm<()> {
            let asset: MultiAsset = (MultiLocation::here(), 1_000_000_000_000).into();

            WithdrawAsset {
                assets: MultiAssets::from(asset.clone()),
                effects: vec![
                    BuyExecution {
                        fees: asset,
                        weight: 2_000_000_000,
                        debt: 600_000_000,
                        halt_on_error: false,
                        instructions: vec![Transact {
                            origin_type: OriginKind::SovereignAccount,
                            require_weight_at_most: u64::MAX,
                            call,
                        }],
                    },
                    DepositAsset {
                        assets: All.into(),
                        max_assets: u32::max_value(),
                        beneficiary: X1(AccountId32 {
                            network: NetworkId::Any,
                            id: Self::para_account_id().into(),
                        })
                        .into(),
                    },
                ],
            }
        }
    }
}

impl<T: Config> ExchangeRateProvider for Pallet<T> {
    fn get_exchange_rate() -> Rate {
        ExchangeRate::<T>::get()
    }
}

impl<T: Config> LiquidStakingCurrenciesProvider<AssetIdOf<T>> for Pallet<T> {
    fn get_staking_currency() -> Option<AssetIdOf<T>> {
        StakingCurrency::<T>::get()
    }

    fn get_liquid_currency() -> Option<AssetIdOf<T>> {
        LiquidCurrency::<T>::get()
    }
}
