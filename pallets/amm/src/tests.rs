use super::*;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok};
use frame_system::RawOrigin;

const MINIMUM_LIQUIDITY: u128 = 1_000;

#[test]
fn create_pool_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::create_pool(
            RawOrigin::Signed(ALICE).into(),
            (DOT, XDOT),
            (1_000, 2_000),
            BOB,
            SAMPLE_LP_TOKEN,
        ));

        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 2_000);
        assert_eq!(Assets::total_issuance(SAMPLE_LP_TOKEN), 1_414);
        // should be issuance minus the min liq locked
        assert_eq!(Assets::balance(SAMPLE_LP_TOKEN, BOB), 414);
    })
}

#[test]
fn add_liquidity_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::create_pool(
            RawOrigin::Signed(ALICE).into(),
            (DOT, XDOT),
            (1_000, 2_000),
            ALICE,
            SAMPLE_LP_TOKEN,
        ));
        assert_ok!(AMM::add_liquidity(
            RawOrigin::Signed(ALICE).into(),
            (DOT, XDOT),
            (1_000, 2_000),
            (5, 5),
        ));

        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 4_000);
    })
}

#[test]
fn add_more_liquidity_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::create_pool(
            RawOrigin::Signed(ALICE).into(),
            (DOT, XDOT),
            (1_000, 2_000),
            ALICE,
            SAMPLE_LP_TOKEN
        ));

        assert_ok!(AMM::add_liquidity(
            RawOrigin::Signed(ALICE).into(),
            (DOT, XDOT),
            (3_000, 4_000),
            (5, 5),
        ));

        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 6_000);
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().quote_amount, 3_000);
    })
}

#[test]
fn add_more_liquidity_should_not_work_if_minimum_base_amount_is_higher() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::create_pool(
            RawOrigin::Signed(ALICE).into(),
            (DOT, XDOT),
            (1_000, 2_000),
            ALICE,
            SAMPLE_LP_TOKEN
        ));

        assert_noop!(
            AMM::add_liquidity(
                RawOrigin::Signed(ALICE).into(),
                (DOT, XDOT),
                (3_000, 4_000),
                (5_500, 5_00)
            ),
            Error::<Test>::NotAIdealPriceRatio
        );
    })
}

#[test]
fn add_more_liquidity_with_low_balance_should_not_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::create_pool(
            RawOrigin::Signed(ALICE).into(),
            (DOT, XDOT),
            (1_000, 2_000),
            ALICE,
            SAMPLE_LP_TOKEN
        ));

        assert_ok!(AMM::add_liquidity(
            RawOrigin::Signed(ALICE).into(),
            (DOT, XDOT),
            (3_000, 4_000),
            (1, 1),
        ));

        assert_noop!(
            AMM::add_liquidity(
                RawOrigin::Signed(ALICE).into(),
                (DOT, XDOT),
                (5000_000_000, 6000_000_000),
                (5, 5),
            ),
            pallet_assets::Error::<Test>::BalanceLow
        );
    })
}

#[test]
fn add_liquidity_by_another_user_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::create_pool(
            RawOrigin::Signed(ALICE).into(),
            (DOT, XDOT),
            (1_000, 2_000),
            ALICE,
            SAMPLE_LP_TOKEN
        ));

        assert_ok!(AMM::add_liquidity(
            RawOrigin::Signed(ALICE).into(),
            (DOT, XDOT),
            (3_000, 4_000),
            (5, 5),
        ));

        assert_ok!(AMM::add_liquidity(
            RawOrigin::Signed(BOB).into(),
            (DOT, XDOT),
            (500, 1_000),
            (5, 5),
        ));

        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 7_000);
    })
}

#[test]
fn cannot_create_pool_twice() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::create_pool(
            RawOrigin::Signed(ALICE).into(),
            (DOT, XDOT),
            (1_000, 2_000),
            ALICE,
            SAMPLE_LP_TOKEN
        ));

        assert_noop!(
            AMM::create_pool(
                RawOrigin::Signed(ALICE).into(),
                (DOT, XDOT),
                (1_000, 2_000),
                ALICE,
                SAMPLE_LP_TOKEN
            ),
            Error::<Test>::PoolAlreadyExists,
        );
    })
}

#[test]
fn remove_liquidity_whole_share_should_work() {
    new_test_ext().execute_with(|| {
        // A pool with a single LP provider
        // who deposit tokens and withdraws their whole share
        // (most simple case)

        let _ = AMM::create_pool(
            RawOrigin::Signed(ALICE).into(),
            (DOT, XDOT),
            (1_000, 9_000),
            ALICE,
            SAMPLE_LP_TOKEN,
        );

        assert_ok!(AMM::remove_liquidity(
            RawOrigin::Signed(ALICE).into(),
            (DOT, XDOT),
            3_000 - MINIMUM_LIQUIDITY
        ));
    })
}

#[test]
fn remove_liquidity_only_portion_should_work() {
    new_test_ext().execute_with(|| {
        // A pool with a single LP provider who
        // deposit tokens and withdraws
        // a portion of their total shares (simple case)

        let _ = AMM::create_pool(
            RawOrigin::Signed(ALICE).into(),
            (DOT, XDOT),
            (1_000, 9_000),
            ALICE,
            SAMPLE_LP_TOKEN,
        );

        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 9_000);
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().quote_amount, 1_000);

        assert_ok!(AMM::remove_liquidity(
            RawOrigin::Signed(ALICE).into(),
            (DOT, XDOT),
            1_500
        ));

        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 4_500);
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().quote_amount, 500);
    })
}

