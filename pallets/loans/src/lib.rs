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

use frame_support::{pallet_prelude::*, transactional, PalletId};
use frame_system::pallet_prelude::*;
use orml_traits::{MultiCurrency, MultiCurrencyExtended};
use primitives::{Amount, Balance, CurrencyId, Multiplier, PriceFeeder, Rate, Ratio};
use sp_runtime::traits::Zero;
use sp_runtime::{traits::AccountIdConversion, FixedPointNumber, RuntimeDebug};
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
    /// CurrencyType -> Owner -> BorrowSnapshot
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

    /// The multiplier of utilization rate that gives the slope of the interest rate
    #[pallet::storage]
    #[pallet::getter(fn multipler_per_block)]
    pub type MultiplierPerBlock<T: Config> = StorageValue<_, Multiplier, ValueQuery>;

    /// The base interest rate which is the y-intercept when utilization rate is 0
    #[pallet::storage]
    #[pallet::getter(fn base_rate_per_block)]
    pub type BaseRatePerBlock<T: Config> = StorageValue<_, Rate, ValueQuery>;

    /// The multiplierPerBlock after hitting a specified utilization point
    #[pallet::storage]
    #[pallet::getter(fn jump_multiplier_per_block)]
    pub type JumpMultiplierPerBlock<T: Config> = StorageValue<_, Multiplier, ValueQuery>;

    /// The utilization point at which the jump multiplier is applied
    #[pallet::storage]
    #[pallet::getter(fn kink)]
    pub type Kink<T: Config> = StorageValue<_, Ratio, ValueQuery>;

    /// Mapping of borrow rate to currency type
    #[pallet::storage]
    #[pallet::getter(fn borrow_rate)]
    pub type BorrowRate<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, Rate, ValueQuery>;

    /// Mapping of supply rate to currency type
    #[pallet::storage]
    #[pallet::getter(fn supply_rate)]
    pub type SupplyRate<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, u128, ValueQuery>;

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
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig {
        fn build(&self) {
            self.currencies.iter().for_each(|currency_id| {
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
            Kink::<T>::put(self.kink.clone());
            Currencies::<T>::put(self.currencies.clone());
            Pallet::<T>::init_jump_rate_model(
                self.base_rate_per_year,
                self.multiplier_per_year,
                self.jump_multiplier_per_year,
            )
            .unwrap();
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
            Self::currencies().into_iter().for_each(|currency_id| {
                let total_cash = Self::get_total_cash(currency_id);
                let total_borrows = Self::total_borrows(currency_id);
                let _ = Self::accrue_interest(currency_id);
                let _ = Self::update_supply_rate(currency_id, total_cash, total_borrows, 0, 0);
                let _ = Self::update_exchange_rate(currency_id);
            });
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
    }
}

impl<T: Config> Pallet<T> {
    pub fn account_id() -> T::AccountId {
        T::PalletId::get().into_account()
    }

    pub fn update_exchange_rate(currency_id: CurrencyId) -> DispatchResult {
        // exchangeRate = (totalCash + totalBorrows - totalReserves) / totalSupply
        let total_borrows = Self::total_borrows(currency_id);
        let total_supply = Self::total_supply(currency_id);
        let total_cash = Self::get_total_cash(currency_id);

        let cash_plus_borrows = total_cash
            .checked_add(total_borrows)
            .ok_or(Error::<T>::CalcAccrueInterestFailed)?;
        let exchange_rate = Rate::checked_from_rational(cash_plus_borrows, total_supply)
            .ok_or(Error::<T>::CalcExchangeRateFailed)?;

        ExchangeRate::<T>::insert(currency_id, exchange_rate);

        Ok(())
    }
}
