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

//! # Loans pallet
//!
//! ## Overview
//!
//! Loans pallet implement the lending protocol by using a pool-based strategy
//! that aggregates each user's supplied assets. The interest rate is dynamically
//! determined by the supply and demand.

#![cfg_attr(not(feature = "std"), no_std)]

pub use crate::rate_model::*;

use frame_support::{
    log,
    pallet_prelude::*,
    storage::{with_transaction, TransactionOutcome},
    traits::{
        tokens::fungibles::{Inspect, Mutate, Transfer},
        UnixTime,
    },
    transactional, PalletId,
};
use frame_system::pallet_prelude::*;
pub use pallet::*;
use primitives::{CurrencyId, Liquidity, Price, PriceFeeder, Rate, Ratio, Shortfall, Timestamp};
use sp_runtime::{
    traits::{
        AccountIdConversion, AtLeast32BitUnsigned, CheckedAdd, CheckedDiv, CheckedMul, CheckedSub,
        One, Saturating, StaticLookup, Zero,
    },
    ArithmeticError, FixedPointNumber, FixedPointOperand, FixedU128, SaturatedConversion,
};
use sp_std::{convert::TryInto, result::Result};

pub use types::{BorrowSnapshot, Deposits, EarnedSnapshot, Market, MarketState};
pub use weights::WeightInfo;

mod benchmarking;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

mod interest;
mod rate_model;
mod types;

pub mod weights;

type AssetIdOf<T> =
    <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;
