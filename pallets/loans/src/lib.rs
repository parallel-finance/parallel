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

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::collapsible_if)]

use crate::rate::InterestRateModel;
use crate::util::*;
use frame_support::{pallet_prelude::*, transactional, PalletId};
use frame_system::pallet_prelude::*;
use orml_traits::{MultiCurrency, MultiCurrencyExtended};
use primitives::{Amount, Balance, CurrencyId, Multiplier, PriceFeeder, Rate, Ratio};
use sp_runtime::{
    traits::{AccountIdConversion, CheckedAdd, CheckedDiv, CheckedMul, StaticLookup, Zero},
    FixedPointNumber,
};
use sp_std::vec::Vec;

pub use module::*;

mod benchmarking;
mod loan;
#[cfg(test)]
mod mock;
mod rate;
#[cfg(test)]
mod tests;
mod util;

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
    /// Exchange rate,  after applying the most recent balance-changing action
    pub exchange_rate_prior: Rate,
}

#[frame_support::pallet]
pub mod module {
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
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Collateral amount overflow when calculating
        CollateralOverflow,
        /// Collateral amount too low to redeem
        CollateralTooLow,
        /// Insufficient collateral asset to borrow more
        InsufficientCollateral,
        /// Repay amount greater than borrow balance
        RepayAmountTooBig,
        /// Amount type convert failed
        AmountConvertFailed,
        /// Calculate accrue interest failed
        CalcAccrueInterestFailed,
        /// Calculate exchange rate failed
        CalcExchangeRateFailed,
        /// Calculate collateral amount failed
        CalcCollateralFailed,
        /// Calculate interest rate failed
        CalcInterestRateFailed,
        /// Calculate borrow balance failed
        CalcBorrowBalanceFailed,
        /// Calculate earned amount failed
        CalcEarnedFailed,
        /// Please enable collateral for one of your assets before borrowing
        NoCollateralAsset,
        /// Currency's oracle price not ready
        OracleCurrencyPriceNotReady,
        /// Asset already enabled collateral
        AlreadyEnabledCollateral,
        /// Asset already disabled collateral
        AlreadyDisabledCollateral,
        /// Please deposit before collateral
        DepositRequiredBeforeCollateral,
        /// Collateral disable action denied
        CollateralDisableActionDenied,
        /// Repay amount more than collateral amount
        RepayValueGreaterThanCollateral,
        /// Liquidator is same as borrower
        LiquidatorIsBorrower,
        /// There is no borrow balance
        NoBorrowBalance,
        /// Calculate borrow balance with close factor failed
        CalcCloseBorrowsFailed,
        /// Calculate incentive value failed
        CalcDiscdCollateralValueFailed,
        /// Liquidate value overflow
        LiquidateValueOverflow,
        /// Equivalent collateral amount overflow
        EquivalentCollateralAmountOverflow,
        /// Real collateral amount overflow
        RealCollateralAmountOverflow,
        /// Insufficient reserves
        InsufficientReserves,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub (crate) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Initialize the interest rate parameter
        /// [base_rate, multiplier_per_block, jump_multiplier_per_block]
        InitInterestRateModel(Rate, Multiplier, Multiplier),
        /// Enable collateral for certain asset
        /// [sender, currency_id]
        CollateralAssetAdded(T::AccountId, CurrencyId),
        /// Disable collateral for certain asset
        /// [sender, currency_id]
        CollateralAssetRemoved(T::AccountId, CurrencyId),

