use super::*;
use crate::mock::*;
use frame_support::assert_ok;

#[test]
fn add_liquidity_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::add_liquidity(
            Origin::signed(1.into()),
            (DOT, XDOT),
            (10, 20)
        ));

        assert_eq!(AMM::pools(XDOT, DOT).base_amount, 10);

        assert_eq!(
            AMM::liquidity_providers((AccountId(1u64), XDOT, DOT)).base_amount,
            10
        );

        // Check balance is correct
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::DOT, &1.into()),
            80
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(CurrencyId::xDOT, &1.into()),
            90
        );
    })
}

#[test]
fn remove_liquidity_whole_share_should_work() {
    new_test_ext().execute_with(|| {
        // A pool with a single LP provider
        // who deposit tokens and withdraws their whole share
        // (most simple case)

        let _ = AMM::add_liquidity(Origin::signed(1.into()), (DOT, XDOT), (10, 90));

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

        let _ = AMM::add_liquidity(Origin::signed(1.into()), (DOT, XDOT), (10, 90));

        assert_ok!(AMM::remove_liquidity(
            Origin::signed(1.into()),
            (DOT, XDOT),
            5
        ));

        assert_eq!(
            AMM::liquidity_providers((AccountId(1u64), XDOT, DOT)).ownership,
            25
        );

        assert_eq!(AMM::pools(XDOT, DOT).ownership, 25);

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
