use super::*;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok};

#[test]
fn add_liquidity_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::add_liquidity(
            Origin::signed(1.into()),
            (DOT, XDOT),
            (10, 20),
            (5, 5)
        ));

        assert_eq!(AMM::pools(XDOT, DOT).base_amount, 20);

        assert_eq!(
            AMM::liquidity_providers((AccountId(1u64), XDOT, DOT)).base_amount,
            20
        );

        // Check balance is correct
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::DOT, &1.into()),
            90
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::xDOT, &1.into()),
            80
        );
    })
}

#[test]
fn add_more_liquidity_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::add_liquidity(
            Origin::signed(1.into()),
            (DOT, XDOT),
            (10, 20),
            (5, 5)
        ));

        assert_ok!(AMM::add_liquidity(
            Origin::signed(1.into()),
            (DOT, XDOT),
            (30, 40),
            (5, 5)
        ));

        assert_eq!(AMM::pools(XDOT, DOT).base_amount, 60);

        assert_eq!(
            AMM::liquidity_providers((AccountId(1u64), XDOT, DOT)).base_amount,
            60
        );

        assert_eq!(
            AMM::liquidity_providers((AccountId(1u64), XDOT, DOT)).quote_amount,
            30
        );

        assert_eq!(AMM::pools(XDOT, DOT).base_amount, 60);

        assert_eq!(AMM::pools(XDOT, DOT).quote_amount, 30);
    })
}

#[test]
fn add_more_liquidity_should_not_work_if_minimum_base_amount_is_higher() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::add_liquidity(
            Origin::signed(1.into()),
            (DOT, XDOT),
            (10, 20),
            (5, 5)
        ));

        assert_noop!(
            AMM::add_liquidity(Origin::signed(1.into()), (DOT, XDOT), (30, 40), (55, 5)),
            Error::<Test>::NotAIdealPriceRatio
        );
    })
}

#[test]
fn add_more_liquidity_with_low_balance_should_not_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::add_liquidity(
            Origin::signed(1.into()),
            (DOT, XDOT),
            (10, 20),
            (5, 5)
        ));

        assert_ok!(AMM::add_liquidity(
            Origin::signed(1.into()),
            (DOT, XDOT),
            (30, 40),
            (1, 1)
        ));

        assert_noop!(
            AMM::add_liquidity(Origin::signed(1.into()), (DOT, XDOT), (50, 60), (5, 5)),
            orml_tokens::Error::<Test>::BalanceTooLow,
        );
    })
}

#[test]
fn add_liquidity_by_another_user_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::add_liquidity(
            Origin::signed(1.into()),
            (DOT, XDOT),
            (10, 20),
            (5, 5)
        ));

        assert_ok!(AMM::add_liquidity(
            Origin::signed(1.into()),
            (DOT, XDOT),
            (30, 40),
            (5, 5)
        ));

        assert_ok!(AMM::add_liquidity(
            Origin::signed(2.into()),
            (DOT, XDOT),
            (5, 10),
            (5, 5)
        ));

        assert_eq!(AMM::pools(XDOT, DOT).base_amount, 70);
    })
}

#[test]
fn remove_liquidity_whole_share_should_work() {
    new_test_ext().execute_with(|| {
        // A pool with a single LP provider
        // who deposit tokens and withdraws their whole share
        // (most simple case)

        let _ = AMM::add_liquidity(Origin::signed(1.into()), (DOT, XDOT), (10, 90), (5, 5));

        assert_ok!(AMM::remove_liquidity(
            Origin::signed(1.into()),
            (DOT, XDOT),
            30
        ));

        assert_eq!(
            AMM::liquidity_providers((AccountId(1u64), XDOT, DOT)).ownership,
            0
        );

        assert_eq!(AMM::pools(XDOT, DOT).ownership, 0);

        // Check balance is correct
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::DOT, &1.into()),
            100
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::xDOT, &1.into()),
            100
        );
    })
}

#[test]
fn remove_liquidity_only_portion_should_work() {
    new_test_ext().execute_with(|| {
        // A pool with a single LP provider who
        // deposit tokens and withdraws
        // a portion of their total shares (simple case)

        let _ = AMM::add_liquidity(Origin::signed(1.into()), (DOT, XDOT), (10, 90), (5, 5));

        assert_ok!(AMM::remove_liquidity(
            Origin::signed(1.into()),
            (DOT, XDOT),
            15
        ));

        assert_eq!(
            AMM::liquidity_providers((AccountId(1u64), XDOT, DOT)).ownership,
            15
        );

        assert_eq!(AMM::pools(XDOT, DOT).ownership, 15);

        // Check balance is correct
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::DOT, &1.into()),
            95
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::xDOT, &1.into()),
            55
        );
    })
}

#[test]
fn remove_liquidity_user_more_liquidity_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::add_liquidity(
            Origin::signed(1.into()),
            (DOT, XDOT),
            (10, 25),
            (5, 5)
        ));
        assert_ok!(AMM::add_liquidity(
            Origin::signed(1.into()),
            (DOT, XDOT),
            (15, 30),
            (5, 5)
        ));

        assert_ok!(AMM::remove_liquidity(
            Origin::signed(1.into()),
            (DOT, XDOT),
            15
        ));

        assert_eq!(
            AMM::liquidity_providers((AccountId(1u64), XDOT, DOT)).ownership,
            3
        );

        assert_eq!(AMM::pools(XDOT, DOT).ownership, 3);

        // Check balance is correct
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::DOT, &1.into()),
            96
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::xDOT, &1.into()),
            90
        );
    })
}

#[test]
fn remove_liquidity_when_pool_does_not_exist_should_not_work() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            AMM::remove_liquidity(Origin::signed(1.into()), (DOT, XDOT), 15),
            Error::<Test>::PoolDoesNotExist
        );
    })
}

#[test]
fn remove_liquidity_with_more_liquidity_should_not_work() {
    new_test_ext().execute_with(|| {
        // A pool with a single LP provider
        // who deposit tokens and withdraws their whole share
        // (most simple case)

        let _ = AMM::add_liquidity(Origin::signed(1.into()), (DOT, XDOT), (10, 90), (5, 5));

        assert_noop!(
            AMM::remove_liquidity(Origin::signed(1.into()), (DOT, XDOT), 300),
            Error::<Test>::MoreLiquidity
        );
    })
}
