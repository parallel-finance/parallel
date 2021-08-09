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
    ) -> Result<AmountEvaluation, DispatchError>;
}

impl AMMCurve for StandardSwap {
    // let [x|y] = k / [x|y];
    fn calculate_amount(
        _: Rate,
        new_amount: Balance,
        pool: &Pool,
    ) -> Result<AmountEvaluation, DispatchError> {
        let k = pool
            .base_amount
            .checked_mul(pool.quote_amount)
            .ok_or_else::<DispatchError, _>(|| ArithmeticError::Overflow.into())?;
        let rslt = k
            .checked_div(new_amount)
            .ok_or_else::<DispatchError, _>(|| ArithmeticError::DivisionByZero.into())?;
        calculate_amount_evaluation(rslt, pool)
    }
}

impl AMMCurve for StableSwap {
    // let [x|y] = (k * (4*A*k + k - 4*A*[x|y])) / (4 * (A*k + [x|y]));
    fn calculate_amount(
        exchange_rate: Rate,
        new_amount: Balance,
        pool: &Pool,
    ) -> Result<AmountEvaluation, DispatchError> {
        let k = pool
            .base_amount
            .checked_add(pool.quote_amount)
            .ok_or_else::<DispatchError, _>(|| ArithmeticError::Overflow.into())?;
        let f = || {
            let ak = amplification_coeficient_mul(exchange_rate, k)?;
            let _4ax =
                4u128.checked_mul(amplification_coeficient_mul(exchange_rate, new_amount)?)?;
            let _4ak = 4u128.checked_mul(ak)?;
            let dividend = k.checked_mul(_4ak.checked_add(k)?.checked_sub(_4ax)?)?;
            let divisor = 4u128.checked_mul(ak.checked_add(new_amount)?)?;
            dividend.checked_div(divisor)
        };
        let rslt = f().ok_or_else::<DispatchError, _>(|| ArithmeticError::DivisionByZero.into())?;
        calculate_amount_evaluation(rslt, pool)
    }
}

impl AMMCurve for StakingSwap {
    fn calculate_amount(_: Rate, _: Balance, _: &Pool) -> Result<AmountEvaluation, DispatchError> {
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

fn calculate_amount_evaluation(
    pool_amount: Balance,
    pool: &Pool,
) -> Result<AmountEvaluation, DispatchError> {
    let [greater, lesser] = if pool.quote_amount > pool_amount {
        [pool.quote_amount, pool_amount]
    } else {
        [pool_amount, pool.quote_amount]
    };
    let diff = greater
        .checked_sub(lesser)
        .ok_or_else::<DispatchError, _>(|| ArithmeticError::Underflow.into())?;
    Ok(AmountEvaluation {
        account_amount: diff,
        pool_amount,
    })
}
