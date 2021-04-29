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

use primitives::{Multiplier, Rate, Ratio, BLOCK_PER_YEAR};
use sp_runtime::traits::{CheckedAdd, CheckedDiv, Saturating};

use crate::*;

/// Parallel interest rate model
#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, Default)]
pub struct InterestRateModel {
    /// The base interest rate which is the y-intercept when utilization rate is 0
    pub base_rate_per_block: Rate,
    /// The multiplier of utilization rate that gives the slope of the interest rate
    pub multiplier_per_block: Multiplier,
    /// The multiplierPerBlock after hitting a specified utilization point
    pub jump_multiplier_per_block: Multiplier,
    /// The utilization point at which the jump multiplier is applied
    pub kink: Ratio,
}

impl InterestRateModel {
    /// Initialize the interest rate model
    pub fn init_model(
        base_rate_per_year: Rate,
        multiplier_per_year: Multiplier,
        jump_multiplier_per_year: Multiplier,
        kink: Ratio,
    ) -> Option<InterestRateModel> {
        let base_rate_per_block =
            base_rate_per_year.checked_div(&Rate::saturating_from_integer(BLOCK_PER_YEAR))?;
        let multiplier_per_block =
            multiplier_per_year.checked_div(&Rate::saturating_from_integer(BLOCK_PER_YEAR))?;
        let jump_multiplier_per_block =
            jump_multiplier_per_year.checked_div(&Rate::saturating_from_integer(BLOCK_PER_YEAR))?;

        Some(Self {
            base_rate_per_block,
            multiplier_per_block,
            jump_multiplier_per_block,
            kink,
        })
    }

    /// Calculates the current borrow interest rate per block
    pub fn get_borrow_rate(&self, util: Ratio) -> Option<Rate> {
        if util <= self.kink {
            // borrowRate = multiplier * utilizationRatio + baseRate
            self.multiplier_per_block
                .saturating_mul(util.into())
                .checked_add(&self.base_rate_per_block)
        } else {
            // borrowRate = (multiplier * kink + baseRate) + (jumpMultiplier * (utilizationRatio - kink))
            let normal_rate = self
                .multiplier_per_block
                .saturating_mul(self.kink.into())
                .checked_add(&self.base_rate_per_block)?;
            let excess_util = util.saturating_sub(self.kink);

            self.jump_multiplier_per_block
                .saturating_mul(excess_util.into())
                .checked_add(&normal_rate)
        }
    }

    /// Calculates the current supply interest rate per block
    pub fn get_supply_rate(borrow_rate: Rate, util: Ratio, reserve_factor: Ratio) -> Option<Rate> {
        // supplyRate = ((1 - reserveFactor) * borrowRate) * utilizationRatio
        let one_minus_reserve_factor = Ratio::one().saturating_sub(reserve_factor);
        let rate_to_pool = borrow_rate.saturating_mul(one_minus_reserve_factor.into());

        Some(rate_to_pool.saturating_mul(util.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_model_works() {
        let base_rate_per_year = Rate::saturating_from_rational(2, 100);
        let multiplier_per_year = Multiplier::saturating_from_rational(1, 10);
        let jump_multiplier_per_year = Multiplier::saturating_from_rational(11, 10);
        let kink = Ratio::from_percent(80);

        assert_eq!(
            InterestRateModel::init_model(
                base_rate_per_year,
                multiplier_per_year,
                jump_multiplier_per_year,
                kink
            ),
            Some(InterestRateModel {
                base_rate_per_block: Rate::saturating_from_rational(2, 100 * BLOCK_PER_YEAR),
                multiplier_per_block: Multiplier::saturating_from_rational(1, 10 * BLOCK_PER_YEAR),
                jump_multiplier_per_block: Multiplier::saturating_from_rational(
                    11,
                    10 * BLOCK_PER_YEAR
                ),
                kink: Ratio::from_percent(80),
            })
        );
    }

    #[test]
    fn get_borrow_rate_works() {
        // init
        let base_rate_per_year = Rate::saturating_from_rational(2, 100);
        let multiplier_per_year = Multiplier::saturating_from_rational(1, 10);
        let jump_multiplier_per_year = Multiplier::saturating_from_rational(11, 10);
        let kink = Ratio::from_percent(80);
        let interest_model = InterestRateModel::init_model(
            base_rate_per_year,
            multiplier_per_year,
            jump_multiplier_per_year,
            kink,
        )
        .unwrap();

        // normal rate
        let mut cash: u128 = 500;
        let borrows: u128 = 1000;
        let util = Ratio::from_rational(borrows, cash + borrows);
        let borrow_rate = interest_model.get_borrow_rate(util).unwrap();
        assert_eq!(
            borrow_rate,
            interest_model
                .multiplier_per_block
                .saturating_mul(util.into())
                + interest_model.base_rate_per_block,
        );

        // jump rate
        cash = 100;
        let util = Ratio::from_rational(borrows, cash + borrows);
        let borrow_rate = interest_model.get_borrow_rate(util).unwrap();
        let normal_rate = interest_model
            .multiplier_per_block
            .saturating_mul(kink.into())
            + interest_model.base_rate_per_block;
        let excess_util = util.saturating_sub(kink);
        assert_eq!(
            borrow_rate,
            interest_model
                .jump_multiplier_per_block
                .saturating_mul(excess_util.into())
                + normal_rate,
        );
    }

    #[test]
    fn get_supply_rate_works() {
        let borrow_rate = Rate::saturating_from_rational(2, 100 * BLOCK_PER_YEAR);
        let util = Ratio::from_percent(50);
        let reserve_factor = Ratio::zero();
        let supply_rate =
            InterestRateModel::get_supply_rate(borrow_rate, util, reserve_factor).unwrap();
        assert_eq!(
            supply_rate,
            borrow_rate
                .saturating_mul(((Ratio::one().saturating_sub(reserve_factor)) * util).into()),
        );
    }
}