type BalanceOf<T> =
    <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {

    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The oracle price feeder
        type PriceFeeder: PriceFeeder;

        /// The loan's module id, keep all collaterals of CDPs.
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// The origin which can add/reduce reserves.
        type ReserveOrigin: EnsureOrigin<Self::Origin>;

        /// The origin which can update rate model, liquidate incentive and
        /// add/reduce reserves. Root can always do this.
        type UpdateOrigin: EnsureOrigin<Self::Origin>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;

        /// Unix time
        type UnixTime: UnixTime;

        /// Assets for deposit/withdraw collateral assets to/from loans module
        type Assets: Transfer<Self::AccountId> + Inspect<Self::AccountId> + Mutate<Self::AccountId>;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Insufficient liquidity to borrow more or disable collateral
        InsufficientLiquidity,
        /// Insufficient deposit to redeem
        InsufficientDeposit,
        /// Repay amount greater than allowed
        TooMuchRepay,
        /// Asset already enabled/disabled collateral
        DuplicateOperation,
        /// No deposit asset
        NoDeposit,
        /// Repay amount more than collateral amount
        InsufficientCollateral,
        /// Liquidator is same as borrower
        LiquidatorIsBorrower,
        /// Deposits are not used as a collateral
        DepositsAreNotCollateral,
        /// Insufficient shortfall to repay
        InsufficientShortfall,
        /// Liquidate value overflow
        LiquidateValueOverflow,
        /// Insufficient reserves
        InsufficientReserves,
        /// Invalid rate model params
        InvalidRateModelParam,
        /// Market not activated
        MarketNotActivated,
        /// Currency's oracle price not ready
        PriceOracleNotReady,
        /// Invalid asset id
        InvalidCurrencyId,
        /// Market does not exist
        MarketDoesNotExist,
        /// Market already exists
        MarketAlredyExists,
        /// New markets must have a pending state
        NewMarketMustHavePendingState,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub (crate) fn deposit_event)]
    #[pallet::metadata(T::AccountId = "AccountId", AssetIdOf<T> = "CurrencyId", BalanceOf<T> = "Balance")]
    pub enum Event<T: Config> {
        /// Enable collateral for certain asset
        /// [sender, asset_id]
        CollateralAssetAdded(T::AccountId, AssetIdOf<T>),
        /// Disable collateral for certain asset
        /// [sender, asset_id]
        CollateralAssetRemoved(T::AccountId, AssetIdOf<T>),
        /// Event emitted when assets are deposited
        /// [sender, asset_id, amount]
        Deposited(T::AccountId, AssetIdOf<T>, BalanceOf<T>),
        /// Event emitted when assets are redeemed
        /// [sender, asset_id, amount]
        Redeemed(T::AccountId, AssetIdOf<T>, BalanceOf<T>),
        /// Event emitted when cash is borrowed
        /// [sender, asset_id, amount]
        Borrowed(T::AccountId, AssetIdOf<T>, BalanceOf<T>),
        /// Event emitted when a borrow is repaid
        /// [sender, asset_id, amount]
        RepaidBorrow(T::AccountId, AssetIdOf<T>, BalanceOf<T>),
        /// Event emitted when a borrow is liquidated
        /// [liquidator, borrower, liquidate_token, collateral_token, repay_amount, collateral_amount]
        LiquidatedBorrow(
            T::AccountId,
            T::AccountId,
            AssetIdOf<T>,
            AssetIdOf<T>,
            BalanceOf<T>,
            BalanceOf<T>,
        ),
        /// Event emitted when the reserves are reduced
        /// [admin, asset_id, reduced_amount, total_reserves]
        ReservesReduced(T::AccountId, AssetIdOf<T>, BalanceOf<T>, BalanceOf<T>),
        /// Event emitted when the reserves are added
        /// [admin, asset_id, added_amount, total_reserves]
        ReservesAdded(T::AccountId, AssetIdOf<T>, BalanceOf<T>, BalanceOf<T>),
        /// New interest rate model is set
        /// [new_interest_rate_model]
        NewMarket(Market),
        /// Event emitted when a market is activated
        /// [admin, asset_id]
        ActivatedMarket(AssetIdOf<T>),
        /// Event emitted when a market is activated
        /// [admin, asset_id]
        UpdatedMarket(Market),
    }

    /// The timestamp of the last calculation of accrued interest
    #[pallet::storage]
    #[pallet::getter(fn last_block_timestamp)]
    pub type LastBlockTimestamp<T: Config> = StorageValue<_, Timestamp, ValueQuery>;

    /// Total number of collateral tokens in circulation
    /// CollateralType -> Balance
    #[pallet::storage]
    #[pallet::getter(fn total_supply)]
    pub type TotalSupply<T: Config> =
        StorageMap<_, Blake2_128Concat, AssetIdOf<T>, BalanceOf<T>, ValueQuery>;

    /// Total amount of outstanding borrows of the underlying in this market
    /// CurrencyId -> Balance
    #[pallet::storage]
    #[pallet::getter(fn total_borrows)]
    pub type TotalBorrows<T: Config> =
        StorageMap<_, Blake2_128Concat, AssetIdOf<T>, BalanceOf<T>, ValueQuery>;

    /// Total amount of reserves of the underlying held in this market
    /// CurrencyId -> Balance
    #[pallet::storage]
    #[pallet::getter(fn total_reserves)]
    pub type TotalReserves<T: Config> =
        StorageMap<_, Blake2_128Concat, AssetIdOf<T>, BalanceOf<T>, ValueQuery>;

    /// Mapping of account addresses to outstanding borrow balances
    /// CurrencyId -> Owner -> BorrowSnapshot
    #[pallet::storage]
    #[pallet::getter(fn account_borrows)]
    pub type AccountBorrows<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        AssetIdOf<T>,
        Blake2_128Concat,
        T::AccountId,
        BorrowSnapshot<BalanceOf<T>>,
        ValueQuery,
    >;

    /// Mapping of account addresses to deposit details
    /// CollateralType -> Owner -> Deposits
    #[pallet::storage]
    #[pallet::getter(fn account_deposits)]
    pub type AccountDeposits<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        AssetIdOf<T>,
        Blake2_128Concat,
        T::AccountId,
        Deposits<BalanceOf<T>>,
        ValueQuery,
    >;

    /// Mapping of account addresses to total deposit interest accrual
    /// CurrencyId -> Owner -> EarnedSnapshot
    #[pallet::storage]
    #[pallet::getter(fn account_earned)]
    pub type AccountEarned<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        AssetIdOf<T>,
        Blake2_128Concat,
        T::AccountId,
        EarnedSnapshot<BalanceOf<T>>,
        ValueQuery,
    >;

    /// Accumulator of the total earned interest rate since the opening of the market
    /// CurrencyId -> u128
    #[pallet::storage]
    #[pallet::getter(fn borrow_index)]
    pub type BorrowIndex<T: Config> =
        StorageMap<_, Blake2_128Concat, AssetIdOf<T>, Rate, ValueQuery>;

    /// The exchange rate from the underlying to the internal collateral
    #[pallet::storage]
    #[pallet::getter(fn exchange_rate)]
    pub type ExchangeRate<T: Config> =
        StorageMap<_, Blake2_128Concat, AssetIdOf<T>, Rate, ValueQuery>;

    /// Mapping of borrow rate to currency type
    #[pallet::storage]
    #[pallet::getter(fn borrow_rate)]
    pub type BorrowRate<T: Config> =
        StorageMap<_, Blake2_128Concat, AssetIdOf<T>, Rate, ValueQuery>;

    /// Mapping of supply rate to currency type
    #[pallet::storage]
    #[pallet::getter(fn supply_rate)]
    pub type SupplyRate<T: Config> =
        StorageMap<_, Blake2_128Concat, AssetIdOf<T>, Rate, ValueQuery>;

    /// Borrow utilization ratio
    #[pallet::storage]
    #[pallet::getter(fn utilization_ratio)]
    pub type UtilizationRatio<T: Config> =
        StorageMap<_, Blake2_128Concat, AssetIdOf<T>, Ratio, ValueQuery>;

    /// Mapping of asset id to its market
    #[pallet::storage]
    pub type Markets<T: Config> = StorageMap<_, Blake2_128Concat, AssetIdOf<T>, Market>;

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T>
    where
        BalanceOf<T>: FixedPointOperand,
        AssetIdOf<T>: AtLeast32BitUnsigned,
    {
        /// Called by substrate on block initialization which is fallible.
        /// When an error occurs, stop counting interest. When the error is resolved,
        /// the interest will be restored, because we use delta time to calculate the
        /// interest.
        fn on_initialize(block_number: T::BlockNumber) -> frame_support::weights::Weight {
            let last_block_timestamp = Self::last_block_timestamp();
            let now = T::UnixTime::now().as_secs();
            // For the initialization
            if last_block_timestamp.is_zero() {
                LastBlockTimestamp::<T>::put(now);
            }
            if now <= last_block_timestamp {
                return 0;
            }
            with_transaction(|| {
                match <Pallet<T>>::accrue_interest(now - last_block_timestamp) {
                    Ok(()) => {
                        LastBlockTimestamp::<T>::put(now);
                        TransactionOutcome::Commit(
                            T::WeightInfo::accrue_interest()
                                * Self::active_markets().count() as u64,
                        )
                    }
                    Err(err) => {
                        // This should never happen...
                        log::error!(
                            "Could not initialize block!!! {:#?} {:#?}",
                            block_number,
                            err
                        );
                        TransactionOutcome::Rollback(0)
                    }
                }
            })
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T>
    where
        BalanceOf<T>: FixedPointOperand,
        AssetIdOf<T>: AtLeast32BitUnsigned,
    {
        /// Activates a market. Returns `Err` if the market currency does not exist.
        ///
        /// If the market is already activated, does nothing.
        ///
        /// - `asset_id`: Market related currency
        #[pallet::weight(T::WeightInfo::active_market())]
        #[transactional]
        pub fn active_market(
            origin: OriginFor<T>,
            asset_id: AssetIdOf<T>,
        ) -> DispatchResultWithPostInfo {
            T::UpdateOrigin::ensure_origin(origin)?;
            Self::mutate_market(asset_id, |stored_market| {
                if let MarketState::Active = stored_market.state {
                    return;
                }
                stored_market.state = MarketState::Active
            })?;
            Self::deposit_event(Event::<T>::ActivatedMarket(asset_id));
            Ok(().into())
        }

        /// Stores a new market and its related currency. Returns `Err` if a currency
        /// is not attached to an existent market.
        ///
        /// All provided market states must be `Pending`, otherwise an error will be returned.
        ///
        /// If a currency is already attached to a market, then the market will be replaced
        /// by the new provided value.
        ///
        /// - `asset_id`: Market related currency
        /// - `market`: The market that is going to be stored
        #[pallet::weight(T::WeightInfo::add_market())]
        #[transactional]
        pub fn add_market(
            origin: OriginFor<T>,
            asset_id: AssetIdOf<T>,
            market: Market,
        ) -> DispatchResultWithPostInfo {
            T::UpdateOrigin::ensure_origin(origin)?;
            ensure!(
                !Markets::<T>::contains_key(asset_id),
                Error::<T>::MarketAlredyExists
            );
            ensure!(
                market.state == MarketState::Pending,
                Error::<T>::NewMarketMustHavePendingState
            );
            ensure!(
                market.rate_model.check_model(),
                Error::<T>::InvalidRateModelParam
            );
            Markets::<T>::insert(asset_id, market.clone());

            // Init the ExchangeRate and BorrowIndex for asset
            ExchangeRate::<T>::insert(asset_id, Rate::saturating_from_rational(2, 100));
            BorrowIndex::<T>::insert(asset_id, Rate::one());

            Self::deposit_event(Event::<T>::NewMarket(market));
            Ok(().into())
        }

        /// Updates a stored market. Returns `Err` if the market currency does not exist.
        ///
        /// Market state won't be modified, regardless of the provided value.
        ///
        /// - `asset_id`: Market related currency
        /// - `market`: The new market parameters
        #[pallet::weight(T::WeightInfo::update_market())]
        #[transactional]
        pub fn update_market(
            origin: OriginFor<T>,
            asset_id: AssetIdOf<T>,
            market: Market,
        ) -> DispatchResultWithPostInfo {
            T::UpdateOrigin::ensure_origin(origin)?;
            ensure!(
                market.rate_model.check_model(),
                Error::<T>::InvalidRateModelParam
            );
            Self::mutate_market(asset_id, |stored_market| {
                let Market {
                    collateral_factor,
                    reserve_factor,
                    close_factor,
                    liquidate_incentive,
                    rate_model,
                    state: _,
                } = stored_market;
                *collateral_factor = market.collateral_factor;
                *reserve_factor = market.reserve_factor;
                *close_factor = market.close_factor;
                *liquidate_incentive = market.liquidate_incentive;
                *rate_model = market.rate_model;
            })?;

            Self::deposit_event(Event::<T>::UpdatedMarket(market));
            Ok(().into())
        }

        /// Sender supplies assets into the market and receives internal supplies in exchange.
        ///
        /// - `asset_id`: the asset to be deposited.
        /// - `mint_amount`: the amount to be deposited.
        #[pallet::weight(T::WeightInfo::mint())]
        #[transactional]
        pub fn mint(
            origin: OriginFor<T>,
            asset_id: AssetIdOf<T>,
            mint_amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::ensure_currency(asset_id)?;

            T::Assets::transfer(asset_id, &who, &Self::account_id(), mint_amount, false)?;
            Self::update_earned_stored(&who, asset_id)?;
            let exchange_rate = Self::exchange_rate(asset_id);
            let voucher_amount = Self::calc_collateral_amount(mint_amount, exchange_rate)?;
            AccountDeposits::<T>::try_mutate(asset_id, &who, |deposits| -> DispatchResult {
                deposits.voucher_balance = deposits
                    .voucher_balance
                    .checked_add(&voucher_amount)
                    .ok_or(ArithmeticError::Overflow)?;
                Ok(())
            })?;
            TotalSupply::<T>::try_mutate(asset_id, |total_balance| -> DispatchResult {
                let new_balance = total_balance
                    .checked_add(&voucher_amount)
                    .ok_or(ArithmeticError::Overflow)?;
                *total_balance = new_balance;
                Ok(())
            })?;

            Self::deposit_event(Event::<T>::Deposited(who, asset_id, mint_amount));

            Ok(().into())
        }

        /// Sender redeems some of internal supplies in exchange for the underlying asset.
        ///
        /// - `asset_id`: the asset to be redeemed.
        /// - `redeem_amount`: the amount to be redeemed.
        #[pallet::weight(T::WeightInfo::redeem())]
        #[transactional]
        pub fn redeem(
            origin: OriginFor<T>,
            asset_id: AssetIdOf<T>,
            redeem_amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::ensure_currency(asset_id)?;

            let exchange_rate = Self::exchange_rate(asset_id);
            let voucher_amount = Self::calc_collateral_amount(redeem_amount, exchange_rate)?;
            Self::update_earned_stored(&who, asset_id)?;
            let redeem_amount = Self::redeem_internal(&who, asset_id, voucher_amount)?;

            Self::deposit_event(Event::<T>::Redeemed(who, asset_id, redeem_amount));

            Ok(().into())
        }

        /// Sender redeems all of internal supplies in exchange for the underlying asset.
        ///
        /// - `asset_id`: the asset to be redeemed.
        #[pallet::weight(T::WeightInfo::redeem_all())]
        #[transactional]
        pub fn redeem_all(
            origin: OriginFor<T>,
            asset_id: AssetIdOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::ensure_currency(asset_id)?;

            Self::update_earned_stored(&who, asset_id)?;
            let deposits = AccountDeposits::<T>::get(asset_id, &who);
            let redeem_amount = Self::redeem_internal(&who, asset_id, deposits.voucher_balance)?;

            Self::deposit_event(Event::<T>::Redeemed(who, asset_id, redeem_amount));

            Ok(().into())
        }

        /// Sender borrows assets from the protocol to their own address.
        ///
        /// - `asset_id`: the asset to be borrowed.
        /// - `borrow_amount`: the amount to be borrowed.
        #[pallet::weight(T::WeightInfo::borrow())]
        #[transactional]
        pub fn borrow(
            origin: OriginFor<T>,
            asset_id: AssetIdOf<T>,
            borrow_amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::ensure_currency(asset_id)?;

            Self::borrow_allowed(asset_id, &who, borrow_amount)?;
            let account_borrows = Self::current_borrow_balance(&who, asset_id)?;
            let account_borrows_new = account_borrows
                .checked_add(&borrow_amount)
                .ok_or(ArithmeticError::Overflow)?;
            let total_borrows = Self::total_borrows(asset_id);
            let total_borrows_new = total_borrows
                .checked_add(&borrow_amount)
                .ok_or(ArithmeticError::Overflow)?;
            AccountBorrows::<T>::insert(
                asset_id,
                &who,
                BorrowSnapshot {
                    principal: account_borrows_new,
                    borrow_index: Self::borrow_index(asset_id),
                },
            );
            TotalBorrows::<T>::insert(asset_id, total_borrows_new);
            T::Assets::transfer(asset_id, &Self::account_id(), &who, borrow_amount, false)?;

            Self::deposit_event(Event::<T>::Borrowed(who, asset_id, borrow_amount));

            Ok(().into())
        }

        /// Sender repays some of their debts.
        ///
        /// - `asset_id`: the asset to be repaid.
        /// - `repay_amount`: the amount to be repaid.
        #[pallet::weight(T::WeightInfo::repay_borrow())]
        #[transactional]
        pub fn repay_borrow(
            origin: OriginFor<T>,
            asset_id: AssetIdOf<T>,
            repay_amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::ensure_currency(asset_id)?;

            let account_borrows = Self::current_borrow_balance(&who, asset_id)?;
            Self::repay_borrow_internal(&who, asset_id, account_borrows, repay_amount)?;

            Self::deposit_event(Event::<T>::RepaidBorrow(who, asset_id, repay_amount));

            Ok(().into())
        }

        /// Sender repays all of their debts.
        ///
        /// - `asset_id`: the asset to be repaid.
        #[pallet::weight(T::WeightInfo::repay_borrow_all())]
        #[transactional]
        pub fn repay_borrow_all(
            origin: OriginFor<T>,
            asset_id: AssetIdOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::ensure_currency(asset_id)?;

            let account_borrows = Self::current_borrow_balance(&who, asset_id)?;
            Self::repay_borrow_internal(&who, asset_id, account_borrows, account_borrows)?;

            Self::deposit_event(Event::<T>::RepaidBorrow(who, asset_id, account_borrows));

            Ok(().into())
        }

        /// Set the collateral asset.
        ///
        /// - `asset_id`: the asset to be set.
        /// - `enable`: turn on/off the collateral option.
        #[pallet::weight(T::WeightInfo::collateral_asset())]
        #[transactional]
        pub fn collateral_asset(
            origin: OriginFor<T>,
            asset_id: AssetIdOf<T>,
            enable: bool,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::ensure_currency(asset_id)?;
            ensure!(
                AccountDeposits::<T>::contains_key(asset_id, &who),
                Error::<T>::NoDeposit
            );
            let mut deposits = Self::account_deposits(asset_id, &who);
            if deposits.is_collateral == enable {
                return Err(Error::<T>::DuplicateOperation.into());
            }
            // turn on the collateral button
            if enable {
                deposits.is_collateral = true;
                AccountDeposits::<T>::insert(asset_id, &who, deposits);
                Self::deposit_event(Event::<T>::CollateralAssetAdded(who, asset_id));

                return Ok(().into());
            }
            // turn off the collateral button after checking the liquidity
            let total_collateral_value = Self::total_collateral_value(&who)?;
            let market = Self::market(asset_id)?;
            let collateral_asset_value = Self::collateral_asset_value(&who, asset_id, &market)?;
            let total_borrowed_value = Self::total_borrowed_value(&who)?;
            if total_collateral_value
                < total_borrowed_value
                    .checked_add(&collateral_asset_value)
                    .ok_or(ArithmeticError::Overflow)?
            {
                return Err(Error::<T>::InsufficientLiquidity.into());
            }

            deposits.is_collateral = false;
            AccountDeposits::<T>::insert(asset_id, &who, deposits);
            Self::deposit_event(Event::<T>::CollateralAssetRemoved(who, asset_id));

            Ok(().into())
        }

        /// The sender liquidates the borrower's collateral.
        ///
        /// - `borrower`: the borrower to be liquidated.
        /// - `liquidate_token`: the assert to be liquidated.
        /// - `repay_amount`: the amount to be repaid borrow.
        /// - `collateral_token`: The collateral to seize from the borrower.
        #[pallet::weight(T::WeightInfo::liquidate_borrow())]
        #[transactional]
        pub fn liquidate_borrow(
            origin: OriginFor<T>,
            borrower: T::AccountId,
            liquidate_token: AssetIdOf<T>,
            repay_amount: BalanceOf<T>,
            collateral_token: AssetIdOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::liquidate_borrow_internal(
                who,
                borrower,
                liquidate_token,
                repay_amount,
                collateral_token,
            )?;
            Ok(().into())
        }

        /// Add reserves by transferring from payer.
        ///
        /// May only be called from `T::ReserveOrigin`.
        ///
        /// - `payer`: the payer account.
        /// - `asset_id`: the assets to be added.
        /// - `add_amount`: the amount to be added.
        #[pallet::weight(T::WeightInfo::add_reserves())]
        #[transactional]
        pub fn add_reserves(
            origin: OriginFor<T>,
            payer: <T::Lookup as StaticLookup>::Source,
            asset_id: AssetIdOf<T>,
            add_amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            T::ReserveOrigin::ensure_origin(origin)?;
            let payer = T::Lookup::lookup(payer)?;
            Self::ensure_currency(asset_id)?;

            T::Assets::transfer(asset_id, &payer, &Self::account_id(), add_amount, false)?;
            let total_reserves = Self::total_reserves(asset_id);
            let total_reserves_new = total_reserves
                .checked_add(&add_amount)
                .ok_or(ArithmeticError::Overflow)?;
            TotalReserves::<T>::insert(asset_id, total_reserves_new);

            Self::deposit_event(Event::<T>::ReservesAdded(
                payer,
                asset_id,
                add_amount,
                total_reserves_new,
            ));

            Ok(().into())
        }

        /// Reduces reserves by transferring to receiver.
        ///
        /// May only be called from `T::ReserveOrigin`.
        ///
        /// - `receiver`: the receiver account.
        /// - `asset_id`: the assets to be reduced.
        /// - `reduce_amount`: the amount to be reduced.
        #[pallet::weight(T::WeightInfo::reduce_reserves())]
        #[transactional]
        pub fn reduce_reserves(
            origin: OriginFor<T>,
            receiver: <T::Lookup as StaticLookup>::Source,
            asset_id: AssetIdOf<T>,
            reduce_amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            T::ReserveOrigin::ensure_origin(origin)?;
            let receiver = T::Lookup::lookup(receiver)?;
            Self::ensure_currency(asset_id)?;

            let total_reserves = Self::total_reserves(asset_id);
            if reduce_amount > total_reserves {
                return Err(Error::<T>::InsufficientReserves.into());
            }
            let total_reserves_new = total_reserves
                .checked_sub(&reduce_amount)
                .ok_or(ArithmeticError::Underflow)?;
            TotalReserves::<T>::insert(asset_id, total_reserves_new);
            T::Assets::transfer(
                asset_id,
                &Self::account_id(),
                &receiver,
                reduce_amount,
                false,
            )?;

            Self::deposit_event(Event::<T>::ReservesReduced(
                receiver,
                asset_id,
                reduce_amount,
                total_reserves_new,
            ));

            Ok(().into())
        }
    }
}

impl<T: Config> Pallet<T>
where
    BalanceOf<T>: FixedPointOperand,
    AssetIdOf<T>: AtLeast32BitUnsigned,
{
    pub fn account_id() -> T::AccountId {
        T::PalletId::get().into_account()
    }

    pub fn get_account_liquidity(
        account: &T::AccountId,
    ) -> Result<(Liquidity, Shortfall), DispatchError> {
        let total_borrow_value = Self::total_borrowed_value(account)?;
        let total_collateral_value = Self::total_collateral_value(account)?;
        if total_collateral_value > total_borrow_value {
            Ok((
                total_collateral_value - total_borrow_value,
                FixedU128::zero(),
            ))
        } else {
            Ok((
                FixedU128::zero(),
                total_borrow_value - total_collateral_value,
            ))
        }
    }

    fn total_borrowed_value(borrower: &T::AccountId) -> Result<FixedU128, DispatchError> {
        let mut total_borrow_value: FixedU128 = FixedU128::zero();

        for (asset_id, _) in Self::active_markets() {
            let currency_borrow_amount = Self::current_borrow_balance(borrower, asset_id)?;
            if currency_borrow_amount.is_zero() {
                continue;
            }
            let borrow_currency_price = Self::get_price(asset_id)?;
            total_borrow_value = borrow_currency_price
                .checked_mul(&FixedU128::from_inner(
                    currency_borrow_amount.saturated_into(),
                ))
                .and_then(|r| r.checked_add(&total_borrow_value))
                .ok_or(ArithmeticError::Overflow)?;
        }

        Ok(total_borrow_value)
    }

    fn collateral_asset_value(
        borrower: &T::AccountId,
        asset_id: AssetIdOf<T>,
        market: &Market,
    ) -> Result<FixedU128, DispatchError> {
        if !AccountDeposits::<T>::contains_key(asset_id, borrower) {
            return Ok(FixedU128::zero());
        }
        let deposits = Self::account_deposits(asset_id, borrower);
        if !deposits.is_collateral {
            return Ok(FixedU128::zero());
        }
        if deposits.voucher_balance.is_zero() {
            return Ok(FixedU128::zero());
        }
        let exchange_rate = ExchangeRate::<T>::get(asset_id);
        let currency_price = Self::get_price(asset_id)?;
        let collateral_amount = exchange_rate
            .checked_mul_int(market.collateral_factor.mul_floor(deposits.voucher_balance))
            .ok_or(ArithmeticError::Overflow)?;

        Ok(currency_price
            .checked_mul(&FixedU128::from_inner(collateral_amount.saturated_into()))
            .ok_or(ArithmeticError::Overflow)?)
    }

    fn total_collateral_value(borrower: &T::AccountId) -> Result<FixedU128, DispatchError> {
        let mut total_asset_value: FixedU128 = FixedU128::zero();
        for (asset_id, market) in Self::active_markets() {
            total_asset_value = total_asset_value
                .checked_add(&Self::collateral_asset_value(borrower, asset_id, &market)?)
                .ok_or(ArithmeticError::Overflow)?;
        }

        Ok(total_asset_value)
    }

    /// Checks if the redeemer should be allowed to redeem tokens in given market
    fn redeem_allowed(
        asset_id: AssetIdOf<T>,
        redeemer: &T::AccountId,
        voucher_amount: BalanceOf<T>,
        market: &Market,
    ) -> DispatchResult {
        let deposit = Self::account_deposits(asset_id, redeemer);
        if deposit.voucher_balance < voucher_amount {
            return Err(Error::<T>::InsufficientDeposit.into());
        }
        if !deposit.is_collateral {
            return Ok(());
        }
        let price = Self::get_price(asset_id)?;
        let exchange_rate = Self::exchange_rate(asset_id);
        let redeem_amount = Self::calc_underlying_amount(voucher_amount, exchange_rate)?;
        let redeem_effects_value = price
            .checked_mul(&FixedU128::from_inner(
                market
                    .collateral_factor
                    .mul_ceil(redeem_amount.saturated_into()),
            ))
            .ok_or(ArithmeticError::Overflow)?;

        let (liquidity, _) = Self::get_account_liquidity(redeemer)?;
        if liquidity < redeem_effects_value {
            return Err(Error::<T>::InsufficientLiquidity.into());
        }

        Ok(())
    }

    pub fn redeem_internal(
        who: &T::AccountId,
        asset_id: AssetIdOf<T>,
        voucher_amount: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        let market = Self::market(asset_id)?;
        Self::redeem_allowed(asset_id, who, voucher_amount, &market)?;
        let exchange_rate = Self::exchange_rate(asset_id);
        let redeem_amount = Self::calc_underlying_amount(voucher_amount, exchange_rate)?;
        AccountDeposits::<T>::try_mutate_exists(asset_id, who, |deposits| -> DispatchResult {
            let mut d = deposits.unwrap_or_default();
            d.voucher_balance = d
                .voucher_balance
                .checked_sub(&voucher_amount)
                .ok_or(ArithmeticError::Underflow)?;
            if d.voucher_balance.is_zero() {
                // remove deposits storage if zero balance
                *deposits = None;
            } else {
                *deposits = Some(d);
            }
            Ok(())
        })?;
        TotalSupply::<T>::try_mutate(asset_id, |total_balance| -> DispatchResult {
            let new_balance = total_balance
                .checked_sub(&voucher_amount)
                .ok_or(ArithmeticError::Underflow)?;
            *total_balance = new_balance;
            Ok(())
        })?;
        T::Assets::transfer(asset_id, &Self::account_id(), who, redeem_amount, false)?;

        Ok(redeem_amount)
    }

    /// Borrower shouldn't borrow more than his total collateral value
    fn borrow_allowed(
        asset_id: AssetIdOf<T>,
        borrower: &T::AccountId,
        borrow_amount: BalanceOf<T>,
    ) -> DispatchResult {
        let price = Self::get_price(asset_id)?;
        let borrow_value = price
            .checked_mul(&FixedU128::from_inner(borrow_amount.saturated_into()))
            .ok_or(ArithmeticError::Overflow)?;

        let (liquidity, _) = Self::get_account_liquidity(borrower)?;
        if liquidity < borrow_value {
            return Err(Error::<T>::InsufficientLiquidity.into());
        }

        Ok(())
    }

    fn repay_borrow_internal(
        borrower: &T::AccountId,
        asset_id: AssetIdOf<T>,
        account_borrows: BalanceOf<T>,
        repay_amount: BalanceOf<T>,
    ) -> DispatchResult {
        if account_borrows < repay_amount {
            return Err(Error::<T>::TooMuchRepay.into());
        }

        T::Assets::transfer(asset_id, borrower, &Self::account_id(), repay_amount, false)?;

        let account_borrows_new = account_borrows
            .checked_sub(&repay_amount)
            .ok_or(ArithmeticError::Underflow)?;
        let total_borrows = Self::total_borrows(asset_id);
        // NOTE : total_borrows use a different way to calculate interest
        // so when user repays all borrows, total_borrows can be smaller than account_borrows
        // which will cause it to fail with `ArithmeticError::Underflow`
        //
        // Change it back to checked_sub will cause Underflow
        let total_borrows_new = total_borrows.saturating_sub(repay_amount);

        AccountBorrows::<T>::insert(
            asset_id,
            borrower,
            BorrowSnapshot {
                principal: account_borrows_new,
                borrow_index: Self::borrow_index(asset_id),
            },
        );

        TotalBorrows::<T>::insert(asset_id, total_borrows_new);

        Ok(())
    }

    // Calculates and returns the most recent amount of borrowed balance of `currency_id`
    // for `who`.
    pub fn current_borrow_balance(
        who: &T::AccountId,
        asset_id: AssetIdOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        let snapshot: BorrowSnapshot<BalanceOf<T>> = Self::account_borrows(asset_id, who);
        Self::current_balance_from_snapshot(asset_id, snapshot)
    }

    /// Same as `current_borrow_balance` but takes a given `snapshot` instead of fetching
    /// the storage
    pub fn current_balance_from_snapshot(
        asset_id: AssetIdOf<T>,
        snapshot: BorrowSnapshot<BalanceOf<T>>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        if snapshot.principal.is_zero() || snapshot.borrow_index.is_zero() {
            return Ok(Zero::zero());
        }
        // Calculate new borrow balance using the interest index:
        // recent_borrow_balance = snapshot.principal * borrow_index / snapshot.borrow_index
        let recent_borrow_balance = Self::borrow_index(asset_id)
            .checked_div(&snapshot.borrow_index)
            .and_then(|r| r.checked_mul_int(snapshot.principal))
            .ok_or(ArithmeticError::Overflow)?;

        Ok(recent_borrow_balance)
    }

    fn update_earned_stored(who: &T::AccountId, asset_id: AssetIdOf<T>) -> DispatchResult {
        let deposits = AccountDeposits::<T>::get(asset_id, who);
        let exchange_rate = ExchangeRate::<T>::get(asset_id);
        let account_earned = AccountEarned::<T>::get(asset_id, who);
        let total_earned_prior_new = exchange_rate
            .checked_sub(&account_earned.exchange_rate_prior)
            .and_then(|r| r.checked_mul_int(deposits.voucher_balance))
            .and_then(|r| r.checked_add(&account_earned.total_earned_prior))
            .ok_or(ArithmeticError::Overflow)?;

        AccountEarned::<T>::insert(
            asset_id,
            who,
            EarnedSnapshot {
                exchange_rate_prior: exchange_rate,
                total_earned_prior: total_earned_prior_new,
            },
        );

        Ok(())
    }

    /// Checks if the liquidation should be allowed to occur
    fn liquidate_borrow_allowed(
        borrower: &T::AccountId,
        liquidate_asset_id: AssetIdOf<T>,
        repay_amount: BalanceOf<T>,
        market: &Market,
    ) -> DispatchResult {
        let (_, shortfall) = Self::get_account_liquidity(borrower)?;
        if shortfall.is_zero() {
            return Err(Error::<T>::InsufficientShortfall.into());
        }

        // The liquidator may not repay more than 50%(close_factor) of the borrower's borrow balance.
        let account_borrows = Self::current_borrow_balance(borrower, liquidate_asset_id)?;
        if market.close_factor.mul_ceil(account_borrows) < repay_amount {
            return Err(Error::<T>::TooMuchRepay.into());
        }

        Ok(())
    }

    /// Note:
    /// - liquidate_token is borrower's debt asset.
    /// - collateral_token is borrower's collateral asset.
    /// - repay_amount is amount of liquidate_token
    ///
    /// The liquidator will repay a certain amount of liquidate_token from own
    /// account for borrower. Then the protocol will reduce borrower's debt
    /// and liquidator will receive collateral_token(as voucher amount) from
    /// borrower.
    pub fn liquidate_borrow_internal(
        liquidator: T::AccountId,
        borrower: T::AccountId,
        liquidate_asset_id: AssetIdOf<T>,
        repay_amount: BalanceOf<T>,
        collateral_asset_id: AssetIdOf<T>,
    ) -> DispatchResult {
        Self::ensure_currency(liquidate_asset_id)?;
        Self::ensure_currency(collateral_asset_id)?;

        let market = Self::market(liquidate_asset_id)?;

        if borrower == liquidator {
            return Err(Error::<T>::LiquidatorIsBorrower.into());
        }
        Self::liquidate_borrow_allowed(&borrower, liquidate_asset_id, repay_amount, &market)?;

        let deposits = AccountDeposits::<T>::get(collateral_asset_id, &borrower);
        if !deposits.is_collateral {
            return Err(Error::<T>::DepositsAreNotCollateral.into());
        }
        let exchange_rate = Self::exchange_rate(collateral_asset_id);
        let borrower_deposit_amount = exchange_rate
            .checked_mul_int(deposits.voucher_balance)
            .ok_or(ArithmeticError::Overflow)?;

        // Calculate the collateral value
        let collateral_token_price = Self::get_price(collateral_asset_id)?;
        let collateral_value = collateral_token_price
            .checked_mul(&FixedU128::from_inner(
                borrower_deposit_amount.saturated_into(),
            ))
            .ok_or(ArithmeticError::Overflow)?;

        // The incentive for liquidator and punishment for the borrower
        let liquidate_value = Self::get_price(liquidate_asset_id)?
            .checked_mul(&FixedU128::from_inner(repay_amount.saturated_into()))
            .and_then(|a| a.checked_mul(&market.liquidate_incentive))
            .ok_or(Error::<T>::LiquidateValueOverflow)?;

        if collateral_value < liquidate_value {
            return Err(Error::<T>::InsufficientCollateral.into());
        }

        // Calculate the collateral will get
        //
        // amount: 1 Unit = 10^12 pico
        // price is for 1 pico: 1$ = FixedU128::saturating_from_rational(1, 10^12)
        // if price is N($) and amount is M(Unit):
        // liquidate_value = price * amount = (N / 10^12) * (M * 10^12) = N * M
        // if liquidate_value >= 340282366920938463463.374607431768211455,
        // FixedU128::saturating_from_integer(liquidate_value) will overflow, so we use from_inner
        // instead of saturating_from_integer, and after calculation use into_inner to get final value.
        let real_collateral_underlying_amount = liquidate_value
            .checked_div(&collateral_token_price)
            .ok_or(ArithmeticError::Underflow)?
            .into_inner();

        //inside transfer token
        Self::liquidate_repay_borrow_internal(
            &liquidator,
            &borrower,
            liquidate_asset_id,
            collateral_asset_id,
            repay_amount,
            real_collateral_underlying_amount.saturated_into(),
        )?;

        Ok(())
    }

    fn liquidate_repay_borrow_internal(
        liquidator: &T::AccountId,
        borrower: &T::AccountId,
        liquidate_asset_id: AssetIdOf<T>,
        collateral_asset_id: AssetIdOf<T>,
        repay_amount: BalanceOf<T>,
        collateral_underlying_amount: BalanceOf<T>,
    ) -> DispatchResult {
        // 1.liquidator repay borrower's debt,
        // transfer from liquidator to module account
        T::Assets::transfer(
            liquidate_asset_id,
            liquidator,
            &Self::account_id(),
            repay_amount,
            false,
        )?;

        // 2.the system reduce borrower's debt
        let account_borrows = Self::current_borrow_balance(borrower, liquidate_asset_id)?;
        let account_borrows_new = account_borrows
            .checked_sub(&repay_amount)
            .ok_or(ArithmeticError::Underflow)?;
        let total_borrows = Self::total_borrows(liquidate_asset_id);
        let total_borrows_new = total_borrows
            .checked_sub(&repay_amount)
            .ok_or(ArithmeticError::Underflow)?;
        AccountBorrows::<T>::insert(
            liquidate_asset_id,
            borrower,
            BorrowSnapshot {
                principal: account_borrows_new,
                borrow_index: Self::borrow_index(liquidate_asset_id),
            },
        );
        TotalBorrows::<T>::insert(liquidate_asset_id, total_borrows_new);

        // 3.the liquidator will receive voucher token from borrower
        let exchange_rate = Self::exchange_rate(collateral_asset_id);
        let collateral_amount =
            Self::calc_collateral_amount(collateral_underlying_amount, exchange_rate)?;
        AccountDeposits::<T>::try_mutate(
            collateral_asset_id,
            borrower,
            |deposits| -> DispatchResult {
                deposits.voucher_balance = deposits
                    .voucher_balance
                    .checked_sub(&collateral_amount)
                    .ok_or(ArithmeticError::Underflow)?;
                Ok(())
            },
        )?;
        // increase liquidator's voucher_balance
        AccountDeposits::<T>::try_mutate(
            collateral_asset_id,
            liquidator,
            |deposits| -> DispatchResult {
                deposits.voucher_balance = deposits
                    .voucher_balance
                    .checked_add(&collateral_amount)
                    .ok_or(ArithmeticError::Overflow)?;
                Ok(())
            },
        )?;

        Self::deposit_event(Event::<T>::LiquidatedBorrow(
            liquidator.clone(),
            borrower.clone(),
            liquidate_asset_id,
            collateral_asset_id,
            repay_amount,
            collateral_underlying_amount,
        ));

        Ok(())
    }

    // Ensures a given `asset_id` exists on the `Currencies` storage.
    fn ensure_currency(asset_id: AssetIdOf<T>) -> DispatchResult {
        if Self::active_markets().any(|(id, _)| id == asset_id) {
            Ok(())
        } else {
            Err(<Error<T>>::MarketNotActivated.into())
        }
    }

    pub fn calc_underlying_amount(
        voucher_amount: BalanceOf<T>,
        exchange_rate: Rate,
    ) -> Result<BalanceOf<T>, DispatchError> {
        Ok(exchange_rate
            .checked_mul_int(voucher_amount)
            .ok_or(ArithmeticError::Overflow)?)
    }

    pub fn calc_collateral_amount(
        underlying_amount: BalanceOf<T>,
        exchange_rate: Rate,
    ) -> Result<BalanceOf<T>, DispatchError> {
        Ok(FixedU128::from_inner(underlying_amount.saturated_into())
            .checked_div(&exchange_rate)
            .map(|r| r.into_inner())
            .ok_or(ArithmeticError::Underflow)?
            .saturated_into())
    }

    pub fn get_price(asset_id: AssetIdOf<T>) -> Result<Price, DispatchError> {
        let id: CurrencyId = asset_id
            .try_into()
            .map_err(|_| Error::<T>::InvalidCurrencyId)?;
        let (price, _) = T::PriceFeeder::get_price(&id).ok_or(Error::<T>::PriceOracleNotReady)?;
        if price.is_zero() {
            return Err(Error::<T>::PriceOracleNotReady.into());
        }

        Ok(price)
    }

    // Returns a stored Market.
    //
    // Returns `Err` if market does not exist.
    pub fn market(asset_id: AssetIdOf<T>) -> Result<Market, DispatchError> {
        Markets::<T>::try_get(asset_id).map_err(|_err| Error::<T>::MarketDoesNotExist.into())
    }

    // Mutates a stored Market.
    //
    // Returns `Err` if market does not exist.
    pub(crate) fn mutate_market<F>(asset_id: AssetIdOf<T>, cb: F) -> Result<(), DispatchError>
    where
        F: FnOnce(&mut Market),
    {
        Markets::<T>::try_mutate(asset_id, |opt| {
            if let Some(market) = opt {
                cb(market);
                return Ok(());
            }
            Err(Error::<T>::MarketDoesNotExist.into())
        })
    }

    // All markets that are `MarketStatus::Active`.
    fn active_markets() -> impl Iterator<Item = (AssetIdOf<T>, Market)> {
        Markets::<T>::iter().filter(|(_, market)| market.state == MarketState::Active)
    }
}
