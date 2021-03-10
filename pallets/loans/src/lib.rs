#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::collapsible_if)]

use frame_support::pallet_prelude::*;
use frame_support::transactional;
use frame_system::pallet_prelude::*;
use orml_traits::{MultiCurrency, MultiCurrencyExtended};
use primitives::{Amount, Balance, CurrencyId};
use sp_runtime::{
    traits::{AccountIdConversion, Zero},
    DispatchResult, ModuleId, RuntimeDebug,
};
use sp_std::vec::Vec;

pub use module::*;

mod loan;
mod rate;
mod util;

/// A collateralized debit position.
#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, Default)]
pub struct Position {
    /// The amount of collateral.
    pub collateral: Balance,
    /// The amount of debit.
    pub debit: Balance,
}

/// Container for borrow balance information
#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, Default)]
pub struct BorrowSnapshot {
    /// Principal Total balance (with accrued interest), after applying the most recent balance-changing action
    pub principal: Balance,
    /// InterestIndex Global borrowIndex as of the most recent balance-changing action
    pub interest_index: u128,
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

        /// The loan's module id, keep all collaterals of CDPs.
        #[pallet::constant]
        type ModuleId: Get<ModuleId>;
    }

    #[pallet::error]
    pub enum Error<T> {
        DebitOverflow,
        DebitTooLow,
        CollateralOverflow,
        CollateralTooLow,
        InsufficientCash,
        RepayAmountTooBig,
        AmountConvertFailed,
        GetBlockDeltaFailed,
        CalcAccrueInterestFailed,
        CalcExchangeRateFailed,
        CalcCollateralFailed,
        CalcInterestRateFailed,
        CalcBorrowBalanceFailed,
        MarketNotFresh,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub (crate) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Position updated. \[owner, collateral_type, collateral_adjustment,
        /// debit_adjustment\]
        PositionUpdated(T::AccountId, CurrencyId, Amount, Amount),
        /// Confiscate CDP's collateral assets and eliminate its debit. \[owner,
        /// collateral_type, confiscated_collateral_amount,
        /// deduct_debit_amount\]
        ConfiscateCollateralAndDebit(T::AccountId, CurrencyId, Balance, Balance),
        /// Transfer loan. \[from, to, currency_id\]
        TransferLoan(T::AccountId, T::AccountId, CurrencyId),

        AccrueInterest(CurrencyId),

        NewInterestParams2(T::AccountId, u128, u128, u128, u128),
        NewInterestParams(u128, u128, u128, u128),
        BorrowRateUpdated(CurrencyId, u128),
        SupplyRateUpdated(CurrencyId, u128),
        UtilityRateUpdated(CurrencyId, u128),
        Test(u128),
    }

    // Loan storage
    /// The collateralized debit positions, map from
    /// Owner -> CollateralType -> Position
    #[pallet::storage]
    #[pallet::getter(fn positions)]
    pub type Positions<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        CurrencyId,
        Twox64Concat,
        T::AccountId,
        Position,
        ValueQuery,
    >;

    /// The total collateralized debit positions, map from
    /// CollateralType -> Position
    #[pallet::storage]
    #[pallet::getter(fn total_positions)]
    pub type TotalPositions<T: Config> =
        StorageMap<_, Twox64Concat, CurrencyId, Position, ValueQuery>;

    // new design
    /// Total number of collateral tokens in circulation
    /// CollateralType -> Balance
    #[pallet::storage]
    #[pallet::getter(fn total_supply)]
    pub type TotalSupply<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, Balance, ValueQuery>;

    /// Total amount of outstanding borrows of the underlying in this market
    /// CollateralType -> Balance
    #[pallet::storage]
    #[pallet::getter(fn total_borrows)]
    pub type TotalBorrows<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, Balance, ValueQuery>;

    /// Mapping of account addresses to outstanding borrow balances
    /// CollateralType -> Owner -> BorrowSnapshot
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

    #[pallet::storage]
    #[pallet::getter(fn borrow_index)]
    pub type BorrowIndex<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, u128, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn currencies)]
    pub type Currencies<T: Config> = StorageValue<_, Vec<CurrencyId>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn exchange_rate)]
    pub type ExchangeRate<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, u128, ValueQuery>;

    // Rate storage
    #[pallet::storage]
    #[pallet::getter(fn multipler_per_block)]
    pub type MultiplierPerBlock<T: Config> = StorageValue<_, Option<u128>, ValueQuery>;
    #[pallet::storage]
    #[pallet::getter(fn base_rate_per_block)]
    pub type BaseRatePerBlock<T: Config> = StorageValue<_, Option<u128>, ValueQuery>;
    #[pallet::storage]
    #[pallet::getter(fn jump_multiplier_per_block)]
    pub type JumpMultiplierPerBlock<T: Config> = StorageValue<_, Option<u128>, ValueQuery>;
    #[pallet::storage]
    #[pallet::getter(fn kink)]
    pub type Kink<T: Config> = StorageValue<_, Option<u128>, ValueQuery>;
    #[pallet::storage]
    #[pallet::getter(fn borrow_rate)]
    pub type BorrowRate<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, u128, ValueQuery>;
    #[pallet::storage]
    #[pallet::getter(fn supply_rate)]
    pub type SupplyRate<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, u128, ValueQuery>;
    #[pallet::storage]
    #[pallet::getter(fn utility_rate)]
    pub type UtilityRate<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, u128, ValueQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig {
        pub currencies: Vec<CurrencyId>,
        pub total_supply: Balance,
        pub total_borrows: Balance,
        pub borrow_index: u128,
        pub exchange_rate: u128,
        pub base_rate: u128,
        pub multiplier_per_year: u128,
        pub jump_muiltiplier: u128,
        pub kink: u128,
    }

    #[cfg(feature = "std")]
    impl Default for GenesisConfig {
        fn default() -> Self {
            GenesisConfig {
                currencies: vec![],
                total_supply: 0,
                total_borrows: 0,
                borrow_index: 0,
                exchange_rate: 0,
                base_rate: 0,
                multiplier_per_year: 0,
                jump_muiltiplier: 0,
                kink: 0,
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig {
        fn build(&self) {
            self.currencies.iter().for_each(|currency_id| {
                TotalSupply::<T>::insert(currency_id, self.total_supply);
                TotalBorrows::<T>::insert(currency_id, self.total_borrows);
                ExchangeRate::<T>::insert(currency_id, self.exchange_rate);
                BorrowIndex::<T>::insert(currency_id, self.borrow_index);
            });
            Currencies::<T>::put(self.currencies.clone());
            Pallet::<T>::update_jump_rate_model(
                self.base_rate,
                self.multiplier_per_year,
                self.jump_muiltiplier,
                self.kink,
            );
        }
    }

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
        fn on_finalize(_now: T::BlockNumber) {
            Self::currencies().iter().for_each(|currency_id| {
                let total_position: Position = Self::total_positions(currency_id);
                let total_cash = Self::get_total_cash(currency_id.clone());

                Self::accrue_interest(currency_id);
                Self::update_supply_rate(
                    *currency_id,
                    total_cash,
                    total_position.debit,
                    0,
                    1 * rate::DECIMAL,
                );
                Self::calc_exchange_rate(currency_id);
            });
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        #[transactional]
        pub fn adjust_loan(
            origin: OriginFor<T>,
            currency_id: CurrencyId,
            collateral_adjustment: Amount,
            debit_adjustment: Amount,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::adjust_position(&who, currency_id, collateral_adjustment, debit_adjustment)?;
            Ok(().into())
        }

        #[pallet::weight(10_000)]
        #[transactional]
        pub fn mint_collateral(
            origin: OriginFor<T>,
            currency_id: CurrencyId,
            mint_amount: Balance,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
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
    }
}

impl<T: Config> Pallet<T> {
    pub fn account_id() -> T::AccountId {
        T::ModuleId::get().into_account()
    }

    /// adjust the position.
    ///
    /// Ensured atomic.
    #[transactional]
    pub fn adjust_position(
        who: &T::AccountId,
        currency_id: CurrencyId,
        collateral_adjustment: Amount,
        debit_adjustment: Amount,
    ) -> DispatchResult {
        // mutate collateral and debit
        Self::update_loan(who, currency_id, collateral_adjustment, debit_adjustment)?;

        let collateral_balance_adjustment =
            Self::balance_try_from_amount_abs(collateral_adjustment)?;
        let debit_balance_adjustment = Self::balance_try_from_amount_abs(debit_adjustment)?;
        let module_account = Self::account_id();

        if collateral_adjustment.is_positive() {
            T::Currency::transfer(
                currency_id,
                who,
                &module_account,
                collateral_balance_adjustment,
            )?;
        } else if collateral_adjustment.is_negative() {
            T::Currency::transfer(
                currency_id,
                &module_account,
                who,
                collateral_balance_adjustment,
            )?;
        }

        if debit_adjustment.is_positive() {
            T::Currency::transfer(currency_id, &module_account, who, debit_balance_adjustment)?;
        } else if debit_adjustment.is_negative() {
            T::Currency::transfer(currency_id, who, &module_account, debit_balance_adjustment)?;
        }

        Ok(())
    }

    /// mutate records of collaterals and debits
    pub fn update_loan(
        who: &T::AccountId,
        currency_id: CurrencyId,
        collateral_adjustment: Amount,
        debit_adjustment: Amount,
    ) -> DispatchResult {
        let collateral_balance = Self::balance_try_from_amount_abs(collateral_adjustment)?;
        let debit_balance = Self::balance_try_from_amount_abs(debit_adjustment)?;

        <Positions<T>>::try_mutate_exists(currency_id, who, |may_be_position| -> DispatchResult {
            let mut p = may_be_position.take().unwrap_or_default();
            let new_collateral = if collateral_adjustment.is_positive() {
                p.collateral
                    .checked_add(collateral_balance)
                    .ok_or(Error::<T>::CollateralOverflow)
            } else {
                p.collateral
                    .checked_sub(collateral_balance)
                    .ok_or(Error::<T>::CollateralTooLow)
            }?;
            let new_debit = if debit_adjustment.is_positive() {
                p.debit
                    .checked_add(debit_balance)
                    .ok_or(Error::<T>::DebitOverflow)
            } else {
                p.debit
                    .checked_sub(debit_balance)
                    .ok_or(Error::<T>::DebitTooLow)
            }?;

            p.collateral = new_collateral;
            p.debit = new_debit;

            if p.collateral.is_zero() && p.debit.is_zero() {
                // decrease account ref if zero position
                frame_system::Module::<T>::dec_consumers(who);

                // remove position storage if zero position
                *may_be_position = None;
            } else {
                *may_be_position = Some(p);
            }

            Ok(())
        })?;

        TotalPositions::<T>::try_mutate(currency_id, |total_positions| -> DispatchResult {
            total_positions.collateral = if collateral_adjustment.is_positive() {
                total_positions
                    .collateral
                    .checked_add(collateral_balance)
                    .ok_or(Error::<T>::CollateralOverflow)
            } else {
                total_positions
                    .collateral
                    .checked_sub(collateral_balance)
                    .ok_or(Error::<T>::CollateralTooLow)
            }?;

            total_positions.debit = if debit_adjustment.is_positive() {
                total_positions
                    .debit
                    .checked_add(debit_balance)
                    .ok_or(Error::<T>::DebitOverflow)
            } else {
                total_positions
                    .debit
                    .checked_sub(debit_balance)
                    .ok_or(Error::<T>::DebitTooLow)
            }?;

            Ok(())
        })
    }
}
