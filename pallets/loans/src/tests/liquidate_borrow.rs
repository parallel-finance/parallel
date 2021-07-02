use crate::{
    mock::{Loans, MockPriceFeeder, Origin, Runtime, ALICE, BOB, DOT, KSM, USDT},
    tests::{million_dollar, ExtBuilder},
    Config, Error,
};
use frame_support::{assert_noop, assert_ok};
use orml_traits::MultiCurrency;
use primitives::Rate;
use sp_runtime::FixedPointNumber;

#[test]
fn liquidate_borrow_allowed_works() {
    ExtBuilder::default().build().execute_with(|| {
        // Borrower should have a positive shortfall
        let dot_market = Loans::market(&DOT).unwrap();
        assert_noop!(
            Loans::liquidate_borrow_allowed(&ALICE, DOT, 100, &dot_market),
            Error::<Runtime>::InsufficientShortfall
        );
        initial_setup();
        alice_borrows_100_ksm();
        // Adjust KSM price to make shortfall
        MockPriceFeeder::set_price(KSM, 2.into());
        let ksm_market = Loans::market(&KSM).unwrap();
        assert_noop!(
            Loans::liquidate_borrow_allowed(&ALICE, KSM, million_dollar(51), &ksm_market),
            Error::<Runtime>::TooMuchRepay
        );
        assert_ok!(Loans::liquidate_borrow_allowed(
            &ALICE,
            KSM,
            million_dollar(50),
            &ksm_market
        ));
    })
}

#[test]
fn deposit_of_borrower_must_be_collateral() {
    ExtBuilder::default().build().execute_with(|| {
        initial_setup();
        alice_borrows_100_ksm();
        // Adjust KSM price to make shortfall
        MockPriceFeeder::set_price(KSM, 2.into());
        let market = Loans::market(&KSM).unwrap();
        assert_noop!(
            Loans::liquidate_borrow_allowed(&ALICE, KSM, million_dollar(51), &market),
            Error::<Runtime>::TooMuchRepay
        );
        assert_noop!(
            Loans::liquidate_borrow(Origin::signed(BOB), ALICE, KSM, 10, USDT),
            Error::<Runtime>::DepositsAreNotCollateral
        );
    })
}

pub(super) fn collateral_value_must_be_greater_than_liquidation_value() {
    ExtBuilder::default().build().execute_with(|| {
        initial_setup();
        alice_borrows_100_ksm();
        MockPriceFeeder::set_price(KSM, Rate::from_float(2000.0));
        Loans::mutate_market(&KSM, |market| {
            market.liquidate_incentive = Rate::from_float(200.0);
        })
        .unwrap();
        assert_noop!(
            Loans::liquidate_borrow(Origin::signed(BOB), ALICE, KSM, million_dollar(50), DOT),
            Error::<Runtime>::InsufficientCollateral
        );
        MockPriceFeeder::reset();
    })
}

pub(super) fn full_workflow_works_as_expected() {
    ExtBuilder::default().build().execute_with(|| {
        initial_setup();
        alice_borrows_100_ksm();
        // adjust KSM price to make ALICE generate shortfall
        MockPriceFeeder::set_price(KSM, 2.into());
        // BOB repay the KSM borrow balance and get DOT from ALICE
        assert_ok!(Loans::liquidate_borrow(
            Origin::signed(BOB),
            ALICE,
            KSM,
            million_dollar(50),
            DOT
        ));

        // KSM price = 2
        // incentive = repay KSM value * 1.1 = (50 * 2) * 1.1 = 110
        // Alice DOT: cash - deposit = 1000 - 200 = 800
        // Alice DOT collateral: deposit - incentive = 200 - 110 = 90
        // Alice KSM: cash + borrow = 1000 + 100 = 1100
        // Alice KSM borrow balance: origin borrow balance - repay amount = 100 - 50 = 50
        // Bob KSM: cash - deposit - repay = 1000 - 200 - 50 = 750
        // Bob DOT collateral: incentive = 110
        assert_eq!(
            <Runtime as Config>::Currency::free_balance(DOT, &ALICE),
            million_dollar(800),
        );
        assert_eq!(
            Loans::exchange_rate(DOT)
                .saturating_mul_int(Loans::account_deposits(DOT, ALICE).voucher_balance),
            90000000000000000000,
        );
        assert_eq!(
            <Runtime as Config>::Currency::free_balance(KSM, &ALICE),
            million_dollar(1100),
        );
        assert_eq!(
            Loans::account_borrows(KSM, ALICE).principal,
            million_dollar(50)
        );
        assert_eq!(
            <Runtime as Config>::Currency::free_balance(KSM, &BOB),
            million_dollar(750)
        );
        assert_eq!(
            Loans::exchange_rate(DOT)
                .saturating_mul_int(Loans::account_deposits(DOT, BOB).voucher_balance),
            110000000000000000000,
        );
        MockPriceFeeder::reset();
    })
}

pub(super) fn liquidator_can_not_repay_more_than_the_close_factor_pct_multiplier() {
    ExtBuilder::default().build().execute_with(|| {
        initial_setup();
        alice_borrows_100_ksm();
        MockPriceFeeder::set_price(KSM, 20.into());
        assert_noop!(
            Loans::liquidate_borrow(Origin::signed(BOB), ALICE, KSM, million_dollar(51), DOT),
            Error::<Runtime>::TooMuchRepay
        );
        MockPriceFeeder::reset();
    })
}

#[test]
fn liquidator_must_not_be_borrower() {
    ExtBuilder::default().build().execute_with(|| {
        initial_setup();
        assert_noop!(
            Loans::liquidate_borrow(Origin::signed(ALICE), ALICE, KSM, 0, DOT),
            Error::<Runtime>::LiquidatorIsBorrower
        );
    })
}

fn alice_borrows_100_ksm() {
    assert_ok!(Loans::borrow(
        Origin::signed(ALICE),
        KSM,
        million_dollar(100)
    ));
}

fn initial_setup() {
    // Bob deposits 200 KSM
    assert_ok!(Loans::mint(Origin::signed(BOB), KSM, million_dollar(200)));
    // Alice deposits 200 DOT as collateral
    assert_ok!(Loans::mint(Origin::signed(ALICE), DOT, million_dollar(200)));
    assert_ok!(Loans::collateral_asset(Origin::signed(ALICE), DOT, true));
}