#[test]
fn remove_liquidity_user_more_liquidity_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::create_pool(
            RawOrigin::Signed(ALICE).into(),
            (DOT, XDOT),
            (1_000, 2_500),
            ALICE,
            SAMPLE_LP_TOKEN
        ));
        assert_ok!(AMM::add_liquidity(
            RawOrigin::Signed(ALICE).into(),
            (DOT, XDOT),
            (1_500, 3_000),
            (5, 5),
        ));

        assert_ok!(AMM::remove_liquidity(
            RawOrigin::Signed(ALICE).into(),
            (DOT, XDOT),
            1_500
        ));
    })
}

#[test]
fn remove_liquidity_when_pool_does_not_exist_should_not_work() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            AMM::remove_liquidity(RawOrigin::Signed(ALICE).into(), (DOT, XDOT), 15),
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

        let _ = AMM::create_pool(
            RawOrigin::Signed(ALICE).into(),
            (DOT, XDOT),
            (1_000, 9_000),
            ALICE,
            SAMPLE_LP_TOKEN,
        );

        assert_noop!(
            AMM::remove_liquidity(RawOrigin::Signed(ALICE).into(), (DOT, XDOT), 3_0000),
            Error::<Test>::MoreLiquidity
        );
    })
}

#[test]
fn trade_should_work() {
    new_test_ext().execute_with(|| {
        use primitives::AMM as _;

        let trader = EVE;

        // create pool and add liquidity
        assert_ok!(AMM::create_pool(
            RawOrigin::Signed(ALICE).into(),
            (DOT, XDOT),
            (100_000_000, 100_000_000),
            CHARLIE,
            SAMPLE_LP_TOKEN,
        ));

        // check that pool was funded correctly
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 100_000_000); // XDOT
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().quote_amount, 100_000_000); // DOT

        // calculate amount out
        let amount_out = AMM::trade(&trader, (DOT, XDOT), 1_000, 980);

        // amount out should be 994
        assert_eq!(amount_out.unwrap(), 994);

        // // pools values should be updated - we should have less XDOT
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 99_999_006);

        // // pools values should be updated - we should have more DOT in the pool
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().quote_amount, 100_000_998);
    })
}

#[test]
fn trade_should_not_work_if_insufficient_amount_in() {
    new_test_ext().execute_with(|| {
        use primitives::AMM as _;

        let trader = EVE;

        assert_ok!(AMM::create_pool(
            RawOrigin::Signed(ALICE).into(),
            (DOT, XDOT),
            (100_000, 100_000),
            CHARLIE,
            SAMPLE_LP_TOKEN,
        ));

        // create pool and add liquidity
        assert_ok!(AMM::add_liquidity(
            RawOrigin::Signed(CHARLIE).into(),
            (DOT, XDOT),
            (100_000, 100_000),
            (99_999, 99_999),
        ));

        // check that pool was funded correctly
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 200_000); // XDOT
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().quote_amount, 200_000); // DOT

        // amount out is less than minimum_amount_out
        assert_noop!(
            AMM::trade(&trader, (DOT, XDOT), 332, 300),
            Error::<Test>::InsufficientAmountIn
        );
    })
}

#[test]
fn trade_should_work_flipped_currencies() {
    new_test_ext().execute_with(|| {
        use primitives::AMM as _;

        let trader = EVE;

        // create pool and add liquidity
        assert_ok!(AMM::create_pool(
            RawOrigin::Signed(ALICE).into(),
            (DOT, XDOT),
            (100_000, 50_000),
            CHARLIE,
            SAMPLE_LP_TOKEN
        ));

        // check that pool was funded correctly
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().quote_amount, 100_000); // DOT
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 50_000); // XDOT

        // calculate amount out
        let amount_out = AMM::trade(&trader, (XDOT, DOT), 500, 800);
        // fees
        // lp = 1.5 (rounded to 1)
        // protocol = 1
        // total = 2

        // amount out should be 986
        assert_eq!(amount_out.unwrap(), 986);

        // pools values should be updated - we should have less DOT in the pool
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().quote_amount, 99_014);

        // pools values should be updated - we should have more XDOT
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 50_499);
    })
}

#[test]
fn trade_should_not_work_if_amount_less_than_miniumum() {
    new_test_ext().execute_with(|| {
        use primitives::AMM as _;

        let trader = EVE;

        // create pool and add liquidity
        assert_ok!(AMM::create_pool(
            RawOrigin::Signed(ALICE).into(),
            (DOT, XDOT),
            (100_000, 100_000),
            CHARLIE,
            SAMPLE_LP_TOKEN
        ));
        // check that pool was funded correctly
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 100_000);
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().quote_amount, 100_000);

        // amount out is less than minimum_amount_out
        assert_noop!(
            AMM::trade(&trader, (DOT, XDOT), 1_000, 1_000),
            Error::<Test>::InsufficientAmountOut
        );
    })
}

#[test]
fn trade_should_not_work_if_amount_in_is_zero() {
    new_test_ext().execute_with(|| {
        use primitives::AMM as _;

        let trader = EVE;

        // create pool and add liquidity
        assert_ok!(AMM::create_pool(
            RawOrigin::Signed(ALICE).into(),
            (DOT, XDOT),
            (1_000, 1_000),
            ALICE,
            SAMPLE_LP_TOKEN
        ));

        // fail if amount_in is zero
        assert_noop!(
            AMM::trade(&trader, (DOT, XDOT), 0, 0),
            Error::<Test>::InsufficientAmountIn
        );
    })
}

#[test]
fn trade_should_not_work_if_pool_does_not_exist() {
    new_test_ext().execute_with(|| {
        use primitives::AMM as _;

        let trader = EVE;

        // try to trade in pool with no liquidity
        assert_noop!(
            AMM::trade(&trader, (DOT, XDOT), 10, 10),
            Error::<Test>::PoolDoesNotExist
        );
    })
}
