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
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, XDOT),                     // Currency pool, in which liquidity will be added
            (1_000, 2_000),                  // Liquidity amounts to be added in pool
            BOB,                             // LPToken receiver
            SAMPLE_LP_TOKEN,                 // Liquidity pool share representative token
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
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, XDOT),                     // Currency pool, in which liquidity will be added
            (1_000, 2_000),                  // Liquidity amounts to be added in pool
            ALICE,                           // LPToken receiver
            SAMPLE_LP_TOKEN,                 // Liquidity pool share representative token
        ));
        assert_ok!(AMM::add_liquidity(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, XDOT),                     // Currency pool, in which liquidity will be added
            (1_000, 2_000),                  // Liquidity amounts to be added in pool
            (5, 5),                          // specifying its worst case ratio when pool already
        ));

        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 4_000);
    })
}

#[test]
fn add_more_liquidity_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, XDOT),                     // Currency pool, in which liquidity will be added
            (1_000, 2_000),                  // Liquidity amounts to be added in pool
            ALICE,                           // LPToken receiver
            SAMPLE_LP_TOKEN                  // Liquidity pool share representative token
        ));

        assert_ok!(AMM::add_liquidity(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, XDOT),                     // Currency pool, in which liquidity will be added
            (3_000, 4_000),                  // Liquidity amounts to be added in pool
            (5, 5), // specifying its worst case ratio when pool already exists
        ));

        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 6_000);
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().quote_amount, 3_000);
    })
}

#[test]
fn add_more_liquidity_should_not_work_if_minimum_base_amount_is_higher() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, XDOT),                     // Currency pool, in which liquidity will be added
            (1_000, 2_000),                  // Liquidity amounts to be added in pool
            ALICE,                           // LPToken receiver
            SAMPLE_LP_TOKEN                  // Liquidity pool share representative token
        ));

        assert_noop!(
            AMM::add_liquidity(
                RawOrigin::Signed(ALICE).into(), // Origin
                (DOT, XDOT),                     // Currency pool, in which liquidity will be added
                (3_000, 4_000),                  // Liquidity amounts to be added in pool
                (5_500, 5_00)                    // specifying its worst case ratio when pool already
            ),
            Error::<Test>::NotAnIdealPrice // Not an ideal price ratio
        );
    })
}

#[test]
fn add_more_liquidity_with_low_balance_should_not_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, XDOT),                     // Currency pool, in which liquidity will be added
            (1_000, 2_000),                  // Liquidity amounts to be added in pool
            ALICE,                           // LPToken receiver
            SAMPLE_LP_TOKEN                  // Liquidity pool share representative token
        ));

        assert_ok!(AMM::add_liquidity(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, XDOT),                     // Currency pool, in which liquidity will be added
            (3_000, 4_000),                  // Liquidity amounts to be added in pool
            (1, 1),                          // specifying its worst case ratio when pool already
        ));

        assert_noop!(
            AMM::add_liquidity(
                RawOrigin::Signed(ALICE).into(), // Origin
                (DOT, XDOT),                     // Currency pool, in which liquidity will be added
                (5000_000_000, 6000_000_000),    // Liquidity amounts to be added in pool
                (5, 5), // specifying its worst case ratio when pool already
            ),
            pallet_assets::Error::<Test>::BalanceLow
        );
    })
}

#[test]
fn add_liquidity_by_another_user_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, XDOT),                     // Currency pool, in which liquidity will be added
            (1_000, 2_000),                  // Liquidity amounts to be added in pool
            ALICE,                           // LPToken receiver
            SAMPLE_LP_TOKEN                  // Liquidity pool share representative token
        ));

        assert_ok!(AMM::add_liquidity(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, XDOT),                     // Currency pool, in which liquidity will be added
            (3_000, 4_000),                  // Liquidity amounts to be added in pool
            (5, 5),                          // specifying its worst case ratio when pool already
        ));

        assert_ok!(AMM::add_liquidity(
            RawOrigin::Signed(BOB).into(), // Origin
            (DOT, XDOT),                   // Currency pool, in which liquidity will be added
            (500, 1_000),                  // Liquidity amounts to be added in pool
            (5, 5),                        // specifying its worst case ratio when pool already
        ));

        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 7_000);
    })
}

