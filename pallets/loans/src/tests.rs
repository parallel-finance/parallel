//! Unit tests for the loans module.

#![cfg(test)]

use super::*;

use mock::{*};

#[test]
fn test_mock_genesis_ok() {
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(TotalBorrows::<Runtime>::get(DOT), 50 * 10u128.pow(18));
        assert_eq!(TotalSupply::<Runtime>::get(BTC), 100 * 10u128.pow(18));
        assert_eq!(BorrowIndex::<Runtime>::get(USDC), 10u128.pow(18));
        assert_eq!(CollateralRate::<Runtime>::get(KSM), 5 * 10u128.pow(17));
    });
}

// Test rate module
#[test]
fn test_utilization_rate() {
    assert_eq!(
        Loans::utilization_rate(1, 1, 0).unwrap(),
        5 * 10u128.pow(17)
    )
}

#[test]
fn test_utilization_rate_no_borrow() {
    assert_eq!(
        Loans::utilization_rate(1, 0, 0).unwrap(),
        0 * 10u128.pow(17)
    )
}

#[test]
fn test_utilization_rate_full_borrow() {
    assert_eq!(
        Loans::utilization_rate(0, 1, 0).unwrap(),
        1 * 10u128.pow(18)
    )
}
