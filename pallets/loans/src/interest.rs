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

use primitives::{Timestamp, SECONDS_PER_YEAR};
use sp_runtime::{traits::Zero, DispatchResult};

use crate::*;

impl<T: Config> Pallet<T> {
    /// Accrue interest and update corresponding storage
    pub(crate) fn accrue_interest(asset_id: AssetIdOf<T>) -> DispatchResult {
        let now = T::UnixTime::now().as_secs();
        let last_accrued_interest_time = Self::last_accrued_interest_time(asset_id);
        if last_accrued_interest_time.is_zero() {
            // For the initialization
            Self::update_last_accrued_interest_time(asset_id, now)?;
            return Ok(());
        }
        if now <= last_accrued_interest_time {
            return Ok(());
        }
        Self::update_last_accrued_interest_time(asset_id, now)?;
        let delta_time = now - last_accrued_interest_time;
        let market = Self::market(asset_id)?;
        let total_cash = Self::get_total_cash(asset_id);
        let total_borrows = Self::total_borrows(asset_id);
        let total_reserves = Self::total_reserves(asset_id);
        let util = Self::calc_utilization_ratio(total_cash, total_borrows, total_reserves)?;
        let borrow_rate = market
            .rate_model
            .get_borrow_rate(util)
            .ok_or(ArithmeticError::Overflow)?;
        let supply_rate =
            InterestRateModel::get_supply_rate(borrow_rate, util, market.reserve_factor);
        UtilizationRatio::<T>::insert(asset_id, util);
        BorrowRate::<T>::insert(asset_id, &borrow_rate);
        SupplyRate::<T>::insert(asset_id, supply_rate);
        Self::update_borrow_index(borrow_rate, asset_id, &market, delta_time)?;
        Self::update_exchange_rate(asset_id)?;

        Ok(())
    }

    /// Update the exchange rate according to the totalCash, totalBorrows and totalSupply.
    ///
    /// exchangeRate = (totalCash + totalBorrows - totalReserves) / totalSupply
    pub(crate) fn update_exchange_rate(asset_id: AssetIdOf<T>) -> DispatchResult {
        let total_supply = Self::total_supply(asset_id);
        if total_supply.is_zero() {
            return Ok(());
        }
        let total_cash = Self::get_total_cash(asset_id);
        let total_borrows = Self::total_borrows(asset_id);
        let total_reserves = Self::total_reserves(asset_id);

        let cash_plus_borrows_minus_reserves = total_cash
            .checked_add(total_borrows)
            .and_then(|r| r.checked_sub(total_reserves))
            .ok_or(ArithmeticError::Overflow)?;
        let exchange_rate =
            Rate::checked_from_rational(cash_plus_borrows_minus_reserves, total_supply)
                .ok_or(ArithmeticError::Underflow)?;
        Self::ensure_valid_exchange_rate(exchange_rate)?;

        ExchangeRate::<T>::insert(asset_id, exchange_rate);

        Ok(())
    }

    /// Calculate the borrowing utilization ratio of the specified market
    ///
    /// utilizationRatio = totalBorrows / (totalCash + totalBorrows − totalReserves)
    pub(crate) fn calc_utilization_ratio(
        cash: BalanceOf<T>,
        borrows: BalanceOf<T>,
        reserves: BalanceOf<T>,
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
        asset_id: AssetIdOf<T>,
        market: &Market<BalanceOf<T>>,
        delta_time: u64,
    ) -> DispatchResult {
        let borrows_prior = Self::total_borrows(asset_id);
        let reserve_prior = Self::total_reserves(asset_id);
        let interest_accumulated = Self::accrued_interest(borrow_rate, borrows_prior, delta_time)
            .ok_or(ArithmeticError::Overflow)?;
        let total_borrows_new = interest_accumulated
            .checked_add(borrows_prior)
            .ok_or(ArithmeticError::Overflow)?;
        let total_reserves_new = market
            .reserve_factor
            .mul_floor(interest_accumulated)
            .checked_add(reserve_prior)
            .ok_or(ArithmeticError::Overflow)?;
        let borrow_index = Self::borrow_index(asset_id);
        let borrow_index_new = Self::increment_index(borrow_rate, borrow_index, delta_time)
            .and_then(|r| r.checked_add(&borrow_index))
            .ok_or(ArithmeticError::Overflow)?;

        TotalBorrows::<T>::insert(asset_id, total_borrows_new);
        TotalReserves::<T>::insert(asset_id, total_reserves_new);
        BorrowIndex::<T>::insert(asset_id, borrow_index_new);

        Ok(())
    }

    /// The exchange rate should be greater than 0.02 and less than 1
    pub(crate) fn ensure_valid_exchange_rate(exchange_rate: Rate) -> DispatchResult {
        ensure!(
            exchange_rate >= Rate::from_inner(MIN_EXCHANGE_RATE)
                && exchange_rate < Rate::from_inner(MAX_EXCHANGE_RATE),
            Error::<T>::InvalidExchangeRate
        );

        Ok(())
    }

    pub(crate) fn update_last_accrued_interest_time(
        asset_id: AssetIdOf<T>,
        time: Timestamp,
    ) -> DispatchResult {
        LastAccruedInterestTime::<T>::try_mutate(asset_id, |last_time| -> DispatchResult {
            *last_time = time;
            Ok(())
        })
    }

    fn accrued_interest(
        borrow_rate: Rate,
        amount: BalanceOf<T>,
        delta_time: Timestamp,
    ) -> Option<BalanceOf<T>> {
        borrow_rate
            .checked_mul_int(amount)?
            .checked_mul(delta_time.into())?
            .checked_div(SECONDS_PER_YEAR.into())
    }

    fn increment_index(borrow_rate: Rate, index: Rate, delta_time: Timestamp) -> Option<Rate> {
        borrow_rate
            .checked_mul(&index)?
            .checked_mul(&FixedU128::saturating_from_integer(delta_time))?
            .checked_div(&FixedU128::saturating_from_integer(SECONDS_PER_YEAR))
    }
}
