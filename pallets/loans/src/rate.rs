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

use primitives::{Rate, Ratio, BLOCK_PER_YEAR};
use sp_runtime::traits::{CheckedAdd, CheckedDiv, CheckedSub, Saturating};

use crate::*;

/// Error enum for interest rates
#[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub enum RatesError {
    ModelRateOutOfBounds,
    BaseAboveKink,
    KinkAboveFull,
    KinkUtilizationTooHigh,
    Overflowed,
}

/// Annualized interest rate
#[derive(Encode, Decode, Eq, PartialEq, PartialOrd, Copy, Clone, RuntimeDebug, Default)]
#[allow(clippy::upper_case_acronyms)]
pub struct APR(pub Rate);

impl From<Rate> for APR {
    fn from(x: Rate) -> Self {
        APR(x)
    }
}

impl APR {
    pub const MAX: Rate = Rate::from_inner(350_000_000_000_000_000); // 35%

    pub fn rate_per_block(&self) -> Option<Rate> {
        self.0
            .checked_div(&Rate::saturating_from_integer(BLOCK_PER_YEAR))
    }

    pub fn accrued_interest_per_block(&self, amount: u128) -> Option<u128> {
        self.0.checked_mul_int(amount)?.checked_div(BLOCK_PER_YEAR)
    }

    pub fn increment_index_per_block(&self, index: Rate) -> Option<Rate> {
        self.0
            .checked_mul(&index)?
            .checked_div(&Rate::saturating_from_integer(BLOCK_PER_YEAR))
    }
}

/// Parallel interest rate model
#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, Default)]
pub struct InterestRateModel {
    /// The base interest rate which is the y-intercept when utilization rate is 0
    pub base_rate: APR,
    /// The multiplier of utilization rate that gives the slope of the interest rate
    pub kink_rate: APR,
    /// The multiplier after hitting a specified utilization point
    pub full_rate: APR,
    /// The utilization point at which the full_rate is applied
    pub kink_utilization: Ratio,
}

impl InterestRateModel {
    /// Create a new rate model
    pub fn new_model(
        base_rate: Rate,
        kink_rate: Rate,
        full_rate: Rate,
        kink_utilization: Ratio,
    ) -> InterestRateModel {
        Self {
            base_rate: base_rate.into(),
            kink_rate: kink_rate.into(),
            full_rate: full_rate.into(),
            kink_utilization,
        }
    }

    /// Check the model parameters for sanity
    pub fn check_parameters(&self) -> Result<(), RatesError> {
        if self.base_rate.0 > APR::MAX || self.kink_rate.0 > APR::MAX || self.full_rate.0 > APR::MAX
        {
            return Err(RatesError::ModelRateOutOfBounds);
        }
        if self.base_rate.0 >= self.kink_rate.0 {
            return Err(RatesError::BaseAboveKink);
        }
        if self.kink_rate.0 >= self.full_rate.0 {
            return Err(RatesError::KinkAboveFull);
        }
        if self.kink_utilization >= Ratio::one() {
            return Err(RatesError::KinkUtilizationTooHigh);
        }

        Ok(())
    }

    /// The borrow rate when utilization is between 0 and kink_utilization.
    fn base_to_kink(&self, utilization: Ratio) -> Option<APR> {
        // utilization * (kink_rate - zero_rate) / kink_utilization + zero_rate
        let result = self
            .kink_rate
            .0
            .checked_sub(&self.base_rate.0)?
            .saturating_mul(utilization.into())
            .checked_div(&self.kink_utilization.into())?
            .checked_add(&self.base_rate.0)?;

        Some(result.into())
    }

    /// The borrow rate when utilization is between kink_utilization and 100%.
    fn kink_to_full(&self, utilization: Ratio) -> Option<APR> {
        // (utilization - kink_utilization)*(full_rate - kink_rate) / ( 1 - kink_utilization) + kink_rate
        let excess_util = utilization.saturating_sub(self.kink_utilization);
        let result = self
            .full_rate
            .0
            .checked_sub(&self.kink_rate.0)?
            .saturating_mul(excess_util.into())
            .checked_div(&(Ratio::one().saturating_sub(self.kink_utilization).into()))?
            .checked_add(&self.kink_rate.0)?;

        Some(result.into())
    }

