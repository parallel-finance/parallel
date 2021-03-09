#![cfg_attr(not(feature = "std"), no_std)]

use frame_system::pallet_prelude::*;
use primitives::{Amount, Balance, CurrencyId};
use sp_runtime::{
    traits::{AccountIdConversion, Zero, CheckedSub},
    DispatchResult, ModuleId, RuntimeDebug, SaturatedConversion,
};
use sp_std::{convert::TryInto, result, vec::Vec};
use sp_std::prelude::*;

use crate::*;

const DECIMAL: u128 = 1_000_000_000_000_000_000;

impl<T: Config> Pallet<T> {
    /// This calculates interest accrued from the last checkpointed block
    /// up to the current block and writes new checkpoint to storage.
    pub fn accrue_interest(currency_id: &CurrencyId) -> DispatchResult {
        // Read the previous values out of storage
        let mut total_position: Position = Self::total_positions(currency_id);
        let cash_prior = Self::get_total_cash(currency_id.clone());
        let borrows_prior = total_position.debit;

        // Calculate the current borrow interest rate
        Self::update_borrow_rate(
            currency_id.clone(),
            cash_prior,
            borrows_prior,
            0,
        )?;

         /*
         * Compound protocol:
         * Calculate the interest accumulated into borrows and reserves and the new index:
         *  simpleInterestFactor = borrowRate * blockDelta
         *  interestAccumulated = simpleInterestFactor * totalBorrows
         *  totalBorrowsNew = interestAccumulated + totalBorrows
         *  totalReservesNew = interestAccumulated * reserveFactor + totalReserves
         *  borrowIndexNew = simpleInterestFactor * borrowIndex + borrowIndex
         */

        let borrow_rate_per_block = BorrowRate::<T>::get(currency_id);
        let interest_accumulated = borrow_rate_per_block.checked_mul(borrows_prior)
            .and_then(|r| r.checked_div(DECIMAL)).ok_or(Error::<T>::CalcAccrueInterestFailed)?;
        let total_borrows_new = interest_accumulated.checked_add(borrows_prior)
            .ok_or(Error::<T>::CalcAccrueInterestFailed)?;

        total_position.debit = total_borrows_new;
        TotalPositions::<T>::insert(currency_id, total_position);

        Self::deposit_event(Event::AccrueInterest(
            currency_id.clone(),
        ));

        Ok(())
    }

    pub fn get_total_cash(currency_id: CurrencyId) -> Balance {
        T::Currency::free_balance(currency_id, &Self::account_id())
    }

    /// mint collateral
    ///
    /// Ensured atomic.
    #[transactional]
    pub fn mint_internal(who: &T::AccountId, currency_id: &CurrencyId, mint_amount: Balance) -> DispatchResult {
        let exchange_rate = Self::exchange_rate(currency_id);
        let collateral = mint_amount.checked_mul(DECIMAL)
            .and_then(|r| r.checked_div(exchange_rate)).ok_or(Error::<T>::CalcCollateralFailed)?;

        let collateral_amount = Self::amount_try_from_balance(collateral)?;

        Self::update_loan(who, currency_id.clone(), collateral_amount, 0)?;

        T::Currency::transfer(currency_id.clone(), who, &Self::account_id(), mint_amount)?;

        Ok(())
    }

    /// mint collateral
    ///
    /// Ensured atomic.
    #[transactional]
    pub fn redeem_internal(who: &T::AccountId, currency_id: &CurrencyId, redeem_amount: Balance) -> DispatchResult {
        let exchange_rate = Self::exchange_rate(currency_id);
        let collateral = redeem_amount.checked_mul(DECIMAL)
            .and_then(|r| r.checked_div(exchange_rate)).ok_or(Error::<T>::CalcCollateralFailed)?;

        let collateral_amount = Self::amount_try_from_balance(collateral)?;

        Self::update_loan(who, currency_id.clone(), -collateral_amount, 0)?;

        T::Currency::transfer(currency_id.clone(), &Self::account_id(), who, redeem_amount)?;

        Ok(())
    }
}