#[test]
fn cannot_create_pool_twice() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, XDOT),                     // Currency pool, in which liquidity will be added
            (1_000, 2_000),                  // Liquidity amounts to be added in pool
            ALICE,                           // LPToken receiver
            SAMPLE_LP_TOKEN                  // Liquidity pool share representative token
        ));

        assert_noop!(
            AMM::create_pool(
                RawOrigin::Signed(ALICE).into(), // Origin
                (DOT, XDOT),                     // Currency pool, in which liquidity will be added
                (1_000, 2_000),                  // Liquidity amounts to be added in pool
                ALICE,                           // LPToken receiver
                SAMPLE_LP_TOKEN                  // Liquidity pool share representative token
            ),
            Error::<Test>::PoolAlreadyExists, // Pool already not exist
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
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, XDOT),                     // Currency pool, in which liquidity will be added
            (1_000, 9_000),                  // Liquidity amounts to be added in pool
            ALICE,                           // LPToken receiver
            SAMPLE_LP_TOKEN,                 // Liquidity pool share representative token
        );

        assert_ok!(AMM::remove_liquidity(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, XDOT),                     // Currency pool, in which liquidity will be removed
            3_000 - MINIMUM_LIQUIDITY        // liquidity to be removed from user's liquidity
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
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, XDOT),                     // Currency pool, in which liquidity will be added
            (1_000, 9_000),                  // Liquidity amounts to be added in pool
            ALICE,                           // LPToken receiver
            SAMPLE_LP_TOKEN,                 // Liquidity pool share representative token
        );

        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 9_000);
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().quote_amount, 1_000);

        assert_ok!(AMM::remove_liquidity(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, XDOT),                     // Currency pool, in which liquidity will be removed
            1_500                            // Liquidity to be removed from user's liquidity
        ));

        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 4_500);
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().quote_amount, 500);
    })
}

#[test]
fn remove_liquidity_user_more_liquidity_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, XDOT),                     // Currency pool, in which liquidity will be added
            (1_000, 2_500),                  // Liquidity amounts to be added in pool
            ALICE,                           // LPToken receiver
            SAMPLE_LP_TOKEN                  // Liquidity pool share representative token
        ));
        assert_ok!(AMM::add_liquidity(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, XDOT),                     // Currency pool, in which liquidity will be added
            (1_500, 3_000),                  // Liquidity amounts to be added in pool
            (5, 5),                          // specifying its worst case ratio when pool already
        ));

        assert_ok!(AMM::remove_liquidity(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, XDOT),                     // Currency pool, in which liquidity will be removed
            1_500                            // Liquidity to be removed from user's liquidity
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
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, XDOT),                     // Currency pool, in which liquidity will be added
            (1_000, 9_000),                  // Liquidity amounts to be added in pool
            ALICE,                           // LPToken receiver
            SAMPLE_LP_TOKEN,                 // Liquidity pool share representative token
        );

        assert_noop!(
            AMM::remove_liquidity(RawOrigin::Signed(ALICE).into(), (DOT, XDOT), 3_0000),
            Error::<Test>::InsufficientLiquidity
        );
    })
}

#[test]
fn swap_should_work_base_to_quote() {
    new_test_ext().execute_with(|| {
        let trader = EVE;

        // create pool and add liquidity
        assert_ok!(AMM::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, XDOT),                     // Currency pool, in which liquidity will be added
            (100_000_000, 100_000_000),      // Liquidity amounts to be added in pool
            CHARLIE,                         // LPToken receiver
            SAMPLE_LP_TOKEN,                 // Liquidity pool share representative token
        ));

        // XDOT is base_asset 1001
        // DOT is quote_asset 101

        // check that pool was funded correctly
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 100_000_000); // XDOT
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().quote_amount, 100_000_000); // DOT

        let path = vec![DOT, XDOT];

        let amount_in = 1_000;

        let amounts_out = AMM::get_amounts_out(amount_in, path).unwrap();

        // check balances before swap
        assert_eq!(Assets::balance(DOT, trader), 1_000_000_000);
        assert_eq!(Assets::balance(XDOT, trader), 1_000_000_000);

        assert_ok!(AMM::swap(&trader, (DOT, XDOT), amounts_out[0]));

        assert_eq!(
            Assets::balance(DOT, trader),
            1_000_000_000 - amount_in // 999_999_000
        );

        assert_eq!(
            Assets::balance(XDOT, trader),
            1_000_000_000 + amounts_out[1] // 1_000_000_996
        );
    })
}

