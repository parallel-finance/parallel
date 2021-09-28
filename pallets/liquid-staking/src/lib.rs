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
pub mod relaychain;
#[cfg(test)]
mod tests;
pub mod types;
pub mod weights;

use primitives::{ExchangeRateProvider, LiquidStakingCurrenciesProvider};

pub use self::pallet::*;

#[frame_support::pallet]
mod pallet {
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
        BoundedVec, PalletId, Twox64Concat,
    };
    use frame_system::{
        ensure_signed,
        pallet_prelude::{BlockNumberFor, OriginFor},
    };
    use orml_traits::XcmTransfer;
    use sp_runtime::{
        traits::{AccountIdConversion, AtLeast32BitUnsigned, CheckedAdd, CheckedSub, Zero},
        ArithmeticError, FixedPointNumber, FixedPointOperand,
    };
    use sp_std::vec::Vec;
    use xcm::v0::{Junction, MultiLocation, NetworkId, SendXcm};

    use primitives::{DerivativeProvider, EraIndex, Rate, Ratio};

    use crate::{
        types::{MatchingLedger, RewardDestination, StakingSettlementKind},
        weights::WeightInfo,
    };

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
        type Assets: Transfer<Self::AccountId> + Inspect<Self::AccountId> + Mutate<Self::AccountId>;

        /// Offchain bridge accout who manages staking currency in relaychain.
        type BridgeOrigin: EnsureOrigin<Self::Origin>;

        /// The origin which can update liquid currency, staking currency
        type UpdateOrigin: EnsureOrigin<Self::Origin>;

        /// The pallet id of liquid staking, keeps all the staking assets.
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// XCM transfer
        type XcmTransfer: XcmTransfer<Self::AccountId, BalanceOf<Self>, AssetIdOf<Self>>;

        /// XCM transact
        type XcmSender: SendXcm;

        /// Base xcm transaction weight
        #[pallet::constant]
        type BaseXcmWeight: Get<Weight>;

        /// Parachain account on relaychain
        #[pallet::constant]
        type RelayAgent: Get<Self::AccountId>;

        /// Account derivative index
        #[pallet::constant]
        type DerivativeIndex: Get<u16>;

        /// Account derivative functionality provider
        type DerivativeProvider: DerivativeProvider<Self::AccountId>;

        /// Unstake queue capacity
        #[pallet::constant]
        type UnstakeQueueCapacity: Get<u32>;

        /// Basis of period.
        #[pallet::constant]
        type PeriodBasis: Get<BlockNumberFor<Self>>;

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
        /// Reward/Slash has been recorded.
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
        /// Send staking.payout_stakers call to relaychain
        PayoutStakersCallSent(T::AccountId, u32),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Reward/Slash has been recorded.
        StakingSettlementAlreadyRecorded,
        /// Exchange rate is invalid.
        InvalidExchangeRate,
        /// Era has been pushed before.
        EraAlreadyPushed,
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
        /// Failed to send staking.payout_stakers call
        PayoutStakersCallFailed,
        /// Liquid currency hasn't been set
        LiquidCurrencyNotSet,
        /// Staking currency hasn't been set
        StakingCurrencyNotSet,
        /// UnstakeQueue is already full
        ExceededUnstakeQueueCapacity,
    }

    /// The exchange rate between relaychain native asset and the voucher.
    #[pallet::storage]
    #[pallet::getter(fn exchange_rate)]
    pub type ExchangeRate<T: Config> = StorageValue<_, Rate, ValueQuery>;

    /// Total amount of staked assets in relaycahin.
    #[pallet::storage]
    #[pallet::getter(fn staking_pool)]
    pub type StakingPool<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    /// Fraction of reward currently set aside for reserves
    #[pallet::storage]
    #[pallet::getter(fn reserve_factor)]
    pub type ReserveFactor<T: Config> = StorageValue<_, Ratio, ValueQuery>;

    /// Records reward or slash of era.
    #[pallet::storage]
    #[pallet::getter(fn staking_settlement_records)]
    pub type StakingSettlementRecords<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        EraIndex,
        Twox64Concat,
        StakingSettlementKind,
        BalanceOf<T>,
    >;

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
    #[pallet::getter(fn liquid_currency)]
    pub type LiquidCurrency<T: Config> = StorageValue<_, AssetIdOf<T>, OptionQuery>;

    /// Staking currency asset id
    #[pallet::storage]
    #[pallet::getter(fn staking_currency)]
    pub type StakingCurrency<T: Config> = StorageValue<_, AssetIdOf<T>, OptionQuery>;

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
            if Self::staking_currency().is_none() {
                return remaining_weight;
            }
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

                if T::Assets::transfer(
                    Self::staking_currency().unwrap(),
                    &Self::account_id(),
                    who,
                    *amount,
                    true,
                )
                .is_err()
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

            // Check if current period end.
            if !(n % basis).is_zero() {
                return;
            }

            // Check if there are staking to be settled.
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
        BalanceOf<T>: FixedPointOperand,
        AssetIdOf<T>: AtLeast32BitUnsigned,
    {
        /// Put assets under staking, the native assets will be transferred to the account
        /// owned by the pallet, user receive derivative in return, such derivative can be
        /// further used as collateral for lending.
        ///
        /// - `amount`: the amount of staking assets
        #[pallet::weight(T::WeightInfo::stake())]
        #[transactional]
        pub fn stake(
            origin: OriginFor<T>,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            let exchange_rate = ExchangeRate::<T>::get();
            let liquid_amount = exchange_rate
                .reciprocal()
                .and_then(|r| r.checked_mul_int(amount))
                .ok_or(Error::<T>::InvalidExchangeRate)?;

            T::Assets::transfer(
                Self::staking_currency().ok_or(Error::<T>::StakingCurrencyNotSet)?,
                &who,
                &Self::account_id(),
                amount,
                true,
            )?;
            T::Assets::mint_into(
                Self::liquid_currency().ok_or(Error::<T>::LiquidCurrencyNotSet)?,
                &who,
                liquid_amount,
            )?;

            StakingPool::<T>::try_mutate(|b| -> DispatchResult {
                *b = b.checked_add(&amount).ok_or(ArithmeticError::Overflow)?;
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
        #[pallet::weight(T::WeightInfo::unstake())]
        #[transactional]
        pub fn unstake(
            origin: OriginFor<T>,
            #[pallet::compact] liquid_amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            let exchange_rate = ExchangeRate::<T>::get();
            let asset_amount = exchange_rate
                .checked_mul_int(liquid_amount)
                .ok_or(Error::<T>::InvalidExchangeRate)?;

            if T::Assets::transfer(
                Self::staking_currency().unwrap(),
                &Self::account_id(),
                &who,
                asset_amount,
                true,
            )
            .is_err()
            {
                Self::push_unstake_task(&who, asset_amount)?;
            }

            T::Assets::burn_from(
                Self::liquid_currency().ok_or(Error::<T>::LiquidCurrencyNotSet)?,
                &who,
                liquid_amount,
            )?;
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

        /// Handle staking settlement at the end of an era, such as getting reward or been slashed in relaychain.
        #[pallet::weight(<T as Config>::WeightInfo::record_staking_settlement())]
        #[transactional]
        pub fn record_staking_settlement(
            origin: OriginFor<T>,
            era_index: EraIndex,
            #[pallet::compact] amount: BalanceOf<T>,
            kind: StakingSettlementKind,
        ) -> DispatchResultWithPostInfo {
            T::BridgeOrigin::ensure_origin(origin)?;
            Self::ensure_settlement_not_recorded(era_index, kind)?;
            Self::update_staking_pool(kind, amount)?;

            StakingSettlementRecords::<T>::insert(era_index, kind, amount);
            Self::deposit_event(Event::<T>::StakingSettlementRecorded(kind, amount));
            Ok(().into())
        }

        /// Do settlement for matching pool.
        ///
        /// Calculate the imbalance of current state and send corresponding operations to
        /// relay-chain.
        ///
        /// NOTE: currently it finished by stake-client.
        #[pallet::weight(<T as Config>::WeightInfo::settlement())]
        #[transactional]
        pub fn settlement(
            origin: OriginFor<T>,
            #[pallet::compact] unbonding_amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            T::BridgeOrigin::ensure_origin(origin)?;
            let (bond_amount, rebond_amount, unbond_amount) =
                MatchingPool::<T>::take().matching(unbonding_amount);

            if !bond_amount.is_zero() {
                T::XcmTransfer::transfer(
                    Self::account_id(),
                    Self::staking_currency().ok_or(Error::<T>::StakingCurrencyNotSet)?,
                    bond_amount,
                    MultiLocation::X2(
                        Junction::Parent,
                        Junction::AccountId32 {
                            network: NetworkId::Any,
                            id: T::RelayAgent::get().into(),
                        },
                    ),
                    T::BaseXcmWeight::get(),
                )?;
            }

            Self::deposit_event(Event::<T>::StakingOpRequest(
                bond_amount,
                rebond_amount,
                unbond_amount,
            ));
            Ok(().into())
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
        BalanceOf<T>: FixedPointOperand,
        AssetIdOf<T>: AtLeast32BitUnsigned,
    {
        /// Ensure settlement not recorded for this `era_index`.
        #[inline]
        fn ensure_settlement_not_recorded(
            era_index: EraIndex,
            kind: StakingSettlementKind,
        ) -> DispatchResult {
            ensure!(
                !StakingSettlementRecords::<T>::contains_key(era_index, kind),
                Error::<T>::StakingSettlementAlreadyRecorded
            );
            Ok(())
        }

        /// Increase/Decrease staked asset in staking pool, and synchronized the exchange rate.
        fn update_staking_pool(
            kind: StakingSettlementKind,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            use StakingSettlementKind::*;
            match kind {
                Reward => StakingPool::<T>::try_mutate(|p| -> DispatchResult {
                    *p = p.checked_add(&amount).ok_or(ArithmeticError::Overflow)?;
                    Ok(())
                }),
                Slash => StakingPool::<T>::try_mutate(|p| -> DispatchResult {
                    *p = p.checked_sub(&amount).ok_or(ArithmeticError::Underflow)?;
                    Ok(())
                }),
            }?;

            // Update exchange rate.
            let exchange_rate = Rate::checked_from_rational(
                StakingPool::<T>::get(),
                T::Assets::total_issuance(
                    Self::liquid_currency().ok_or(Error::<T>::LiquidCurrencyNotSet)?,
                ),
            )
            .ok_or(Error::<T>::InvalidExchangeRate)?;
            ExchangeRate::<T>::put(exchange_rate);
            Ok(())
        }

        /// Push an unstake task into queue.
        #[inline]
        fn push_unstake_task(who: &T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
            // UnstakeQueue::<T>::mutate(|q| q.push((who.clone(), amount)))
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
    }

    impl<T: Config> Pallet<T> {
        pub fn account_id() -> T::AccountId {
            T::PalletId::get().into_account()
        }

        /// account derived from parachain account
        pub fn derivative_account_id() -> T::AccountId {
            T::DerivativeProvider::derivative_account_id(
                T::RelayAgent::get(),
                T::DerivativeIndex::get(),
            )
        }
    }
}

impl<T: Config> ExchangeRateProvider for Pallet<T> {
    fn get_exchange_rate() -> primitives::Rate {
        ExchangeRate::<T>::get()
    }
}

impl<T: Config> LiquidStakingCurrenciesProvider<AssetIdOf<T>> for Pallet<T> {
    fn get_staking_currency() -> Option<AssetIdOf<T>> {
        Self::staking_currency()
    }

    fn get_liquid_currency() -> Option<AssetIdOf<T>> {
        Self::liquid_currency()
    }
}
