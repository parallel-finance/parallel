use crate::{
    mock::{
        market_mock, new_test_ext, Loans, Origin, Test, ALICE, DAVE, HKO, KSM, MARKET_MOCK, PHKO,
        XDOT,
    },
    tests::dollar,
    Error,
};
use frame_support::{
    assert_noop, assert_ok,
    traits::tokens::fungibles::{Inspect, Transfer},
};
use sp_runtime::FixedPointNumber;

#[test]
fn trait_inspect_methods_works() {
    new_test_ext().execute_with(|| {
        assert_eq!(Loans::total_issuance(HKO), 0);
        assert_eq!(Loans::total_issuance(KSM), 0);

        let minimum_balance = Loans::minimum_balance(HKO);
        assert_eq!(minimum_balance, 0);

        assert_eq!(Loans::balance(HKO, &DAVE), 0);

        // DAVE Deposit 100 HKO
        assert_ok!(Loans::mint(Origin::signed(DAVE), HKO, dollar(100)));
        assert_eq!(Loans::balance(HKO, &DAVE), dollar(100) * 50);

        assert_eq!(Loans::reducible_balance(HKO, &DAVE, true), dollar(100) * 50);
        assert_ok!(Loans::collateral_asset(Origin::signed(DAVE), HKO, true));
        // Borrow 25 HKO will reduce 25 HKO liquidity for collateral_factor is 50%
        assert_ok!(Loans::borrow(Origin::signed(DAVE), HKO, dollar(25)));

        assert_eq!(
            Loans::exchange_rate(HKO)
                .saturating_mul_int(Loans::account_deposits(HKO, DAVE).voucher_balance),
            dollar(100)
        );

        // DAVE Deposit 100 HKO, Borrow 25 HKO
        // Liquidity HKO 50
        // Formula: ptokens = liquidity / price(1) / collateral(0.5) / exchange_rate(0.02)
        assert_eq!(
            Loans::reducible_balance(HKO, &DAVE, true),
            dollar(25) * 2 * 50
        );

        assert_ok!(Loans::borrow(Origin::signed(DAVE), HKO, dollar(25)));
        assert_eq!(Loans::reducible_balance(HKO, &DAVE, true), 0);

        assert_ok!(Loans::can_deposit(HKO, &DAVE, 100).into_result());
        assert_ok!(Loans::can_withdraw(HKO, &DAVE, 100).into_result());
    })
}

#[test]
fn ptoken_unique_works() {
    new_test_ext().execute_with(|| {
        // ptoken_id already exists in `UnderlyingAssetId`
        assert_noop!(
            Loans::add_market(Origin::root(), XDOT, market_mock(PHKO)),
            Error::<Test>::InvalidCurrencyId
        );

        // ptoken_id token id cannot as the same as the asset id in `Markets`
        assert_noop!(
            Loans::add_market(Origin::root(), XDOT, market_mock(KSM)),
            Error::<Test>::InvalidCurrencyId
        );
    })
}

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
        Loans::transfer(HKO, &DAVE, &ALICE, dollar(50) * 50, true).unwrap();
        // Loans::transfer_ptokens(Origin::signed(DAVE), ALICE, HKO, dollar(50) * 50).unwrap();

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
        Loans::transfer(HKO, &DAVE, &ALICE, dollar(20) * 50, true).unwrap();

        // DAVE Deposit HKO = 100 - 20 = 80
        // DAVE Borrow HKO = 0 + 50 - 40 = 10
        // DAVE liquidity HKO = 80 * 0.5 - 10 = 30
        assert_eq!(
            Loans::exchange_rate(HKO)
                .saturating_mul_int(Loans::account_deposits(HKO, DAVE).voucher_balance),
            dollar(80)
        );
        // DAVE Borrow 31 HKO should cause InsufficientLiquidity
        assert_noop!(
            Loans::borrow(Origin::signed(DAVE), HKO, dollar(31)),
            Error::<Test>::InsufficientLiquidity
        );
        assert_ok!(Loans::borrow(Origin::signed(DAVE), HKO, dollar(30)));

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