#[test]
fn swap_should_work_different_ratio_base_to_quote() {
    new_test_ext().execute_with(|| {
        let trader = EVE;

        // create pool and add liquidity
        assert_ok!(AMM::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, XDOT),                     // Currency pool, in which liquidity will be added
            (100_000_000, 50_000_000),       // Liquidity amounts to be added in pool
            CHARLIE,                         // LPToken receiver
            SAMPLE_LP_TOKEN,                 // Liquidity pool share representative token
        ));

        // XDOT is base_asset 1001
        // DOT is quote_asset 101

        // check that pool was funded correctly
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 50_000_000); // XDOT
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().quote_amount, 100_000_000); // DOT

        let path = vec![DOT, XDOT];

        let amount_in = 1_000;

        let amounts_out = AMM::get_amounts_out(amount_in, path).unwrap();

        // check balances before swap
        assert_eq!(Assets::balance(DOT, trader), 1_000_000_000);
        assert_eq!(Assets::balance(XDOT, trader), 1_000_000_000);

        assert_ok!(AMM::swap(&trader, (DOT, XDOT), amounts_out[0],));

        assert_eq!(
            Assets::balance(DOT, trader),
            1_000_000_000 - amount_in // 999_999_000
        );

        assert_eq!(
            Assets::balance(XDOT, trader),
            1_000_000_000 + amounts_out[1] // 1_000_000_996
        );
    })
}

#[test]
fn swap_should_work_quote_to_base() {
    new_test_ext().execute_with(|| {
        let trader = EVE;

        // create pool and add liquidity
        assert_ok!(AMM::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (XDOT, DOT),                     // Currency pool, in which liquidity will be added
            (50_000_000, 100_000_000),       // Liquidity amounts to be added in pool
            CHARLIE,                         // LPToken receiver
            SAMPLE_LP_TOKEN,                 // Liquidity pool share representative token
        ));

        // XDOT is base_asset 1001
        // DOT is quote_asset 101

        // check that pool was funded correctly
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 50_000_000); // XDOT
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().quote_amount, 100_000_000); // DOT

        let path = vec![DOT, XDOT];

        let amount_in = 1_000;

        let amounts_out = AMM::get_amounts_out(amount_in, path).unwrap();

        // check balances before swap
        assert_eq!(Assets::balance(DOT, trader), 1_000_000_000);
        assert_eq!(Assets::balance(XDOT, trader), 1_000_000_000);

        assert_ok!(AMM::swap(&trader, (DOT, XDOT), amounts_out[0],));

        assert_eq!(
            Assets::balance(DOT, trader),
            1_000_000_000 - amount_in // 999_999_000
        );

        assert_eq!(
            Assets::balance(XDOT, trader),
            1_000_000_000 + amounts_out[1] // 1_000_000_996
        );
    })
}

