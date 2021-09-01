#![cfg_attr(not(feature = "std"), no_std)]

use primitives::{Balance, CurrencyId};
use sp_runtime::{traits::Zero, DispatchResult};

use crate::*;

impl<T: Config> Pallet<T> {
    /// Accrue interest per block and update corresponding storage
    pub(crate) fn accrue_interest() -> DispatchResult {
        for (currency_id, market) in Self::active_markets() {
            let total_cash = Self::get_total_cash(currency_id);
            let total_borrows = Self::total_borrows(currency_id);
            let total_reserves = Self::total_reserves(currency_id);
            let util = Self::calc_utilization_ratio(total_cash, total_borrows, total_reserves)?;

            let borrow_rate = market
                .rate_model
                .get_borrow_rate(util)
                .ok_or(ArithmeticError::Overflow)?;
            let supply_rate =
                InterestRateModel::get_supply_rate(borrow_rate, util, market.reserve_factor);

            UtilizationRatio::<T>::insert(currency_id, util);
            BorrowRate::<T>::insert(currency_id, &borrow_rate);
            SupplyRate::<T>::insert(currency_id, supply_rate);

            Self::update_borrow_index(borrow_rate, currency_id, &market)?;
            Self::update_exchange_rate(currency_id)?;
        }

        Ok(())
    }

    /// Update the exchange rate according to the totalCash, totalBorrows and totalSupply.
    ///
    /// exchangeRate = (totalCash + totalBorrows - totalReserves) / totalSupply
    pub(crate) fn update_exchange_rate(currency_id: CurrencyId) -> DispatchResult {
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

    /// Calculate the borrowing utilization ratio of the specified market
    ///
    /// utilizationRatio = totalBorrows / (totalCash + totalBorrows − totalReserves)
    pub(crate) fn calc_utilization_ratio(
        cash: Balance,
        borrows: Balance,
        reserves: Balance,
    ) -> Result<Ratio, DispatchError> {
        // utilization ratio is 0 when there are no borrows
        if borrows.is_zero() {
            return Ok(Ratio::zero());
        }
        let total = cash
            .checked_add(borrows)
            .and_then(|r| r.checked_sub(reserves))
            .ok_or(ArithmeticError::Overflow)?;

        Ok(Ratio::from_rational(borrows, total))
    }

    /// Update the borrow index by borrow rate, the total borrows and
    /// total reserves will be updated simultaneously.
    ///
    /// interestAccumulated = totalBorrows * borrowRate
    /// totalBorrows = interestAccumulated + totalBorrows
    /// totalReserves = interestAccumulated * reserveFactor + totalReserves
    /// borrowIndex = borrowIndex * (1 + borrowRate)
    pub(crate) fn update_borrow_index(
        borrow_rate: Rate,
        currency_id: CurrencyId,
        market: &Market,
    ) -> DispatchResult {
        let borrows_prior = Self::total_borrows(currency_id);
        let reserve_prior = Self::total_reserves(currency_id);
        let delta_time = T::UnixTime::now()
            .as_secs()
            .checked_sub(Self::last_block_timestamp())
            .ok_or(ArithmeticError::Underflow)?;
        let interest_accumulated = accrued_interest(borrow_rate, borrows_prior, delta_time)
            .ok_or(ArithmeticError::Overflow)?;
        let total_borrows_new = interest_accumulated
            .checked_add(borrows_prior)
            .ok_or(ArithmeticError::Overflow)?;
        let total_reserves_new = market
            .reserve_factor
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
}
