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

mod benchmarking;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub mod types;
pub mod weights;

#[macro_use]
extern crate primitives;

use frame_support::traits::Get;
use primitives::{ExchangeRateProvider, LiquidStakingCurrenciesProvider, Rate};
use sp_runtime::traits::Zero;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        dispatch::{DispatchResult, DispatchResultWithPostInfo},
        ensure, log,
        pallet_prelude::*,
        require_transactional,
        traits::{
            fungibles::{Inspect, InspectMetadata, Mutate, Transfer},
            IsType,
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
        traits::{AccountIdConversion, Convert},
        ArithmeticError, FixedPointNumber,
    };
    use sp_std::vec::Vec;
    use xcm::latest::prelude::*;

    use primitives::{ump::*, Balance, CurrencyId, ParaId, Rate, Ratio};

    use super::{types::*, weights::WeightInfo, *};
    use pallet_xcm_helper::XcmHelper;

    pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
    pub type AssetIdOf<T> =
        <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;
    pub type BalanceOf<T> =
        <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_utility::Config + pallet_xcm::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// Assets for deposit/withdraw assets to/from pallet account
        type Assets: Transfer<Self::AccountId, AssetId = CurrencyId>
            + Mutate<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
            + InspectMetadata<Self::AccountId, AssetId = CurrencyId, Balance = Balance>;

        /// The origin which can do operation on relaychain using parachain's sovereign account
        type RelayOrigin: EnsureOrigin<<Self as frame_system::Config>::Origin>;

        /// The origin which can update liquid currency, staking currency and other parameters
        type UpdateOrigin: EnsureOrigin<<Self as frame_system::Config>::Origin>;

        /// The pallet id of liquid staking, keeps all the staking assets
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// Returns the parachain ID we are running with.
        #[pallet::constant]
        type SelfParaId: Get<ParaId>;

        /// Account derivative index
        #[pallet::constant]
        type DerivativeIndex: Get<u16>;

        /// Convert `T::AccountId` to `MultiLocation`.
        type AccountIdToMultiLocation: Convert<Self::AccountId, MultiLocation>;

        /// Staking currency
        #[pallet::constant]
        type StakingCurrency: Get<AssetIdOf<Self>>;

        /// Liquid currency
        #[pallet::constant]
        type LiquidCurrency: Get<AssetIdOf<Self>>;

        /// Unstake queue capacity
        #[pallet::constant]
        type UnstakeQueueCapacity: Get<u32>;

        /// Minimum stake amount
        #[pallet::constant]
        type MinStakeAmount: Get<BalanceOf<Self>>;

        /// Minimum unstake amount
        #[pallet::constant]
        type MinUnstakeAmount: Get<BalanceOf<Self>>;

        /// Weight information
        type WeightInfo: WeightInfo;

        /// To expose XCM helper functions
        type XCM: XcmHelper<Self, BalanceOf<Self>, AssetIdOf<Self>, Self::AccountId>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// The assets get staked successfully
        Staked(T::AccountId, BalanceOf<T>),
        /// The derivative get unstaked successfully
        Unstaked(T::AccountId, BalanceOf<T>, BalanceOf<T>),
        /// Rewards/Slashes has been recorded.
        StakingSettlementRecorded(StakingSettlementKind, BalanceOf<T>),
        /// Request to perform bond/rebond/unbond on relay chain
        ///
        /// Send `(bond_amount, rebond_amount, unbond_amount)` as args.
        Settlement(BalanceOf<T>, BalanceOf<T>, BalanceOf<T>),
        /// Sent staking.bond call to relaychain
        Bonding(T::AccountId, BalanceOf<T>, RewardDestination<T::AccountId>),
        /// Sent staking.bond_extra call to relaychain
        BondingExtra(BalanceOf<T>),
        /// Sent staking.unbond call to relaychain
        Unbonding(BalanceOf<T>),
        /// Sent staking.rebond call to relaychain
        Rebonding(BalanceOf<T>),
        /// Sent staking.withdraw_unbonded call to relaychain
        WithdrawingUnbonded(u32),
        /// Sent staking.nominate call to relaychain
        Nominating(Vec<T::AccountId>),
        /// Compensation for extrinsics on relaychain was set to new value
        XcmFeesUpdated(BalanceOf<T>),
        /// Capacity of staking pool was set to new value
        StakingPoolCapacityUpdated(BalanceOf<T>),
        /// Xcm weight in BuyExecution message
        XcmWeightUpdated(XcmWeightMisc<Weight>),
        /// InsurancePool's reserve_factor updated
        ReserveFactorUpdated(Ratio),
        /// Add asset to insurance pool
        InsurancesAdded(T::AccountId, BalanceOf<T>),
        /// Slash was paid by insurance pool
        SlashPaid(BalanceOf<T>),
        /// Exchange rate was set to new value
        ExchangeRateUpdated(Rate),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Exchange rate is invalid.
        InvalidExchangeRate,
        /// Stake amount is too small
        StakeAmountTooSmall,
        /// Unstake amount is too small
        UnstakeAmountTooSmall,
        /// Liquid currency hasn't been set
        LiquidCurrencyNotReady,
        /// Staking currency hasn't been set
        StakingCurrencyNotReady,
        /// Exceeded unstake queue's capacity
        ExceededUnstakeQueueCapacity,
        /// The cap cannot be zero
        ZeroCap,
        /// The factor should be bigger than 0% and smaller than 100%
        InvalidFactor,
        /// fees cannot be zero
        ZeroFees,
    }

    /// The exchange rate between relaychain native asset and the voucher.
    #[pallet::storage]
    #[pallet::getter(fn exchange_rate)]
    pub type ExchangeRate<T: Config> = StorageValue<_, Rate, ValueQuery>;

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

    /// Xcm weight in BuyExecution
    #[pallet::storage]
    #[pallet::getter(fn xcm_weight)]
    pub type XcmWeight<T: Config> = StorageValue<_, XcmWeightMisc<Weight>, ValueQuery>;

    /// Staking pool capacity
    #[pallet::storage]
    #[pallet::getter(fn staking_pool_capacity)]
    pub type StakingPoolCapacity<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    /// Total amount of charged assets to be used as xcm fees.
    #[pallet::storage]
    #[pallet::getter(fn insurance_pool)]
    pub type InsurancePool<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

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
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// Try to pay off over the `UnstakeQueue` while blockchain is on idle.
        ///
        /// It breaks when:
        ///     - Pallet's balance is insufficiant.
        ///     - Queue is empty.
        ///     - `remaining_weight` is less than one pop_queue needed.
        fn on_idle(_n: BlockNumberFor<T>, mut remaining_weight: Weight) -> Weight {
            // on_idle shouldn't run out of all remaining_weight normally
            let base_weight = <T as Config>::WeightInfo::on_idle();
            let staking_currency = Self::staking_currency();

            // return if staking_currency haven't been set.
            if staking_currency.is_err() {
                return remaining_weight;
            }

            let staking_currency = staking_currency.expect("It must be ok; qed");

            loop {
                // check weight is enough
                if remaining_weight < base_weight {
                    break;
                }

                if Self::unstake_queue().is_empty() {
                    break;
                }

                // get the front of the queue.
                let (who, amount) = &Self::unstake_queue()[0];
                let account_id = Self::account_id();

                // InsurancePool should not be embazzled.
                let free_balance =
                    T::Assets::reducible_balance(staking_currency, &account_id, false)
                        .saturating_sub(Self::insurance_pool());

                log::trace!(
                    target: "liquidstaking::on_idle",
                    "account: {:?}, unstake_amount: {:?}, remaining_weight: {:?}, pallet_free_balance: {:?}",
                    who,
                    amount,
                    remaining_weight,
                    free_balance,
                );
                if free_balance < *amount {
                    return remaining_weight;
                }

                if let Err(err) =
                    T::Assets::transfer(staking_currency, &account_id, who, *amount, false)
                {
                    log::error!(target: "liquidstaking::on_idle", "Transfer failed {:?}", err);
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
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
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

            // calculate staking fee and add it to insurance pool
            let fees = Self::reserve_factor().mul_floor(amount);
            let amount = amount.checked_sub(fees).ok_or(ArithmeticError::Underflow)?;

            T::Assets::transfer(
                Self::staking_currency()?,
                &who,
                &Self::account_id(),
                amount,
                false,
            )?;

            T::XCM::add_xcm_fees(Self::staking_currency()?, &who, fees)?;
            let liquid_amount = Self::exchange_rate()
                .reciprocal()
                .and_then(|r| r.checked_mul_int(amount))
                .ok_or(Error::<T>::InvalidExchangeRate)?;
            T::Assets::mint_into(Self::liquid_currency()?, &who, liquid_amount)?;

            MatchingPool::<T>::try_mutate(|p| -> DispatchResult {
                p.total_stake_amount = p
                    .total_stake_amount
                    .checked_add(amount)
                    .ok_or(ArithmeticError::Overflow)?;
                Ok(())
            })?;

            Self::deposit_event(Event::<T>::Staked(who, amount));
            Ok(().into())
        }

        /// Unstake by exchange derivative for assets, the assets will not be avaliable immediately.
        /// Instead, the request is recorded and pending for the nomination accounts on relaychain
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

            MatchingPool::<T>::try_mutate(|p| -> DispatchResult {
                p.total_unstake_amount = p
                    .total_unstake_amount
                    .checked_add(liquid_amount)
                    .ok_or(ArithmeticError::Overflow)?;
                Ok(())
            })?;

            Self::deposit_event(Event::<T>::Unstaked(who, liquid_amount, asset_amount));
            Ok(().into())
        }

        /// Update default xcm fees
        /// it reflects xcm fees consumed on relaychain
        #[pallet::weight(<T as Config>::WeightInfo::update_xcm_fees())]
        #[transactional]
        pub fn update_xcm_fees(
            origin: OriginFor<T>,
            #[pallet::compact] fees: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            T::UpdateOrigin::ensure_origin(origin)?;

            ensure!(fees > Zero::zero(), Error::<T>::ZeroFees);

            T::XCM::update_xcm_fees(fees);
            Self::deposit_event(Event::<T>::XcmFeesUpdated(fees));
            Ok(().into())
        }

        /// Update insurance pool's reserve_factor
        #[pallet::weight(<T as Config>::WeightInfo::update_reserve_factor())]
        #[transactional]
        pub fn update_reserve_factor(
            origin: OriginFor<T>,
            reserve_factor: Ratio,
        ) -> DispatchResultWithPostInfo {
            T::UpdateOrigin::ensure_origin(origin)?;

            ensure!(
                reserve_factor > Ratio::zero() && reserve_factor < Ratio::one(),
                Error::<T>::InvalidFactor,
            );

            ReserveFactor::<T>::mutate(|v| *v = reserve_factor);
            Self::deposit_event(Event::<T>::ReserveFactorUpdated(reserve_factor));
            Ok(().into())
        }

        /// Update xcm transact's weight configuration
        #[pallet::weight(<T as Config>::WeightInfo::update_xcm_weight())]
        #[transactional]
        pub fn update_xcm_weight(
            origin: OriginFor<T>,
            xcm_weight_misc: XcmWeightMisc<Weight>,
        ) -> DispatchResultWithPostInfo {
            T::UpdateOrigin::ensure_origin(origin)?;
            XcmWeight::<T>::mutate(|v| *v = xcm_weight_misc);
            Self::deposit_event(Event::<T>::XcmWeightUpdated(xcm_weight_misc));
            Ok(().into())
        }

        /// Update staking's market cap
        /// stake will be blocked if passed the cap
        #[pallet::weight(<T as Config>::WeightInfo::update_staking_pool_capacity())]
        #[transactional]
        pub fn update_staking_pool_capacity(
            origin: OriginFor<T>,
            #[pallet::compact] cap: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            T::UpdateOrigin::ensure_origin(origin)?;

            ensure!(cap > Zero::zero(), Error::<T>::ZeroCap);

            StakingPoolCapacity::<T>::mutate(|v| *v = cap);
            Self::deposit_event(Event::<T>::StakingPoolCapacityUpdated(cap));
            Ok(().into())
        }

        /// Payout slashed amount.
        ///
        /// Clear `TotalSlashed`, subtract `InsurancePool`, and bond corresponding amount in relay
        /// chain.
        #[pallet::weight(<T as Config>::WeightInfo::payout_slashed())]
        #[transactional]
        pub fn payout_slashed(
            origin: OriginFor<T>,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            T::RelayOrigin::ensure_origin(origin)?;
            InsurancePool::<T>::try_mutate(|v| -> DispatchResult {
                *v = v.checked_sub(amount).ok_or(ArithmeticError::Underflow)?;
                Ok(())
            })?;
            Self::do_bond_extra(amount)?;
            Self::deposit_event(Event::<T>::SlashPaid(amount));
            Ok(().into())
        }

        /// Do settlement for matching pool.
        ///
        /// The extrinsic does two things:
        /// 1. Update exchange rate
        /// 2. Calculate the imbalance of current matching state and send corresponding operations to
        /// relay-chain.
        #[pallet::weight(<T as Config>::WeightInfo::settlement())]
        #[transactional]
        pub fn settlement(
            origin: OriginFor<T>,
            #[pallet::compact] bonded_amount: BalanceOf<T>,
            #[pallet::compact] unbonding_amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            T::RelayOrigin::ensure_origin(origin)?;

            let bond_extra = !bonded_amount.is_zero();

            // Update exchange rate
            let matching_pool = MatchingPool::<T>::get();
            let old_exchange_rate = Self::exchange_rate();
            let exchange_rate = Rate::checked_from_rational(
                bonded_amount
                    .checked_add(matching_pool.total_stake_amount)
                    .ok_or(ArithmeticError::Overflow)?,
                T::Assets::total_issuance(Self::liquid_currency()?)
                    .checked_add(matching_pool.total_unstake_amount)
                    .ok_or(ArithmeticError::Overflow)?,
            )
            .ok_or(Error::<T>::InvalidExchangeRate)?;
            if exchange_rate > old_exchange_rate {
                ExchangeRate::<T>::put(exchange_rate);
                Self::deposit_event(Event::<T>::ExchangeRateUpdated(exchange_rate));
            }

            let (bond_amount, rebond_amount, unbond_amount) =
                MatchingPool::<T>::take().matching::<Self>(unbonding_amount)?;
            let staking_currency = Self::staking_currency()?;
            let account_id = Self::account_id();

            if !bond_amount.is_zero() {
                T::Assets::burn_from(staking_currency, &account_id, bond_amount)?;

                if !bond_extra {
                    Self::do_bond(bond_amount, RewardDestination::Staked)?;
                } else {
                    Self::do_bond_extra(bond_amount)?;
                }
            }

            if !unbond_amount.is_zero() {
                Self::do_unbond(unbond_amount)?;
            }

            if !rebond_amount.is_zero() {
                Self::do_rebond(rebond_amount)?;
            }

            Self::deposit_event(Event::<T>::Settlement(
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
            #[pallet::compact] value: BalanceOf<T>,
            payee: RewardDestination<T::AccountId>,
        ) -> DispatchResult {
            T::RelayOrigin::ensure_origin(origin)?;
            Self::do_bond(value, payee)?;
            Ok(())
        }

        /// Bond_extra on relaychain via xcm.transact
        #[pallet::weight(<T as Config>::WeightInfo::bond_extra())]
        #[transactional]
        pub fn bond_extra(
            origin: OriginFor<T>,
            #[pallet::compact] value: BalanceOf<T>,
        ) -> DispatchResult {
            T::RelayOrigin::ensure_origin(origin)?;
            Self::do_bond_extra(value)?;
            Ok(())
        }

        /// Unbond on relaychain via xcm.transact
        #[pallet::weight(<T as Config>::WeightInfo::unbond())]
        #[transactional]
        pub fn unbond(
            origin: OriginFor<T>,
            #[pallet::compact] value: BalanceOf<T>,
        ) -> DispatchResult {
            T::RelayOrigin::ensure_origin(origin)?;
            Self::do_unbond(value)?;
            Ok(())
        }

        /// Rebond on relaychain via xcm.transact
        #[pallet::weight(<T as Config>::WeightInfo::rebond())]
        #[transactional]
        pub fn rebond(
            origin: OriginFor<T>,
            #[pallet::compact] value: BalanceOf<T>,
        ) -> DispatchResult {
            T::RelayOrigin::ensure_origin(origin)?;
            Self::do_rebond(value)?;
            Ok(())
        }

        /// Withdraw unbonded on relaychain via xcm.transact
        #[pallet::weight(<T as Config>::WeightInfo::withdraw_unbonded())]
        #[transactional]
        pub fn withdraw_unbonded(
            origin: OriginFor<T>,
            num_slashing_spans: u32,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResult {
            T::RelayOrigin::ensure_origin(origin)?;
            T::XCM::do_withdraw_unbonded(
                num_slashing_spans,
                amount,
                T::AccountIdToMultiLocation::convert(Self::para_account_id()),
                Self::para_account_id(),
                Self::staking_currency()?,
                T::DerivativeIndex::get(),
            )?;
            Self::deposit_event(Event::<T>::WithdrawingUnbonded(num_slashing_spans));
            Ok(())
        }

        /// Nominate on relaychain via xcm.transact
        #[pallet::weight(<T as Config>::WeightInfo::nominate())]
        #[transactional]
        pub fn nominate(origin: OriginFor<T>, targets: Vec<T::AccountId>) -> DispatchResult {
            T::RelayOrigin::ensure_origin(origin)?;
            T::XCM::do_nominate(
                targets.clone(),
                T::AccountIdToMultiLocation::convert(Self::para_account_id()),
                Self::staking_currency()?,
                T::DerivativeIndex::get(),
            )?;
            Self::deposit_event(Event::<T>::Nominating(targets));
            Ok(())
        }

        /// Anyone can transfer asset to the insurance pool
        #[pallet::weight(<T as Config>::WeightInfo::add_insurances())]
        #[transactional]
        pub fn add_insurances(
            origin: OriginFor<T>,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            T::Assets::transfer(
                Self::staking_currency()?,
                &who,
                &Self::account_id(),
                amount,
                false,
            )?;
            InsurancePool::<T>::try_mutate(|b| -> DispatchResult {
                *b = b.checked_add(amount).ok_or(ArithmeticError::Overflow)?;
                Ok(())
            })?;
            Self::deposit_event(Event::<T>::InsurancesAdded(who, amount));
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Staking pool account
        pub fn account_id() -> T::AccountId {
            T::PalletId::get().into_account()
        }

        /// Parachain sovereign account
        pub fn para_account_id() -> T::AccountId {
            T::SelfParaId::get().into_account()
        }

        /// Get staking currency or return back an error
        pub fn staking_currency() -> Result<AssetIdOf<T>, DispatchError> {
            Self::get_staking_currency()
                .ok_or(Error::<T>::StakingCurrencyNotReady)
                .map_err(Into::into)
        }

        /// Get liquid currency or return back an error
        pub fn liquid_currency() -> Result<AssetIdOf<T>, DispatchError> {
            Self::get_liquid_currency()
                .ok_or(Error::<T>::LiquidCurrencyNotReady)
                .map_err(Into::into)
        }

        /// Derivative parachain account
        pub fn derivative_para_account_id() -> T::AccountId {
            let para_account = Self::para_account_id();
            let derivative_index = T::DerivativeIndex::get();
            pallet_utility::Pallet::<T>::derivative_account_id(para_account, derivative_index)
        }

        #[require_transactional]
        fn do_bond(value: BalanceOf<T>, payee: RewardDestination<T::AccountId>) -> DispatchResult {
            T::XCM::do_bond(
                value,
                payee.clone(),
                Self::derivative_para_account_id(),
                T::AccountIdToMultiLocation::convert(Self::para_account_id()),
                Self::staking_currency()?,
                T::DerivativeIndex::get(),
            )?;
            Self::deposit_event(Event::<T>::Bonding(
                Self::derivative_para_account_id(),
                value,
                payee,
            ));
            Ok(())
        }

        #[require_transactional]
        fn do_bond_extra(value: BalanceOf<T>) -> DispatchResult {
            T::XCM::do_bond_extra(
                value,
                Self::derivative_para_account_id(),
                T::AccountIdToMultiLocation::convert(Self::para_account_id()),
                Self::staking_currency()?,
                T::DerivativeIndex::get(),
            )?;
            Self::deposit_event(Event::<T>::BondingExtra(value));
            Ok(())
        }

        #[require_transactional]
        fn do_unbond(value: BalanceOf<T>) -> DispatchResult {
            T::XCM::do_unbond(
                value,
                T::AccountIdToMultiLocation::convert(Self::para_account_id()),
                Self::staking_currency()?,
                T::DerivativeIndex::get(),
            )?;
            Self::deposit_event(Event::<T>::Unbonding(value));
            Ok(())
        }

        #[require_transactional]
        fn do_rebond(value: BalanceOf<T>) -> DispatchResult {
            T::XCM::do_rebond(
                value,
                T::AccountIdToMultiLocation::convert(Self::para_account_id()),
                Self::staking_currency()?,
                T::DerivativeIndex::get(),
            )?;
            Self::deposit_event(Event::<T>::Rebonding(value));
            Ok(())
        }

        #[inline]
        fn push_unstake_task(who: &T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
            UnstakeQueue::<T>::try_mutate(|q| -> DispatchResult {
                q.try_push((who.clone(), amount))
                    .map_err(|_| Error::<T>::ExceededUnstakeQueueCapacity)?;
                Ok(())
            })
        }

        #[inline]
        fn pop_unstake_task() {
            UnstakeQueue::<T>::mutate(|v| v.remove(0));
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
        let asset_id = T::StakingCurrency::get();
        // if !<T::Assets as InspectMetadata<AccountIdOf<T>>>::decimals(&asset_id).is_zero() {
        Some(asset_id)
        // } else {
        //     None
        // }
    }

    fn get_liquid_currency() -> Option<AssetIdOf<T>> {
        let asset_id = T::LiquidCurrency::get();
        // if !<T::Assets as InspectMetadata<AccountIdOf<T>>>::decimals(&asset_id).is_zero() {
        Some(asset_id)
        // } else {
        //     None
        // }
    }
}
