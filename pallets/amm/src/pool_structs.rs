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

// Groups common pool related structures

use parallel_primitives::{Balance, CurrencyId, Rate};
use sp_runtime::{traits::Saturating, ArithmeticError, DispatchError, FixedPointNumber};

// Amplification Coefficient Weight.
//
// In this pallet, the actual amplification coefficient will be `exchange_rate` * `ACW`.
const ACW: Rate = Rate::from_inner(Rate::DIV / 100 * 50); // 50%

#[derive(
    Clone,
    PartialEq,
    parity_scale_codec::Decode,
    parity_scale_codec::Encode,
    sp_runtime::RuntimeDebug,
)]
pub enum SwapType {
    Buy,
    Sell,
}

#[derive(Debug)]
// Wrapper around the result of `Pallet::calculate_amount`
pub struct AmountEvaluation {
    pub account_amount: Balance,
    pub pool_amount: Balance,
}

#[derive(
    Clone,
    PartialEq,
    parity_scale_codec::Decode,
    parity_scale_codec::Encode,
    sp_runtime::RuntimeDebug,
)]
pub struct LiquidityProviderAmounts {
    pub base_amount: Balance,
    pub quote_amount: Balance,
}

#[derive(
    Clone,
    PartialEq,
    parity_scale_codec::Decode,
    parity_scale_codec::Encode,
    sp_runtime::RuntimeDebug,
)]
pub struct Pool {
    pub base_amount: Balance,
    pub quote_amount: Balance,
    pub base_asset: CurrencyId,
    pub quote_asset: CurrencyId,
}

pub struct StandardSwap;

pub struct StableSwap;

pub struct StakingSwap;

pub trait StabilityPool {}

impl StabilityPool for () {}

pub trait AMMCurve {
    // Calculates the amount according to the underlying formula and the provided pool.
    fn calculate_amount(
        exchange_rate: Rate,
        new_amount: Balance,
        pool: &Pool,
    ) -> Result<Balance, DispatchError>;
}

impl AMMCurve for StandardSwap {
    // let [x|y] = k / [x|y];
    fn calculate_amount(
        _: Rate,
        new_amount: Balance,
        pool: &Pool,
    ) -> Result<Balance, DispatchError> {
        let k = pool
            .base_amount
            .checked_mul(pool.quote_amount)
            .ok_or_else::<DispatchError, _>(|| ArithmeticError::Overflow.into())?;
        let result = k
            .checked_div(new_amount)
            .ok_or_else::<DispatchError, _>(|| ArithmeticError::DivisionByZero.into())?;
        Ok(result)
    }
}

impl AMMCurve for StableSwap {
    // let [x|y] = (k * (4*A*k + k - 4*A*[x|y])) / (4 * (A*k + [x|y]));
    fn calculate_amount(
        exchange_rate: Rate,
        new_amount: Balance,
        pool: &Pool,
    ) -> Result<Balance, DispatchError> {
        let k = pool
            .base_amount
            .checked_add(pool.quote_amount)
            .ok_or_else::<DispatchError, _>(|| ArithmeticError::Overflow.into())?;
        let evaluation_option = || {
            let a_multiplied_by_k = amplification_coeficient_mul(exchange_rate, k)?;
            let _4_multiplied_by_a_multiplied_by_x =
                4u128.checked_mul(amplification_coeficient_mul(exchange_rate, new_amount)?)?;
            let _4_multiplied_by_a_multiplied_by_k = 4u128.checked_mul(a_multiplied_by_k)?;
            let dividend = k.checked_mul(
                _4_multiplied_by_a_multiplied_by_k
                    .checked_add(k)?
                    .checked_sub(_4_multiplied_by_a_multiplied_by_x)?,
            )?;
            let divisor = 4u128.checked_mul(a_multiplied_by_k.checked_add(new_amount)?)?;
            dividend.checked_div(divisor)
        };
        let result = evaluation_option()
            .ok_or_else::<DispatchError, _>(|| ArithmeticError::DivisionByZero.into())?;
        Ok(result)
    }
}

impl AMMCurve for StakingSwap {
    fn calculate_amount(_: Rate, _: Balance, _: &Pool) -> Result<Balance, DispatchError> {
        unimplemented!()
    }
}

// Multiplies an arbitrary coefficient value with the current amplification coefficient.
fn amplification_coeficient_mul(exchange_rate: Rate, n: u128) -> Option<u128> {
    // Saturates because a very large amplification coefficient
    // will simply shape the curve as a constant sum equation.
    let amplif_coefficient = ACW.saturating_add(exchange_rate);
    amplif_coefficient.checked_mul_int(n)
}

#[cfg(test)]
mod tests {
    use super::{AMMCurve, Pool, StableSwap, StandardSwap};
    use parallel_primitives::CurrencyId;

    const DEFAULT_DYNAMIC_POOL: Pool = Pool {
        base_amount: 40,
        base_asset: CurrencyId::DOT,
        quote_amount: 60,
        quote_asset: CurrencyId::xDOT,
    };
    const DEFAULT_STABLE_POOL: Pool = Pool {
        base_asset: CurrencyId::DOT,
        quote_asset: CurrencyId::xDOT,
        base_amount: 40,
        quote_amount: 60,
    };

    #[test]
    fn dynamic_curve_correctly_evaluates() {
        let amount = StandardSwap::calculate_amount(1.into(), 20, &DEFAULT_DYNAMIC_POOL).unwrap();
        assert_eq!(amount, 120);
    }

    #[test]
    fn stable_curve_correctly_evaluates() {
        let amount = StableSwap::calculate_amount(1.into(), 20, &DEFAULT_STABLE_POOL).unwrap();
        assert_eq!(amount, 85);
    }
}
