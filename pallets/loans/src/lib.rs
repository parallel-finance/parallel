#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::collapsible_if)]

use frame_support::pallet_prelude::*;
use frame_support::transactional;
use frame_system::pallet_prelude::*;
use orml_traits::{MultiCurrency, MultiCurrencyExtended};
use primitives::{Amount, Balance, CurrencyId, RATE_DECIMAL};
use sp_runtime::{traits::AccountIdConversion, ModuleId, RuntimeDebug};
use sp_std::vec::Vec;

pub use module::*;

mod loan;
mod mock;
mod rate;
mod staking;
mod tests;
mod util;

/// Container for borrow balance information
#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, Default)]
pub struct BorrowSnapshot {
    /// Principal Total balance (with accrued interest), after applying the most recent balance-changing action
    pub principal: Balance,
    /// InterestIndex Global borrowIndex as of the most recent balance-changing action
    pub interest_index: u128,
}

/// Container for earned amount information
#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, Default)]
pub struct EarnedSnapshot {
    /// Total deposit interest, after applying the most recent balance-changing action
    pub total_earned_prior: Balance,
    /// Exchange rate,  after applying the most recent balance-changing action
    pub exchange_rate_prior: u128,
}

#[frame_support::pallet]
pub mod module {
    use super::*;
    use crate::util::mul_then_div;

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
        InsufficientCollateral,
        RepayAmountTooBig,
        AmountConvertFailed,
        InvalidAmountParam,
        GetBlockDeltaFailed,
        CalcAccrueInterestFailed,
        CalcExchangeRateFailed,
        CalcCollateralFailed,
        CalcRedeemBalanceFailed,
        CalcInterestRateFailed,
        CalcBorrowBalanceFailed,
        CalcEarnedFailed,
        MarketNotFresh,
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
        /// repay more than collateral
        RepayValueGreaterThanCollateral,
        /// real repay amount should less than repay amount
        NotEnoughRepayAmount,
        LiquidatorIsBorrower,
        LiquidateAmountIsZero,
        NoBorrowBalance,
        CalcCloseBorrowsFailed,
        CalcDiscdCollateralValueFailed,
        LiquidateValueOverflow,
        EquivalentCollateralAmountOverflow,
        RealCollateralAmountOverflow,
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

        /// Enable/Disable collateral for certain asset
        CollateralAssetAdded(T::AccountId, CurrencyId),
        CollateralAssetRemoved(T::AccountId, CurrencyId),

        /// a collateral has been liquidatied
        /// liquidator, borrower,liquidate_token,collateral_token,liquidate_token_repay_amount,collateral_token_amount
        LiquidationOccur(
            T::AccountId,
            T::AccountId,
            CurrencyId,
            CurrencyId,
            Balance,
            Balance,
        ),

        Test(u128),
    }

    // Loan storage
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

    /// Mapping of account addresses to total deposit interest accrual
    /// CollateralType -> Owner -> BorrowSnapshot
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

    #[pallet::storage]
    #[pallet::getter(fn account_collateral_assets)]
    pub type AccountCollateralAssets<T: Config> =
        StorageMap<_, Twox64Concat, T::AccountId, Vec<CurrencyId>, ValueQuery>;

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
    #[pallet::storage]
    #[pallet::getter(fn collateral_rate)]
    pub type CollateralRate<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, u128, ValueQuery>;
    #[pallet::storage]
    #[pallet::getter(fn liquidation_incentive)]
    pub type LiquidationIncentive<T: Config> =
        StorageMap<_, Twox64Concat, CurrencyId, u128, ValueQuery>;
    #[pallet::storage]
    #[pallet::getter(fn liquidation_threshold)]
    pub type LiquidationThreshold<T: Config> =
        StorageMap<_, Twox64Concat, CurrencyId, u128, ValueQuery>;
    #[pallet::storage]
    #[pallet::getter(fn close_factor)]
    pub type CloseFactor<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, u128, ValueQuery>;

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
        pub collateral_rate: Vec<(CurrencyId, u128)>,
        pub liquidation_incentive: Vec<(CurrencyId, u128)>,
        pub liquidation_threshold: Vec<(CurrencyId, u128)>,
        pub close_factor: Vec<(CurrencyId, u128)>,
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
                collateral_rate: vec![],
                liquidation_incentive: vec![],
                liquidation_threshold: vec![],
                close_factor: vec![],
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig {
        fn build(&self) {
            T::Currency::update_balance(
                CurrencyId::xDOT,
                &Pallet::<T>::account_id(),
                1_000_000_000_000_000_000_000_000_000_000,
            )
            .unwrap();
            self.currencies.iter().for_each(|currency_id| {
                TotalSupply::<T>::insert(currency_id, self.total_supply);
                TotalBorrows::<T>::insert(currency_id, self.total_borrows);
                ExchangeRate::<T>::insert(currency_id, self.exchange_rate);
                BorrowIndex::<T>::insert(currency_id, self.borrow_index);
            });
            self.collateral_rate
                .iter()
                .for_each(|(currency_id, collateral_rate)| {
                    CollateralRate::<T>::insert(currency_id, collateral_rate);
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
            Currencies::<T>::put(self.currencies.clone());
            Pallet::<T>::update_jump_rate_model(
                self.base_rate,
                self.multiplier_per_year,
                self.jump_muiltiplier,
                self.kink,
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
                let _ = Self::calc_exchange_rate(currency_id);
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
            let redeem_amount = mul_then_div(collateral, exchange_rate, RATE_DECIMAL)
                .ok_or(Error::<T>::CollateralOverflow)?;
            Self::redeem_internal(&who, &currency_id, redeem_amount.into())?;
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
        pub fn stake(origin: OriginFor<T>, amount: Balance) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::stake_internal(&who, amount)?;

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        #[transactional]
        pub fn unstake(origin: OriginFor<T>, amount: Balance) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::unstake_internal(&who, amount)?;

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
        T::ModuleId::get().into_account()
    }
}
