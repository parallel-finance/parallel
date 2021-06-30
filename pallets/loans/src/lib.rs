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
#![allow(clippy::unused_unit)]
#![allow(clippy::collapsible_if)]

pub use crate::rate_model::*;
use frame_support::{
    log,
    pallet_prelude::*,
    storage::{with_transaction, TransactionOutcome},
    traits::UnixTime,
    transactional, PalletId,
};
use frame_system::pallet_prelude::*;
use orml_traits::{MultiCurrency, MultiCurrencyExtended};
pub use pallet::*;
use primitives::{
    Amount, Balance, CurrencyId, Multiplier, Price, PriceFeeder, Rate, Ratio, Timestamp,
};
use sp_runtime::ArithmeticError;
use sp_runtime::{
    traits::{
        AccountIdConversion, CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, StaticLookup, Zero,
    },
    FixedPointNumber, FixedU128,
};
use sp_std::result::Result;
use sp_std::vec::Vec;
pub use weights::WeightInfo;

mod mock;
mod rate_model;
mod tests;
pub mod weights;

/// Container for borrow balance information
#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, Default)]
pub struct BorrowSnapshot {
    /// Principal Total balance (with accrued interest), after applying the most recent balance-changing action
    pub principal: Balance,
    /// InterestIndex Global borrowIndex as of the most recent balance-changing action
    pub borrow_index: Rate,
}

/// Container for earned amount information
#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, Default)]
pub struct EarnedSnapshot {
    /// Total deposit interest, after applying the most recent balance-changing action
    pub total_earned_prior: Balance,
    /// Exchange rate, after applying the most recent balance-changing action
    pub exchange_rate_prior: Rate,
}

