#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::collapsible_if)]

use frame_system::pallet_prelude::*;
use frame_support::pallet_prelude::*;
use frame_support::transactional;
use orml_traits::{Happened, MultiCurrency, MultiCurrencyExtended};
use primitives::{Amount, Balance, CurrencyId};
use sp_runtime::{
    traits::{AccountIdConversion, Convert, Zero},
    DispatchResult, ModuleId, RuntimeDebug,
};
use sp_std::{convert::TryInto, result};

pub use module::*;

/// A collateralized debit position.
#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, Default)]
pub struct Position {
    /// The amount of collateral.
    pub collateral: Balance,
    /// The amount of debit.
    pub debit: Balance,
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
        AmountConvertFailed,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
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
    }

    /// The collateralized debit positions, map from
    /// Owner -> CollateralType -> Position
    #[pallet::storage]
    #[pallet::getter(fn positions)]
    pub type Positions<T: Config> =
        StorageDoubleMap<_, Twox64Concat, CurrencyId, Twox64Concat, T::AccountId, Position, ValueQuery>;

    /// The total collateralized debit positions, map from
    /// CollateralType -> Position
    #[pallet::storage]
    #[pallet::getter(fn total_positions)]
    pub type TotalPositions<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, Position, ValueQuery>;

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

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

        let collateral_balance_adjustment = Self::balance_try_from_amount_abs(collateral_adjustment)?;
        let debit_balance_adjustment = Self::balance_try_from_amount_abs(debit_adjustment)?;
        let module_account = Self::account_id();

        if collateral_adjustment.is_positive() {
            T::Currency::transfer(currency_id, who, &module_account, collateral_balance_adjustment)?;
        } else if collateral_adjustment.is_negative() {
            T::Currency::transfer(currency_id, &module_account, who, collateral_balance_adjustment)?;
        }

        Self::deposit_event(Event::PositionUpdated(
            who.clone(),
            currency_id,
            collateral_adjustment,
            debit_adjustment,
        ));
        Ok(())
    }

    /// mutate records of collaterals and debits
    fn update_loan(
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
                p.debit.checked_add(debit_balance).ok_or(Error::<T>::DebitOverflow)
            } else {
                p.debit.checked_sub(debit_balance).ok_or(Error::<T>::DebitTooLow)
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

impl<T: Config> Pallet<T> {
    /// Convert `Balance` to `Amount`.
    fn amount_try_from_balance(b: Balance) -> result::Result<Amount, Error<T>> {
        TryInto::<Amount>::try_into(b).map_err(|_| Error::<T>::AmountConvertFailed)
    }

    /// Convert the absolute value of `Amount` to `Balance`.
    fn balance_try_from_amount_abs(a: Amount) -> result::Result<Balance, Error<T>> {
        TryInto::<Balance>::try_into(a.saturating_abs()).map_err(|_| Error::<T>::AmountConvertFailed)
    }
}