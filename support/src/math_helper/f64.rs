use sp_runtime::{FixedPointNumber, FixedU128};
use substrate_fixed::{
    traits::LossyInto,
    transcendental::pow as fpow,
    types::{I32F32, I64F64},
};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_power_float_should_work() {
        let total_term_by_year = FixedU128::from_inner(1_753_333_333_000_000_000);
        let term_rate = FixedU128::from_inner(FixedU128::DIV / 100 * 12);
        let start_exchange_rate = FixedU128::from_inner(FixedU128::DIV / 100 * 45);
        let remaining_year =
            fixed_u128_to_float(total_term_by_year) * (1_f64 - fixed_u128_to_float(term_rate));
        let current_rate = power_float(
            1_f64 + fixed_u128_to_float(start_exchange_rate),
            remaining_year,
        )
        .ok()
        .unwrap();
        let current_rate = fixed_u128_from_float(current_rate as f64)
            .reciprocal()
            .unwrap();
        assert_eq!(current_rate, FixedU128::from_inner(563663518378716343));
    }
}
