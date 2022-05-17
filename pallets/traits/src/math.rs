use sp_runtime::{FixedPointNumber, FixedU128};

pub fn fixed_u128_to_float(rate: FixedU128) -> f64 {
    rate.into_inner() as f64 / (FixedU128::DIV as f64)
}

pub fn fixed_u128_from_float(rate: f64) -> FixedU128 {
    FixedU128::from_inner((rate * (FixedU128::DIV as f64)) as u128)
}
