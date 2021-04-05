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

use primitives::{Balance, CurrencyId, BLOCK_PER_YEAR, RATE_DECIMAL};
use sp_runtime::{traits::Zero, DispatchResult};

use crate::{util::*, *};

impl<T: Config> Pallet<T> {
    fn insert_borrow_rate(currency_id: CurrencyId, rate: u128) {
        BorrowRate::<T>::insert(currency_id, rate);
        Self::deposit_event(Event::BorrowRateUpdated(currency_id, rate));
    }

    fn insert_supply_rate(currency_id: CurrencyId, rate: u128) {
        SupplyRate::<T>::insert(currency_id, rate);
        Self::deposit_event(Event::SupplyRateUpdated(currency_id, rate));
    }

    pub fn to_decimal(n: Option<u128>) -> Result<u128, Error<T>> {
        n.and_then(|r| r.checked_div(RATE_DECIMAL))
            .ok_or(Error::<T>::CalcInterestRateFailed)
    }

    pub fn utilization_rate(
        cash: Balance,
        borrows: Balance,
        reserves: Balance,
    ) -> Result<u128, Error<T>> {
        // Utilization rate is 0 when there are no borrows
        if borrows.is_zero() {
            return Ok(Zero::zero());
        }

        // utilizationRate = totalBorrows / (totalCash + totalBorrows − totalReserves)
        let total =
            add_then_sub(cash, borrows, reserves).ok_or(Error::<T>::CalcInterestRateFailed)?;

        mul_then_div(borrows, RATE_DECIMAL, total).ok_or(Error::<T>::CalcInterestRateFailed)
    }

    pub fn update_jump_rate_model(
        base_rate_per_year: u128,
        multiplier_per_year: u128,
        jump_multiplier_per_year: u128,
        kink: u128,
    ) -> DispatchResult {
        let base = base_rate_per_year
            .checked_div(BLOCK_PER_YEAR)
            .ok_or(Error::<T>::CalcInterestRateFailed)?;

        let temp = BLOCK_PER_YEAR
            .checked_mul(kink)
            .ok_or(Error::<T>::CalcInterestRateFailed)?;

        let multiplier = mul_then_div(multiplier_per_year, RATE_DECIMAL, temp)
            .ok_or(Error::<T>::CalcInterestRateFailed)?;

        let jump = jump_multiplier_per_year
            .checked_div(BLOCK_PER_YEAR)
            .ok_or(Error::<T>::CalcInterestRateFailed)?;

        BaseRatePerBlock::<T>::put(Some(base));
        MultiplierPerBlock::<T>::put(Some(multiplier));
        JumpMultiplierPerBlock::<T>::put(Some(jump));
        Kink::<T>::put(Some(kink));

        Self::deposit_event(Event::NewInterestParams(base, multiplier, jump, kink));
        Ok(())
    }

    pub fn update_borrow_rate(
        currency_id: CurrencyId,
        cash: Balance,
        borrows: Balance,
        reserves: Balance,
    ) -> DispatchResult {
        let util = Self::utilization_rate(cash, borrows, reserves)?;
        UtilityRate::<T>::insert(currency_id, util);
        Self::deposit_event(Event::UtilityRateUpdated(currency_id, util));

        let multiplier_per_block =
            MultiplierPerBlock::<T>::get().ok_or(Error::<T>::CalcInterestRateFailed)?;
        let base_rate_per_block =
            BaseRatePerBlock::<T>::get().ok_or(Error::<T>::CalcInterestRateFailed)?;
        let kink = Kink::<T>::get().ok_or(Error::<T>::CalcInterestRateFailed)?;
        let jump_multiplier_per_block = Self::to_decimal(JumpMultiplierPerBlock::<T>::get())?;

        if util <= kink {
            let rate = mul_then_div_then_add(
                util,
                multiplier_per_block,
                RATE_DECIMAL,
                base_rate_per_block,
            )
            .ok_or(Error::<T>::CalcInterestRateFailed)?;
            Self::insert_borrow_rate(currency_id, rate);
        } else {
            let normal_rate = mul_then_div_then_add(
                kink,
                multiplier_per_block,
                RATE_DECIMAL,
                base_rate_per_block,
            )
            .ok_or(Error::<T>::CalcInterestRateFailed)?;

            let excess_util = util.saturating_sub(kink);
            let rate = excess_util
                .checked_mul(jump_multiplier_per_block)
                .and_then(|r| r.checked_add(normal_rate))
                .ok_or(Error::<T>::CalcInterestRateFailed)?;

            Self::insert_borrow_rate(currency_id, rate);
        }
        Ok(())
    }

    pub fn update_supply_rate(
        currency_id: CurrencyId,
        cash: Balance,
        borrows: Balance,
        reserves: Balance,
        reserve_factor_mantissa: u128,
    ) -> DispatchResult {
        let one_minus_reserve_factor = RATE_DECIMAL.saturating_sub(reserve_factor_mantissa);

        let borrow_rate = BorrowRate::<T>::get(currency_id);
        let rate_to_pool = Self::to_decimal(borrow_rate.checked_mul(one_minus_reserve_factor))?;

        let rate = Self::to_decimal(
            Self::utilization_rate(cash, borrows, reserves)?.checked_mul(rate_to_pool),
        )?;
        Self::insert_supply_rate(currency_id, rate);

        Ok(())
    }

    pub fn calc_exchange_rate(currency_id: CurrencyId) -> DispatchResult {
        /*
         *  exchangeRate = (totalCash + totalBorrows - totalReserves) / totalSupply
         */
        let total_borrows = Self::total_borrows(currency_id);
        let total_supply = Self::total_supply(currency_id);
        let total_cash = Self::get_total_cash(currency_id);

        let cash_plus_borrows = total_cash
            .checked_add(total_borrows)
            .ok_or(Error::<T>::CalcAccrueInterestFailed)?;
        let exchage_rate = cash_plus_borrows
            .checked_mul(RATE_DECIMAL)
            .and_then(|r| r.checked_div(total_supply))
            .ok_or(Error::<T>::CalcExchangeRateFailed)?;

        ExchangeRate::<T>::insert(currency_id, exchage_rate);

        Ok(())
    }
}
