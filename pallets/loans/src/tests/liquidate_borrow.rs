use crate::{
    mock::{new_test_ext, Loans, MockPriceFeeder, Origin, Test, ALICE, BOB, DOT, KSM, USDT},
    tests::dollar,
    Config, Error, MarketState,
};
use frame_support::{assert_noop, assert_ok};
use primitives::Rate;
use sp_runtime::FixedPointNumber;

#[test]
fn liquidate_borrow_allowed_works() {
    new_test_ext().execute_with(|| {
        // Borrower should have a positive shortfall
        let dot_market = Loans::market(DOT).unwrap();
        assert_noop!(
            Loans::liquidate_borrow_allowed(&ALICE, DOT, 100, &dot_market),
            Error::<Test>::InsufficientShortfall
        );
        initial_setup();
        alice_borrows_100_ksm();
        // Adjust KSM price to make shortfall
        MockPriceFeeder::set_price(KSM, 2.into());
        let ksm_market = Loans::market(KSM).unwrap();
        assert_noop!(
            Loans::liquidate_borrow_allowed(&ALICE, KSM, dollar(51), &ksm_market),
            Error::<Test>::TooMuchRepay
        );
        assert_ok!(Loans::liquidate_borrow_allowed(
            &ALICE,
            KSM,
            dollar(50),
            &ksm_market
        ));
    })
}

#[test]
fn deposit_of_borrower_must_be_collateral() {
    new_test_ext().execute_with(|| {
        initial_setup();
        alice_borrows_100_ksm();
        // Adjust KSM price to make shortfall
        MockPriceFeeder::set_price(KSM, 2.into());
        let market = Loans::market(KSM).unwrap();
        assert_noop!(
            Loans::liquidate_borrow_allowed(&ALICE, KSM, dollar(51), &market),
            Error::<Test>::TooMuchRepay
        );
        assert_noop!(
            Loans::liquidate_borrow(Origin::signed(BOB), ALICE, KSM, 10, USDT),
            Error::<Test>::DepositsAreNotCollateral
        );
    })
}

#[test]
fn collateral_value_must_be_greater_than_liquidation_value() {
    new_test_ext().execute_with(|| {
        initial_setup();
        alice_borrows_100_ksm();
        MockPriceFeeder::set_price(KSM, Rate::from_float(2000.0));
        Loans::mutate_market(KSM, |market| {
            market.liquidate_incentive = Rate::from_float(200.0);
        })
        .unwrap();
        assert_noop!(
            Loans::liquidate_borrow(Origin::signed(BOB), ALICE, KSM, dollar(50), DOT),
            Error::<Test>::InsufficientCollateral
        );
    })
}

#[test]
fn full_workflow_works_as_expected() {
    new_test_ext().execute_with(|| {
        initial_setup();
        alice_borrows_100_ksm();
        // adjust KSM price to make ALICE generate shortfall
        MockPriceFeeder::set_price(KSM, 2.into());
        // BOB repay the KSM borrow balance and get DOT from ALICE
        assert_ok!(Loans::liquidate_borrow(
            Origin::signed(BOB),
            ALICE,
            KSM,
            dollar(50),
            DOT
        ));

        // KSM price = 2
        // incentive = repay KSM value * 1.1 = (50 * 2) * 1.1 = 110
        // Alice DOT: cash - deposit = 1000 - 200 = 800
        // Alice DOT collateral: deposit - incentive = 200 - 110 = 90
        // Alice KSM: cash + borrow = 1000 + 100 = 1100
        // Alice KSM borrow balance: origin borrow balance - liquidate amount = 100 - 50 = 50
        // Bob KSM: cash - deposit - repay = 1000 - 200 - 50 = 750
        // Bob DOT collateral: incentive = 110
        assert_eq!(<Test as Config>::Assets::balance(DOT, &ALICE), dollar(800),);
        assert_eq!(
            Loans::exchange_rate(DOT)
                .saturating_mul_int(Loans::account_deposits(DOT, ALICE).voucher_balance),
            dollar(90),
        );
        assert_eq!(<Test as Config>::Assets::balance(KSM, &ALICE), dollar(1100),);
        assert_eq!(Loans::account_borrows(KSM, ALICE).principal, dollar(50));
        assert_eq!(<Test as Config>::Assets::balance(KSM, &BOB), dollar(750));
        assert_eq!(
            Loans::exchange_rate(DOT)
                .saturating_mul_int(Loans::account_deposits(DOT, BOB).voucher_balance),
            dollar(110),
        );
    })
}

#[test]
fn liquidator_cannot_take_inactive_market_currency() {
    new_test_ext().execute_with(|| {
        initial_setup();
        alice_borrows_100_ksm();
        // Adjust KSM price to make shortfall
        MockPriceFeeder::set_price(KSM, 2.into());
        assert_ok!(Loans::mutate_market(DOT, |stored_market| {
            stored_market.state = MarketState::Supervision;
        }));
        assert_noop!(
            Loans::liquidate_borrow(Origin::signed(BOB), ALICE, KSM, dollar(50), DOT),
            Error::<Test>::MarketNotActivated
        );
    })
}

#[test]
fn liquidator_can_not_repay_more_than_the_close_factor_pct_multiplier() {
    new_test_ext().execute_with(|| {
        initial_setup();
        alice_borrows_100_ksm();
        MockPriceFeeder::set_price(KSM, 20.into());
        assert_noop!(
            Loans::liquidate_borrow(Origin::signed(BOB), ALICE, KSM, dollar(51), DOT),
            Error::<Test>::TooMuchRepay
        );
    })
}

#[test]
fn liquidator_must_not_be_borrower() {
    new_test_ext().execute_with(|| {
        initial_setup();
        assert_noop!(
            Loans::liquidate_borrow(Origin::signed(ALICE), ALICE, KSM, 0, DOT),
            Error::<Test>::LiquidatorIsBorrower
        );
    })
}

fn alice_borrows_100_ksm() {
    assert_ok!(Loans::borrow(Origin::signed(ALICE), KSM, dollar(100)));
}

fn initial_setup() {
    // Bob deposits 200 KSM
    assert_ok!(Loans::mint(Origin::signed(BOB), KSM, dollar(200)));
    // Alice deposits 200 DOT as collateral
    assert_ok!(Loans::mint(Origin::signed(ALICE), DOT, dollar(200)));
    assert_ok!(Loans::collateral_asset(Origin::signed(ALICE), DOT, true));
}