#[test]
fn trade_should_work_base_to_quote_flipped_currencies_on_pool_creation() {
    new_test_ext().execute_with(|| {
        let trader = EVE;

        // create pool and add liquidity
        assert_ok!(AMM::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (XDOT, DOT),                     // Currency pool, in which liquidity will be added
            (100_000_000, 100_000_000),      // Liquidity amounts to be added in pool
            CHARLIE,                         // LPToken receiver
            SAMPLE_LP_TOKEN,                 // Liquidity pool share representative token
        ));

        // XDOT is base_asset 1001
        // DOT is quote_asset 101

        // check that pool was funded correctly
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 100_000_000); // XDOT
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().quote_amount, 100_000_000); // DOT

        // calculate amount out
        assert_ok!(AMM::swap(&trader, (DOT, XDOT), 1_000));

        assert_eq!(
            Assets::balance(XDOT, trader),
            1_000_000_000 + 996 // 1_000_000_996
        );

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
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, XDOT),                     // Currency pool, in which liquidity will be added
            (100_000_000, 100_000_000),      // Liquidity amounts to be added in pool
            CHARLIE,                         // LPToken receiver
            SAMPLE_LP_TOKEN,                 // Liquidity pool share representative token
        ));

        // XDOT is base_asset 1001
        // DOT is quote_asset 101

        // check that pool was funded correctly
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 100_000_000); // XDOT
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().quote_amount, 100_000_000); // DOT

        // calculate amount out
        // trade base for quote
        assert_ok!(AMM::swap(&trader, (DOT, XDOT), 1_000));

        assert_eq!(
            Assets::balance(XDOT, trader),
            1_000_000_000 + 996 // 1_000_000_996
        );

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
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, XDOT),                     // Currency pool, in which liquidity will be added
            (100_000, 100_000),              // Liquidity amounts to be added in pool
            CHARLIE,                         // LPToken receiver
            SAMPLE_LP_TOKEN,                 // Liquidity pool share representative token
        ));

        // create pool and add liquidity
        assert_ok!(AMM::add_liquidity(
            RawOrigin::Signed(CHARLIE).into(), // Origin
            (DOT, XDOT),                       // Currency pool, in which liquidity will be added
            (100_000, 100_000),                // Liquidity amounts to be added in pool
            (99_999, 99_999),                  // specifying its worst case ratio when pool already
        ));

        // check that pool was funded correctly
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 200_000); // XDOT
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().quote_amount, 200_000); // DOT

        // amount out is less than minimum_amount_out
        assert_noop!(
            AMM::swap(&trader, (DOT, XDOT), 332),
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
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, XDOT),                     // Currency pool, in which liquidity will be added
            (100_000, 50_000),               // Liquidity amounts to be added in pool
            CHARLIE,                         // LPToken receiver
            SAMPLE_LP_TOKEN                  // Liquidity pool share representative token
        ));

        // check that pool was funded correctly
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().quote_amount, 100_000); // DOT
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 50_000); // XDOT

        // calculate amount out
        assert_ok!(AMM::swap(&trader, (DOT, XDOT), 500));

        assert_eq!(
            Assets::balance(XDOT, trader),
            1_000_000_000 + 248 //
        );

        // pools values should be updated - we should have less DOT in the pool
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().quote_amount, 100_000 + 500);

        // pools values should be updated - we should have more XDOT
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().base_amount, 50_000 - 248);
    })
}

#[test]
fn trade_should_not_work_if_amount_in_is_zero() {
    new_test_ext().execute_with(|| {
        let trader = EVE;

        // create pool and add liquidity
        assert_ok!(AMM::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, XDOT),                     // Currency pool, in which liquidity will be added
            (1_000, 1_000),                  // Liquidity amounts to be added in pool
            ALICE,                           // LPToken receiver
            SAMPLE_LP_TOKEN                  // Liquidity pool share representative token
        ));

        // fail if amount_in is zero
        assert_noop!(
            AMM::swap(&trader, (DOT, XDOT), 0),
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
            AMM::swap(&trader, (DOT, XDOT), 10),
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
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, XDOT),                     // Currency pool, in which liquidity will be added
            (1_000, 2_000),                  // Liquidity amounts to be added in pool
            BOB,                             // LPToken receiver
            SAMPLE_LP_TOKEN,                 // Liquidity pool share representative token
        ));

        assert_ok!(AMM::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (KSM, DOT),                      // Currency pool, in which liquidity will be added
            (1_000, 1_000),                  // Liquidity amounts to be added in pool
            BOB,                             // LPToken receiver
            SAMPLE_LP_TOKEN_2,               // Liquidity pool share representative token
        ));

        let path = vec![XDOT, DOT, KSM];

        let amount_in = 1_000;

        let amounts_out = AMM::get_amounts_out(amount_in, path).unwrap();

        assert_eq!(amounts_out, [1000, 332, 249]);
    })
}

