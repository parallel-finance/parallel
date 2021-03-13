use primitives::{Amount, Balance};

use sp_std::{convert::TryInto, result};

use crate::module::*;

impl<T: Config> Pallet<T> {
    /// Convert `Balance` to `Amount`.
    pub fn amount_try_from_balance(b: Balance) -> result::Result<Amount, Error<T>> {
        TryInto::<Amount>::try_into(b).map_err(|_| Error::<T>::AmountConvertFailed)
    }

    /// Convert the absolute value of `Amount` to `Balance`.
    pub fn balance_try_from_amount_abs(a: Amount) -> result::Result<Balance, Error<T>> {
        TryInto::<Balance>::try_into(a.saturating_abs())
            .map_err(|_| Error::<T>::AmountConvertFailed)
    }
}

pub fn mul_then_div(multiplier_l: u128, multiplier_r: u128, divisor: u128) -> Option<u128> {
    multiplier_l
        .checked_mul(multiplier_r)
        .and_then(|r| r.checked_div(divisor))
}

pub fn mul_then_div_then_add(
    multiplier_l: u128,
    multiplier_r: u128,
    divisor: u128,
    addend: u128,
) -> Option<u128> {
    multiplier_l
        .checked_mul(multiplier_r)
        .and_then(|r| r.checked_div(divisor))
        .and_then(|r| r.checked_add(addend))
}

pub fn add_then_sub(addend_a: u128, addend_b: u128, subtrahend: u128) -> Option<u128> {
    addend_a
        .checked_add(addend_b)
        .and_then(|r| r.checked_sub(subtrahend))
}
