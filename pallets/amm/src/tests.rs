use super::*;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok};

#[test]
fn add_liquidity_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::add_liquidity(
            Origin::signed(ALICE),
            (DOT, XDOT),
            (10, 20),
            (5, 5),
            10
        ));

        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 20);

        assert_eq!(AMM::liquidity_providers((ALICE, XDOT, DOT)).base_amount, 20);
    })
}

#[test]
fn add_more_liquidity_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::add_liquidity(
            Origin::signed(ALICE),
            (DOT, XDOT),
            (10, 20),
            (5, 5),
            10
        ));

        assert_ok!(AMM::add_liquidity(
            Origin::signed(ALICE),
            (DOT, XDOT),
            (30, 40),
            (5, 5),
            10
        ));

        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 60);

        assert_eq!(AMM::liquidity_providers((ALICE, XDOT, DOT)).base_amount, 60);

        assert_eq!(
            AMM::liquidity_providers((ALICE, XDOT, DOT)).quote_amount,
            30
        );

        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 60);

        assert_eq!(AMM::pools(XDOT, DOT).unwrap().quote_amount, 30);
    })
}

#[test]
fn add_more_liquidity_should_not_work_if_minimum_base_amount_is_higher() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::add_liquidity(
            Origin::signed(ALICE),
            (DOT, XDOT),
            (10, 20),
            (5, 5),
            10
        ));

        assert_noop!(
            AMM::add_liquidity(Origin::signed(ALICE), (DOT, XDOT), (30, 40), (55, 5), 10),
            Error::<Test, Instance1>::NotAIdealPriceRatio
        );
    })
}

#[test]
fn add_more_liquidity_should_not_work_for_same_assetid() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::add_liquidity(
            Origin::signed(ALICE),
            (DOT, XDOT),
            (10, 20),
            (5, 5),
            10
        ));

        assert_noop!(
            AMM::add_liquidity(Origin::signed(ALICE), (DOT, HKO), (30, 40), (55, 5), 10),
            pallet_assets::Error::<Test>::InUse
        );
    })
}

#[test]
fn add_liquidity_should_not_work_if_not_allowed_for_normal_user() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            PermissionedAMM::add_liquidity(
                Origin::signed(ALICE),
                (DOT, XDOT),
                (30, 40),
                (55, 5),
                10
            ),
            Error::<Test, Instance2>::PoolCreationDisabled
        );
    })
}

#[test]
fn add_more_liquidity_with_low_balance_should_not_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::add_liquidity(
            Origin::signed(ALICE),
            (DOT, XDOT),
            (10, 20),
            (5, 5),
            10
        ));

        assert_ok!(AMM::add_liquidity(
            Origin::signed(ALICE),
            (DOT, XDOT),
            (30, 40),
            (1, 1),
            10
        ));

        assert_noop!(
            AMM::add_liquidity(
                Origin::signed(ALICE),
                (DOT, XDOT),
                (5000_000_000, 6000_000_000),
                (5, 5),
                10
            ),
            pallet_assets::Error::<Test>::BalanceLow
        );
    })
}

#[test]
fn add_liquidity_by_another_user_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::add_liquidity(
            Origin::signed(ALICE),
            (DOT, XDOT),
            (10, 20),
            (5, 5),
            10
        ));

        assert_ok!(AMM::add_liquidity(
            Origin::signed(ALICE),
            (DOT, XDOT),
            (30, 40),
            (5, 5),
            10
        ));

        assert_ok!(AMM::add_liquidity(
            Origin::signed(BOB),
            (DOT, XDOT),
            (5, 10),
            (5, 5),
            10
        ));

        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 70);
    })
}

#[test]
fn add_liquidity_should_work_if_created_by_root() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::force_create_pool(
            frame_system::RawOrigin::Root.into(),
            (DOT, XDOT),
            (10, 20),
            ALICE,
            12
        ));

        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 20);

        assert_eq!(AMM::liquidity_providers((ALICE, XDOT, DOT)).base_amount, 20);
    })
}

#[test]
fn add_liquidity_by_root_should_not_work_if_pool_already_exists() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::add_liquidity(
            Origin::signed(ALICE),
            (DOT, XDOT),
            (10, 20),
            (5, 5),
            10
        ));

        assert_noop!(
            AMM::force_create_pool(
                frame_system::RawOrigin::Root.into(),
                (DOT, XDOT),
                (10, 20),
                ALICE,
                10
            ),
            Error::<Test, Instance1>::PoolAlreadyExists,
        );
    })
}