#[test]
fn long_route_amounts_in_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, XDOT),                     // Currency pool, in which liquidity will be added
            (10_000, 20_000),                // Liquidity amounts to be added in pool
            BOB,                             // LPToken receiver
            SAMPLE_LP_TOKEN,                 // Liquidity pool share representative token
        ));

        assert_ok!(AMM::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (KSM, DOT),                      // Currency pool, in which liquidity will be added
            (10_000, 10_000),                // Liquidity amounts to be added in pool
            BOB,                             // LPToken receiver
            SAMPLE_LP_TOKEN_2,               // Liquidity pool share representative token
        ));

        let path = vec![XDOT, DOT, KSM];

        let amount_out = 1_000;

        let amounts_in = AMM::get_amounts_in(amount_out, path).unwrap();

        assert_eq!(amounts_in, [2518, 1115, 1000]);
    })
}

#[test]
fn short_route_amounts_in_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, XDOT),                     // Currency pool, in which liquidity will be added
            (10_000_000, 10_000_000),        // Liquidity amounts to be added in pool
            BOB,                             // LPToken receiver
            SAMPLE_LP_TOKEN,                 // Liquidity pool share representative token
        ));

        let path = vec![DOT, XDOT];

        let amount_out = 1_000;

        let amounts_in = AMM::get_amounts_in(amount_out, path).unwrap();

        assert_eq!(amounts_in, [1004, 1000]);
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
fn amount_in_uneven_should_work() {
    new_test_ext().execute_with(|| {
        let amount_out = 1_000;
        let supply_in = 100_000_000;
        let supply_out = 1_344_312_043;

        let amount_in = AMM::get_amount_in(amount_out, supply_in, supply_out).unwrap();

        assert_eq!(amount_in, 75);
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

#[test]
fn update_oracle_should_work() {
    new_test_ext().execute_with(|| {
        let trader = EVE;

        assert_ok!(AMM::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (XDOT, DOT),                     // Currency pool, in which liquidity will be added
            (100_000, 100_000),              // Liquidity amounts to be added in pool
            BOB,                             // LPToken receiver
            SAMPLE_LP_TOKEN,                 // Liquidity pool share representative token
        ));

        assert_eq!(AMM::pools(XDOT, DOT).unwrap().block_timestamp_last, 0);
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().price_0_cumulative_last, 0);
        assert_eq!(AMM::pools(XDOT, DOT).unwrap().price_1_cumulative_last, 0);

        run_to_block(2);

        assert_ok!(AMM::swap(&trader, (DOT, XDOT), 1_000));

        assert_eq!(AMM::pools(XDOT, DOT).unwrap().block_timestamp_last, 2);
        assert_eq!(
            AMM::pools(XDOT, DOT).unwrap().price_0_cumulative_last,
            2_040136143738700978
        );
        assert_eq!(
            AMM::pools(XDOT, DOT).unwrap().price_1_cumulative_last,
            1_960653465346534653
        );

        run_to_block(4);

        assert_ok!(AMM::swap(&trader, (DOT, XDOT), 1_000));

        assert_eq!(AMM::pools(XDOT, DOT).unwrap().block_timestamp_last, 4);
        assert_eq!(
            AMM::pools(XDOT, DOT).unwrap().price_0_cumulative_last,
            4_120792162342213614
        );
        assert_eq!(
            AMM::pools(XDOT, DOT).unwrap().price_1_cumulative_last,
            3_883124053581828770
        );
    })
}

#[test]
fn oracle_big_block_no_overflow() {
    new_test_ext().execute_with(|| {
        let trader = FRANK;

        assert_ok!(AMM::create_pool(
            RawOrigin::Signed(ALICE).into(),                     // Origin
            (DOT, KSM), // Currency pool, in which liquidity will be added
            (9_999_650_729_873_433, 30_001_051_000_000_000_000), // Liquidity amounts to be added in pool
            FRANK,                                               // LPToken receiver
            SAMPLE_LP_TOKEN, // Liquidity pool share representative token
        ));

        assert_eq!(AMM::pools(DOT, KSM).unwrap().block_timestamp_last, 0);
        assert_eq!(AMM::pools(DOT, KSM).unwrap().price_0_cumulative_last, 0);
        assert_eq!(AMM::pools(DOT, KSM).unwrap().price_1_cumulative_last, 0);

        let mut big_block = 30_000;
        run_to_block(big_block);

        for _ in 0..5 {
            big_block += 1000;
            run_to_block(big_block);
            assert_ok!(AMM::swap(&trader, (DOT, KSM), 1000));
        }

        assert_eq!(
            AMM::pools(DOT, KSM).unwrap().block_timestamp_last,
            big_block
        );
        assert_eq!(
            AMM::pools(DOT, KSM).unwrap().price_0_cumulative_last,
            105007346_092879071079611686
        );
        assert_eq!(
            AMM::pools(DOT, KSM).unwrap().price_1_cumulative_last,
            11_665850491226458031
        );

        // increment a block
        big_block += 4;
        run_to_block(big_block);

        // this would swap used to overflow
        assert_ok!(AMM::swap(&trader, (DOT, KSM), 10_000_000_000));
    })
}

// ignore this test because it take >5 minutes to run
#[ignore]
#[test]
fn oracle_huge_block_should_work() {
    new_test_ext().execute_with(|| {
        let trader = FRANK;

        assert_ok!(AMM::create_pool(
            RawOrigin::Signed(ALICE).into(),                     // Origin
            (DOT, KSM), // Currency pool, in which liquidity will be added
            (9_999_650_729_873_433, 30_001_051_000_000_000_000), // Liquidity amounts to be added in pool
            FRANK,                                               // LPToken receiver
            SAMPLE_LP_TOKEN, // Liquidity pool share representative token
        ));

        assert_eq!(AMM::pools(DOT, KSM).unwrap().block_timestamp_last, 0);
        assert_eq!(AMM::pools(DOT, KSM).unwrap().price_0_cumulative_last, 0);
        assert_eq!(AMM::pools(DOT, KSM).unwrap().price_1_cumulative_last, 0);

        // let mut big_block = 100_000_000;
        let mut big_block = 10_000_000;

        // 100 Million blocks should take ~42.5 years to create at ~12 seconds a block

        // Calculations
        // avg_block_time = (1645493658865 - 1639798590500) / (424950 - 1)
        // avg_block_time == 13401.769071112063 == 13.4 seconds per block
        // total_time = (avg_block_time * 100_000_000) / (1000 * 60 * 60 * 24 * 365)
        // total_time == 42.496730945941344

        run_to_block(big_block);

        for _ in 0..5 {
            big_block += 100_000;
            run_to_block(big_block);
            assert_ok!(AMM::swap(&trader, (DOT, KSM), 1000));
        }

        assert_eq!(
            AMM::pools(DOT, KSM).unwrap().block_timestamp_last,
            big_block
        );
        assert_eq!(
            AMM::pools(DOT, KSM).unwrap().price_0_cumulative_last,
            // 301521093780_997938040922975491
            31502203827_864919649515113416
        );
        assert_eq!(
            AMM::pools(DOT, KSM).unwrap().price_1_cumulative_last,
            // 33497_656410519841854583
            3499_755147367804281224
        );
    })
}

#[test]
fn stable_swap_amount_out_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, XDOT),                     // Currency pool, in which liquidity will be added
            (1_000_000, 1_000_000),          // Liquidity amounts to be added in pool
            BOB,                             // LPToken receiver
            SAMPLE_LP_TOKEN,                 // Liquidity pool share representative token
        ));

        let y = AMM::get_alternative_var(10_000, (DOT, XDOT)).unwrap();
        // println!("{:?}", y);

        // let dy = 1_000_000 - y;

        let dy = 1_000_000u128.checked_sub(y).unwrap();

        // println!("{:?}", dy);

        // assert_eq!(dy, 9998.837230932666);
        assert_eq!(dy, 9998);
    })
}

