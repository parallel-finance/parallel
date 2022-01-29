use super::*;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok};
use frame_system::RawOrigin;
use primitives::AMM as _;

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
            Error::<Test>::NotAnIdealPrice
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
            Error::<Test>::InsufficientLiquidity
        );
    })
}

#[test]
fn trade_should_work_base_to_quote() {
    new_test_ext().execute_with(|| {
        let trader = EVE;

        // create pool and add liquidity
        assert_ok!(AMM::create_pool(
            RawOrigin::Signed(ALICE).into(),
            (DOT, XDOT),
            (100_000_000, 100_000_000),
            CHARLIE,
            SAMPLE_LP_TOKEN,
        ));

        // XDOT is base_asset 1001
        // DOT is quote_asset 101

        // check that pool was funded correctly
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 100_000_000); // XDOT
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().quote_amount, 100_000_000); // DOT

        // calculate amount out
        let amount_out = AMM::trade(&trader, (DOT, XDOT), 1_000, 980);

        // amount out should be 994
        assert_eq!(amount_out.unwrap(), 996);

        // pools values should be updated - we should have less XDOT
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 99_999_004);

        // pools values should be updated - we should have more DOT in the pool
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().quote_amount, 100_001_000);
    })
}

#[test]
fn trade_should_work_base_to_quote_flipped_currencies_on_pool_creation() {
    new_test_ext().execute_with(|| {
        let trader = EVE;

        // create pool and add liquidity
        assert_ok!(AMM::create_pool(
            RawOrigin::Signed(ALICE).into(),
            (XDOT, DOT),
            (100_000_000, 100_000_000),
            CHARLIE,
            SAMPLE_LP_TOKEN,
        ));

        // XDOT is base_asset 1001
        // DOT is quote_asset 101

        // check that pool was funded correctly
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 100_000_000); // XDOT
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().quote_amount, 100_000_000); // DOT

        // calculate amount out
        let amount_out = AMM::trade(&trader, (DOT, XDOT), 1_000, 980);

        // amount out should be 994
        assert_eq!(amount_out.unwrap(), 996);

        // pools values should be updated - we should have less XDOT
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 99_999_004);

        // pools values should be updated - we should have more DOT in the pool
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().quote_amount, 100_001_000);
    })
}

#[test]
fn trade_should_work_quote_to_base() {
    new_test_ext().execute_with(|| {
        let trader = EVE;

        // create pool and add liquidity
        assert_ok!(AMM::create_pool(
            RawOrigin::Signed(ALICE).into(),
            (DOT, XDOT),
            (100_000_000, 100_000_000),
            CHARLIE,
            SAMPLE_LP_TOKEN,
        ));

        // XDOT is base_asset 1001
        // DOT is quote_asset 101

        // check that pool was funded correctly
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 100_000_000); // XDOT
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().quote_amount, 100_000_000); // DOT

        // calculate amount out
        // trade base for quote
        let amount_out = AMM::trade(&trader, (DOT, XDOT), 1_000, 980);

        // amount out should be 996
        assert_eq!(amount_out.unwrap(), 996);

        // we should have more DOT in the pool since were trading it for DOT
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().quote_amount, 100_001_000);

        // we should have less XDOT since we traded it for XDOT
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 99_999_004);
    })
}

#[test]
fn trade_should_not_work_if_insufficient_amount_in() {
    new_test_ext().execute_with(|| {
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
        // fees = 1.5 (rounded to 1)

        assert_eq!(amount_out.unwrap(), 988);

        // pools values should be updated - we should have less DOT in the pool
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().quote_amount, 99_012);

        // pools values should be updated - we should have more XDOT
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 50_500);
    })
}

#[test]
fn trade_should_not_work_if_amount_less_than_miniumum() {
    new_test_ext().execute_with(|| {
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
            Error::<Test>::NotAnIdealPrice
        );
    })
}

#[test]
fn trade_should_not_work_if_amount_in_is_zero() {
    new_test_ext().execute_with(|| {
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
        let trader = EVE;

        // try to trade in pool with no liquidity
        assert_noop!(
            AMM::trade(&trader, (DOT, XDOT), 10, 10),
            Error::<Test>::PoolDoesNotExist
        );
    })
}

#[test]
fn amount_out_should_work() {
    new_test_ext().execute_with(|| {
        let amount_in = 1_000;
        let supply_in = 100_000_000;
        let supply_out = 100_000_000;

        let amount_out = AMM::get_amount_out(amount_in, supply_in, supply_out).unwrap();

        // actual value == 996.9900600091017
        // TODO: assumes we round down to int
        assert_eq!(amount_out, 996);
    })
}

#[test]
fn amounts_out_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::create_pool(
            RawOrigin::Signed(ALICE).into(),
            (DOT, XDOT),
            (1_000, 2_000),
            BOB,
            SAMPLE_LP_TOKEN,
        ));

        assert_ok!(AMM::create_pool(
            RawOrigin::Signed(ALICE).into(),
            (KSM, DOT),
            (1_000, 1_000),
            BOB,
            SAMPLE_LP_TOKEN,
        ));

        let path = Path::<Test, ()>::try_from(vec![XDOT, DOT, KSM]).unwrap();

        let amount_in = 1_000;

        let amounts_out = AMM::get_amounts_out(amount_in, path).unwrap();

        assert_eq!(amounts_out, [1000, 332, 249]);
    })
}

#[test]
fn amounts_in_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::create_pool(
            RawOrigin::Signed(ALICE).into(),
            (DOT, XDOT),
            (1_000, 2_000),
            BOB,
            SAMPLE_LP_TOKEN,
        ));

        assert_ok!(AMM::create_pool(
            RawOrigin::Signed(ALICE).into(),
            (KSM, DOT),
            (1_000, 1_000),
            BOB,
            SAMPLE_LP_TOKEN,
        ));

        let path = Path::<Test, ()>::try_from(vec![XDOT, DOT, KSM]).unwrap();

        let amount_out = 1_000;

        let amounts_in = AMM::get_amounts_in(amount_out, path).unwrap();

        assert_eq!(amounts_in, [1, 0, 1000]);
    })
}

#[test]
fn amount_in_should_work() {
    new_test_ext().execute_with(|| {
        let amount_out = 1_000;
        let supply_in = 100_000_000;
        let supply_out = 100_000_000;

        let amount_in = AMM::get_amount_in(amount_out, supply_in, supply_out).unwrap();

        // actual value == 1004.0190572718165
        // TODO: assumes we round down to int
        assert_eq!(amount_in, 1004);
    })
}

#[test]
fn amount_out_and_in_should_work() {
    new_test_ext().execute_with(|| {
        let amount_out = 1_000;
        let supply_in = 100_000_000;
        let supply_out = 100_000_000;

        let amount_in = AMM::get_amount_in(amount_out, supply_in, supply_out).unwrap();

        assert_eq!(amount_in, 1004);

        let amount_out = AMM::get_amount_out(amount_in, supply_in, supply_out).unwrap();

        assert_eq!(amount_out, 1000);
    })
}