        // TODO: add event for dispatchables `mint, redeem, borrow, repay, liquidate` (#32)
        /// Liquidation occurred
        /// [liquidator, borrower,liquidate_token,collateral_token,liquidate_token_repay_amount,collateral_token_amount]
        LiquidationOccur(
            T::AccountId,
            T::AccountId,
            CurrencyId,
            CurrencyId,
            Balance,
            Balance,
        ),
        /// Event emitted when the reserves are reduced
        /// [admin, currency_id, reduced_amount, total_reserves]
        ReservesReduced(T::AccountId, CurrencyId, Balance, Balance),
        /// Event emitted when the reserves are added
        /// [admin, currency_id, added_amount, total_reserves]
        ReservesAdded(T::AccountId, CurrencyId, Balance, Balance),
    }

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
        Twox64Concat,
        T::AccountId,
        BorrowSnapshot,
        ValueQuery,
    >;

    /// Mapping of account addresses to collateral tokens balances
    /// CollateralType -> Owner -> Balance
    #[pallet::storage]
    #[pallet::getter(fn account_collateral)]
    pub type AccountCollateral<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        CurrencyId,
        Twox64Concat,
        T::AccountId,
        Balance,
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
        Twox64Concat,
        T::AccountId,
        EarnedSnapshot,
        ValueQuery,
    >;

    /// Mapping of account addresses to assets which allowed as collateral
    /// Owner -> Vec<CurrencyId>
    #[pallet::storage]
    #[pallet::getter(fn account_collateral_assets)]
    pub type AccountCollateralAssets<T: Config> =
        StorageMap<_, Twox64Concat, T::AccountId, Vec<CurrencyId>, ValueQuery>;

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
        StorageMap<_, Twox64Concat, CurrencyId, u128, ValueQuery>;

    /// The collateral utilization ratio will triggering the liquidation
    #[pallet::storage]
    #[pallet::getter(fn liquidation_threshold)]
    pub type LiquidationThreshold<T: Config> =
        StorageMap<_, Twox64Concat, CurrencyId, u128, ValueQuery>;

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
        pub base_rate_per_year: Rate,
        pub multiplier_per_year: Multiplier,
        pub jump_multiplier_per_year: Multiplier,
        pub kink: Ratio,
        pub collateral_factor: Vec<(CurrencyId, Ratio)>,
        pub liquidation_incentive: Vec<(CurrencyId, u128)>,
        pub liquidation_threshold: Vec<(CurrencyId, u128)>,
        pub close_factor: Vec<(CurrencyId, Ratio)>,
        pub reserve_factor: Vec<(CurrencyId, Ratio)>,
    }

    #[cfg(feature = "std")]
    impl Default for GenesisConfig {
        fn default() -> Self {
            GenesisConfig {
                currencies: vec![],
                borrow_index: Rate::zero(),
                exchange_rate: Rate::zero(),
                base_rate_per_year: Rate::zero(),
                multiplier_per_year: Multiplier::zero(),
                jump_multiplier_per_year: Multiplier::zero(),
                kink: Ratio::zero(),
                collateral_factor: vec![],
                liquidation_incentive: vec![],
                liquidation_threshold: vec![],
                close_factor: vec![],
                reserve_factor: vec![],
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig {
        fn build(&self) {
            self.currencies.iter().for_each(|currency_id| {
                let interest_model = InterestRateModel::init_model(
                    self.base_rate_per_year,
                    self.multiplier_per_year,
                    self.jump_multiplier_per_year,
                    self.kink,
                )
                .unwrap();
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
            self.liquidation_threshold
                .iter()
                .for_each(|(currency_id, liquidation_threshold)| {
                    LiquidationThreshold::<T>::insert(currency_id, liquidation_threshold);
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
        fn on_finalize(_now: T::BlockNumber) {
            let _ = <Pallet<T>>::accrue_interest();
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        #[transactional]
        pub fn mint(
            origin: OriginFor<T>,
            currency_id: CurrencyId,
            mint_amount: Balance,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::update_earned_stored(&who, &currency_id)?;
            Self::mint_internal(&who, &currency_id, mint_amount)?;
            Ok(().into())
        }

        #[pallet::weight(10_000)]
        #[transactional]
        pub fn redeem(
            origin: OriginFor<T>,
            currency_id: CurrencyId,
            redeem_amount: Balance,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::update_earned_stored(&who, &currency_id)?;
            Self::redeem_internal(&who, &currency_id, redeem_amount)?;
            Ok(().into())
        }

        #[pallet::weight(10_000)]
        #[transactional]
        pub fn redeem_all(
            origin: OriginFor<T>,
            currency_id: CurrencyId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::update_earned_stored(&who, &currency_id)?;
            let collateral = AccountCollateral::<T>::get(&currency_id, &who);
            let exchange_rate = Self::exchange_rate(currency_id);
            let redeem_amount = exchange_rate
                .checked_mul_int(collateral)
                .ok_or(Error::<T>::CollateralOverflow)?;
            Self::redeem_internal(&who, &currency_id, redeem_amount)?;
            Ok(().into())
        }

        #[pallet::weight(10_000)]
        #[transactional]
        pub fn borrow(
            origin: OriginFor<T>,
            currency_id: CurrencyId,
            borrow_amount: Balance,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::borrow_internal(&who, &currency_id, borrow_amount)?;
            Ok(().into())
        }

        #[pallet::weight(10_000)]
        #[transactional]
        pub fn repay_borrow(
            origin: OriginFor<T>,
            currency_id: CurrencyId,
            repay_amount: Balance,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::repay_borrow_internal(&who, &currency_id, repay_amount)?;
            Ok(().into())
        }

        #[pallet::weight(10_000)]
        #[transactional]
        pub fn repay_borrow_all(
            origin: OriginFor<T>,
            currency_id: CurrencyId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let account_borrows = Self::borrow_balance_stored(&who, &currency_id)?;
            Self::repay_borrow_internal(&who, &currency_id, account_borrows)?;
            Ok(().into())
        }

        #[pallet::weight(10_000)]
        #[transactional]
        pub fn transfer_token(
            origin: OriginFor<T>,
            to: T::AccountId,
            currency_id: CurrencyId,
            amount: Balance,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            T::Currency::transfer(currency_id, &who, &to, amount)?;

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        #[transactional]
        pub fn collateral_asset(
            origin: OriginFor<T>,
            currency_id: CurrencyId,
            enable: bool,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::collateral_asset_internal(who, currency_id, enable)?;

            Ok(().into())
        }

        #[pallet::weight(10_000)]
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

        #[pallet::weight(10_000)]
        #[transactional]
        pub fn add_reserves(
            origin: OriginFor<T>,
            payer: <T::Lookup as StaticLookup>::Source,
            currency_id: CurrencyId,
            add_amount: Balance,
        ) -> DispatchResultWithPostInfo {
            T::ReserveOrigin::ensure_origin(origin)?;
            let payer = T::Lookup::lookup(payer)?;

            T::Currency::transfer(currency_id.clone(), &payer, &Self::account_id(), add_amount)?;
            let total_reserves = Self::total_reserves(currency_id);
            let total_reserves_new = total_reserves + add_amount;
            TotalReserves::<T>::insert(currency_id, total_reserves_new);

            Self::deposit_event(Event::<T>::ReservesAdded(
                Self::account_id().clone(),
                currency_id,
                add_amount,
                total_reserves_new,
            ));

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        #[transactional]
        pub fn reduce_reserves(
            origin: OriginFor<T>,
            receiver: <T::Lookup as StaticLookup>::Source,
            currency_id: CurrencyId,
            reduce_amount: Balance,
        ) -> DispatchResultWithPostInfo {
            T::ReserveOrigin::ensure_origin(origin)?;
            let receiver = T::Lookup::lookup(receiver)?;

            let total_reserves = Self::total_reserves(currency_id);
            if reduce_amount > total_reserves {
                return Err(Error::<T>::InsufficientReserves.into());
            }
            let total_reserves_new = total_reserves - reduce_amount;
            T::Currency::transfer(
                currency_id.clone(),
                &Self::account_id(),
                &receiver,
                reduce_amount,
            )?;
            TotalReserves::<T>::insert(currency_id, total_reserves_new);
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

    fn accrue_interest() -> DispatchResult {
        for currency_id in Self::currencies() {
            let total_cash = Self::get_total_cash(currency_id);
            let total_borrows = Self::total_borrows(currency_id);
            let total_reserves = Self::total_reserves(currency_id);
            let util = Self::calc_utilization_ratio(total_cash, total_borrows, total_reserves)?;

            let interest_model = Self::currency_interest_model(currency_id);
            let borrow_rate = interest_model
                .get_borrow_rate(util)
                .ok_or(Error::<T>::CollateralOverflow)?;
            let supply_rate = InterestRateModel::get_supply_rate(
                borrow_rate,
                util,
                Self::reserve_factor(currency_id),
            )
            .ok_or(Error::<T>::CollateralOverflow)?;

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
        let total_cash = Self::get_total_cash(currency_id);
        let total_borrows = Self::total_borrows(currency_id);
        let total_reserves = Self::total_reserves(currency_id);
        let total_supply = Self::total_supply(currency_id);

        let cash_plus_borrows_minus_reserves = total_cash
            .checked_add(total_borrows)
            .and_then(|r| r.checked_sub(total_reserves))
            .ok_or(Error::<T>::CalcAccrueInterestFailed)?;
        let exchange_rate =
            Rate::checked_from_rational(cash_plus_borrows_minus_reserves, total_supply)
                .ok_or(Error::<T>::CalcExchangeRateFailed)?;

        ExchangeRate::<T>::insert(currency_id, exchange_rate);

        Ok(())
    }

    pub fn calc_utilization_ratio(
        cash: Balance,
        borrows: Balance,
        reserves: Balance,
    ) -> Result<Ratio, Error<T>> {
        // utilization ratio is 0 when there are no borrows
        if borrows.is_zero() {
            return Ok(Ratio::zero());
        }
        // utilizationRatio = totalBorrows / (totalCash + totalBorrows − totalReserves)
        let total =
            add_then_sub(cash, borrows, reserves).ok_or(Error::<T>::CalcInterestRateFailed)?;

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
        let interest_accumulated = borrow_rate
            .checked_mul_int(borrows_prior)
            .ok_or(Error::<T>::CalcAccrueInterestFailed)?;
        let total_borrows_new = interest_accumulated
            .checked_add(borrows_prior)
            .ok_or(Error::<T>::CalcAccrueInterestFailed)?;
        let total_reserves_new = reserve_factor
            .mul_floor(interest_accumulated)
            .checked_add(reserve_prior)
            .ok_or(Error::<T>::CalcAccrueInterestFailed)?;
        let borrow_index = Self::borrow_index(currency_id);
        let borrow_index_new = borrow_index
            .checked_mul(&borrow_rate)
            .and_then(|r| r.checked_add(&borrow_index))
            .ok_or(Error::<T>::CalcAccrueInterestFailed)?;

        TotalBorrows::<T>::insert(currency_id, total_borrows_new);
        TotalReserves::<T>::insert(currency_id, total_reserves_new);
        BorrowIndex::<T>::insert(currency_id, borrow_index_new);

        Ok(())
    }

    pub fn get_total_cash(currency_id: CurrencyId) -> Balance {
        T::Currency::free_balance(currency_id, &Self::account_id())
    }
}
