use crate::{
    mock::{new_test_ext, Loans, Origin, Test, ALICE, DAVE, HKO, MARKET_MOCK},
    tests::dollar,
    Error,
};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::FixedPointNumber;

#[test]
fn transfer_ptoken_works() {
    new_test_ext().execute_with(|| {
        // DAVE Deposit 100 HKO
        assert_ok!(Loans::mint(Origin::signed(DAVE), HKO, dollar(100)));

        // DAVE HKO collateral: deposit = 100
        // HKO: cash - deposit = 1000 - 100 = 900
        assert_eq!(
            Loans::exchange_rate(HKO)
                .saturating_mul_int(Loans::account_deposits(HKO, DAVE).voucher_balance),
            dollar(100)
        );

        // ALICE HKO collateral: deposit = 0
        assert_eq!(
            Loans::exchange_rate(HKO)
                .saturating_mul_int(Loans::account_deposits(HKO, ALICE).voucher_balance),
            dollar(0)
        );

        // Transfer ptokens from DAVE to ALICE
        Loans::transfer_ptokens(Origin::signed(DAVE), ALICE, HKO, dollar(50) * 50).unwrap();

        // DAVE HKO collateral: deposit = 50
        assert_eq!(
            Loans::exchange_rate(HKO)
                .saturating_mul_int(Loans::account_deposits(HKO, DAVE).voucher_balance),
            dollar(50)
        );
        // DAVE Redeem 51 HKO should cause InsufficientDeposit
        assert_noop!(
            Loans::redeem_allowed(HKO, &DAVE, dollar(51) * 50, &MARKET_MOCK),
            Error::<Test>::InsufficientDeposit
        );

        // ALICE HKO collateral: deposit = 50
        assert_eq!(
            Loans::exchange_rate(HKO)
                .saturating_mul_int(Loans::account_deposits(HKO, ALICE).voucher_balance),
            dollar(50)
        );
        // ALICE Redeem 50 HKO should be succeeded
        assert_ok!(Loans::redeem_allowed(
            HKO,
            &ALICE,
            dollar(50) * 50,
            &MARKET_MOCK
        ));
    })
}

#[test]
fn transfer_ptokens_under_collateral_works() {
    new_test_ext().execute_with(|| {
        // DAVE Deposit 100 HKO
        assert_ok!(Loans::mint(Origin::signed(DAVE), HKO, dollar(100)));
        assert_ok!(Loans::collateral_asset(Origin::signed(DAVE), HKO, true));

        // Borrow 50 HKO will reduce 50 HKO liquidity for collateral_factor is 50%
        assert_ok!(Loans::borrow(Origin::signed(DAVE), HKO, dollar(50)));
        // Repay 40 HKO
        assert_ok!(Loans::repay_borrow(Origin::signed(DAVE), HKO, dollar(40)));

        // Transfer 20 ptokens from DAVE to ALICE
        Loans::transfer_ptokens(Origin::signed(DAVE), ALICE, HKO, dollar(20) * 50).unwrap();

        // DAVE Deposit HKO = 100 - 20 = 80
        // DAVE Borrow HKO = 0 + 50 - 40 = 10
        // DAVE liquidity HKO = 80 * 0.5 - 10 = 30
        assert_eq!(
            Loans::exchange_rate(HKO)
                .saturating_mul_int(Loans::account_deposits(HKO, DAVE).voucher_balance),
            dollar(80)
        );
        assert_ok!(Loans::redeem_allowed(
            HKO,
            &DAVE,
            dollar(60) * 50,
            &MARKET_MOCK
        ));
        // DAVE Borrow 31 HKO should cause InsufficientLiquidity
        assert_noop!(
            Loans::borrow(Origin::signed(DAVE), HKO, dollar(31)),
            Error::<Test>::InsufficientLiquidity
        );

        // ALICE Deposit HKO 20
        assert_eq!(
            Loans::exchange_rate(HKO)
                .saturating_mul_int(Loans::account_deposits(HKO, ALICE).voucher_balance),
            dollar(20)
        );
        // ALICE Redeem 20 HKO should be succeeded
        assert_ok!(Loans::redeem_allowed(
            HKO,
            &ALICE,
            dollar(20) * 50,
            &MARKET_MOCK
        ));
    })
}
