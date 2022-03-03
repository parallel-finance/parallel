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

use frame_support::traits::{fungibles::InspectMetadata, tokens::Balance as BalanceT, Get};
use pallet_xcm::ensure_response;
use primitives::{
    ExchangeRateProvider, LiquidStakingConvert, LiquidStakingCurrenciesProvider, Rate,
};
use sp_runtime::{
    traits::{Saturating, Zero},
    FixedPointNumber, FixedPointOperand,
};

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        dispatch::{DispatchResult, DispatchResultWithPostInfo},
        ensure,
        error::BadOrigin,
        log,
        pallet_prelude::*,
        require_transactional,
        traits::{
            fungibles::{Inspect, InspectMetadata, Mutate, Transfer},
            IsType, SortedMembers,
        },
        transactional, PalletId,
    };
    use frame_system::{
        ensure_signed,
        pallet_prelude::{BlockNumberFor, OriginFor},
    };
    use sp_runtime::{
        traits::{AccountIdConversion, BlockNumberProvider, StaticLookup},
        ArithmeticError, FixedPointNumber,
    };
    use sp_std::{boxed::Box, result::Result, vec::Vec};

    use primitives::{
        ump::*, ArithmeticKind, Balance, CurrencyId, LiquidStakingConvert, ParaId, Rate, Ratio,
    };

    use super::{types::*, weights::WeightInfo, *};
    use pallet_xcm_helper::XcmHelper;
    use xcm::latest::prelude::*;

    pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
    pub type AssetIdOf<T> =
        <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;
    pub type BalanceOf<T> =
        <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance;
    pub type UnbondIndex = u32;

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

        /// Approved accouts which can call `withdraw_unbonded` and `settlement`
        type Members: SortedMembers<Self::AccountId>;

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

        /// Minimum stake amount
        #[pallet::constant]
        type MinStake: Get<BalanceOf<Self>>;

        /// Minimum unstake amount
        #[pallet::constant]
        type MinUnstake: Get<BalanceOf<Self>>;

        /// Weight information
        type WeightInfo: WeightInfo;

        /// Number of unbond indexes for unlocking.
        #[pallet::constant]
        type BondingDuration: Get<UnbondIndex>;

        /// Number of blocknumbers that each period contains.
        /// SessionsPerEra * EpochDuration / MILLISECS_PER_BLOCK
        #[pallet::constant]
        type EraLength: Get<BlockNumberFor<Self>>;

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
        /// Exchange rate was updated
        ExchangeRateUpdated(Rate),
        /// Notification received
        /// [multi_location, query_id, res]
        NotificationReceived(Box<MultiLocation>, QueryId, Option<(u32, XcmError)>),
        /// Claim user's unbonded staking assets
        /// [unbond_index, account_id, amount]
        ClaimedFor(UnbondIndex, T::AccountId, BalanceOf<T>),
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
        /// Exceeded liquid currency's market cap
        CapExceeded,
        /// Invalid market cap
        InvalidCap,
        /// The factor should be bigger than 0% and smaller than 100%
        InvalidFactor,
        /// Settlement locked
        SettlementLocked,
        /// Nothing to claim yet
        NothingToClaim,
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

    /// Liquid currency's market cap
    #[pallet::storage]
    #[pallet::getter(fn market_cap)]
    pub type MarketCap<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    /// Flying & failed xcm requests
    #[pallet::storage]
    #[pallet::getter(fn xcm_request)]
    pub type XcmRequests<T> = StorageMap<_, Blake2_128Concat, QueryId, XcmRequest<T>, OptionQuery>;

    /// Current unbond index
    /// Users can come to claim their unbonded staking assets back once this value arrived
    /// at certain height decided by `BondingDuration` and `EraLength`
    #[pallet::storage]
    #[pallet::getter(fn current_unbond_index)]
    pub type CurrentUnbondIndex<T: Config> = StorageValue<_, UnbondIndex, ValueQuery>;

    /// Last settlement time
    /// Settlement must be executed once and only once in every relaychain era
    #[pallet::storage]
    #[pallet::getter(fn last_settlement_time)]
    pub type LastSettlementTime<T: Config> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

    /// Pending unstake requests
    #[pallet::storage]
    #[pallet::getter(fn pending_unstake)]
    pub type PendingUnstake<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        UnbondIndex,
        Blake2_128Concat,
        T::AccountId,
        BalanceOf<T>,
        ValueQuery,
    >;

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
            let liquid_amount =
                Self::staking_to_liquid(amount).ok_or(Error::<T>::InvalidExchangeRate)?;
            let liquid_currency = Self::liquid_currency()?;
            Self::ensure_market_cap(liquid_currency, liquid_amount)?;

            T::Assets::mint_into(liquid_currency, &who, liquid_amount)?;

            log::trace!(
                target: "liquidStaking::stake",
                "stake_amount: {:?}, liquid_amount: {:?}, reserved: {:?}",
                &amount,
                &liquid_amount,
                &reserves
            );

            MatchingPool::<T>::try_mutate(|p| -> DispatchResult {
                p.update_total_stake_amount(amount, ArithmeticKind::Addition)
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

            let amount =
                Self::liquid_to_staking(liquid_amount).ok_or(Error::<T>::InvalidExchangeRate)?;

            PendingUnstake::<T>::try_mutate(
                Self::current_unbond_index(),
                &who,
                |unstake_amount| -> DispatchResult {
                    *unstake_amount = unstake_amount
                        .checked_add(amount)
                        .ok_or(ArithmeticError::Overflow)?;
                    Ok(())
                },
            )?;

            T::Assets::burn_from(Self::liquid_currency()?, &who, liquid_amount)?;

            log::trace!(
                target: "liquidStaking::unstake",
                "unstake_amount: {:?}, liquid_amount: {:?}",
                &amount,
                &liquid_amount,
            );

            MatchingPool::<T>::try_mutate(|p| -> DispatchResult {
                p.update_total_unstake_amount(amount, ArithmeticKind::Addition)
            })?;

            Self::deposit_event(Event::<T>::Unstaked(who, liquid_amount, amount));
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

            log::trace!(
                target: "liquidStaking::update_reserve_factor",
                 "reserve_factor: {:?}",
                &reserve_factor,
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

            log::trace!(
                target: "liquidStaking::update_market_cap",
                "cap: {:?}",
                &cap,
            );
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
            Self::ensure_origin(origin)?;

            let relaychain_blocknumber = T::RelayChainBlockNumberProvider::current_block_number();
            ensure!(
                relaychain_blocknumber.saturating_sub(Self::last_settlement_time())
                    >= T::EraLength::get(),
                Error::<T>::SettlementLocked
            );

            let (bond_amount, rebond_amount, unbond_amount) =
                Self::matching_pool().matching(unbonding_amount)?;

            if bonding_amount.is_zero() && unbonding_amount.is_zero() {
                Self::do_bond(bond_amount, RewardDestination::Staked)?;
            } else {
                Self::do_bond_extra(bond_amount)?;
            }

            Self::do_unbond(unbond_amount)?;
            Self::do_rebond(rebond_amount)?;

            Self::do_update_exchange_rate(bonding_amount)?;

            log::trace!(
                target: "liquidStaking::settlement",
                "bonding_amount: {:?}, unbonding_amount: {:?}, bond_amount: {:?}, rebond_amount: {:?}, unbond_amount: {:?}",
                &bonding_amount,
                &unbonding_amount,
                &bond_amount,
                &rebond_amount,
                &unbond_amount,
            );

            CurrentUnbondIndex::<T>::mutate(|v| *v += 1);
            LastSettlementTime::<T>::put(relaychain_blocknumber);

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
            Self::ensure_origin(origin)?;

            let query_id = T::XCM::do_withdraw_unbonded(
                num_slashing_spans,
                Self::para_account_id(),
                Self::staking_currency()?,
                T::DerivativeIndex::get(),
                Self::notify_placeholder(),
            )?;

            log::trace!(
                target: "liquidStaking::withdraw_unbonded",
                "num_slashing_spans: {:?}",
                &num_slashing_spans,
            );

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
            Self::ensure_origin(origin)?;

            let query_id = T::XCM::do_nominate(
                targets.clone(),
                Self::staking_currency()?,
                T::DerivativeIndex::get(),
                Self::notify_placeholder(),
            )?;

            log::trace!(
                target: "liquidStaking::nominate",
                "targets: {:?}",
                &targets,
            );

            XcmRequests::<T>::insert(
                query_id,
                XcmRequest::Nominate {
                    targets: targets.clone(),
                },
            );
            Self::deposit_event(Event::<T>::Nominating(targets));
            Ok(())
        }

        /// Internal call which is expected to be triggered only by xcm instruction
        #[pallet::weight(<T as Config>::WeightInfo::notification_received())]
        #[transactional]
        pub fn notification_received(
            origin: OriginFor<T>,
            query_id: QueryId,
            response: Response,
        ) -> DispatchResultWithPostInfo {
            let responder = ensure_response(<T as Config>::Origin::from(origin))?;
            log::trace!(
                target: "liquidStaking::notification_received",
                "query_id: {:?}, response: {:?}",
                &query_id,
                &response
            );
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

        /// Claim assets back when unbond_index arrived at certain height
        #[pallet::weight(<T as Config>::WeightInfo::claim_for())]
        #[transactional]
        pub fn claim_for(
            origin: OriginFor<T>,
            unbond_index: UnbondIndex,
            dest: <T::Lookup as StaticLookup>::Source,
        ) -> DispatchResultWithPostInfo {
            Self::ensure_origin(origin)?;
            let who = T::Lookup::lookup(dest)?;

            ensure!(
                Self::current_unbond_index().saturating_sub(unbond_index)
                    >= T::BondingDuration::get(),
                Error::<T>::NothingToClaim
            );

            PendingUnstake::<T>::try_mutate_exists(unbond_index, &who, |d| -> DispatchResult {
                let amount = d.take().unwrap_or_default();
                if amount.is_zero() {
                    return Err(Error::<T>::NothingToClaim.into());
                }
                T::Assets::transfer(
                    Self::staking_currency()?,
                    &Self::account_id(),
                    &who,
                    amount,
                    false,
                )?;

                log::trace!(
                    target: "liquidStaking::claim_for",
                    "unbond_index: {:?}, beneficiary: {:?}, amount: {:?}",
                    &unbond_index,
                    &who,
                    amount
                );

                Self::deposit_event(Event::<T>::ClaimedFor(unbond_index, who.clone(), amount));
                Ok(())
            })?;
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

            log::trace!(
                target: "liquidStaking::bond",
                "amount: {:?}",
                &amount,
            );

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

            log::trace!(
                target: "liquidStaking::bond_extra",
                "amount: {:?}",
                &amount,
            );

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

            log::trace!(
                target: "liquidStaking::unbond",
                "amount: {:?}",
                &amount,
            );

            let query_id = T::XCM::do_unbond(
                amount,
                Self::staking_currency()?,
                T::DerivativeIndex::get(),
                Self::notify_placeholder(),
            )?;

            XcmRequests::<T>::insert(query_id, XcmRequest::Unbond { amount });

            Self::deposit_event(Event::<T>::Unbonding(amount));

            Ok(())
        }

        #[require_transactional]
        fn do_rebond(amount: BalanceOf<T>) -> DispatchResult {
            if amount.is_zero() {
                return Ok(());
            }

            log::trace!(
                target: "liquidStaking::rebond",
                "amount: {:?}",
                &amount,
            );

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
        fn do_notification_received(
            query_id: QueryId,
            request: XcmRequest<T>,
            res: Option<(u32, XcmError)>,
        ) -> DispatchResult {
            let executed = res.is_none();
            use ArithmeticKind::*;
            use XcmRequest::*;

            match request {
                Bond { amount, .. } | BondExtra { amount } if executed => {
                    MatchingPool::<T>::try_mutate(|p| -> DispatchResult {
                        p.update_total_stake_amount(amount, Subtraction)
                    })?;
                    T::Assets::burn_from(Self::staking_currency()?, &Self::account_id(), amount)?;
                }
                Unbond { amount } if executed => {
                    MatchingPool::<T>::try_mutate(|p| -> DispatchResult {
                        p.update_total_unstake_amount(amount, Subtraction)
                    })?;
                }
                Rebond { amount } if executed => {
                    MatchingPool::<T>::try_mutate(|p| -> DispatchResult {
                        p.update_total_stake_amount(amount, Subtraction)
                    })?;
                }
                WithdrawUnbonded {
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

        #[require_transactional]
        fn do_update_exchange_rate(bonding_amount: BalanceOf<T>) -> DispatchResult {
            let matching_ledger = Self::matching_pool();
            let total_issuance = T::Assets::total_issuance(Self::liquid_currency()?);
            if total_issuance.is_zero() {
                return Ok(());
            }
            let new_exchange_rate = Rate::checked_from_rational(
                bonding_amount
                    .checked_add(matching_ledger.total_stake_amount)
                    .and_then(|r| r.checked_sub(matching_ledger.total_unstake_amount))
                    .ok_or(ArithmeticError::Overflow)?,
                total_issuance,
            )
            .ok_or(Error::<T>::InvalidExchangeRate)?;
            if new_exchange_rate != Self::exchange_rate() {
                ExchangeRate::<T>::put(new_exchange_rate);
                Self::deposit_event(Event::<T>::ExchangeRateUpdated(new_exchange_rate));
            }
            Ok(())
        }

        fn ensure_origin(origin: OriginFor<T>) -> DispatchResult {
            T::RelayOrigin::ensure_origin(origin.clone())
                .map(|_| ())
                .or_else(|_| match ensure_signed(origin) {
                    Ok(who) if T::Members::contains(&who) => Ok(()),
                    _ => Err(BadOrigin),
                })?;
            Ok(())
        }

        fn ensure_market_cap(asset_id: AssetIdOf<T>, amount: BalanceOf<T>) -> DispatchResult {
            ensure!(
                T::Assets::total_issuance(asset_id)
                    .checked_add(amount)
                    .ok_or(ArithmeticError::Overflow)?
                    <= Self::market_cap(),
                Error::<T>::CapExceeded
            );
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

impl<T: Config, Balance: BalanceT + FixedPointOperand> LiquidStakingConvert<Balance> for Pallet<T> {
    fn staking_to_liquid(amount: Balance) -> Option<Balance> {
        Self::exchange_rate()
            .reciprocal()
            .and_then(|r| r.checked_mul_int(amount))
    }

    fn liquid_to_staking(liquid_amount: Balance) -> Option<Balance> {
        Self::exchange_rate().checked_mul_int(liquid_amount)
    }
}
