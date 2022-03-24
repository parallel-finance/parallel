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

pub mod migrations;
pub mod types;
pub mod weights;

#[macro_use]
extern crate primitives;

use frame_support::traits::{fungibles::InspectMetadata, tokens::Balance as BalanceT, Get};
use primitives::{
    ExchangeRateProvider, LiquidStakingConvert, LiquidStakingCurrenciesProvider, Rate,
};
use sp_runtime::{traits::Zero, FixedPointNumber, FixedPointOperand};

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
        storage::with_transaction,
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
    use pallet_xcm::ensure_response;
    use sp_runtime::{
        traits::{AccountIdConversion, BlockNumberProvider, CheckedDiv, CheckedSub, StaticLookup},
        ArithmeticError, FixedPointNumber, TransactionOutcome,
    };
    use sp_std::{boxed::Box, result::Result, vec::Vec};

    use primitives::{
        ump::*, ArithmeticKind, Balance, CurrencyId, DerivativeIndex, EraIndex,
        LiquidStakingConvert, ParaId, Rate, Ratio,
    };

    use super::{types::*, weights::WeightInfo, *};
    use pallet_xcm_helper::XcmHelper;
    use xcm::latest::prelude::*;

    pub const MAX_UNLOCKING_CHUNKS: usize = 32;

    pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
    pub type AssetIdOf<T> =
        <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;
    pub type BalanceOf<T> =
        <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    /// Utility type for managing upgrades/migrations.
    #[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo)]
    pub enum Versions {
        V1,
        V2,
        V3,
    }

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

        /// Derivative index list
        #[pallet::constant]
        type DerivativeIndexList: Get<Vec<DerivativeIndex>>;

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
        type BondingDuration: Get<EraIndex>;

        /// The minimum active bond to become and maintain the role of a nominator.
        #[pallet::constant]
        type MinNominatorBond: Get<BalanceOf<Self>>;

        /// Number of blocknumbers that each period contains.
        /// SessionsPerEra * EpochDuration / MILLISECS_PER_BLOCK
        #[pallet::constant]
        type EraLength: Get<BlockNumberFor<Self>>;

        #[pallet::constant]
        type NumSlashingSpans: Get<u32>;

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
        /// Staking ledger feeded
        StakingLedgerUpdated(DerivativeIndex, StakingLedger<T::AccountId, BalanceOf<T>>),
        /// Sent staking.bond call to relaychain
        Bonding(
            DerivativeIndex,
            T::AccountId,
            BalanceOf<T>,
            RewardDestination<T::AccountId>,
        ),
        /// Sent staking.bond_extra call to relaychain
        BondingExtra(DerivativeIndex, BalanceOf<T>),
        /// Sent staking.unbond call to relaychain
        Unbonding(DerivativeIndex, BalanceOf<T>),
        /// Sent staking.rebond call to relaychain
        Rebonding(DerivativeIndex, BalanceOf<T>),
        /// Sent staking.withdraw_unbonded call to relaychain
        WithdrawingUnbonded(DerivativeIndex, u32),
        /// Sent staking.nominate call to relaychain
        Nominating(DerivativeIndex, Vec<T::AccountId>),
        /// Staking ledger's cap was updated
        StakingLedgerCapUpdated(BalanceOf<T>),
        /// Reserve_factor was updated
        ReserveFactorUpdated(Ratio),
        /// Exchange rate was updated
        ExchangeRateUpdated(Rate),
        /// Notification received
        /// [multi_location, query_id, res]
        NotificationReceived(Box<MultiLocation>, QueryId, Option<(u32, XcmError)>),
        /// Claim user's unbonded staking assets
        /// [account_id, amount]
        ClaimedFor(T::AccountId, BalanceOf<T>),
        /// New era
        /// [era_index]
        NewEra(EraIndex),
        /// Matching stakes & unstakes for optimizing operations to be done
        /// on relay chain
        /// [bond_amount, rebond_amount, unbond_amount]
        Matching(BalanceOf<T>, BalanceOf<T>, BalanceOf<T>),
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
        /// Invalid derivative index
        InvalidDerivativeIndex,
        /// Invalid staking ledger
        InvalidStakingLedger,
        /// Exceeded liquid currency's market cap
        CapExceeded,
        /// Invalid market cap
        InvalidCap,
        /// The factor should be bigger than 0% and smaller than 100%
        InvalidFactor,
        /// Nothing to claim yet
        NothingToClaim,
        /// Stash wasn't bonded yet
        NotBonded,
        /// Stash is already bonded.
        AlreadyBonded,
        /// Can not schedule more unlock chunks.
        NoMoreChunks,
        /// Staking ledger is locked due to mutation in notification_received
        StakingLedgerLocked,
        /// Not withdrawn unbonded yet
        NotWithdrawn,
        /// Cannot have a nominator role with value less than the minimum defined by
        /// `MinNominatorBond`
        InsufficientBond,
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

    /// Staking ledger's cap
    #[pallet::storage]
    #[pallet::getter(fn staking_ledger_cap)]
    pub type StakingLedgerCap<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    /// Flying & failed xcm requests
    #[pallet::storage]
    #[pallet::getter(fn xcm_request)]
    pub type XcmRequests<T> = StorageMap<_, Blake2_128Concat, QueryId, XcmRequest<T>, OptionQuery>;

    /// Current era index
    /// Users can come to claim their unbonded staking assets back once this value arrived
    /// at certain height decided by `BondingDuration` and `EraLength`
    #[pallet::storage]
    #[pallet::getter(fn current_era)]
    pub type CurrentEra<T: Config> = StorageValue<_, EraIndex, ValueQuery>;

    /// Current era's start relaychain block
    #[pallet::storage]
    #[pallet::getter(fn era_start_block)]
    pub type EraStartBlock<T: Config> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

    /// Unbonding requests to be handled after arriving at target era
    #[pallet::storage]
    #[pallet::getter(fn unlockings)]
    pub type Unlockings<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, Vec<UnlockChunk<BalanceOf<T>>>, OptionQuery>;

    /// Platform's staking ledgers
    #[pallet::storage]
    #[pallet::getter(fn staking_ledger)]
    pub type StakingLedgers<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        DerivativeIndex,
        StakingLedger<T::AccountId, BalanceOf<T>>,
        OptionQuery,
    >;

    /// Set to true if staking ledger has been modified in this block
    #[pallet::storage]
    #[pallet::getter(fn is_updated)]
    pub type IsUpdated<T: Config> = StorageMap<_, Twox64Concat, DerivativeIndex, bool, ValueQuery>;

    /// DefaultVersion is using for initialize the StorageVersion
    #[pallet::type_value]
    pub(super) fn DefaultVersion<T: Config>() -> Versions {
        Versions::V2
    }
    /// Storage version of the pallet.
    #[pallet::storage]
    pub(crate) type StorageVersion<T: Config> =
        StorageValue<_, Versions, ValueQuery, DefaultVersion<T>>;

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
            Self::ensure_market_cap(amount)?;

            T::Assets::mint_into(liquid_currency, &who, liquid_amount)?;

            log::trace!(
                target: "liquidStaking::stake",
                "stake_amount: {:?}, liquid_amount: {:?}, reserved: {:?}",
                &amount,
                &liquid_amount,
                &reserves
            );

            MatchingPool::<T>::try_mutate(|p| -> DispatchResult { p.add_stake_amount(amount) })?;
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

            Unlockings::<T>::try_mutate(&who, |b| -> DispatchResult {
                // TODO: check if we can bond before the next era
                // so that the one era's delay can be removed
                let mut chunks = b.take().unwrap_or_default();
                let target_era = Self::current_era() + T::BondingDuration::get() + 1;
                if let Some(mut chunk) = chunks.last_mut().filter(|chunk| chunk.era == target_era) {
                    chunk.value = chunk.value.saturating_add(amount);
                } else {
                    chunks.push(UnlockChunk {
                        value: amount,
                        era: target_era,
                    });
                }
                ensure!(
                    chunks.len() <= MAX_UNLOCKING_CHUNKS,
                    Error::<T>::NoMoreChunks
                );
                *b = Some(chunks);
                Ok(())
            })?;

            T::Assets::burn_from(Self::liquid_currency()?, &who, liquid_amount)?;

            log::trace!(
                target: "liquidStaking::unstake",
                "unstake_amount: {:?}, liquid_amount: {:?}",
                &amount,
                &liquid_amount,
            );

            MatchingPool::<T>::try_mutate(|p| -> DispatchResult { p.add_unstake_amount(amount) })?;

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

        /// Update ledger's max bonded cap
        #[pallet::weight(<T as Config>::WeightInfo::update_staking_ledger_cap())]
        #[transactional]
        pub fn update_staking_ledger_cap(
            origin: OriginFor<T>,
            #[pallet::compact] cap: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            T::UpdateOrigin::ensure_origin(origin)?;

            ensure!(!cap.is_zero(), Error::<T>::InvalidCap);

            log::trace!(
                target: "liquidStaking::update_staking_ledger_cap",
                "cap: {:?}",
                &cap,
            );
            StakingLedgerCap::<T>::mutate(|v| *v = cap);
            Self::deposit_event(Event::<T>::StakingLedgerCapUpdated(cap));
            Ok(().into())
        }

        /// Bond on relaychain via xcm.transact
        #[pallet::weight(<T as Config>::WeightInfo::bond())]
        #[transactional]
        pub fn bond(
            origin: OriginFor<T>,
            derivative_index: DerivativeIndex,
            #[pallet::compact] amount: BalanceOf<T>,
            payee: RewardDestination<T::AccountId>,
        ) -> DispatchResult {
            T::RelayOrigin::ensure_origin(origin)?;
            Self::do_bond(derivative_index, amount, payee)?;
            Ok(())
        }

        /// Bond_extra on relaychain via xcm.transact
        #[pallet::weight(<T as Config>::WeightInfo::bond_extra())]
        #[transactional]
        pub fn bond_extra(
            origin: OriginFor<T>,
            derivative_index: DerivativeIndex,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResult {
            T::RelayOrigin::ensure_origin(origin)?;
            Self::do_bond_extra(derivative_index, amount)?;
            Ok(())
        }

        /// Unbond on relaychain via xcm.transact
        #[pallet::weight(<T as Config>::WeightInfo::unbond())]
        #[transactional]
        pub fn unbond(
            origin: OriginFor<T>,
            derivative_index: DerivativeIndex,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResult {
            T::RelayOrigin::ensure_origin(origin)?;
            Self::do_unbond(derivative_index, amount)?;
            Ok(())
        }

        /// Rebond on relaychain via xcm.transact
        #[pallet::weight(<T as Config>::WeightInfo::rebond())]
        #[transactional]
        pub fn rebond(
            origin: OriginFor<T>,
            derivative_index: DerivativeIndex,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResult {
            T::RelayOrigin::ensure_origin(origin)?;
            Self::do_rebond(derivative_index, amount)?;
            Ok(())
        }

        /// Withdraw unbonded on relaychain via xcm.transact
        #[pallet::weight(<T as Config>::WeightInfo::withdraw_unbonded())]
        #[transactional]
        pub fn withdraw_unbonded(
            origin: OriginFor<T>,
            derivative_index: DerivativeIndex,
            num_slashing_spans: u32,
        ) -> DispatchResult {
            T::RelayOrigin::ensure_origin(origin)?;
            Self::do_withdraw_unbonded(derivative_index, num_slashing_spans)?;
            Ok(())
        }

        /// Nominate on relaychain via xcm.transact
        #[pallet::weight(<T as Config>::WeightInfo::nominate())]
        #[transactional]
        pub fn nominate(
            origin: OriginFor<T>,
            derivative_index: DerivativeIndex,
            targets: Vec<T::AccountId>,
        ) -> DispatchResult {
            T::RelayOrigin::ensure_origin(origin)?;
            Self::do_nominate(derivative_index, targets)?;
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
            let responder =
                ensure_response(<T as Config>::Origin::from(origin.clone())).or_else(|_| {
                    T::UpdateOrigin::ensure_origin(origin).map(|_| MultiLocation::here())
                })?;
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

        /// Claim assets back when current era index arrived
        /// at target era
        #[pallet::weight(<T as Config>::WeightInfo::claim_for())]
        #[transactional]
        pub fn claim_for(
            origin: OriginFor<T>,
            dest: <T::Lookup as StaticLookup>::Source,
        ) -> DispatchResultWithPostInfo {
            Self::ensure_origin(origin)?;
            let who = T::Lookup::lookup(dest)?;
            let current_era = Self::current_era();

            Unlockings::<T>::try_mutate_exists(&who, |b| -> DispatchResult {
                let mut amount: BalanceOf<T> = Zero::zero();
                let chunks = b.as_mut().ok_or(Error::<T>::NothingToClaim)?;
                chunks.retain(|chunk| {
                    if chunk.era > current_era {
                        true
                    } else {
                        amount += chunk.value;
                        false
                    }
                });

                let total_unclaimed = Self::get_total_unclaimed(Self::staking_currency()?);

                log::trace!(
                    target: "liquidStaking::claim_for",
                    "current_era: {:?}, beneficiary: {:?}, total_unclaimed: {:?}, amount: {:?}",
                    &current_era,
                    &who,
                    &total_unclaimed,
                    amount
                );

                if amount.is_zero() {
                    return Err(Error::<T>::NothingToClaim.into());
                }

                if total_unclaimed < amount {
                    return Err(Error::<T>::NotWithdrawn.into());
                }

                T::Assets::transfer(
                    Self::staking_currency()?,
                    &Self::account_id(),
                    &who,
                    amount,
                    false,
                )?;

                if chunks.is_empty() {
                    *b = None;
                }

                Self::deposit_event(Event::<T>::ClaimedFor(who.clone(), amount));
                Ok(())
            })?;
            Ok(().into())
        }

        /// Force set era start block
        #[pallet::weight(<T as Config>::WeightInfo::force_set_era_start_block())]
        #[transactional]
        pub fn force_set_era_start_block(
            origin: OriginFor<T>,
            block_number: BlockNumberFor<T>,
        ) -> DispatchResult {
            T::UpdateOrigin::ensure_origin(origin)?;
            EraStartBlock::<T>::put(block_number);
            Ok(())
        }

        /// Force set current era
        #[pallet::weight(<T as Config>::WeightInfo::force_set_current_era())]
        #[transactional]
        pub fn force_set_current_era(origin: OriginFor<T>, era: EraIndex) -> DispatchResult {
            T::UpdateOrigin::ensure_origin(origin)?;
            CurrentEra::<T>::put(era);
            Ok(())
        }

        /// Force advance era
        #[pallet::weight(<T as Config>::WeightInfo::force_advance_era())]
        #[transactional]
        pub fn force_advance_era(
            origin: OriginFor<T>,
            offset: EraIndex,
        ) -> DispatchResultWithPostInfo {
            T::UpdateOrigin::ensure_origin(origin)?;

            Self::do_advance_era(offset)?;

            Ok(().into())
        }

        /// Force set staking_ledger for updating exchange rate in next era
        #[pallet::weight(<T as Config>::WeightInfo::force_set_staking_ledger())]
        #[transactional]
        pub fn force_set_staking_ledger(
            origin: OriginFor<T>,
            derivative_index: DerivativeIndex,
            staking_ledger: StakingLedger<T::AccountId, BalanceOf<T>>,
        ) -> DispatchResultWithPostInfo {
            Self::ensure_origin(origin)?;

            Self::do_update_ledger(derivative_index, |ledger| {
                ensure!(
                    !Self::is_updated(derivative_index),
                    Error::<T>::StakingLedgerLocked
                );
                ensure!(
                    ledger.active >= T::MinNominatorBond::get(),
                    Error::<T>::InsufficientBond
                );
                log::trace!(
                    target: "liquidStaking::force_set_staking_ledger",
                    "index: {:?}, staking_ledger: {:?}",
                    &derivative_index,
                    &staking_ledger,
                );
                // TODO: using storage proof to validate submitted staking_ledger
                *ledger = staking_ledger;
                Ok(())
            })?;

            Ok(().into())
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
        fn on_initialize(block_number: T::BlockNumber) -> frame_support::weights::Weight {
            let relaychain_block_number = T::RelayChainBlockNumberProvider::current_block_number();
            let offset = Self::offset(relaychain_block_number);
            log::trace!(
                target: "liquidStaking::on_initialize",
                "relaychain_block_number: {:?}, block_number: {:?}, advance_offset: {:?}",
                &relaychain_block_number,
                &block_number,
                &offset
            );
            if offset.is_zero() {
                return <T as Config>::WeightInfo::on_initialize();
            }
            with_transaction(|| match Self::do_advance_era(offset) {
                Ok(()) => TransactionOutcome::Commit(
                    <T as Config>::WeightInfo::on_initialize_with_advance_era(),
                ),
                Err(err) => {
                    log::error!(
                        target: "liquidStaking::do_advance_era",
                        "Could not advance era! block_number: {:#?}, err: {:?}",
                        &block_number,
                        &err
                    );
                    TransactionOutcome::Rollback(
                        <T as Config>::WeightInfo::on_initialize_with_advance_era(),
                    )
                }
            })
        }

        fn on_finalize(_n: T::BlockNumber) {
            IsUpdated::<T>::remove_all(None);
        }
    }

    impl<T: Config> Pallet<T> {
        /// Staking pool account
        pub fn account_id() -> T::AccountId {
            T::PalletId::get().into_account()
        }

        /// Parachain's sovereign account
        pub fn sovereign_account_id() -> T::AccountId {
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

        /// Get total unclaimed
        pub fn get_total_unclaimed(staking_currency: AssetIdOf<T>) -> BalanceOf<T> {
            let matching_pool = Self::matching_pool();
            T::Assets::reducible_balance(staking_currency, &Self::account_id(), false)
                .saturating_sub(Self::total_reserves())
                .saturating_sub(matching_pool.total_stake_amount.free)
                .saturating_sub(matching_pool.total_stake_amount.reserved)
        }

        /// Derivative of parachain's account
        pub fn derivative_sovereign_account_id(index: DerivativeIndex) -> T::AccountId {
            let para_account = Self::sovereign_account_id();
            pallet_utility::Pallet::<T>::derivative_account_id(para_account, index)
        }

        fn offset(relaychain_block_number: BlockNumberFor<T>) -> EraIndex {
            relaychain_block_number
                .checked_sub(&Self::era_start_block())
                .and_then(|r| r.checked_div(&T::EraLength::get()))
                .and_then(|r| TryInto::<EraIndex>::try_into(r).ok())
                .unwrap_or_else(Zero::zero)
        }

        fn bonded_of(index: DerivativeIndex) -> BalanceOf<T> {
            Self::staking_ledger(&index).map_or(Zero::zero(), |ledger| ledger.active)
        }

        fn unbonding_of(index: DerivativeIndex) -> BalanceOf<T> {
            Self::staking_ledger(&index).map_or(Zero::zero(), |ledger| {
                ledger.total.saturating_sub(ledger.active)
            })
        }

        fn unbonded_of(index: DerivativeIndex) -> BalanceOf<T> {
            let current_era = Self::current_era();
            Self::staking_ledger(&index).map_or(Zero::zero(), |ledger| {
                ledger.unlocking.iter().fold(Zero::zero(), |acc, chunk| {
                    if chunk.era <= current_era {
                        acc.saturating_add(chunk.value)
                    } else {
                        acc
                    }
                })
            })
        }

        fn get_total_unbonding() -> BalanceOf<T> {
            StakingLedgers::<T>::iter_values().fold(Zero::zero(), |acc, ledger| {
                acc.saturating_add(ledger.total.saturating_sub(ledger.active))
            })
        }

        fn get_total_bonded() -> BalanceOf<T> {
            StakingLedgers::<T>::iter_values().fold(Zero::zero(), |acc, ledger| {
                acc.saturating_add(ledger.active)
            })
        }

        fn get_market_cap() -> BalanceOf<T> {
            Self::staking_ledger_cap()
                .saturating_mul(T::DerivativeIndexList::get().len() as BalanceOf<T>)
        }

        #[require_transactional]
        fn do_bond(
            derivative_index: DerivativeIndex,
            amount: BalanceOf<T>,
            payee: RewardDestination<T::AccountId>,
        ) -> DispatchResult {
            if amount.is_zero() {
                return Ok(());
            }

            if StakingLedgers::<T>::contains_key(&derivative_index) {
                return Self::do_bond_extra(derivative_index, amount);
            }

            ensure!(
                T::DerivativeIndexList::get().contains(&derivative_index),
                Error::<T>::InvalidDerivativeIndex
            );
            ensure!(
                amount >= T::MinNominatorBond::get(),
                Error::<T>::InsufficientBond
            );
            Self::ensure_staking_ledger_cap(derivative_index, amount)?;

            log::trace!(
                target: "liquidStaking::bond",
                "index: {:?}, amount: {:?}",
                &derivative_index,
                &amount,
            );

            MatchingPool::<T>::try_mutate(|p| -> DispatchResult {
                p.set_stake_amount_lock(amount)
            })?;
            let staking_currency = Self::staking_currency()?;
            let derivative_account_id = Self::derivative_sovereign_account_id(derivative_index);
            let query_id = T::XCM::do_bond(
                amount,
                payee.clone(),
                derivative_account_id.clone(),
                staking_currency,
                derivative_index,
                Self::notify_placeholder(),
            )?;

            XcmRequests::<T>::insert(
                query_id,
                XcmRequest::Bond {
                    index: derivative_index,
                    amount,
                },
            );

            Self::deposit_event(Event::<T>::Bonding(
                derivative_index,
                derivative_account_id,
                amount,
                payee,
            ));

            Ok(())
        }

        #[require_transactional]
        fn do_bond_extra(
            derivative_index: DerivativeIndex,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            if amount.is_zero() {
                return Ok(());
            }

            ensure!(
                T::DerivativeIndexList::get().contains(&derivative_index),
                Error::<T>::InvalidDerivativeIndex
            );
            ensure!(
                StakingLedgers::<T>::contains_key(&derivative_index),
                Error::<T>::NotBonded
            );
            Self::ensure_staking_ledger_cap(derivative_index, amount)?;

            log::trace!(
                target: "liquidStaking::bond_extra",
                "index: {:?}, amount: {:?}",
                &derivative_index,
                &amount,
            );

            MatchingPool::<T>::try_mutate(|p| -> DispatchResult {
                p.set_stake_amount_lock(amount)
            })?;
            let query_id = T::XCM::do_bond_extra(
                amount,
                Self::derivative_sovereign_account_id(derivative_index),
                Self::staking_currency()?,
                derivative_index,
                Self::notify_placeholder(),
            )?;

            XcmRequests::<T>::insert(
                query_id,
                XcmRequest::BondExtra {
                    index: derivative_index,
                    amount,
                },
            );

            Self::deposit_event(Event::<T>::BondingExtra(derivative_index, amount));

            Ok(())
        }

        #[require_transactional]
        fn do_unbond(derivative_index: DerivativeIndex, amount: BalanceOf<T>) -> DispatchResult {
            if amount.is_zero() {
                return Ok(());
            }

            ensure!(
                T::DerivativeIndexList::get().contains(&derivative_index),
                Error::<T>::InvalidDerivativeIndex
            );

            let ledger: StakingLedger<T::AccountId, BalanceOf<T>> =
                Self::staking_ledger(&derivative_index).ok_or(Error::<T>::NotBonded)?;
            ensure!(
                ledger.unlocking.len() < MAX_UNLOCKING_CHUNKS,
                Error::<T>::NoMoreChunks
            );
            ensure!(
                ledger.active.saturating_sub(amount) >= T::MinNominatorBond::get(),
                Error::<T>::InsufficientBond
            );

            log::trace!(
                target: "liquidStaking::unbond",
                "index: {:?} , amount: {:?}",
                &derivative_index,
                &amount,
            );

            MatchingPool::<T>::try_mutate(|p| -> DispatchResult {
                p.set_unstake_amount_lock(amount)
            })?;
            let query_id = T::XCM::do_unbond(
                amount,
                Self::staking_currency()?,
                derivative_index,
                Self::notify_placeholder(),
            )?;

            XcmRequests::<T>::insert(
                query_id,
                XcmRequest::Unbond {
                    index: derivative_index,
                    amount,
                },
            );

            Self::deposit_event(Event::<T>::Unbonding(derivative_index, amount));

            Ok(())
        }

        #[require_transactional]
        fn do_rebond(derivative_index: DerivativeIndex, amount: BalanceOf<T>) -> DispatchResult {
            if amount.is_zero() {
                return Ok(());
            }

            ensure!(
                T::DerivativeIndexList::get().contains(&derivative_index),
                Error::<T>::InvalidDerivativeIndex
            );
            ensure!(
                StakingLedgers::<T>::contains_key(&derivative_index),
                Error::<T>::NotBonded
            );
            Self::ensure_staking_ledger_cap(derivative_index, amount)?;

            log::trace!(
                target: "liquidStaking::rebond",
                "index: {:?}, amount: {:?}",
                &derivative_index,
                &amount,
            );

            MatchingPool::<T>::try_mutate(|p| -> DispatchResult {
                p.set_stake_amount_lock(amount)
            })?;
            let query_id = T::XCM::do_rebond(
                amount,
                Self::staking_currency()?,
                derivative_index,
                Self::notify_placeholder(),
            )?;

            XcmRequests::<T>::insert(
                query_id,
                XcmRequest::Rebond {
                    index: derivative_index,
                    amount,
                },
            );

            Self::deposit_event(Event::<T>::Rebonding(derivative_index, amount));

            Ok(())
        }

        #[require_transactional]
        fn do_withdraw_unbonded(
            derivative_index: DerivativeIndex,
            num_slashing_spans: u32,
        ) -> DispatchResult {
            if Self::unbonded_of(derivative_index).is_zero() {
                return Ok(());
            }

            ensure!(
                T::DerivativeIndexList::get().contains(&derivative_index),
                Error::<T>::InvalidDerivativeIndex
            );
            ensure!(
                StakingLedgers::<T>::contains_key(&derivative_index),
                Error::<T>::NotBonded
            );

            log::trace!(
                target: "liquidStaking::withdraw_unbonded",
                "index: {:?}, num_slashing_spans: {:?}",
                &derivative_index,
                &num_slashing_spans,
            );

            let query_id = T::XCM::do_withdraw_unbonded(
                num_slashing_spans,
                Self::sovereign_account_id(),
                Self::staking_currency()?,
                derivative_index,
                Self::notify_placeholder(),
            )?;

            XcmRequests::<T>::insert(
                query_id,
                XcmRequest::WithdrawUnbonded {
                    index: derivative_index,
                    num_slashing_spans,
                },
            );

            Self::deposit_event(Event::<T>::WithdrawingUnbonded(
                derivative_index,
                num_slashing_spans,
            ));

            Ok(())
        }

        #[require_transactional]
        fn do_nominate(
            derivative_index: DerivativeIndex,
            targets: Vec<T::AccountId>,
        ) -> DispatchResult {
            ensure!(
                T::DerivativeIndexList::get().contains(&derivative_index),
                Error::<T>::InvalidDerivativeIndex
            );
            ensure!(
                StakingLedgers::<T>::contains_key(&derivative_index),
                Error::<T>::NotBonded
            );

            log::trace!(
                target: "liquidStaking::nominate",
                "index: {:?}",
                &derivative_index,
            );

            let query_id = T::XCM::do_nominate(
                targets.clone(),
                Self::staking_currency()?,
                derivative_index,
                Self::notify_placeholder(),
            )?;

            XcmRequests::<T>::insert(
                query_id,
                XcmRequest::Nominate {
                    index: derivative_index,
                    targets: targets.clone(),
                },
            );

            Self::deposit_event(Event::<T>::Nominating(derivative_index, targets));

            Ok(())
        }

        #[require_transactional]
        fn do_multi_bond(
            mut total_amount: BalanceOf<T>,
            payee: RewardDestination<T::AccountId>,
        ) -> DispatchResult {
            let total_bonded = Self::get_total_bonded();
            let new_total_bonded = total_bonded
                .saturating_add(total_amount)
                .min(Self::get_market_cap());
            total_amount = new_total_bonded.saturating_sub(total_bonded);
            if total_amount.is_zero() {
                return Ok(());
            }

            let mut amounts: Vec<(DerivativeIndex, BalanceOf<T>)> = T::DerivativeIndexList::get()
                .iter()
                .map(|&index| (index, Self::bonded_of(index)))
                .collect();
            let new_avg_bonded = new_total_bonded.saturating_div(amounts.len() as BalanceOf<T>);

            amounts.sort_by(|a, b| a.1.cmp(&b.1));

            let mut distributions: Vec<(DerivativeIndex, BalanceOf<T>)> = vec![];
            let mut remain = total_amount;

            for (index, bonded) in amounts.into_iter() {
                if remain.is_zero() {
                    break;
                }
                let amount = Self::staking_ledger_cap()
                    .saturating_sub(bonded)
                    .min(new_avg_bonded.saturating_sub(bonded));
                if amount.is_zero() || bonded.saturating_add(amount) < T::MinNominatorBond::get() {
                    continue;
                }
                distributions.push((index, amount));
                remain = remain.saturating_sub(amount);
            }

            for (index, amount) in distributions.into_iter() {
                Self::do_bond(index, amount, payee.clone())?;
            }

            Ok(())
        }

        #[require_transactional]
        fn do_multi_unbond(total_amount: BalanceOf<T>) -> DispatchResult {
            let total_bonded = Self::get_total_bonded();
            let mut amounts: Vec<(DerivativeIndex, BalanceOf<T>)> = T::DerivativeIndexList::get()
                .iter()
                .map(|&index| (index, Self::bonded_of(index)))
                .collect();

            amounts.sort_by(|a, b| b.1.cmp(&a.1));

            let mut distributions: Vec<(DerivativeIndex, BalanceOf<T>, BalanceOf<T>)> = vec![];
            let mut remain = total_amount;

            for (index, bonded) in amounts.into_iter() {
                if remain.is_zero() {
                    break;
                }
                let share = Ratio::from_rational(bonded, total_bonded);
                let amount = share.mul_floor(total_amount);
                if amount.is_zero() || bonded.saturating_sub(amount) < T::MinNominatorBond::get() {
                    continue;
                }
                distributions.push((index, amount, bonded));
                remain = remain.saturating_sub(amount);
            }

            let mut idx = 0_usize;
            while !remain.is_zero() && idx < distributions.len() {
                let (_, amount, bonded) = &mut distributions[idx];
                let extra = bonded
                    .saturating_sub(*amount)
                    .saturating_sub(T::MinNominatorBond::get())
                    .min(remain);
                *amount = amount.saturating_add(extra);
                remain = remain.saturating_sub(extra);
                idx += 1;
            }

            for (index, amount, _) in distributions.into_iter() {
                Self::do_unbond(index, amount)?;
            }

            Ok(())
        }

        #[require_transactional]
        fn do_multi_rebond(total_amount: BalanceOf<T>) -> DispatchResult {
            let mut amounts: Vec<(DerivativeIndex, BalanceOf<T>, BalanceOf<T>)> =
                T::DerivativeIndexList::get()
                    .iter()
                    .map(|&index| (index, Self::bonded_of(index), Self::unbonding_of(index)))
                    .collect();

            amounts.sort_by(|a, b| a.2.cmp(&b.2));

            let mut distributions: Vec<(DerivativeIndex, BalanceOf<T>)> = vec![];
            let mut remain = total_amount;

            for (index, bonded, unbonding) in amounts.into_iter() {
                if remain.is_zero() {
                    break;
                }
                let amount = Self::staking_ledger_cap()
                    .saturating_sub(bonded)
                    .min(unbonding)
                    .min(remain);
                if amount.is_zero() {
                    continue;
                }
                distributions.push((index, amount));
                remain = remain.saturating_sub(amount);
            }

            for (index, amount) in distributions.into_iter() {
                Self::do_rebond(index, amount)?;
            }

            Ok(())
        }

        #[require_transactional]
        fn do_multi_withdraw_unbonded(num_slashing_spans: u32) -> DispatchResult {
            for derivative_index in StakingLedgers::<T>::iter_keys() {
                Self::do_withdraw_unbonded(derivative_index, num_slashing_spans)?;
            }

            Ok(())
        }

        #[require_transactional]
        fn do_notification_received(
            query_id: QueryId,
            req: XcmRequest<T>,
            res: Option<(u32, XcmError)>,
        ) -> DispatchResult {
            use ArithmeticKind::*;
            use XcmRequest::*;

            log::trace!(
                target: "liquidStaking::notification_received",
                "query_id: {:?}, response: {:?}",
                &query_id,
                &res
            );

            let executed = res.is_none();
            if !executed {
                return Ok(());
            }

            match req {
                Bond {
                    index: derivative_index,
                    amount,
                } => {
                    ensure!(
                        !StakingLedgers::<T>::contains_key(&derivative_index),
                        Error::<T>::AlreadyBonded
                    );
                    let staking_ledger = <StakingLedger<T::AccountId, BalanceOf<T>>>::new(
                        Self::derivative_sovereign_account_id(derivative_index),
                        amount,
                    );
                    StakingLedgers::<T>::insert(derivative_index, staking_ledger);
                    MatchingPool::<T>::try_mutate(|p| -> DispatchResult {
                        p.remove_stake_amount_lock(amount)?;
                        p.sub_stake_amount(amount)?;
                        p.clear();
                        Ok(())
                    })?;
                    T::Assets::burn_from(Self::staking_currency()?, &Self::account_id(), amount)?;
                }
                BondExtra {
                    index: derivative_index,
                    amount,
                } => {
                    Self::do_update_ledger(derivative_index, |ledger| {
                        ledger.bond_extra(amount);
                        Ok(())
                    })?;
                    MatchingPool::<T>::try_mutate(|p| -> DispatchResult {
                        p.remove_stake_amount_lock(amount)?;
                        p.sub_stake_amount(amount)?;
                        p.clear();
                        Ok(())
                    })?;
                    T::Assets::burn_from(Self::staking_currency()?, &Self::account_id(), amount)?;
                }
                Unbond {
                    index: derivative_index,
                    amount,
                } => {
                    let target_era = Self::current_era() + T::BondingDuration::get();
                    Self::do_update_ledger(derivative_index, |ledger| {
                        ledger.unbond(amount, target_era);
                        Ok(())
                    })?;
                    MatchingPool::<T>::try_mutate(|p| -> DispatchResult {
                        p.remove_unstake_amount_lock(amount)?;
                        p.sub_unstake_amount(amount)?;
                        p.clear();
                        Ok(())
                    })?;
                }
                Rebond {
                    index: derivative_index,
                    amount,
                } => {
                    Self::do_update_ledger(derivative_index, |ledger| {
                        ledger.rebond(amount);
                        Ok(())
                    })?;
                    MatchingPool::<T>::try_mutate(|p| -> DispatchResult {
                        p.remove_stake_amount_lock(amount)?;
                        p.sub_stake_amount(amount)?;
                        p.clear();
                        Ok(())
                    })?;
                }
                WithdrawUnbonded {
                    index: derivative_index,
                    num_slashing_spans: _,
                } => {
                    Self::do_update_ledger(derivative_index, |ledger| {
                        let current_era = Self::current_era();
                        let total = ledger.total;
                        let staking_currency = Self::staking_currency()?;
                        let account_id = Self::account_id();
                        ledger.consolidate_unlocked(current_era);
                        let amount = total.saturating_sub(ledger.total);
                        T::Assets::mint_into(staking_currency, &account_id, amount)?;
                        Ok(())
                    })?;
                }
                Nominate { targets: _, .. } => {}
            }
            XcmRequests::<T>::remove(&query_id);
            Ok(())
        }

        #[require_transactional]
        fn do_update_exchange_rate() -> DispatchResult {
            let matching_ledger = Self::matching_pool();
            let total_bonded = Self::get_total_bonded();
            let issuance = T::Assets::total_issuance(Self::liquid_currency()?);
            if issuance.is_zero() {
                return Ok(());
            }
            let new_exchange_rate = Rate::checked_from_rational(
                total_bonded
                    .checked_add(matching_ledger.total_stake_amount.free)
                    .and_then(|r| r.checked_add(matching_ledger.total_stake_amount.reserved))
                    .and_then(|r| r.checked_sub(matching_ledger.total_unstake_amount.free))
                    .and_then(|r| r.checked_sub(matching_ledger.total_unstake_amount.reserved))
                    .ok_or(ArithmeticError::Overflow)?,
                issuance,
            )
            .ok_or(Error::<T>::InvalidExchangeRate)?;
            // slashes should be handled properly offchain
            // by doing `bond_extra` using OrmlXcm or PolkadotXcm
            if new_exchange_rate > Self::exchange_rate() {
                ExchangeRate::<T>::put(new_exchange_rate);
                Self::deposit_event(Event::<T>::ExchangeRateUpdated(new_exchange_rate));
            }
            Ok(())
        }

        #[require_transactional]
        fn do_update_ledger(
            derivative_index: DerivativeIndex,
            cb: impl FnOnce(&mut StakingLedger<T::AccountId, BalanceOf<T>>) -> DispatchResult,
        ) -> DispatchResult {
            StakingLedgers::<T>::try_mutate(derivative_index, |ledger| -> DispatchResult {
                let ledger = ledger.as_mut().ok_or(Error::<T>::NotBonded)?;
                cb(ledger)?;
                IsUpdated::<T>::insert(derivative_index, true);
                Self::deposit_event(Event::<T>::StakingLedgerUpdated(
                    derivative_index,
                    ledger.clone(),
                ));
                Ok(())
            })
        }

        #[require_transactional]
        pub(crate) fn do_advance_era(offset: EraIndex) -> DispatchResult {
            EraStartBlock::<T>::put(T::RelayChainBlockNumberProvider::current_block_number());
            CurrentEra::<T>::mutate(|e| *e = e.saturating_add(offset));

            log::trace!(
                target: "liquidStaking::do_advance_era",
                "offset: {:?}",
                &offset,
            );

            Self::deposit_event(Event::<T>::NewEra(Self::current_era()));

            // ignore error
            let _ = || -> DispatchResult {
                let unbonding_amount = Self::get_total_unbonding();
                let (bond_amount, rebond_amount, unbond_amount) =
                    Self::matching_pool().matching(unbonding_amount)?;
                Self::do_multi_bond(bond_amount, RewardDestination::Staked)?;
                Self::do_multi_rebond(rebond_amount)?;

                Self::do_multi_unbond(unbond_amount)?;

                Self::do_multi_withdraw_unbonded(T::NumSlashingSpans::get())?;

                Self::do_update_exchange_rate()?;

                Self::deposit_event(Event::<T>::Matching(
                    bond_amount,
                    rebond_amount,
                    unbond_amount,
                ));

                Ok(())
            }();

            Ok(())
        }

        fn ensure_origin(origin: OriginFor<T>) -> DispatchResult {
            if T::RelayOrigin::ensure_origin(origin.clone()).is_ok() {
                return Ok(());
            }
            let who = ensure_signed(origin)?;
            if !T::Members::contains(&who) {
                return Err(BadOrigin.into());
            }
            Ok(())
        }

        fn ensure_market_cap(amount: BalanceOf<T>) -> DispatchResult {
            ensure!(
                Self::get_total_bonded().saturating_add(amount) <= Self::get_market_cap(),
                Error::<T>::CapExceeded
            );
            Ok(())
        }

        fn ensure_staking_ledger_cap(
            derivative_index: DerivativeIndex,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            ensure!(
                Self::bonded_of(derivative_index).saturating_add(amount)
                    <= Self::staking_ledger_cap(),
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
