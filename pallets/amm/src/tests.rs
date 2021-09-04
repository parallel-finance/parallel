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

        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 20);

        assert_eq!(
            AMM::liquidity_providers((AccountId(1u64), XDOT, DOT)).base_amount,
            20
        );

        // Check balance is correct
        assert_eq!(
            <Test as Config<pallet_balances::Instance1>>::Currency::free_balance(
                CurrencyId::DOT,
                &1.into()
            ),
            90
        );
        assert_eq!(
            <Test as Config<pallet_balances::Instance1>>::Currency::free_balance(
                CurrencyId::xDOT,
                &1.into()
            ),
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

        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 60);

        assert_eq!(
            AMM::liquidity_providers((AccountId(1u64), XDOT, DOT)).base_amount,
            60
        );

        assert_eq!(
            AMM::liquidity_providers((AccountId(1u64), XDOT, DOT)).quote_amount,
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
            Origin::signed(1.into()),
            (DOT, XDOT),
            (10, 20),
            (5, 5)
        ));

        assert_noop!(
            AMM::add_liquidity(Origin::signed(1.into()), (DOT, XDOT), (30, 40), (55, 5)),
            Error::<Test, Instance1>::NotAIdealPriceRatio
        );
    })
}

#[test]
fn add_liquidity_should_not_work_if_not_allowed_for_normal_user() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            PermissionedAMM::add_liquidity(
                Origin::signed(1.into()),
                (DOT, XDOT),
                (30, 40),
                (55, 5)
            ),
            Error::<Test, Instance2>::PoolCreationDisabled
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
            1.into()
        ));

        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 20);

        assert_eq!(
            AMM::liquidity_providers((AccountId(1u64), XDOT, DOT)).base_amount,
            20
        );

        // Check balance is correct
        assert_eq!(
            <Test as Config<pallet_balances::Instance1>>::Currency::free_balance(
                CurrencyId::DOT,
                &1.into()
            ),
            90
        );
        assert_eq!(
            <Test as Config<pallet_balances::Instance1>>::Currency::free_balance(
                CurrencyId::xDOT,
                &1.into()
            ),
            80
        );
    })
}

#[test]
fn add_liquidity_by_root_should_not_work_if_pool_already_exists() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::add_liquidity(
            Origin::signed(1.into()),
            (DOT, XDOT),
            (10, 20),
            (5, 5)
        ));

        assert_noop!(
            AMM::force_create_pool(
                frame_system::RawOrigin::Root.into(),
                (DOT, XDOT),
                (10, 20),
                1.into()
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

        assert_eq!(AMM::pools(XDOT, DOT).unwrap().ownership, 0);

        // Check balance is correct
        assert_eq!(
            <Test as Config<pallet_balances::Instance1>>::Currency::free_balance(
                CurrencyId::DOT,
                &1.into()
            ),
            100
        );
        assert_eq!(
            <Test as Config<pallet_balances::Instance1>>::Currency::free_balance(
                CurrencyId::xDOT,
                &1.into()
            ),
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

        assert_eq!(AMM::pools(XDOT, DOT).unwrap().ownership, 15);

        // Check balance is correct
        assert_eq!(
            <Test as Config<pallet_balances::Instance1>>::Currency::free_balance(
                CurrencyId::DOT,
                &1.into()
            ),
            95
        );
        assert_eq!(
            <Test as Config<pallet_balances::Instance1>>::Currency::free_balance(
                CurrencyId::xDOT,
                &1.into()
            ),
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

        assert_eq!(AMM::pools(XDOT, DOT).unwrap().ownership, 3);

        // Check balance is correct
        assert_eq!(
            <Test as Config<pallet_balances::Instance1>>::Currency::free_balance(
                CurrencyId::DOT,
                &1.into()
            ),
            96
        );
        assert_eq!(
            <Test as Config<pallet_balances::Instance1>>::Currency::free_balance(
                CurrencyId::xDOT,
                &1.into()
            ),
            90
        );
    })
}

#[test]
fn remove_liquidity_when_pool_does_not_exist_should_not_work() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            AMM::remove_liquidity(Origin::signed(1.into()), (DOT, XDOT), 15),
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

        let _ = AMM::add_liquidity(Origin::signed(1.into()), (DOT, XDOT), (10, 90), (5, 5));

        assert_noop!(
            AMM::remove_liquidity(Origin::signed(1.into()), (DOT, XDOT), 300),
            Error::<Test, Instance1>::MoreLiquidity
        );
    })
}

