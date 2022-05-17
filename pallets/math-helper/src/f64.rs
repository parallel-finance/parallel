use sp_runtime::{FixedPointNumber, FixedU128};
use substrate_fixed::traits::LossyInto;
use substrate_fixed::transcendental::pow as fpow;
use substrate_fixed::types::{I32F32, I64F64};

pub fn fixed_u128_to_float(rate: FixedU128) -> f64 {
    rate.into_inner() as f64 / (FixedU128::DIV as f64)
}

pub fn fixed_u128_from_float(rate: f64) -> FixedU128 {
    FixedU128::from_inner((rate * (FixedU128::DIV as f64)) as u128)
}

pub fn power_float(rate: f64, exp: f64) -> Result<f64, &'static str> {
    let result: I64F64 = fpow(I32F32::from_num(rate), I32F32::from_num(exp))
        .expect("Arithmetic power float overflow");
    Ok(result.lossy_into())
}