#[test]
fn small_stable_swap_amount_out_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, XDOT),                     // Currency pool, in which liquidity will be added
            (1_000_000, 1_000_000),          // Liquidity amounts to be added in pool
            BOB,                             // LPToken receiver
            SAMPLE_LP_TOKEN,                 // Liquidity pool share representative token
        ));

        let amount_in = 10;
        let val = 1_000_000;
        let y = AMM::get_alternative_var(amount_in, (DOT, XDOT)).unwrap();

        let dy: u128;

        if val > y {
            dy = val - y;
        } else {
            dy = y - val;
        }

        let ex_ratio = dy.checked_div(amount_in).unwrap();

        // assert_eq!(ex_ratio, 0.9999998837010935);
        // assert_eq!(dy, 9.999998837010935);
        assert_eq!(ex_ratio, 0); // 33202
        assert_eq!(dy, 9); // 332027
    })
}

#[test]
fn large_stable_swap_amount_out_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, XDOT),                     // Currency pool, in which liquidity will be added
            (1_000_000, 1_000_000),          // Liquidity amounts to be added in pool
            BOB,                             // LPToken receiver
            SAMPLE_LP_TOKEN,                 // Liquidity pool share representative token
        ));

        let amount_in = 999_99;
        let y = AMM::get_alternative_var(amount_in, (DOT, XDOT)).unwrap();

        // let dy = 1_000_000 - y;
        // let ex_ratio = dy / amount_in;
        let dy = 1_000_000u128.checked_sub(y).unwrap();
        let ex_ratio = dy.checked_div(amount_in).unwrap();
        // assert_eq!(ex_ratio, 0.9289610549560947);
        // assert_eq!(dy, 928960.1259950397);
        assert_eq!(ex_ratio, 0);
        assert_eq!(dy, 928960);
    })
}

