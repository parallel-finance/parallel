//! Unit tests for the loans module.

use primitives::{CurrencyId, BLOCK_PER_YEAR, RATE_DECIMAL, TOKEN_DECIMAL};

use super::*;

use mock::*;

#[test]
fn test_mock_genesis_ok() {
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(TotalBorrows::<Runtime>::get(DOT), 0 * TOKEN_DECIMAL);
        assert_eq!(TotalSupply::<Runtime>::get(BTC), 0 * TOKEN_DECIMAL);
        assert_eq!(BorrowIndex::<Runtime>::get(USDC), RATE_DECIMAL);
        assert_eq!(CollateralRate::<Runtime>::get(KSM), 5 * RATE_DECIMAL / 10);
    });
}

// Test rate module
#[test]
fn test_utilization_rate() {
    // 50% borrow
    assert_eq!(
        Loans::utilization_rate(1, 1, 0).unwrap(),
        5 * RATE_DECIMAL / 10
    );
    assert_eq!(
        Loans::utilization_rate(100, 100, 0).unwrap(),
        5 * RATE_DECIMAL / 10
    );
    // no borrow
    assert_eq!(
        Loans::utilization_rate(1, 0, 0).unwrap(),
        0 * RATE_DECIMAL / 10
    );
    // full borrow
    assert_eq!(Loans::utilization_rate(0, 1, 0).unwrap(), 1 * RATE_DECIMAL);
}

#[test]
fn test_update_jump_rate_model() {
    ExtBuilder::default().build().execute_with(|| {
        let base_rate_per_year: u128 = 2 * RATE_DECIMAL / 100;
        let multiplier_per_year: u128 = RATE_DECIMAL / 10;
        let jump_multiplier_per_year: u128 = 11 * RATE_DECIMAL / 10;
        let kink: u128 = 8 * RATE_DECIMAL / 10;
        assert!(Loans::update_jump_rate_model(
            base_rate_per_year,
            multiplier_per_year,
            jump_multiplier_per_year,
            kink,
        )
        .is_ok());
        assert_eq!(
            BaseRatePerBlock::<Runtime>::get(),
            Some(base_rate_per_year / BLOCK_PER_YEAR)
        );
        assert_eq!(
            MultiplierPerBlock::<Runtime>::get(),
            Some(multiplier_per_year * RATE_DECIMAL / (BLOCK_PER_YEAR * kink))
        );
        assert_eq!(
            JumpMultiplierPerBlock::<Runtime>::get(),
            Some(jump_multiplier_per_year / BLOCK_PER_YEAR)
        );
        assert_eq!(Kink::<Runtime>::get(), Some(kink));
    });
}

#[test]
fn test_update_borrow_rate() {
    ExtBuilder::default().build().execute_with(|| {
        // normal rate
        let mut cash: u128 = 5 * TOKEN_DECIMAL;
        let borrows: u128 = 10 * TOKEN_DECIMAL;
        let reserves: u128 = 0;
        assert!(Loans::update_borrow_rate(CurrencyId::DOT, cash, borrows, reserves).is_ok());
        let util = Loans::utilization_rate(cash, borrows, reserves).unwrap();
        let multiplier_per_block = MultiplierPerBlock::<Runtime>::get().unwrap();
        let base_rate_per_block = BaseRatePerBlock::<Runtime>::get().unwrap();
        let kink = Kink::<Runtime>::get().unwrap();
        let jump_multiplier_per_block = JumpMultiplierPerBlock::<Runtime>::get().unwrap();
        assert_eq!(
            BorrowRate::<Runtime>::get(CurrencyId::DOT),
            util * multiplier_per_block / RATE_DECIMAL + base_rate_per_block
        );

        // jump rate
        cash = 1 * TOKEN_DECIMAL;
        assert!(Loans::update_borrow_rate(CurrencyId::KSM, cash, borrows, reserves).is_ok());
        let normal_rate = kink * multiplier_per_block / RATE_DECIMAL + base_rate_per_block;
        let excess_util = util.saturating_sub(kink);
        assert_eq!(
            BorrowRate::<Runtime>::get(CurrencyId::KSM),
            excess_util * (jump_multiplier_per_block / RATE_DECIMAL) + normal_rate
        );
    });
}

#[test]
fn test_calc_exchange_rate() {}
