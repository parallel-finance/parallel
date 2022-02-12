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

use frame_support::traits::{fungibles::InspectMetadata, Get};
use pallet_xcm::ensure_response;
use primitives::{ExchangeRateProvider, LiquidStakingCurrenciesProvider, Rate};
use sp_runtime::traits::Zero;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        dispatch::{DispatchResult, DispatchResultWithPostInfo},
        ensure,
        pallet_prelude::*,
        require_transactional,
        storage::with_transaction,
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
        traits::{AccountIdConversion, BlockNumberProvider, CheckedAdd},
        ArithmeticError, FixedPointNumber, TransactionOutcome,
    };
    use sp_std::{boxed::Box, result::Result, vec::Vec};

    use primitives::{ump::*, Balance, CurrencyId, ParaId, Rate, Ratio};

    use super::{types::*, weights::WeightInfo, *};
    use pallet_xcm_helper::XcmHelper;
    use xcm::latest::prelude::*;

    pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
    pub type AssetIdOf<T> =
        <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;
    pub type BalanceOf<T> =
        <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_utility::Config + pallet_xcm::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type Origin: IsType<<Self as frame_system::Config>::Origin>
            + Into<Result<pallet_xcm::Origin, <Self as Config>::Origin>>;

        type Call: IsType<<Self as pallet_xcm::Config>::Call> + From<Call<Self>>;

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

        /// Xcm fees
        #[pallet::constant]
        type XcmFees: Get<BalanceOf<Self>>;

        /// Staking currency
        #[pallet::constant]
        type StakingCurrency: Get<AssetIdOf<Self>>;

        /// Liquid currency
        #[pallet::constant]
        type LiquidCurrency: Get<AssetIdOf<Self>>;

        /// Unstake queue's capacity
        #[pallet::constant]
        type UnstakeQueueCap: Get<u32>;

        /// Minimum stake amount
        #[pallet::constant]
        type MinStake: Get<BalanceOf<Self>>;

        /// Minimum unstake amount
        #[pallet::constant]
        type MinUnstake: Get<BalanceOf<Self>>;

        /// Weight information
        type WeightInfo: WeightInfo;

        /// Number of blocknumbers that staked funds must remain bonded for.
        /// BondingDuration * SessionsPerEra * EpochDuration / MILLISECS_PER_BLOCK
        #[pallet::constant]
        type BondingDuration: Get<BlockNumberFor<Self>>;

        /// The relay's BlockNumber provider
        type RelayChainBlockNumberProvider: BlockNumberProvider<BlockNumber = BlockNumberFor<Self>>;

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
        /// Liquid currency's market cap was updated
        MarketCapUpdated(BalanceOf<T>),
        /// InsurancePool's reserve_factor was updated
        ReserveFactorUpdated(Ratio),
        /// Slash was paid by insurance pool
        SlashPaid(BalanceOf<T>),
        /// Exchange rate was updated
        ExchangeRateUpdated(Rate),
        /// Notification received
        /// [multi_location, query_id, res]
        NotificationReceived(Box<MultiLocation>, QueryId, Option<(u32, XcmError)>),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Exchange rate is invalid.
        InvalidExchangeRate,
        /// The stake was below the minimum, `MinStake`.
        StakeTooSmall,
        /// The unstake was below the minimum, `MinUnstake`.
        UnstakeTooSmall,
        /// Invalid liquid currency
        InvalidLiquidCurrency,
        /// Invalid staking currency
        InvalidStakingCurrency,
        /// Exceeded unstake queue's capacity
        UnstakeQueueCapExceeded,
        /// Exceeded liquid currency's market cap
        CapExceeded,
        /// Invalid market cap
        InvalidCap,
        /// The factor should be bigger than 0% and smaller than 100%
        InvalidFactor,
    }

    /// The exchange rate between relaychain native asset and the voucher.
    #[pallet::storage]
    #[pallet::getter(fn exchange_rate)]
    pub type ExchangeRate<T: Config> = StorageValue<_, Rate, ValueQuery>;

    /// Fraction of reward currently set aside for reserves.
    #[pallet::storage]
    #[pallet::getter(fn reserve_factor)]
    pub type ReserveFactor<T: Config> = StorageValue<_, Ratio, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn total_reserves)]
    pub type TotalReserves<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

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
        BoundedVec<(T::AccountId, BalanceOf<T>, BlockNumberFor<T>), T::UnstakeQueueCap>,
        ValueQuery,
    >;

    /// Liquid currency's market cap
    #[pallet::storage]
    #[pallet::getter(fn market_cap)]
    pub type MarketCap<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn xcm_request)]
    pub type XcmRequests<T> = StorageMap<_, Blake2_128Concat, QueryId, XcmRequest<T>, OptionQuery>;

    #[derive(Default)]
    #[pallet::genesis_config]
    pub struct GenesisConfig {
        pub exchange_rate: Rate,
        pub reserve_factor: Ratio,
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
        fn on_idle(_n: BlockNumberFor<T>, remaining_weight: Weight) -> Weight {
            let base_weight = <T as Config>::WeightInfo::on_idle();
            if remaining_weight < base_weight {
                return remaining_weight;
            }
            with_transaction(|| match Self::do_pop_front() {
                Ok(_) => TransactionOutcome::Commit(remaining_weight - base_weight),
                Err(_) => TransactionOutcome::Rollback(0),
            })
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

            ensure!(amount >= T::MinStake::get(), Error::<T>::StakeTooSmall);

            let reserves = Self::reserve_factor().mul_floor(amount);

            let xcm_fees = T::XcmFees::get();
            let amount = amount
                .checked_sub(xcm_fees)
                .ok_or(ArithmeticError::Underflow)?;
            T::Assets::transfer(
                Self::staking_currency()?,
                &who,
                &Self::account_id(),
                amount,
                false,
            )?;
            T::XCM::add_xcm_fees(Self::staking_currency()?, &who, xcm_fees)?;

            let amount = amount
                .checked_sub(reserves)
                .ok_or(ArithmeticError::Underflow)?;
            let liquid_amount = Self::exchange_rate()
                .reciprocal()
                .and_then(|r| r.checked_mul_int(amount))
                .ok_or(Error::<T>::InvalidExchangeRate)?;
            let liquid_currency = Self::liquid_currency()?;
            ensure!(
                T::Assets::total_issuance(liquid_currency)
                    .checked_add(liquid_amount)
                    .ok_or(ArithmeticError::Overflow)?
                    <= Self::market_cap(),
                Error::<T>::CapExceeded
            );
            T::Assets::mint_into(liquid_currency, &who, liquid_amount)?;

            MatchingPool::<T>::try_mutate(|p| -> DispatchResult {
                p.total_stake_amount = p
                    .total_stake_amount
                    .checked_add(amount)
                    .ok_or(ArithmeticError::Overflow)?;
                Ok(())
            })?;
            TotalReserves::<T>::try_mutate(|b| -> DispatchResult {
                *b = b.checked_add(reserves).ok_or(ArithmeticError::Overflow)?;
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
                liquid_amount >= T::MinUnstake::get(),
                Error::<T>::UnstakeTooSmall
            );

            let exchange_rate = ExchangeRate::<T>::get();
            let asset_amount = exchange_rate
                .checked_mul_int(liquid_amount)
                .ok_or(Error::<T>::InvalidExchangeRate)?;

            let target_blocknumber = T::RelayChainBlockNumberProvider::current_block_number()
                .checked_add(&T::BondingDuration::get())
                .ok_or(ArithmeticError::Overflow)?;
            Self::do_push_back(&who, asset_amount, target_blocknumber)?;

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

        /// Update liquid currency's market cap
        /// stake will be blocked if passed liquid currency's market cap
        #[pallet::weight(<T as Config>::WeightInfo::update_market_cap())]
        #[transactional]
        pub fn update_market_cap(
            origin: OriginFor<T>,
            #[pallet::compact] cap: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            T::UpdateOrigin::ensure_origin(origin)?;

            ensure!(!cap.is_zero(), Error::<T>::InvalidCap);

            MarketCap::<T>::mutate(|v| *v = cap);
            Self::deposit_event(Event::<T>::MarketCapUpdated(cap));
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
            #[pallet::compact] bonding_amount: BalanceOf<T>,
            #[pallet::compact] unbonding_amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            T::RelayOrigin::ensure_origin(origin)?;

            let old_matching_pool = Self::matching_pool();
            let old_exchange_rate = Self::exchange_rate();
            let (bond_amount, rebond_amount, unbond_amount) =
                MatchingPool::<T>::try_mutate(|b| b.matching::<Self>(unbonding_amount))?;

            if bonding_amount.is_zero() && unbonding_amount.is_zero() {
                Self::do_bond(bond_amount, RewardDestination::Staked)?;
            } else {
                Self::do_bond_extra(bond_amount)?;
            }

            Self::do_unbond(unbond_amount)?;
            Self::do_rebond(rebond_amount)?;

            match Rate::checked_from_rational(
                bonding_amount
                    .checked_add(old_matching_pool.total_stake_amount)
                    .ok_or(ArithmeticError::Overflow)?,
                T::Assets::total_issuance(Self::liquid_currency()?)
                    .checked_add(old_matching_pool.total_unstake_amount)
                    .ok_or(ArithmeticError::Overflow)?,
            ) {
                Some(exchange_rate) if exchange_rate != old_exchange_rate => {
                    ExchangeRate::<T>::put(exchange_rate);
                    Self::deposit_event(Event::<T>::ExchangeRateUpdated(exchange_rate));
                }
                _ => {}
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
            #[pallet::compact] amount: BalanceOf<T>,
            payee: RewardDestination<T::AccountId>,
        ) -> DispatchResult {
            T::RelayOrigin::ensure_origin(origin)?;
            Self::do_bond(amount, payee)?;
            Ok(())
        }

        /// Bond_extra on relaychain via xcm.transact
        #[pallet::weight(<T as Config>::WeightInfo::bond_extra())]
        #[transactional]
        pub fn bond_extra(
            origin: OriginFor<T>,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResult {
            T::RelayOrigin::ensure_origin(origin)?;
            Self::do_bond_extra(amount)?;
            Ok(())
        }

        /// Unbond on relaychain via xcm.transact
        #[pallet::weight(<T as Config>::WeightInfo::unbond())]
        #[transactional]
        pub fn unbond(
            origin: OriginFor<T>,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResult {
            T::RelayOrigin::ensure_origin(origin)?;
            Self::do_unbond(amount)?;
            Ok(())
        }

        /// Rebond on relaychain via xcm.transact
        #[pallet::weight(<T as Config>::WeightInfo::rebond())]
        #[transactional]
        pub fn rebond(
            origin: OriginFor<T>,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResult {
            T::RelayOrigin::ensure_origin(origin)?;
            Self::do_rebond(amount)?;
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
            let query_id = T::XCM::do_withdraw_unbonded(
                num_slashing_spans,
                Self::para_account_id(),
                Self::staking_currency()?,
                T::DerivativeIndex::get(),
                Self::notify_placeholder(),
            )?;
            XcmRequests::<T>::insert(
                query_id,
                XcmRequest::WithdrawUnbonded {
                    num_slashing_spans,
                    amount,
                },
            );
            Self::deposit_event(Event::<T>::WithdrawingUnbonded(num_slashing_spans));
            Ok(())
        }

        /// Nominate on relaychain via xcm.transact
        #[pallet::weight(<T as Config>::WeightInfo::nominate())]
        #[transactional]
        pub fn nominate(origin: OriginFor<T>, targets: Vec<T::AccountId>) -> DispatchResult {
            T::RelayOrigin::ensure_origin(origin)?;
            let query_id = T::XCM::do_nominate(
                targets.clone(),
                Self::staking_currency()?,
                T::DerivativeIndex::get(),
                Self::notify_placeholder(),
            )?;

            XcmRequests::<T>::insert(
                query_id,
                XcmRequest::Nominate {
                    targets: targets.clone(),
                },
            );
            Self::deposit_event(Event::<T>::Nominating(targets));
            Ok(())
        }

        #[pallet::weight(<T as Config>::WeightInfo::notification_received())]
        #[transactional]
        pub fn notification_received(
            origin: OriginFor<T>,
            query_id: QueryId,
            response: Response,
        ) -> DispatchResultWithPostInfo {
            let responder = ensure_response(<T as Config>::Origin::from(origin))?;
            if let Response::ExecutionResult(res) = response {
                if let Some(request) = Self::xcm_request(&query_id) {
                    Self::do_notification_received(query_id, request, res)?;
                }

                Self::deposit_event(Event::<T>::NotificationReceived(
                    Box::new(responder),
                    query_id,
                    res,
                ));
            }
            Ok(().into())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Staking pool account
        pub fn account_id() -> T::AccountId {
            T::PalletId::get().into_account()
        }

        /// Parachain's sovereign account
        pub fn para_account_id() -> T::AccountId {
            T::SelfParaId::get().into_account()
        }

        /// Get staking currency or return back an error
        pub fn staking_currency() -> Result<AssetIdOf<T>, DispatchError> {
            Self::get_staking_currency()
                .ok_or(Error::<T>::InvalidStakingCurrency)
                .map_err(Into::into)
        }

        /// Get liquid currency or return back an error
        pub fn liquid_currency() -> Result<AssetIdOf<T>, DispatchError> {
            Self::get_liquid_currency()
                .ok_or(Error::<T>::InvalidLiquidCurrency)
                .map_err(Into::into)
        }

        /// Derivative parachain account
        pub fn derivative_para_account_id() -> T::AccountId {
            let para_account = Self::para_account_id();
            let derivative_index = T::DerivativeIndex::get();
            pallet_utility::Pallet::<T>::derivative_account_id(para_account, derivative_index)
        }

        #[require_transactional]
        fn do_bond(amount: BalanceOf<T>, payee: RewardDestination<T::AccountId>) -> DispatchResult {
            if amount.is_zero() {
                return Ok(());
            }

            let staking_currency = Self::staking_currency()?;
            let derivative_account_id = Self::derivative_para_account_id();
            let query_id = T::XCM::do_bond(
                amount,
                payee.clone(),
                derivative_account_id.clone(),
                staking_currency,
                T::DerivativeIndex::get(),
                Self::notify_placeholder(),
            )?;

            XcmRequests::<T>::insert(query_id, XcmRequest::Bond { amount });

            Self::deposit_event(Event::<T>::Bonding(derivative_account_id, amount, payee));

            Ok(())
        }

        #[require_transactional]
        fn do_bond_extra(amount: BalanceOf<T>) -> DispatchResult {
            if amount.is_zero() {
                return Ok(());
            }

            let staking_currency = Self::staking_currency()?;
            let query_id = T::XCM::do_bond_extra(
                amount,
                Self::derivative_para_account_id(),
                staking_currency,
                T::DerivativeIndex::get(),
                Self::notify_placeholder(),
            )?;

            XcmRequests::<T>::insert(query_id, XcmRequest::BondExtra { amount });
            Self::deposit_event(Event::<T>::BondingExtra(amount));

            Ok(())
        }

        #[require_transactional]
        fn do_unbond(amount: BalanceOf<T>) -> DispatchResult {
            if amount.is_zero() {
                return Ok(());
            }

            let query_id = T::XCM::do_unbond(
                amount,
                Self::staking_currency()?,
                T::DerivativeIndex::get(),
                Self::notify_placeholder(),
            )?;

            let liquid_amount = Self::exchange_rate()
                .reciprocal()
                .and_then(|r| r.checked_mul_int(amount))
                .ok_or(Error::<T>::InvalidExchangeRate)?;

            XcmRequests::<T>::insert(query_id, XcmRequest::Unbond { liquid_amount });
            Self::deposit_event(Event::<T>::Unbonding(amount));

            Ok(())
        }

        #[require_transactional]
        fn do_rebond(amount: BalanceOf<T>) -> DispatchResult {
            if amount.is_zero() {
                return Ok(());
            }

            let query_id = T::XCM::do_rebond(
                amount,
                Self::staking_currency()?,
                T::DerivativeIndex::get(),
                Self::notify_placeholder(),
            )?;
            XcmRequests::<T>::insert(query_id, XcmRequest::Rebond { amount });
            Self::deposit_event(Event::<T>::Rebonding(amount));

            Ok(())
        }

        #[require_transactional]
        fn do_pop_front() -> Result<(), DispatchError> {
            let unstake_queue = Self::unstake_queue();
            if unstake_queue.is_empty() {
                return Ok(());
            }

            let (who, amount, target_blocknumber) = &unstake_queue[0];
            if T::RelayChainBlockNumberProvider::current_block_number() < *target_blocknumber {
                return Ok(());
            }

            let account_id = Self::account_id();
            let staking_currency = Self::staking_currency()?;
            let free_balance = T::Assets::reducible_balance(staking_currency, &account_id, false)
                .saturating_sub(Self::total_reserves())
                .saturating_sub(Self::matching_pool().total_stake_amount);

            if free_balance >= *amount {
                T::Assets::transfer(staking_currency, &account_id, who, *amount, false)?;
                UnstakeQueue::<T>::mutate(|v| v.remove(0));
            }

            Ok(())
        }

        #[require_transactional]
        fn do_push_back(
            who: &T::AccountId,
            amount: BalanceOf<T>,
            target_blocknumber: BlockNumberFor<T>,
        ) -> DispatchResult {
            UnstakeQueue::<T>::try_mutate(|q| -> DispatchResult {
                q.try_push((who.clone(), amount, target_blocknumber))
                    .map_err(|_| Error::<T>::UnstakeQueueCapExceeded)?;
                Ok(())
            })
        }

        #[require_transactional]
        fn do_notification_received(
            query_id: QueryId,
            request: XcmRequest<T>,
            res: Option<(u32, XcmError)>,
        ) -> DispatchResult {
            let executed = res.is_none();

            match request {
                XcmRequest::Bond { amount, .. } | XcmRequest::BondExtra { amount } if executed => {
                    MatchingPool::<T>::try_mutate(|p| -> DispatchResult {
                        p.total_stake_amount = p
                            .total_stake_amount
                            .checked_sub(amount)
                            .ok_or(ArithmeticError::Underflow)?;
                        Ok(())
                    })?;
                    T::Assets::burn_from(Self::staking_currency()?, &Self::account_id(), amount)?;
                }
                XcmRequest::Unbond { liquid_amount } if executed => {
                    MatchingPool::<T>::try_mutate(|p| -> DispatchResult {
                        p.total_unstake_amount = p
                            .total_unstake_amount
                            .checked_sub(liquid_amount)
                            .ok_or(ArithmeticError::Underflow)?;
                        Ok(())
                    })?;
                }
                XcmRequest::Rebond { amount } if executed => {
                    MatchingPool::<T>::try_mutate(|p| -> DispatchResult {
                        p.total_stake_amount = p
                            .total_stake_amount
                            .checked_sub(amount)
                            .ok_or(ArithmeticError::Underflow)?;
                        Ok(())
                    })?;
                }
                XcmRequest::WithdrawUnbonded {
                    num_slashing_spans: _,
                    amount,
                } if executed => {
                    T::Assets::mint_into(Self::staking_currency()?, &Self::account_id(), amount)?;
                }
                _ => {}
            }

            if executed {
                XcmRequests::<T>::remove(&query_id);
            }
            Ok(())
        }

        fn notify_placeholder() -> <T as Config>::Call {
            <T as Config>::Call::from(Call::<T>::notification_received {
                query_id: Default::default(),
                response: Default::default(),
            })
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
        if !<T::Assets as InspectMetadata<AccountIdOf<T>>>::decimals(&asset_id).is_zero() {
            Some(asset_id)
        } else {
            None
        }
    }

    fn get_liquid_currency() -> Option<AssetIdOf<T>> {
        let asset_id = T::LiquidCurrency::get();
        if !<T::Assets as InspectMetadata<AccountIdOf<T>>>::decimals(&asset_id).is_zero() {
            Some(asset_id)
        } else {
            None
        }
    }
}