#[test]
fn unbalanced_stable_swap_amount_out_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, XDOT),                     // Currency pool, in which liquidity will be added
            (10_000, 1_000_000),             // Liquidity amounts to be added in pool
            BOB,                             // LPToken receiver
            SAMPLE_LP_TOKEN,                 // Liquidity pool share representative token
        ));

        let amount_in = 500;
        let y = AMM::get_alternative_var(amount_in, (DOT, XDOT)).unwrap();

        // let dy = 1_000_000 - y;
        // let ex_ratio = dy / amount_in;
        let dy = 1_000_000u128.checked_sub(y).unwrap();
        let ex_ratio = dy.checked_div(amount_in).unwrap();
        // assert_eq!(ex_ratio, 10.333014499817741);
        // assert_eq!(dy, 5166.507249908871);
        assert_eq!(ex_ratio, 10);
        assert_eq!(dy, 5166);
    })
}

#[test]
fn unbalanced_small_stable_swap_amount_out_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, XDOT),                     // Currency pool, in which liquidity will be added
            (10_000, 1_000_000),             // Liquidity amounts to be added in pool
            BOB,                             // LPToken receiver
            SAMPLE_LP_TOKEN,                 // Liquidity pool share representative token
        ));

        let amount_in = 162;
        let y = AMM::get_alternative_var(amount_in, (DOT, XDOT)).unwrap();

        // let dy = 1_000_000 - y;
        // let ex_ratio = dy / amount_in;

        let dy = 1_000_000u128.checked_sub(y).unwrap();
        let ex_ratio = dy.checked_div(amount_in).unwrap();

        // assert_eq!(ex_ratio, 10.616718051234335);
        // assert_eq!(dy, 1719.9083242999623);
        assert_eq!(ex_ratio, 10);
        assert_eq!(dy, 1719);
    })
}

#[test]
fn close_unbalanced_small_stable_swap_amount_out_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(AMM::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, XDOT),                     // Currency pool, in which liquidity will be added
            (900_000, 1_000_000),            // Liquidity amounts to be added in pool
            BOB,                             // LPToken receiver
            SAMPLE_LP_TOKEN,                 // Liquidity pool share representative token
        ));

        let amount_in = 10_000;
        let y = AMM::get_alternative_var(amount_in, (DOT, XDOT)).unwrap();
        /*
        u128 y = 1282261
        amount_in = 10000

        float
        y  = 989988.61254373263
        dy =  10011.387456267374
        ex =      1.0011387456267373

        */
        // let dy = 1_000_000 - y;
        // let ex_ratio = dy / amount_in;
        let dy = 1_000_000u128.checked_sub(y).unwrap();
        let ex_ratio = dy.checked_div(amount_in).unwrap();

        // assert_eq!(ex_ratio, 1.0011070580483181);
        // assert_eq!(dy, 10011.070580483181);
        assert_eq!(ex_ratio, 1);
        assert_eq!(dy, 10011);
    })
}