#[test]
fn trade_should_work() {
    new_test_ext().execute_with(|| {
        // create pool and add liquidity
        AMM::add_liquidity(
            Origin::signed(3.into()),
            (DOT, XDOT),
            (100_000, 100_000),
            (99_999, 99_999),
        )
        .expect("Error initalizing AMM");

        // check that pool was funded correctly
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 100_000); // XDOT
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().quote_amount, 100_000); // DOT

        // calculate amount out
        let amount_out = AMM::trade(&AccountId::from(4_u64), (DOT, XDOT), 1_000, 980);
        // lp fee is 0.03% or 3/1000 or         0.003*1000=3
        // protocol fee is 0.02% or 2/1000 or   0.002*1000=2
        // total fee is 0.05% or 5/1000 or      0.005*1000=5

        // amount out should be 987
        assert_eq!(amount_out.unwrap(), 985);

        // // pools values should be updated - we should have less XDOT
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 99_015);

        // // pools values should be updated - we should have more DOT in the pool
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().quote_amount, 101_000);
    })
}

#[test]
fn trade_should_work_flipped_currencies() {
    new_test_ext().execute_with(|| {
        // create pool and add liquidity
        AMM::add_liquidity(Origin::signed(1.into()), (DOT, XDOT), (100, 50), (90, 90))
            .expect("Error initalizing AMM");

        // check that pool was funded correctly
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().quote_amount, 100); // DOT
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 50); // XDOT

        // calculate amount out
        let amount_out = AMM::trade(&AccountId::from(2_u64), (XDOT, DOT), 10, 15);
        // fee is 0.03% or 3/1000 or 0.003*16=0.048
        // lp fee is 0.03% or 3/1000 or         0.003*16=0.048
        // protocol fee is 0.02% or 2/1000 or   0.002*16=0.032
        // total fee is 0.05% or 5/1000 or      0.005*16=0.08
        // this is returned as 0

        // amount out should be 16
        assert_eq!(amount_out.unwrap(), 16);

        // pools values should be updated - we should have less DOT in the pool
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().quote_amount, 84);

        // pools values should be updated - we should have more XDOT
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 60);
    })
}

#[test]
fn trade_should_not_work_if_amount_less_than_miniumum() {
    new_test_ext().execute_with(|| {
        // create pool and add liquidity
        AMM::add_liquidity(Origin::signed(1.into()), (DOT, XDOT), (100, 100), (90, 90))
            .expect("Error initalizing AMM");

        // check that pool was funded correctly
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 100);
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().quote_amount, 100);

        // amount out is less than minimum_amount_out
        assert_noop!(
            AMM::trade(&AccountId::from(2_u64), (DOT, XDOT), 10, 10),
            Error::<Test, Instance1>::InsufficientAmountOut
        );
    })
}

#[test]
fn trade_should_not_work_if_amount_in_is_zero() {
    new_test_ext().execute_with(|| {
        // create pool and add liquidity
        AMM::add_liquidity(Origin::signed(1.into()), (DOT, XDOT), (100, 100), (90, 90))
            .expect("Error initalizing AMM");

        // fail if amount_in is zero
        assert_noop!(
            AMM::trade(&AccountId::from(2_u64), (DOT, XDOT), 0, 0),
            Error::<Test, Instance1>::InsufficientAmountOut
        );
    })
}

#[test]
fn trade_should_not_work_if_pool_does_not_exist() {
    new_test_ext().execute_with(|| {
        // try to trade in pool with no liquidity
        assert_noop!(
            AMM::trade(&AccountId::from(2_u64), (DOT, XDOT), 10, 10),
            Error::<Test, Instance1>::PoolDoesNotExist
        );
    })
}
