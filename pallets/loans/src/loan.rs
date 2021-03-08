#![cfg_attr(not(feature = "std"), no_std)]

use frame_system::pallet_prelude::*;
use primitives::{Amount, Balance, CurrencyId};
use sp_runtime::{
    traits::{AccountIdConversion, Zero, CheckedSub},
    DispatchResult, ModuleId, RuntimeDebug, SaturatedConversion,
};
use sp_std::{convert::TryInto, result, vec::{Vec}};
use sp_std::prelude::*;

use crate::*;

const DECIMAL: u128 = 1_000_000_000_000_000_000;

impl<T: Config> Pallet<T> {
    /// This calculates interest accrued from the last checkpointed block
    /// up to the current block and writes new checkpoint to storage.
    pub fn accrue_interest(currency_id: &CurrencyId) -> DispatchResult {
        let current_block_number = frame_system::Module::<T>::block_number();
        let accrual_block_number_prior = Self::accrual_block_number();

        // Short-circuit accumulating 0 interest
        if current_block_number == accrual_block_number_prior {
            return Ok(());
        }

        // Read the previous values out of storage
        let mut total_position: Position = Self::total_positions(currency_id);
        let cash_prior = Self::get_total_cash(currency_id.clone());
        let borrows_prior = total_position.debit;
        let borrow_index_prior = Self::borrow_index();

        // Calculate the current borrow interest rate
        Self::update_borrow_rate(
            currency_id.clone(),
            cash_prior,
            borrows_prior,
            0,
        )?;
        // todo: check borrow rate is absurdly high

        let block_delta = current_block_number.checked_sub(&accrual_block_number_prior)
            .ok_or(Error::<T>::GetBlockDeltaFailed)?;

        /*
         * Calculate the interest accumulated into borrows and reserves and the new index:
         *  simpleInterestFactor = borrowRate * blockDelta
         *  interestAccumulated = simpleInterestFactor * totalBorrows
         *  totalBorrowsNew = interestAccumulated + totalBorrows
         *  totalReservesNew = interestAccumulated * reserveFactor + totalReserves
         *  borrowIndexNew = simpleInterestFactor * borrowIndex + borrowIndex
         */

        let borrow_rate = BorrowRate::<T>::get(currency_id);
        let simple_interest_factor = Self::to_decimal(Some(borrow_rate)).checked_mul(block_delta.saturated_into::<u128>())
            .ok_or(Error::<T>::CalcAccrueInterestFailed)?;
        let interest_accumulated = simple_interest_factor.checked_mul(borrows_prior)
            .ok_or(Error::<T>::CalcAccrueInterestFailed)?;
        let total_borrows_new = interest_accumulated.checked_add(borrows_prior)
            .ok_or(Error::<T>::CalcAccrueInterestFailed)?;
        let borrow_index_new = simple_interest_factor.checked_mul(borrow_index_prior)
            .and_then(|r| r.checked_add(borrow_index_prior)).ok_or(Error::<T>::CalcAccrueInterestFailed)?;

        AccrualBlockNumber::<T>::put(current_block_number);
        BorrowIndex::<T>::put(borrow_index_new);
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
        let exchange_rate = Self::get_exchange_rate_internal(currency_id)?;
        let collateral = mint_amount.checked_mul(DECIMAL)
            .and_then(|r| r.checked_div(exchange_rate)).ok_or(Error::<T>::CalcCollateralFailed)?;

        let collateral_amount = Self::amount_try_from_balance(collateral)?;

        Self::update_loan(who, currency_id.clone(), collateral_amount, 0)?;

        T::Currency::transfer(currency_id.clone(), who, &Self::account_id(), mint_amount)?;

        Ok(())
    }

    fn get_exchange_rate_internal(currency_id: &CurrencyId) -> result::Result<u128, Error<T>> {
        let total_position: Position = Self::total_positions(currency_id);
        if total_position.collateral == 0 {
            /*
             * If there are no tokens minted:
             *  exchangeRate = initialExchangeRate
             */
            Ok(Self::exchange_rate())
        } else {
            /*
             * Otherwise:
             *  exchangeRate = (totalCash + totalBorrows - totalReserves) / totalSupply
             */
            let total_cash = Self::get_total_cash(currency_id.clone());
            let cash_plus_borrows = total_cash.checked_add(total_position.debit)
                .ok_or(Error::<T>::CalcAccrueInterestFailed)?;
            let exchage_rate = cash_plus_borrows.checked_mul(DECIMAL)
                .and_then(|r| r.checked_div(total_position.collateral)).ok_or(Error::<T>::CalcExchangeRateFailed)?;

            Ok(exchage_rate)
        }
    }
}