    /// Calculates the current borrow APR
    pub fn get_borrow_rate(&self, utilization: Ratio) -> Result<APR, RatesError> {
        if utilization <= self.kink_utilization {
            let result = self
                .base_to_kink(utilization)
                .ok_or(RatesError::Overflowed)?;

            Ok(result)
        } else {
            let result = self
                .kink_to_full(utilization)
                .ok_or(RatesError::Overflowed)?;

            Ok(result)
        }
    }

    /// Calculates the current supply APR
    pub fn get_supply_rate(
        borrow_rate: APR,
        util: Ratio,
        reserve_factor: Ratio,
    ) -> Result<APR, RatesError> {
        // ((1 - reserve_factor) * borrow_rate) * utilization
        let one_minus_reserve_factor = Ratio::one().saturating_sub(reserve_factor);
        let rate_to_pool = borrow_rate
            .0
            .checked_mul(&(one_minus_reserve_factor.into()))
            .ok_or(RatesError::Overflowed)?;

        Ok(APR::from(rate_to_pool.saturating_mul(util.into())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use frame_support::assert_ok;
    use sp_runtime::FixedU128;

    #[test]
    fn init_model_works() {
        let base_rate = Rate::saturating_from_rational(2, 100);
        let kink_rate = Rate::saturating_from_rational(10, 100);
        let full_rate = Rate::saturating_from_rational(32, 100);
        let kink_utilization = Ratio::from_percent(80);

        assert_eq!(
            InterestRateModel::new_model(base_rate, kink_rate, full_rate, kink_utilization),
            InterestRateModel {
                base_rate: Rate::from_inner(20_000_000_000_000_000).into(),
                kink_rate: Rate::from_inner(100_000_000_000_000_000).into(),
                full_rate: Rate::from_inner(320_000_000_000_000_000).into(),
                kink_utilization: Ratio::from_percent(80),
            }
        );
    }

    #[test]
    fn get_borrow_rate_works() {
        // init
        let base_rate = Rate::saturating_from_rational(2, 100);
        let kink_rate = Rate::saturating_from_rational(10, 100);
        let full_rate = Rate::saturating_from_rational(32, 100);
        let kink_utilization = Ratio::from_percent(80);
        let interest_model =
            InterestRateModel::new_model(base_rate, kink_rate, full_rate, kink_utilization);
        assert_ok!(interest_model.check_parameters());

        // normal rate
        let mut cash: u128 = 500;
        let borrows: u128 = 1000;
        let util = Ratio::from_rational(borrows, cash + borrows);
        let borrow_rate = interest_model.get_borrow_rate(util).unwrap();
        assert_eq!(
            borrow_rate,
            APR::from(
                interest_model.kink_rate.0.saturating_mul(util.into()) + interest_model.base_rate.0
            ),
        );

        // jump rate
        cash = 100;
        let util = Ratio::from_rational(borrows, cash + borrows);
        let borrow_rate = interest_model.get_borrow_rate(util).unwrap();
        let normal_rate = interest_model
            .kink_rate
            .0
            .saturating_mul(kink_utilization.into())
            + interest_model.base_rate.0;
        let excess_util = util.saturating_sub(kink_utilization);
        assert_eq!(
            borrow_rate,
            APR::from(
                (interest_model.full_rate.0 - interest_model.kink_rate.0)
                    .saturating_mul(excess_util.into())
                    / FixedU128::saturating_from_rational(20, 100)
                    + normal_rate
            ),
        );
    }

    #[test]
    fn get_supply_rate_works() {
        let borrow_rate = APR::from(Rate::saturating_from_rational(2, 100 * BLOCK_PER_YEAR));
        let util = Ratio::from_percent(50);
        let reserve_factor = Ratio::zero();
        let supply_rate =
            InterestRateModel::get_supply_rate(borrow_rate, util, reserve_factor).unwrap();
        assert_eq!(
            supply_rate,
            APR::from(
                borrow_rate
                    .0
                    .saturating_mul(((Ratio::one().saturating_sub(reserve_factor)) * util).into())
            ),
        );
    }
}