/// Deposit information
#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, Default)]
pub struct Deposits {
    /// The voucher amount of the deposit
    pub voucher_balance: Balance,
    /// Can this deposit be used as collateral
    pub is_collateral: bool,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// Currency type for deposit/withdraw collateral assets to/from loans
        /// module
        type Currency: MultiCurrencyExtended<
            Self::AccountId,
            CurrencyId = CurrencyId,
            Balance = Balance,
            Amount = Amount,
        >;

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
        /// There is no borrow balance
        NoBorrowBalance,
        /// Liquidate value overflow
        LiquidateValueOverflow,
        /// Insufficient reserves
        InsufficientReserves,
        /// Invalid rate model params
        InvalidRateModelParam,
        /// Currency not enabled
        CurrencyNotEnabled,
        /// Currency's oracle price not ready
        PriceOracleNotReady,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub (crate) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Enable collateral for certain asset
        /// [sender, currency_id]
        CollateralAssetAdded(T::AccountId, CurrencyId),
        /// Disable collateral for certain asset
        /// [sender, currency_id]
        CollateralAssetRemoved(T::AccountId, CurrencyId),
        /// Event emitted when assets are deposited
        /// [sender, currency_id, amount]
        Deposited(T::AccountId, CurrencyId, Balance),
        /// Event emitted when assets are redeemed
        /// [sender, currency_id, amount]
        Redeemed(T::AccountId, CurrencyId, Balance),
        /// Event emitted when cash is borrowed
        /// [sender, currency_id, amount]
        Borrowed(T::AccountId, CurrencyId, Balance),
        /// Event emitted when a borrow is repaid
        /// [sender, currency_id, amount]
        RepaidBorrow(T::AccountId, CurrencyId, Balance),
        /// Event emitted when a borrow is liquidated
        /// [liquidator, borrower, liquidate_token, collateral_token, repay_amount, collateral_amount]
        LiquidatedBorrow(
            T::AccountId,
            T::AccountId,
            CurrencyId,
            CurrencyId,
            Balance,
            Balance,
        ),
        /// New interest rate model is set
        /// [new_interest_rate_model]
        NewInterestRateModel(InterestRateModel),
        /// Event emitted when the reserves are reduced
        /// [admin, currency_id, reduced_amount, total_reserves]
        ReservesReduced(T::AccountId, CurrencyId, Balance, Balance),
        /// Event emitted when the reserves are added
        /// [admin, currency_id, added_amount, total_reserves]
        ReservesAdded(T::AccountId, CurrencyId, Balance, Balance),
    }

    /// The timestamp of the previous block or defaults to timestamp at genesis.
    #[pallet::storage]
    #[pallet::getter(fn last_block_timestamp)]
    pub type LastBlockTimestamp<T: Config> = StorageValue<_, Timestamp, ValueQuery>;

    /// Total number of collateral tokens in circulation
    /// CollateralType -> Balance
    #[pallet::storage]
    #[pallet::getter(fn total_supply)]
    pub type TotalSupply<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, Balance, ValueQuery>;

    /// Total amount of outstanding borrows of the underlying in this market
    /// CurrencyType -> Balance
    #[pallet::storage]
    #[pallet::getter(fn total_borrows)]
    pub type TotalBorrows<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, Balance, ValueQuery>;

    /// Total amount of reserves of the underlying held in this market
    /// CurrencyType -> Balance
    #[pallet::storage]
    #[pallet::getter(fn total_reserves)]
    pub type TotalReserves<T: Config> =
        StorageMap<_, Twox64Concat, CurrencyId, Balance, ValueQuery>;

    /// Mapping of account addresses to outstanding borrow balances
    /// CurrencyType -> Owner -> BorrowSnapshot
    #[pallet::storage]
    #[pallet::getter(fn account_borrows)]
    pub type AccountBorrows<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        CurrencyId,
        Blake2_128Concat,
        T::AccountId,
        BorrowSnapshot,
        ValueQuery,
    >;

    /// Mapping of account addresses to deposit details
    /// CollateralType -> Owner -> Deposits
    #[pallet::storage]
    #[pallet::getter(fn account_deposits)]
    pub type AccountDeposits<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        CurrencyId,
        Blake2_128Concat,
        T::AccountId,
        Deposits,
        ValueQuery,
    >;

    /// Mapping of account addresses to total deposit interest accrual
    /// CurrencyType -> Owner -> EarnedSnapshot
    #[pallet::storage]
    #[pallet::getter(fn account_earned)]
    pub type AccountEarned<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        CurrencyId,
        Blake2_128Concat,
        T::AccountId,
        EarnedSnapshot,
        ValueQuery,
    >;

    /// Accumulator of the total earned interest rate since the opening of the market
    /// CurrencyType -> u128
    #[pallet::storage]
    #[pallet::getter(fn borrow_index)]
    pub type BorrowIndex<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, Rate, ValueQuery>;

    /// The currency types support on lending markets
    #[pallet::storage]
    #[pallet::getter(fn currencies)]
    pub type Currencies<T: Config> = StorageValue<_, Vec<CurrencyId>, ValueQuery>;

    /// The exchange rate from the underlying to the internal collateral
    #[pallet::storage]
    #[pallet::getter(fn exchange_rate)]
    pub type ExchangeRate<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, Rate, ValueQuery>;

    /// The utilization point at which the jump multiplier is applied
    #[pallet::storage]
    #[pallet::getter(fn currency_interest_model)]
    pub type CurrencyInterestModel<T: Config> =
        StorageMap<_, Twox64Concat, CurrencyId, InterestRateModel, ValueQuery>;

    /// Mapping of borrow rate to currency type
    #[pallet::storage]
    #[pallet::getter(fn borrow_rate)]
    pub type BorrowRate<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, Rate, ValueQuery>;

    /// Mapping of supply rate to currency type
    #[pallet::storage]
    #[pallet::getter(fn supply_rate)]
    pub type SupplyRate<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, Rate, ValueQuery>;

    /// Borrow utilization ratio
    #[pallet::storage]
    #[pallet::getter(fn utilization_ratio)]
    pub type UtilizationRatio<T: Config> =
        StorageMap<_, Twox64Concat, CurrencyId, Ratio, ValueQuery>;

    /// The collateral utilization ratio
    #[pallet::storage]
    #[pallet::getter(fn collateral_factor)]
    pub type CollateralFactor<T: Config> =
        StorageMap<_, Twox64Concat, CurrencyId, Ratio, ValueQuery>;

    /// Fraction of interest currently set aside for reserves
    #[pallet::storage]
    #[pallet::getter(fn reserve_factor)]
    pub type ReserveFactor<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, Ratio, ValueQuery>;

    /// Liquidation incentive ratio
    #[pallet::storage]
    #[pallet::getter(fn liquidation_incentive)]
    pub type LiquidationIncentive<T: Config> =
        StorageMap<_, Twox64Concat, CurrencyId, Rate, ValueQuery>;

    /// The percent, ranging from 0% to 100%, of a liquidatable account's
    /// borrow that can be repaid in a single liquidate transaction.
    #[pallet::storage]
    #[pallet::getter(fn close_factor)]
    pub type CloseFactor<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, Ratio, ValueQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig {
        pub currencies: Vec<CurrencyId>,
        pub borrow_index: Rate,
        pub exchange_rate: Rate,
        pub base_rate: Rate,
        pub jump_rate: Multiplier,
        pub full_rate: Multiplier,
        pub jump_utilization: Ratio,
        pub collateral_factor: Vec<(CurrencyId, Ratio)>,
        pub liquidation_incentive: Vec<(CurrencyId, Rate)>,
        pub close_factor: Vec<(CurrencyId, Ratio)>,
        pub reserve_factor: Vec<(CurrencyId, Ratio)>,
        pub last_block_timestamp: Timestamp,
    }

    #[cfg(feature = "std")]
    impl Default for GenesisConfig {
        fn default() -> Self {
            GenesisConfig {
                currencies: vec![],
                borrow_index: Rate::zero(),
                exchange_rate: Rate::zero(),
                base_rate: Rate::zero(),
                jump_rate: Multiplier::zero(),
                full_rate: Multiplier::zero(),
                jump_utilization: Ratio::zero(),
                collateral_factor: vec![],
                liquidation_incentive: vec![],
                close_factor: vec![],
                reserve_factor: vec![],
                last_block_timestamp: 0,
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig {
        fn build(&self) {
            self.currencies.iter().for_each(|currency_id| {
                let interest_model = InterestRateModel::new_jump_model(
                    self.base_rate,
                    self.jump_rate,
                    self.full_rate,
                    self.jump_utilization,
                );
                if !interest_model.check_model() {
                    panic!(
                        "Could not initialize the interest rate model!!! {:#?}",
                        currency_id
                    );
                }
                CurrencyInterestModel::<T>::insert(currency_id, interest_model);
                ExchangeRate::<T>::insert(currency_id, self.exchange_rate);
                BorrowIndex::<T>::insert(currency_id, self.borrow_index);
            });
            self.collateral_factor
                .iter()
                .for_each(|(currency_id, collateral_factor)| {
                    CollateralFactor::<T>::insert(currency_id, collateral_factor);
                });
            self.liquidation_incentive
                .iter()
                .for_each(|(currency_id, liquidation_incentive)| {
                    LiquidationIncentive::<T>::insert(currency_id, liquidation_incentive);
                });
            self.close_factor
                .iter()
                .for_each(|(currency_id, close_factor)| {
                    CloseFactor::<T>::insert(currency_id, close_factor);
                });
            self.reserve_factor
                .iter()
                .for_each(|(currency_id, reserve_factor)| {
                    ReserveFactor::<T>::insert(currency_id, reserve_factor);
                });
            Currencies::<T>::put(self.currencies.clone());
            LastBlockTimestamp::<T>::put(self.last_block_timestamp.clone());
        }
    }

    #[cfg(feature = "std")]
    impl GenesisConfig {
        /// Direct implementation of `GenesisBuild::build_storage`.
        ///
        /// Kept in order not to break dependency.
        pub fn build_storage<T: Config>(&self) -> Result<sp_runtime::Storage, String> {
            <Self as frame_support::traits::GenesisBuild<T>>::build_storage(self)
        }

        /// Direct implementation of `GenesisBuild::assimilate_storage`.
        ///
        /// Kept in order not to break dependency.
        pub fn assimilate_storage<T: Config>(
            &self,
            storage: &mut sp_runtime::Storage,
        ) -> Result<(), String> {
            <Self as frame_support::traits::GenesisBuild<T>>::assimilate_storage(self, storage)
        }
    }

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
        fn on_finalize(block_number: T::BlockNumber) {
            let now = T::UnixTime::now().as_secs();
            if LastBlockTimestamp::<T>::get().is_zero() {
                LastBlockTimestamp::<T>::put(now);
            }
            with_transaction(|| {
                match <Pallet<T>>::accrue_interest() {
                    Ok(()) => {
                        LastBlockTimestamp::<T>::put(now);
                        TransactionOutcome::Commit(1000)
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
            });

            // This is used to trigger the price aggregation to update the results to the ORML Oracle Pallet.
            for currency_id in Currencies::<T>::get().iter() {
                let _ = Self::get_price(currency_id);
            }
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Sender supplies assets into the market and receives internal supplies in exchange.
        ///
        /// - `currency_id`: the asset to be deposited.
        /// - `mint_amount`: the amount to be deposited.
        #[pallet::weight(T::WeightInfo::mint())]
        #[transactional]
        pub fn mint(
            origin: OriginFor<T>,
            currency_id: CurrencyId,
            mint_amount: Balance,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::ensure_currency(&currency_id)?;

            T::Currency::transfer(currency_id, &who, &Self::account_id(), mint_amount)?;
            Self::update_earned_stored(&who, &currency_id)?;
            let exchange_rate = Self::exchange_rate(currency_id);
            let voucher_amount = Self::calc_collateral_amount(mint_amount, exchange_rate)?;
            AccountDeposits::<T>::try_mutate(&currency_id, &who, |deposits| -> DispatchResult {
                deposits.voucher_balance = deposits
                    .voucher_balance
                    .checked_add(voucher_amount)
                    .ok_or(ArithmeticError::Overflow)?;
                Ok(())
            })?;
            TotalSupply::<T>::try_mutate(&currency_id, |total_balance| -> DispatchResult {
                let new_balance = total_balance
                    .checked_add(voucher_amount)
                    .ok_or(ArithmeticError::Overflow)?;
                *total_balance = new_balance;
                Ok(())
            })?;

            Self::deposit_event(Event::<T>::Deposited(who, currency_id, mint_amount));

            Ok(().into())
        }

        /// Sender redeems some of internal supplies in exchange for the underlying asset.
        ///
        /// - `currency_id`: the asset to be redeemed.
        /// - `redeem_amount`: the amount to be redeemed.
        #[pallet::weight(T::WeightInfo::redeem())]
        #[transactional]
        pub fn redeem(
            origin: OriginFor<T>,
            currency_id: CurrencyId,
            redeem_amount: Balance,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::ensure_currency(&currency_id)?;

            let exchange_rate = Self::exchange_rate(currency_id);
            let voucher_amount = Self::calc_collateral_amount(redeem_amount, exchange_rate)?;
            Self::update_earned_stored(&who, &currency_id)?;
            let redeem_amount = Self::redeem_internal(&who, &currency_id, voucher_amount)?;

            Self::deposit_event(Event::<T>::Redeemed(who, currency_id, redeem_amount));

            Ok(().into())
        }

        /// Sender redeems all of internal supplies in exchange for the underlying asset.
        ///
        /// - `currency_id`: the asset to be redeemed.
        #[pallet::weight(T::WeightInfo::redeem_all())]
        #[transactional]
        pub fn redeem_all(
            origin: OriginFor<T>,
            currency_id: CurrencyId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::ensure_currency(&currency_id)?;

            Self::update_earned_stored(&who, &currency_id)?;
            let deposits = AccountDeposits::<T>::get(&currency_id, &who);
            let redeem_amount =
                Self::redeem_internal(&who, &currency_id, deposits.voucher_balance)?;

            Self::deposit_event(Event::<T>::Redeemed(who, currency_id, redeem_amount));

            Ok(().into())
        }

        /// Sender borrows assets from the protocol to their own address.
        ///
        /// - `currency_id`: the asset to be borrowed.
        /// - `borrow_amount`: the amount to be borrowed.
        #[pallet::weight(T::WeightInfo::borrow())]
        #[transactional]
        pub fn borrow(
            origin: OriginFor<T>,
            currency_id: CurrencyId,
            borrow_amount: Balance,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::ensure_currency(&currency_id)?;

            Self::borrow_allowed(&currency_id, &who, borrow_amount)?;
            let account_borrows = Self::current_borrow_balance(&who, &currency_id)?;
            let account_borrows_new = account_borrows
                .checked_add(borrow_amount)
                .ok_or(ArithmeticError::Overflow)?;
            let total_borrows = Self::total_borrows(&currency_id);
            let total_borrows_new = total_borrows
                .checked_add(borrow_amount)
                .ok_or(ArithmeticError::Overflow)?;
            AccountBorrows::<T>::insert(
                &currency_id,
                &who,
                BorrowSnapshot {
                    principal: account_borrows_new,
                    borrow_index: Self::borrow_index(&currency_id),
                },
            );
            TotalBorrows::<T>::insert(&currency_id, total_borrows_new);
            T::Currency::transfer(currency_id, &Self::account_id(), &who, borrow_amount)?;

            Self::deposit_event(Event::<T>::Borrowed(who, currency_id, borrow_amount));

            Ok(().into())
        }

        /// Sender repays some of their debts.
        ///
        /// - `currency_id`: the asset to be repaid.
        /// - `repay_amount`: the amount to be repaid.
        #[pallet::weight(T::WeightInfo::repay_borrow())]
        #[transactional]
        pub fn repay_borrow(
            origin: OriginFor<T>,
            currency_id: CurrencyId,
            repay_amount: Balance,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::ensure_currency(&currency_id)?;

            let account_borrows = Self::current_borrow_balance(&who, &currency_id)?;
            Self::repay_borrow_internal(&who, &currency_id, account_borrows, repay_amount)?;

            Self::deposit_event(Event::<T>::RepaidBorrow(who, currency_id, repay_amount));

            Ok(().into())
        }

        /// Sender repays all of their debts.
        ///
        /// - `currency_id`: the asset to be repaid.
        #[pallet::weight(T::WeightInfo::repay_borrow_all())]
        #[transactional]
        pub fn repay_borrow_all(
            origin: OriginFor<T>,
            currency_id: CurrencyId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::ensure_currency(&currency_id)?;

            let account_borrows = Self::current_borrow_balance(&who, &currency_id)?;
            Self::repay_borrow_internal(&who, &currency_id, account_borrows, account_borrows)?;

            Self::deposit_event(Event::<T>::RepaidBorrow(who, currency_id, account_borrows));

            Ok(().into())
        }

        /// Sets a new liquidation incentive percentage for `currency_id`.
        ///
        /// Returns `Err` if the provided asset is not attached to an existent incentive.
        ///
        /// - `currency_id`: the asset that is going to be modified.
        #[pallet::weight(T::WeightInfo::set_liquidation_incentive())]
        #[transactional]
        pub fn set_liquidation_incentive(
            origin: OriginFor<T>,
            currency_id: CurrencyId,
            liquidate_incentive: Rate,
        ) -> DispatchResultWithPostInfo {
            T::UpdateOrigin::ensure_origin(origin)?;
            Self::ensure_currency(&currency_id)?;

            if <LiquidationIncentive<T>>::try_get(currency_id).is_err() {
                return Err(<Error<T>>::CurrencyNotEnabled.into());
            }

            <LiquidationIncentive<T>>::insert(currency_id, liquidate_incentive);

            Ok(().into())
        }

        /// Using for development
        #[pallet::weight(T::WeightInfo::transfer_token())]
        #[transactional]
        pub fn transfer_token(
            origin: OriginFor<T>,
            to: T::AccountId,
            currency_id: CurrencyId,
            amount: Balance,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::ensure_currency(&currency_id)?;

            T::Currency::transfer(currency_id, &who, &to, amount)?;

            Ok(().into())
        }

        /// Set the collateral asset.
        ///
        /// - `currency_id`: the asset to be set.
        /// - `enable`: turn on/off the collateral option.
        #[pallet::weight(T::WeightInfo::collateral_asset())]
        #[transactional]
        pub fn collateral_asset(
            origin: OriginFor<T>,
            currency_id: CurrencyId,
            enable: bool,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::ensure_currency(&currency_id)?;
            ensure!(
                AccountDeposits::<T>::contains_key(currency_id, &who),
                Error::<T>::NoDeposit
            );
            let mut deposits = Self::account_deposits(currency_id, &who);
            if deposits.is_collateral == enable {
                return Err(Error::<T>::DuplicateOperation.into());
            }
            // turn on the collateral button
            if enable {
                deposits.is_collateral = true;
                AccountDeposits::<T>::insert(currency_id, &who, deposits);
                Self::deposit_event(Event::<T>::CollateralAssetAdded(who, currency_id));

                return Ok(().into());
            }
            // turn off the collateral button after checking the liquidity
            let total_collateral_value = Self::total_collateral_value(&who)?;
            let collateral_asset_value = Self::collateral_asset_value(&who, &currency_id)?;
            let total_borrowed_value = Self::total_borrowed_value(&who)?;
            if total_collateral_value
                < total_borrowed_value
                    .checked_add(&collateral_asset_value)
                    .ok_or(ArithmeticError::Overflow)?
            {
                return Err(Error::<T>::InsufficientLiquidity.into());
            }

            deposits.is_collateral = false;
            AccountDeposits::<T>::insert(currency_id, &who, deposits);
            Self::deposit_event(Event::<T>::CollateralAssetRemoved(who, currency_id));

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
            liquidate_token: CurrencyId,
            repay_amount: Balance,
            collateral_token: CurrencyId,
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

        /// Update the interest rate model for a given asset.
        ///
        /// May only be called from `T::UpdateOrigin`.
        ///
        /// - `currency_id`: the assets to be set.
        /// - `new_model`: the interest rate model to be set.
        #[pallet::weight(T::WeightInfo::set_rate_model())]
        #[transactional]
        pub fn set_rate_model(
            origin: OriginFor<T>,
            currency_id: CurrencyId,
            new_model: InterestRateModel,
        ) -> DispatchResultWithPostInfo {
            T::UpdateOrigin::ensure_origin(origin)?;
            Self::ensure_currency(&currency_id)?;

            ensure!(new_model.check_model(), Error::<T>::InvalidRateModelParam);

            CurrencyInterestModel::<T>::insert(currency_id, new_model);

            Self::deposit_event(Event::<T>::NewInterestRateModel(new_model));

            Ok(().into())
        }

        /// Add reserves by transferring from payer.
        ///
        /// May only be called from `T::ReserveOrigin`.
        ///
        /// - `payer`: the payer account.
        /// - `currency_id`: the assets to be added.
        /// - `add_amount`: the amount to be added.
        #[pallet::weight(T::WeightInfo::add_reserves())]
        #[transactional]
        pub fn add_reserves(
            origin: OriginFor<T>,
            payer: <T::Lookup as StaticLookup>::Source,
            currency_id: CurrencyId,
            add_amount: Balance,
        ) -> DispatchResultWithPostInfo {
            T::ReserveOrigin::ensure_origin(origin)?;
            let payer = T::Lookup::lookup(payer)?;
            Self::ensure_currency(&currency_id)?;

            T::Currency::transfer(currency_id, &payer, &Self::account_id(), add_amount)?;
            let total_reserves = Self::total_reserves(currency_id);
            let total_reserves_new = total_reserves
                .checked_add(add_amount)
                .ok_or(ArithmeticError::Overflow)?;
            TotalReserves::<T>::insert(currency_id, total_reserves_new);

            Self::deposit_event(Event::<T>::ReservesAdded(
                Self::account_id(),
                currency_id,
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
        /// - `currency_id`: the assets to be reduced.
        /// - `reduce_amount`: the amount to be reduced.
        #[pallet::weight(T::WeightInfo::reduce_reserves())]
        #[transactional]
        pub fn reduce_reserves(
            origin: OriginFor<T>,
            receiver: <T::Lookup as StaticLookup>::Source,
            currency_id: CurrencyId,
            reduce_amount: Balance,
        ) -> DispatchResultWithPostInfo {
            T::ReserveOrigin::ensure_origin(origin)?;
            let receiver = T::Lookup::lookup(receiver)?;
            Self::ensure_currency(&currency_id)?;

            let total_reserves = Self::total_reserves(currency_id);
            if reduce_amount > total_reserves {
                return Err(Error::<T>::InsufficientReserves.into());
            }
            let total_reserves_new = total_reserves
                .checked_sub(reduce_amount)
                .ok_or(ArithmeticError::Underflow)?;
            TotalReserves::<T>::insert(currency_id, total_reserves_new);
            T::Currency::transfer(currency_id, &Self::account_id(), &receiver, reduce_amount)?;

            Self::deposit_event(Event::<T>::ReservesReduced(
                receiver,
                currency_id,
                reduce_amount,
                total_reserves_new,
            ));

            Ok(().into())
        }
    }
}

impl<T: Config> Pallet<T> {
    pub fn account_id() -> T::AccountId {
        T::PalletId::get().into_account()
    }

    fn get_account_liquidity(
        account: &T::AccountId,
    ) -> Result<(FixedU128, FixedU128), DispatchError> {
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

        for currency_id in Currencies::<T>::get().iter() {
            let currency_borrow_amount = Self::current_borrow_balance(borrower, currency_id)?;
            if currency_borrow_amount.is_zero() {
                continue;
            }
            let borrow_currency_price = Self::get_price(currency_id)?;
            total_borrow_value = borrow_currency_price
                .checked_mul(&FixedU128::from_inner(currency_borrow_amount))
                .and_then(|r| r.checked_add(&total_borrow_value))
                .ok_or(ArithmeticError::Overflow)?;
        }

        Ok(total_borrow_value)
    }

    fn collateral_asset_value(
        borrower: &T::AccountId,
        currency_id: &CurrencyId,
    ) -> Result<FixedU128, DispatchError> {
        if !AccountDeposits::<T>::contains_key(currency_id, borrower) {
            return Ok(FixedU128::zero());
        }
        let deposits = Self::account_deposits(currency_id, borrower);
        if !deposits.is_collateral {
            return Ok(FixedU128::zero());
        }
        if deposits.voucher_balance.is_zero() {
            return Ok(FixedU128::zero());
        }
        let collateral_factor = CollateralFactor::<T>::get(currency_id);
        let exchange_rate = ExchangeRate::<T>::get(currency_id);
        let currency_price = Self::get_price(currency_id)?;
        let collateral_amount = exchange_rate
            .checked_mul_int(collateral_factor.mul_floor(deposits.voucher_balance))
            .ok_or(ArithmeticError::Overflow)?;

        Ok(currency_price
            .checked_mul(&FixedU128::from_inner(collateral_amount))
            .ok_or(ArithmeticError::Overflow)?)
    }

    fn total_collateral_value(borrower: &T::AccountId) -> Result<FixedU128, DispatchError> {
        let mut total_asset_value: FixedU128 = FixedU128::zero();
        for currency_id in Currencies::<T>::get().iter() {
            total_asset_value = total_asset_value
                .checked_add(&Self::collateral_asset_value(borrower, currency_id)?)
                .ok_or(ArithmeticError::Overflow)?;
        }

        Ok(total_asset_value)
    }

    /// Checks if the redeemer should be allowed to redeem tokens in given market
    fn redeem_allowed(
        currency_id: &CurrencyId,
        redeemer: &T::AccountId,
        voucher_amount: Balance,
    ) -> DispatchResult {
        let deposit = Self::account_deposits(currency_id, redeemer);
        if deposit.voucher_balance < voucher_amount {
            return Err(Error::<T>::InsufficientDeposit.into());
        }
        if !deposit.is_collateral {
            return Ok(());
        }
        let collateral_factor = Self::collateral_factor(currency_id);
        let price = Self::get_price(currency_id)?;
        let exchange_rate = Self::exchange_rate(currency_id);
        let redeem_amount = Self::calc_underlying_amount(voucher_amount, exchange_rate)?;
        let redeem_effects_value = price
            .checked_mul(&FixedU128::from_inner(
                collateral_factor.mul_ceil(redeem_amount),
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
        currency_id: &CurrencyId,
        voucher_amount: Balance,
    ) -> Result<Balance, DispatchError> {
        Self::redeem_allowed(currency_id, who, voucher_amount)?;
        let exchange_rate = Self::exchange_rate(currency_id);
        let redeem_amount = Self::calc_underlying_amount(voucher_amount, exchange_rate)?;
        AccountDeposits::<T>::try_mutate_exists(currency_id, who, |deposits| -> DispatchResult {
            let mut d = deposits.unwrap_or_default();
            d.voucher_balance = d
                .voucher_balance
                .checked_sub(voucher_amount)
                .ok_or(ArithmeticError::Underflow)?;
            if d.voucher_balance.is_zero() {
                // remove deposits storage if zero balance
                *deposits = None;
            } else {
                *deposits = Some(d);
            }
            Ok(())
        })?;
        TotalSupply::<T>::try_mutate(currency_id, |total_balance| -> DispatchResult {
            let new_balance = total_balance
                .checked_sub(voucher_amount)
                .ok_or(ArithmeticError::Underflow)?;
            *total_balance = new_balance;
            Ok(())
        })?;
        T::Currency::transfer(*currency_id, &Self::account_id(), who, redeem_amount)?;

        Ok(redeem_amount)
    }

    /// Borrower shouldn't borrow more than his total collateral value
    fn borrow_allowed(
        currency_id: &CurrencyId,
        borrower: &T::AccountId,
        borrow_amount: Balance,
    ) -> DispatchResult {
        let price = Self::get_price(currency_id)?;
        let borrow_value = price
            .checked_mul(&FixedU128::from_inner(borrow_amount))
            .ok_or(ArithmeticError::Overflow)?;

        let (liquidity, _) = Self::get_account_liquidity(borrower)?;
        if liquidity < borrow_value {
            return Err(Error::<T>::InsufficientLiquidity.into());
        }

        Ok(())
    }

    fn repay_borrow_internal(
        borrower: &T::AccountId,
        currency_id: &CurrencyId,
        account_borrows: Balance,
        repay_amount: Balance,
    ) -> DispatchResult {
        if account_borrows < repay_amount {
            return Err(Error::<T>::TooMuchRepay.into());
        }

        T::Currency::transfer(*currency_id, borrower, &Self::account_id(), repay_amount)?;

        let account_borrows_new = account_borrows
            .checked_sub(repay_amount)
            .ok_or(ArithmeticError::Underflow)?;
        let total_borrows = Self::total_borrows(currency_id);
        // NOTE : total_borrows use a different way to calculate interest
        // so when user repays all borrows, total_borrows can be smaller than account_borrows
        // which will cause it to fail with `ArithmeticError::Underflow`
        //
        // Change it back to checked_sub will cause Underflow
        let total_borrows_new = total_borrows.saturating_sub(repay_amount);

        AccountBorrows::<T>::insert(
            currency_id,
            borrower,
            BorrowSnapshot {
                principal: account_borrows_new,
                borrow_index: Self::borrow_index(currency_id),
            },
        );

        TotalBorrows::<T>::insert(currency_id, total_borrows_new);

        Ok(())
    }

    // Calculates and returns the most recent amount of borrowed balance of `currency_id`
    // for `who`.
    pub fn current_borrow_balance(
        who: &T::AccountId,
        currency_id: &CurrencyId,
    ) -> Result<Balance, DispatchError> {
        let snapshot: BorrowSnapshot = Self::account_borrows(currency_id, who);
        Self::current_balance_from_snapshot(currency_id, snapshot)
    }

    /// Same as `borrow_balance_stored` but takes a given `snapshot` instead of fetching
    /// the storage
    pub fn current_balance_from_snapshot(
        currency_id: &CurrencyId,
        snapshot: BorrowSnapshot,
    ) -> Result<Balance, DispatchError> {
        if snapshot.principal.is_zero() || snapshot.borrow_index.is_zero() {
            return Ok(Zero::zero());
        }
        // Calculate new borrow balance using the interest index:
        // recent_borrow_balance = snapshot.principal * borrow_index / snapshot.borrow_index
        let recent_borrow_balance = Self::borrow_index(currency_id)
            .checked_div(&snapshot.borrow_index)
            .and_then(|r| r.checked_mul_int(snapshot.principal))
            .ok_or(ArithmeticError::Overflow)?;

        Ok(recent_borrow_balance)
    }

    fn update_earned_stored(who: &T::AccountId, currency_id: &CurrencyId) -> DispatchResult {
        let deposits = AccountDeposits::<T>::get(currency_id, who);
        let exchange_rate = ExchangeRate::<T>::get(currency_id);
        let account_earned = AccountEarned::<T>::get(currency_id, who);
        let total_earned_prior_new = exchange_rate
            .checked_sub(&account_earned.exchange_rate_prior)
            .and_then(|r| r.checked_mul_int(deposits.voucher_balance))
            .and_then(|r| r.checked_add(account_earned.total_earned_prior))
            .ok_or(ArithmeticError::Overflow)?;

        AccountEarned::<T>::insert(
            currency_id,
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
        liquidate_currency_id: CurrencyId,
        repay_amount: Balance,
    ) -> DispatchResult {
        let (_, shortfall) = Self::get_account_liquidity(borrower)?;
        if shortfall.is_zero() {
            return Err(Error::<T>::InsufficientShortfall.into());
        }

        // The liquidator may not repay more than 50%(close_factor) of the borrower's borrow balance.
        let account_borrows = Self::current_borrow_balance(&borrower, &liquidate_currency_id)?;
        let close_factor = CloseFactor::<T>::get(liquidate_currency_id);
        if close_factor.mul_ceil(account_borrows) < repay_amount {
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
        liquidate_currency_id: CurrencyId,
        repay_amount: Balance,
        collateral_currency_id: CurrencyId,
    ) -> DispatchResult {
        Self::ensure_currency(&liquidate_currency_id)?;

        if borrower == liquidator {
            return Err(Error::<T>::LiquidatorIsBorrower.into());
        }
        Self::liquidate_borrow_allowed(&borrower, liquidate_currency_id, repay_amount)?;

        let deposits = AccountDeposits::<T>::get(collateral_currency_id, &borrower);
        if !deposits.is_collateral {
            return Err(Error::<T>::DepositsAreNotCollateral.into());
        }
        let exchange_rate = Self::exchange_rate(collateral_currency_id);
        let borrower_deposit_amount = exchange_rate
            .checked_mul_int(deposits.voucher_balance)
            .ok_or(ArithmeticError::Overflow)?;

        // Calculate the collateral value
        let collateral_token_price = Self::get_price(&collateral_currency_id)?;
        let collateral_value = collateral_token_price
            .checked_mul(&FixedU128::from_inner(borrower_deposit_amount))
            .ok_or(ArithmeticError::Overflow)?;

        // The incentive for liquidator and punishment for the borrower
        let liquidation_incentive = LiquidationIncentive::<T>::get(liquidate_currency_id);
        let liquidate_value = Self::get_price(&liquidate_currency_id)?
            .checked_mul(&FixedU128::from_inner(repay_amount))
            .and_then(|a| a.checked_mul(&liquidation_incentive))
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
            .ok_or(ArithmeticError::Underflow)?;

        //inside transfer token
        Self::liquidate_repay_borrow_internal(
            &liquidator,
            &borrower,
            &liquidate_currency_id,
            &collateral_currency_id,
            repay_amount,
            real_collateral_underlying_amount.into_inner(),
        )?;

        Ok(())
    }

    fn liquidate_repay_borrow_internal(
        liquidator: &T::AccountId,
        borrower: &T::AccountId,
        liquidate_currency_id: &CurrencyId,
        collateral_currency_id: &CurrencyId,
        repay_amount: Balance,
        collateral_underlying_amount: Balance,
    ) -> DispatchResult {
        // 1.liquidator repay borrower's debt,
        // transfer from liquidator to module account
        T::Currency::transfer(
            *liquidate_currency_id,
            liquidator,
            &Self::account_id(),
            repay_amount,
        )?;

        // 2.the system reduce borrower's debt
        let account_borrows = Self::current_borrow_balance(borrower, liquidate_currency_id)?;
        let account_borrows_new = account_borrows
            .checked_sub(repay_amount)
            .ok_or(ArithmeticError::Underflow)?;
        let total_borrows = Self::total_borrows(liquidate_currency_id);
        let total_borrows_new = total_borrows
            .checked_sub(repay_amount)
            .ok_or(ArithmeticError::Underflow)?;
        AccountBorrows::<T>::insert(
            liquidate_currency_id,
            borrower,
            BorrowSnapshot {
                principal: account_borrows_new,
                borrow_index: Self::borrow_index(liquidate_currency_id),
            },
        );
        TotalBorrows::<T>::insert(liquidate_currency_id, total_borrows_new);

        // 3.the liquidator will receive voucher token from borrower
        let exchange_rate = Self::exchange_rate(collateral_currency_id);
        let collateral_amount =
            Self::calc_collateral_amount(collateral_underlying_amount, exchange_rate)?;
        AccountDeposits::<T>::try_mutate(
            collateral_currency_id,
            borrower,
            |deposits| -> DispatchResult {
                deposits.voucher_balance = deposits
                    .voucher_balance
                    .checked_sub(collateral_amount)
                    .ok_or(ArithmeticError::Underflow)?;
                Ok(())
            },
        )?;
        // increase liquidator's voucher_balance
        AccountDeposits::<T>::try_mutate(
            collateral_currency_id,
            liquidator,
            |deposits| -> DispatchResult {
                deposits.voucher_balance = deposits
                    .voucher_balance
                    .checked_add(collateral_amount)
                    .ok_or(ArithmeticError::Overflow)?;
                Ok(())
            },
        )?;

        Self::deposit_event(Event::<T>::LiquidatedBorrow(
            liquidator.clone(),
            borrower.clone(),
            *liquidate_currency_id,
            *collateral_currency_id,
            repay_amount,
            collateral_underlying_amount,
        ));

        Ok(())
    }

    // Ensures a given `currency_id` exists on the `Currencies` storage.
    fn ensure_currency(currency_id: &CurrencyId) -> DispatchResult {
        if Self::currencies()
            .iter()
            .any(|currency| currency == currency_id)
        {
            Ok(())
        } else {
            Err(<Error<T>>::CurrencyNotEnabled.into())
        }
    }

    fn accrue_interest() -> DispatchResult {
        for currency_id in Self::currencies() {
            let total_cash = Self::get_total_cash(currency_id);
            let total_borrows = Self::total_borrows(currency_id);
            let total_reserves = Self::total_reserves(currency_id);
            let util = Self::calc_utilization_ratio(total_cash, total_borrows, total_reserves)?;

            let interest_model = Self::currency_interest_model(currency_id);
            let borrow_rate = interest_model
                .get_borrow_rate(util)
                .ok_or(ArithmeticError::Overflow)?;
            let supply_rate = InterestRateModel::get_supply_rate(
                borrow_rate,
                util,
                Self::reserve_factor(currency_id),
            );

            UtilizationRatio::<T>::insert(currency_id, util);
            BorrowRate::<T>::insert(currency_id, &borrow_rate);
            SupplyRate::<T>::insert(currency_id, supply_rate);

            Self::update_borrow_index(borrow_rate, currency_id)?;
            Self::update_exchange_rate(currency_id)?;
        }

        Ok(())
    }

    pub fn update_exchange_rate(currency_id: CurrencyId) -> DispatchResult {
        // exchangeRate = (totalCash + totalBorrows - totalReserves) / totalSupply
        let total_supply = Self::total_supply(currency_id);
        if total_supply.is_zero() {
            return Ok(());
        }
        let total_cash = Self::get_total_cash(currency_id);
        let total_borrows = Self::total_borrows(currency_id);
        let total_reserves = Self::total_reserves(currency_id);

        let cash_plus_borrows_minus_reserves = total_cash
            .checked_add(total_borrows)
            .and_then(|r| r.checked_sub(total_reserves))
            .ok_or(ArithmeticError::Overflow)?;
        let exchange_rate =
            Rate::checked_from_rational(cash_plus_borrows_minus_reserves, total_supply)
                .ok_or(ArithmeticError::Underflow)?;

        ExchangeRate::<T>::insert(currency_id, exchange_rate);

        Ok(())
    }

    pub fn calc_utilization_ratio(
        cash: Balance,
        borrows: Balance,
        reserves: Balance,
    ) -> Result<Ratio, DispatchError> {
        // utilization ratio is 0 when there are no borrows
        if borrows.is_zero() {
            return Ok(Ratio::zero());
        }
        // utilizationRatio = totalBorrows / (totalCash + totalBorrows − totalReserves)
        let total = cash
            .checked_add(borrows)
            .and_then(|r| r.checked_sub(reserves))
            .ok_or(ArithmeticError::Overflow)?;

        Ok(Ratio::from_rational(borrows, total))
    }

    pub fn update_borrow_index(borrow_rate: Rate, currency_id: CurrencyId) -> DispatchResult {
        // interestAccumulated = totalBorrows * borrowRate
        // totalBorrows = interestAccumulated + totalBorrows
        // totalReserves = interestAccumulated * reserveFactor + totalReserves
        // borrowIndex = borrowIndex * (1 + borrowRate)
        let borrows_prior = Self::total_borrows(currency_id);
        let reserve_prior = Self::total_reserves(currency_id);
        let reserve_factor = Self::reserve_factor(currency_id);
        let delta_time = T::UnixTime::now()
            .as_secs()
            .checked_sub(Self::last_block_timestamp())
            .ok_or(ArithmeticError::Underflow)?;
        let interest_accumulated = accrued_interest(borrow_rate, borrows_prior, delta_time)
            .ok_or(ArithmeticError::Overflow)?;
        let total_borrows_new = interest_accumulated
            .checked_add(borrows_prior)
            .ok_or(ArithmeticError::Overflow)?;
        let total_reserves_new = reserve_factor
            .mul_floor(interest_accumulated)
            .checked_add(reserve_prior)
            .ok_or(ArithmeticError::Overflow)?;
        let borrow_index = Self::borrow_index(currency_id);
        let borrow_index_new = increment_index(borrow_rate, borrow_index, delta_time)
            .and_then(|r| r.checked_add(&borrow_index))
            .ok_or(ArithmeticError::Overflow)?;

        TotalBorrows::<T>::insert(currency_id, total_borrows_new);
        TotalReserves::<T>::insert(currency_id, total_reserves_new);
        BorrowIndex::<T>::insert(currency_id, borrow_index_new);

        Ok(())
    }

    pub fn get_total_cash(currency_id: CurrencyId) -> Balance {
        T::Currency::free_balance(currency_id, &Self::account_id())
    }

    pub fn calc_underlying_amount(
        voucher_amount: u128,
        exchange_rate: Rate,
    ) -> Result<Balance, DispatchError> {
        Ok(exchange_rate
            .checked_mul_int(voucher_amount)
            .ok_or(ArithmeticError::Overflow)?)
    }

    pub fn calc_collateral_amount(
        underlying_amount: u128,
        exchange_rate: Rate,
    ) -> Result<Balance, DispatchError> {
        Ok(FixedU128::from_inner(underlying_amount)
            .checked_div(&exchange_rate)
            .map(|r| r.into_inner())
            .ok_or(ArithmeticError::Underflow)?)
    }

    pub fn get_price(currency_id: &CurrencyId) -> Result<Price, Error<T>> {
        let (price, _) =
            T::PriceFeeder::get_price(currency_id).ok_or(Error::<T>::PriceOracleNotReady)?;
        if price.is_zero() {
            return Err(Error::<T>::PriceOracleNotReady);
        }

        Ok(price)
    }
}
