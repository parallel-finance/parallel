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

use primitives::{Balance, CurrencyId, Multiplier, Rate, Ratio, BLOCK_PER_YEAR, RATE_DECIMAL};
use sp_runtime::{
    traits::{CheckedAdd, Saturating, Zero},
    DispatchResult, Perbill,
};

use crate::{util::*, *};

impl<T: Config> Pallet<T> {
    pub fn calc_utilization_ratio(
        cash: Balance,
        borrows: Balance,
        reserves: Balance,
    ) -> Result<Ratio, Error<T>> {
        // utilization rate is 0 when there are no borrows
        if borrows.is_zero() {
            return Ok(Ratio::zero());
        }
        // utilizationRate = totalBorrows / (totalCash + totalBorrows − totalReserves)
        let total =
            add_then_sub(cash, borrows, reserves).ok_or(Error::<T>::CalcInterestRateFailed)?;

        Ok(Ratio::from_rational(borrows, total))
    }

    pub fn init_jump_rate_model(
        base_rate_per_year: Rate,
        multiplier_per_year: Multiplier,
        jump_multiplier_per_year: Multiplier,
    ) -> DispatchResult {
        let base_rate_per_block =
            base_rate_per_year.saturating_mul(Perbill::from_rational(1, BLOCK_PER_YEAR).into());
        let multiplier_per_block =
            multiplier_per_year.saturating_mul(Perbill::from_rational(1, BLOCK_PER_YEAR).into());
        let jump_multiplier_per_block = jump_multiplier_per_year
            .saturating_mul(Perbill::from_rational(1, BLOCK_PER_YEAR).into());

        BaseRatePerBlock::<T>::put(base_rate_per_block);
        MultiplierPerBlock::<T>::put(multiplier_per_block);
        JumpMultiplierPerBlock::<T>::put(jump_multiplier_per_block);

        Self::deposit_event(Event::InitInterestRateModel(
            base_rate_per_block,
            multiplier_per_block,
            jump_multiplier_per_block,
        ));
        Ok(())
    }

    pub fn update_borrow_rate(
        currency_id: CurrencyId,
        cash: Balance,
        borrows: Balance,
        reserves: Balance,
    ) -> DispatchResult {
        let util = Self::calc_utilization_ratio(cash, borrows, reserves)?;
        UtilizationRatio::<T>::insert(currency_id, util);

        let multiplier_per_block = MultiplierPerBlock::<T>::get();
        let base_rate_per_block = BaseRatePerBlock::<T>::get();
        let jump_multiplier_per_block = JumpMultiplierPerBlock::<T>::get();
        let kink = Kink::<T>::get();

        if util <= kink {
            let rate = multiplier_per_block
                .saturating_mul(util.into())
                .checked_add(&base_rate_per_block)
                .ok_or(Error::<T>::CalcInterestRateFailed)?;
            BorrowRate::<T>::insert(currency_id, rate);
        } else {
            let normal_rate = multiplier_per_block
                .saturating_mul(kink.into())
                .checked_add(&base_rate_per_block)
                .ok_or(Error::<T>::CalcInterestRateFailed)?;
            let excess_util = util.saturating_sub(kink);
            let rate = jump_multiplier_per_block
                .saturating_mul(excess_util.into())
                .checked_add(&normal_rate)
                .ok_or(Error::<T>::CalcInterestRateFailed)?;
            BorrowRate::<T>::insert(currency_id, rate);
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
        let rate_to_pool = borrow_rate
            .checked_mul_int(one_minus_reserve_factor)
            .ok_or(Error::<T>::CalcInterestRateFailed)?;
        let util = Self::calc_utilization_ratio(cash, borrows, reserves)?;
        SupplyRate::<T>::insert(currency_id, util * rate_to_pool);
        Ok(())
    }
}