#[test]
fn remove_liquidity_whole_share_should_work() {
    new_test_ext().execute_with(|| {
        // A pool with a single LP provider
        // who deposit tokens and withdraws their whole share
        // (most simple case)

        let _ = AMM::add_liquidity(Origin::signed(ALICE), (DOT, XDOT), (10, 90), (5, 5), 10);

        assert_eq!(
            <Test as Config<pallet_balances::Instance1>>::Assets::total_issuance(
                AMM::liquidity_providers((ALICE, XDOT, DOT)).pool_assets
            ),
            30
        );

        assert_ok!(AMM::remove_liquidity(
            Origin::signed(ALICE),
            (DOT, XDOT),
            30
        ));

        assert_eq!(
            <Test as Config<pallet_balances::Instance1>>::Assets::total_issuance(
                AMM::liquidity_providers((ALICE, XDOT, DOT)).pool_assets
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

        let _ = AMM::add_liquidity(Origin::signed(ALICE), (DOT, XDOT), (10, 90), (5, 5), 10);

        assert_ok!(AMM::remove_liquidity(
            Origin::signed(ALICE),
            (DOT, XDOT),
            15
        ));

        assert_eq!(
            <Test as Config<pallet_balances::Instance1>>::Assets::total_issuance(
                AMM::liquidity_providers((ALICE, XDOT, DOT)).pool_assets
            ),
            15
        );
    })
}

#[test]
fn remove_liquidity_user_more_liquidity_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::add_liquidity(
            Origin::signed(ALICE),
            (DOT, XDOT),
            (10, 25),
            (5, 5),
            10
        ));
        assert_ok!(AMM::add_liquidity(
            Origin::signed(ALICE),
            (DOT, XDOT),
            (15, 30),
            (5, 5),
            10
        ));

        assert_ok!(AMM::remove_liquidity(
            Origin::signed(ALICE),
            (DOT, XDOT),
            15
        ));

        assert_eq!(
            <Test as Config<pallet_balances::Instance1>>::Assets::total_issuance(
                AMM::liquidity_providers((ALICE, XDOT, DOT)).pool_assets
            ),
            18
        );
    })
}

#[test]
fn remove_liquidity_when_pool_does_not_exist_should_not_work() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            AMM::remove_liquidity(Origin::signed(ALICE), (DOT, XDOT), 15),
            Error::<Test, Instance1>::PoolDoesNotExist
        );
    })
}

#[test]
fn remove_liquidity_with_more_liquidity_should_not_work() {
    new_test_ext().execute_with(|| {
        // A pool with a single LP provider
        // who deposit tokens and withdraws their whole share
        // (most simple case)

        let _ = AMM::add_liquidity(Origin::signed(ALICE), (DOT, XDOT), (10, 90), (5, 5), 10);

        assert_noop!(
            AMM::remove_liquidity(Origin::signed(ALICE), (DOT, XDOT), 300),
            Error::<Test, Instance1>::MoreLiquidity
        );
    })
}

#[test]
fn trade_should_work() {
    new_test_ext().execute_with(|| {
        use primitives::AMM as _;

        let trader = EVE;

        // create pool and add liquidity
        assert_ok!(AMM::add_liquidity(
            Origin::signed(CHARLIE),
            (DOT, XDOT),
            (100_000_000, 100_000_000),
            (99_999, 99_999),
            10
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

        // create pool and add liquidity
        assert_ok!(AMM::add_liquidity(
            Origin::signed(CHARLIE),
            (DOT, XDOT),
            (100_000, 100_000),
            (99_999, 99_999),
            10
        ));

        // check that pool was funded correctly
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 100_000); // XDOT
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().quote_amount, 100_000); // DOT

        // amount out is less than minimum_amount_out
        assert_noop!(
            AMM::trade(&trader, (DOT, XDOT), 332, 300),
            Error::<Test, Instance1>::InsufficientAmountIn
        );
    })
}

#[test]
fn trade_should_work_flipped_currencies() {
    new_test_ext().execute_with(|| {
        use primitives::AMM as _;

        let trader = EVE;

        // create pool and add liquidity
        assert_ok!(AMM::add_liquidity(
            Origin::signed(CHARLIE),
            (DOT, XDOT),
            (100_000, 50_000),
            (99_999, 49_999),
            10
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
        assert_ok!(AMM::add_liquidity(
            Origin::signed(CHARLIE),
            (DOT, XDOT),
            (100_000, 100_000),
            (99_999, 99_999),
            10
        ));
        // check that pool was funded correctly
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 100_000);
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().quote_amount, 100_000);

        // amount out is less than minimum_amount_out
        assert_noop!(
            AMM::trade(&trader, (DOT, XDOT), 1_000, 1_000),
            Error::<Test, Instance1>::InsufficientAmountOut
        );
    })
}

#[test]
fn trade_should_not_work_if_amount_in_is_zero() {
    new_test_ext().execute_with(|| {
        use primitives::AMM as _;

        let trader = EVE;

        // create pool and add liquidity
        assert_ok!(AMM::add_liquidity(
            Origin::signed(ALICE),
            (DOT, XDOT),
            (100, 100),
            (90, 90),
            10
        ));

        // fail if amount_in is zero
        assert_noop!(
            AMM::trade(&trader, (DOT, XDOT), 0, 0),
            Error::<Test, Instance1>::InsufficientAmountIn
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
            Error::<Test, Instance1>::PoolDoesNotExist
        );
    })
}
