use super::*;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok};
use frame_system::RawOrigin;

#[test]
fn create_pool_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::create_pool(
            RawOrigin::Signed(ALICE).into(),
            (DOT, XDOT),
            (10, 20),
            BOB,
            SAMPLE_LP_TOKEN,
        ));

        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 20);
        assert_eq!(
            AMM::liquidity_providers((BOB, XDOT, DOT))
                .unwrap()
                .base_amount,
            20
        );
        assert_eq!(Assets::total_issuance(SAMPLE_LP_TOKEN), 14);
        assert_eq!(Assets::balance(SAMPLE_LP_TOKEN, BOB), 14);
    })
}

#[test]
fn add_liquidity_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::create_pool(
            RawOrigin::Signed(ALICE).into(),
            (DOT, XDOT),
            (10, 20),
            ALICE,
            SAMPLE_LP_TOKEN,
        ));
        assert_ok!(AMM::add_liquidity(
            RawOrigin::Signed(ALICE).into(),
            (DOT, XDOT),
            (10, 20),
            (5, 5),
        ));

        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 40);

        assert_eq!(
            AMM::liquidity_providers((ALICE, XDOT, DOT))
                .unwrap()
                .base_amount,
            40
        );
    })
}

#[test]
fn add_more_liquidity_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::create_pool(
            RawOrigin::Signed(ALICE).into(),
            (DOT, XDOT),
            (10, 20),
            ALICE,
            SAMPLE_LP_TOKEN
        ));

        assert_ok!(AMM::add_liquidity(
            RawOrigin::Signed(ALICE).into(),
            (DOT, XDOT),
            (30, 40),
            (5, 5),
        ));

        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 60);

        assert_eq!(
            AMM::liquidity_providers((ALICE, XDOT, DOT))
                .unwrap()
                .base_amount,
            60
        );

        assert_eq!(
            AMM::liquidity_providers((ALICE, XDOT, DOT))
                .unwrap()
                .quote_amount,
            30
        );

        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 60);

        assert_eq!(AMM::pools(XDOT, DOT).unwrap().quote_amount, 30);
    })
}

#[test]
fn add_more_liquidity_should_not_work_if_minimum_base_amount_is_higher() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::create_pool(
            RawOrigin::Signed(ALICE).into(),
            (DOT, XDOT),
            (10, 20),
            ALICE,
            SAMPLE_LP_TOKEN
        ));

        assert_noop!(
            AMM::add_liquidity(
                RawOrigin::Signed(ALICE).into(),
                (DOT, XDOT),
                (30, 40),
                (55, 5)
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
            (10, 20),
            ALICE,
            SAMPLE_LP_TOKEN
        ));

        assert_ok!(AMM::add_liquidity(
            RawOrigin::Signed(ALICE).into(),
            (DOT, XDOT),
            (30, 40),
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
            (10, 20),
            ALICE,
            SAMPLE_LP_TOKEN
        ));

        assert_ok!(AMM::add_liquidity(
            RawOrigin::Signed(ALICE).into(),
            (DOT, XDOT),
            (30, 40),
            (5, 5),
        ));

        assert_ok!(AMM::add_liquidity(
            RawOrigin::Signed(BOB).into(),
            (DOT, XDOT),
            (5, 10),
            (5, 5),
        ));

        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 70);
    })
}

#[test]
fn cannot_create_pool_twice() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::create_pool(
            RawOrigin::Signed(ALICE).into(),
            (DOT, XDOT),
            (10, 20),
            ALICE,
            SAMPLE_LP_TOKEN
        ));

        assert_noop!(
            AMM::create_pool(
                RawOrigin::Signed(ALICE).into(),
                (DOT, XDOT),
                (10, 20),
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
            (10, 90),
            ALICE,
            SAMPLE_LP_TOKEN,
        );

        assert_eq!(
            <Test as Config>::Assets::total_issuance(
                AMM::liquidity_providers((ALICE, XDOT, DOT))
                    .unwrap()
                    .pool_assets
            ),
            30
        );

        assert_ok!(AMM::remove_liquidity(
            RawOrigin::Signed(ALICE).into(),
            (DOT, XDOT),
            30
        ));

        assert_eq!(
            <Test as Config>::Assets::total_issuance(
                AMM::liquidity_providers((ALICE, XDOT, DOT))
                    .unwrap()
                    .pool_assets
            ),
            0
        );
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
            (10, 90),
            ALICE,
            SAMPLE_LP_TOKEN,
        );

        assert_ok!(AMM::remove_liquidity(
            RawOrigin::Signed(ALICE).into(),
            (DOT, XDOT),
            15
        ));

        assert_eq!(
            <Test as Config>::Assets::total_issuance(
                AMM::liquidity_providers((ALICE, XDOT, DOT))
                    .unwrap()
                    .pool_assets
            ),
            15
        );
    })
}

#[test]
fn remove_liquidity_user_more_liquidity_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::create_pool(
            RawOrigin::Signed(ALICE).into(),
            (DOT, XDOT),
            (10, 25),
            ALICE,
            SAMPLE_LP_TOKEN
        ));
        assert_ok!(AMM::add_liquidity(
            RawOrigin::Signed(ALICE).into(),
            (DOT, XDOT),
            (15, 30),
            (5, 5),
        ));

        assert_ok!(AMM::remove_liquidity(
            RawOrigin::Signed(ALICE).into(),
            (DOT, XDOT),
            15
        ));

        assert_eq!(
            <Test as Config>::Assets::total_issuance(
                AMM::liquidity_providers((ALICE, XDOT, DOT))
                    .unwrap()
                    .pool_assets
            ),
            18
        );
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
            (10, 90),
            ALICE,
            SAMPLE_LP_TOKEN,
        );

        assert_noop!(
            AMM::remove_liquidity(RawOrigin::Signed(ALICE).into(), (DOT, XDOT), 300),
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
            (100, 100),
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
