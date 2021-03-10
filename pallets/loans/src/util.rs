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
