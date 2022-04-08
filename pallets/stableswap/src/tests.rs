use super::*;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok};
use frame_system::RawOrigin;
use primitives::StableSwap as _;

const MINIMUM_LIQUIDITY: u128 = 1_000;

#[test]
fn stable_swap_amount_out_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(DefaultStableSwap::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, SDOT),                     // Currency pool, in which liquidity will be added
            (1_000_000, 1_000_000),          // Liquidity amounts to be added in pool
            BOB,                             // LPToken receiver
            SAMPLE_LP_TOKEN,                 // Liquidity pool share representative token
        ));

        let y = DefaultStableSwap::do_get_alternative_var(10_000, (DOT, SDOT)).unwrap();

        let dy = 1_000_000u128.checked_sub(y).unwrap();

        assert_eq!(dy, 9999);
    })
}

#[test]
fn small_stable_swap_amount_out_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(DefaultStableSwap::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, SDOT),                     // Currency pool, in which liquidity will be added
            (1_000_000, 1_000_000),          // Liquidity amounts to be added in pool
            BOB,                             // LPToken receiver
            SAMPLE_LP_TOKEN,                 // Liquidity pool share representative token
        ));

        let amount_in = 10;
        let val = 1_000_000;
        let y = DefaultStableSwap::do_get_alternative_var(amount_in, (DOT, SDOT)).unwrap();

        let dy: u128;

        if val > y {
            dy = val - y;
        } else {
            dy = y - val;
        }

        let ex_ratio = dy.checked_div(amount_in).unwrap();

        assert_eq!(ex_ratio, 1); // 33202
        assert_eq!(dy, 10); // 332027
    })
}

#[test]
fn large_stable_swap_amount_out_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(DefaultStableSwap::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, SDOT),                     // Currency pool, in which liquidity will be added
            (1_000_000, 1_000_000),          // Liquidity amounts to be added in pool
            BOB,                             // LPToken receiver
            SAMPLE_LP_TOKEN,                 // Liquidity pool share representative token
        ));

        let amount_in = 999_999;
        let y = DefaultStableSwap::do_get_alternative_var(amount_in, (DOT, SDOT)).unwrap();

        let dy = 1_000_000u128.checked_sub(y).unwrap();
        let ex_ratio = dy.checked_div(amount_in).unwrap();

        assert_eq!(ex_ratio, 0);
        assert_eq!(dy, 928960);
    })
}

#[test]
fn unbalanced_stable_swap_amount_out_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(DefaultStableSwap::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, SDOT),                     // Currency pool, in which liquidity will be added
            (10_000, 1_000_000),             // Liquidity amounts to be added in pool
            BOB,                             // LPToken receiver
            SAMPLE_LP_TOKEN,                 // Liquidity pool share representative token
        ));

        let amount_in = 500;
        let y = DefaultStableSwap::do_get_alternative_var(amount_in, (DOT, SDOT)).unwrap();

        let dy = 1_000_000u128.checked_sub(y).unwrap();
        let ex_ratio = dy.checked_div(amount_in).unwrap();

        assert_eq!(ex_ratio, 10);
        assert_eq!(dy, 5167);
    })
}

#[test]
fn unbalanced_small_stable_swap_amount_out_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(DefaultStableSwap::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, SDOT),                     // Currency pool, in which liquidity will be added
            (10_000, 1_000_000),             // Liquidity amounts to be added in pool
            BOB,                             // LPToken receiver
            SAMPLE_LP_TOKEN,                 // Liquidity pool share representative token
        ));

        let amount_in = 162;
        let y = DefaultStableSwap::do_get_alternative_var(amount_in, (DOT, SDOT)).unwrap();

        let dy = 1_000_000u128.checked_sub(y).unwrap();
        let ex_ratio = dy.checked_div(amount_in).unwrap();

        assert_eq!(ex_ratio, 10);
        assert_eq!(dy, 1720);
    })
}

#[test]
fn close_unbalanced_small_stable_swap_amount_out_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(DefaultStableSwap::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, SDOT),                     // Currency pool, in which liquidity will be added
            (900_000, 1_000_000),            // Liquidity amounts to be added in pool
            BOB,                             // LPToken receiver
            SAMPLE_LP_TOKEN,                 // Liquidity pool share representative token
        ));

        let amount_in = 10_000;
        let y = DefaultStableSwap::do_get_alternative_var(amount_in, (DOT, SDOT)).unwrap();

        let dy = 1_000_000u128.checked_sub(y).unwrap();
        let ex_ratio = dy.checked_div(amount_in).unwrap();

        assert_eq!(ex_ratio, 1);
        assert_eq!(dy, 10012);
    })
}

#[test]
fn add_liquidity_with_variant_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(DefaultStableSwap::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, SDOT),                     // Currency pool, in which liquidity will be added
            (1_000, 2_000),                  // Liquidity amounts to be added in pool
            ALICE,                           // LPToken receiver
            SAMPLE_LP_TOKEN,                 // Liquidity pool share representative token
        ));
        assert_eq!(Assets::total_issuance(SAMPLE_LP_TOKEN), 1_414);
        assert_ok!(DefaultStableSwap::add_liquidity(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, SDOT),                     // Currency pool, in which liquidity will be added
            (1_000_000, 2_000_000),          // Liquidity amounts to be added in pool
            (5, 5),                          // specifying its worst case ratio when pool already
        ));
        assert_eq!(Assets::total_issuance(SAMPLE_LP_TOKEN), 1414390653);
        // This fails
        assert_eq!(
            DefaultStableSwap::pools(SDOT, DOT).unwrap().base_amount,
            2002000
        );
    })
}

#[test]
fn add_liquidity_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(DefaultStableSwap::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, SDOT),                     // Currency pool, in which liquidity will be added
            (1_000, 2_000),                  // Liquidity amounts to be added in pool
            ALICE,                           // LPToken receiver
            SAMPLE_LP_TOKEN,                 // Liquidity pool share representative token
        ));
        assert_ok!(DefaultStableSwap::add_liquidity(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, SDOT),                     // Currency pool, in which liquidity will be added
            (1_000, 2_000),                  // Liquidity amounts to be added in pool
            (5, 5),                          // specifying its worst case ratio when pool already
        ));

        assert_eq!(
            DefaultStableSwap::pools(SDOT, DOT).unwrap().base_amount,
            4_000
        );
    })
}

#[test]
fn add_more_liquidity_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(DefaultStableSwap::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, SDOT),                     // Currency pool, in which liquidity will be added
            (1_000, 2_000),                  // Liquidity amounts to be added in pool
            ALICE,                           // LPToken receiver
            SAMPLE_LP_TOKEN                  // Liquidity pool share representative token
        ));

        assert_ok!(DefaultStableSwap::add_liquidity(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, SDOT),                     // Currency pool, in which liquidity will be added
            (3_000, 4_000),                  // Liquidity amounts to be added in pool
            (5, 5), // specifying its worst case ratio when pool already exists
        ));

        assert_eq!(
            DefaultStableSwap::pools(SDOT, DOT).unwrap().base_amount,
            6_000
        );
        assert_eq!(
            DefaultStableSwap::pools(SDOT, DOT).unwrap().quote_amount,
            3_000
        );
    })
}

#[test]
fn add_more_liquidity_should_not_work_if_minimum_base_amount_is_higher() {
    new_test_ext().execute_with(|| {
        assert_ok!(DefaultStableSwap::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, SDOT),                     // Currency pool, in which liquidity will be added
            (1_000, 2_000),                  // Liquidity amounts to be added in pool
            ALICE,                           // LPToken receiver
            SAMPLE_LP_TOKEN                  // Liquidity pool share representative token
        ));

        assert_noop!(
            DefaultStableSwap::add_liquidity(
                RawOrigin::Signed(ALICE).into(), // Origin
                (DOT, SDOT),                     // Currency pool, in which liquidity will be added
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
        assert_ok!(DefaultStableSwap::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, SDOT),                     // Currency pool, in which liquidity will be added
            (1_000, 2_000),                  // Liquidity amounts to be added in pool
            ALICE,                           // LPToken receiver
            SAMPLE_LP_TOKEN                  // Liquidity pool share representative token
        ));

        assert_ok!(DefaultStableSwap::add_liquidity(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, SDOT),                     // Currency pool, in which liquidity will be added
            (3_000, 4_000),                  // Liquidity amounts to be added in pool
            (1, 1),                          // specifying its worst case ratio when pool already
        ));

        assert_noop!(
            DefaultStableSwap::add_liquidity(
                RawOrigin::Signed(ALICE).into(), // Origin
                (DOT, SDOT),                     // Currency pool, in which liquidity will be added
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
        assert_ok!(DefaultStableSwap::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, SDOT),                     // Currency pool, in which liquidity will be added
            (1_000, 2_000),                  // Liquidity amounts to be added in pool
            ALICE,                           // LPToken receiver
            SAMPLE_LP_TOKEN                  // Liquidity pool share representative token
        ));

        assert_ok!(DefaultStableSwap::add_liquidity(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, SDOT),                     // Currency pool, in which liquidity will be added
            (3_000, 4_000),                  // Liquidity amounts to be added in pool
            (5, 5),                          // specifying its worst case ratio when pool already
        ));

        assert_ok!(DefaultStableSwap::add_liquidity(
            RawOrigin::Signed(BOB).into(), // Origin
            (DOT, SDOT),                   // Currency pool, in which liquidity will be added
            (500, 1_000),                  // Liquidity amounts to be added in pool
            (5, 5),                        // specifying its worst case ratio when pool already
        ));

        assert_eq!(
            DefaultStableSwap::pools(SDOT, DOT).unwrap().base_amount,
            7_000
        );
    })
}

#[test]
fn remove_liquidity_whole_share_should_work() {
    new_test_ext().execute_with(|| {
        // A pool with a single LP provider
        // who deposit tokens and withdraws their whole share
        // (most simple case)

        let _ = DefaultStableSwap::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, SDOT),                     // Currency pool, in which liquidity will be added
            (1_000, 9_000),                  // Liquidity amounts to be added in pool
            ALICE,                           // LPToken receiver
            SAMPLE_LP_TOKEN,                 // Liquidity pool share representative token
        );

        assert_ok!(DefaultStableSwap::remove_liquidity(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, SDOT),                     // Currency pool, in which liquidity will be removed
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

        let _ = DefaultStableSwap::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, SDOT),                     // Currency pool, in which liquidity will be added
            (1_000, 9_000),                  // Liquidity amounts to be added in pool
            ALICE,                           // LPToken receiver
            SAMPLE_LP_TOKEN,                 // Liquidity pool share representative token
        );

        assert_eq!(
            DefaultStableSwap::pools(SDOT, DOT).unwrap().base_amount,
            9_000
        );
        assert_eq!(
            DefaultStableSwap::pools(SDOT, DOT).unwrap().quote_amount,
            1_000
        );

        assert_ok!(DefaultStableSwap::remove_liquidity(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, SDOT),                     // Currency pool, in which liquidity will be removed
            1_500                            // Liquidity to be removed from user's liquidity
        ));

        assert_eq!(
            DefaultStableSwap::pools(SDOT, DOT).unwrap().base_amount,
            4_500
        );
        assert_eq!(
            DefaultStableSwap::pools(SDOT, DOT).unwrap().quote_amount,
            500
        );
    })
}

#[test]
fn remove_liquidity_user_more_liquidity_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(DefaultStableSwap::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, SDOT),                     // Currency pool, in which liquidity will be added
            (1_000, 2_500),                  // Liquidity amounts to be added in pool
            ALICE,                           // LPToken receiver
            SAMPLE_LP_TOKEN                  // Liquidity pool share representative token
        ));
        assert_ok!(DefaultStableSwap::add_liquidity(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, SDOT),                     // Currency pool, in which liquidity will be added
            (1_500, 3_000),                  // Liquidity amounts to be added in pool
            (5, 5),                          // specifying its worst case ratio when pool already
        ));

        assert_ok!(DefaultStableSwap::remove_liquidity(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, SDOT),                     // Currency pool, in which liquidity will be removed
            1_500                            // Liquidity to be removed from user's liquidity
        ));
    })
}

#[test]
fn remove_liquidity_when_pool_does_not_exist_should_not_work() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            DefaultStableSwap::remove_liquidity(RawOrigin::Signed(ALICE).into(), (DOT, SDOT), 15),
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

        let _ = DefaultStableSwap::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, SDOT),                     // Currency pool, in which liquidity will be added
            (1_000, 9_000),                  // Liquidity amounts to be added in pool
            ALICE,                           // LPToken receiver
            SAMPLE_LP_TOKEN,                 // Liquidity pool share representative token
        );

        assert_noop!(
            DefaultStableSwap::remove_liquidity(
                RawOrigin::Signed(ALICE).into(),
                (DOT, SDOT),
                3_0000
            ),
            Error::<Test>::InsufficientLiquidity
        );
    })
}

#[test]
fn swap_should_work_base_to_quote() {
    new_test_ext().execute_with(|| {
        let trader = EVE;

        // create pool and add liquidity
        assert_ok!(DefaultStableSwap::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, SDOT),                     // Currency pool, in which liquidity will be added
            (100_000_000, 100_000_000),      // Liquidity amounts to be added in pool
            CHARLIE,                         // LPToken receiver
            SAMPLE_LP_TOKEN,                 // Liquidity pool share representative token
        ));

        // SDOT is base_asset 1001
        // DOT is quote_asset 101

        // check that pool was funded correctly
        assert_eq!(
            DefaultStableSwap::pools(SDOT, DOT).unwrap().base_amount,
            100_000_000
        ); // SDOT
        assert_eq!(
            DefaultStableSwap::pools(SDOT, DOT).unwrap().quote_amount,
            100_000_000
        ); // DOT

        let path = vec![DOT, SDOT];

        let amount_in = 1_000;

        let amounts_out = DefaultStableSwap::get_amounts_out(amount_in, path).unwrap();

        // check balances before swap
        assert_eq!(Assets::balance(DOT, trader), 1_000_000_000);
        assert_eq!(Assets::balance(SDOT, trader), 1_000_000_000);

        assert_ok!(DefaultStableSwap::swap(
            &trader,
            (DOT, SDOT),
            amounts_out[0]
        ));

        assert_eq!(
            Assets::balance(DOT, trader),
            1_000_000_000 - amount_in // 999_999_000
        );

        assert_eq!(
            Assets::balance(SDOT, trader),
            1_000_000_000 + amounts_out[1] // 1_000_000_996
        );
    })
}

#[test]
fn swap_should_work_different_ratio_base_to_quote() {
    new_test_ext().execute_with(|| {
        let trader = EVE;

        // create pool and add liquidity
        assert_ok!(DefaultStableSwap::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, SDOT),                     // Currency pool, in which liquidity will be added
            (100_000_000, 50_000_000),       // Liquidity amounts to be added in pool
            CHARLIE,                         // LPToken receiver
            SAMPLE_LP_TOKEN,                 // Liquidity pool share representative token
        ));

        // SDOT is base_asset 1001
        // DOT is quote_asset 101

        // check that pool was funded correctly
        assert_eq!(
            DefaultStableSwap::pools(SDOT, DOT).unwrap().base_amount,
            50_000_000
        ); // SDOT
        assert_eq!(
            DefaultStableSwap::pools(SDOT, DOT).unwrap().quote_amount,
            100_000_000
        ); // DOT

        let path = vec![DOT, SDOT];

        let amount_in = 1_000;

        let amounts_out = DefaultStableSwap::get_amounts_out(amount_in, path).unwrap();

        // check balances before swap
        assert_eq!(Assets::balance(DOT, trader), 1_000_000_000);
        assert_eq!(Assets::balance(SDOT, trader), 1_000_000_000);

        assert_ok!(DefaultStableSwap::swap(
            &trader,
            (DOT, SDOT),
            amounts_out[0],
        ));

        assert_eq!(
            Assets::balance(DOT, trader),
            1_000_000_000 - amount_in // 999_999_000
        );

        assert_eq!(
            Assets::balance(SDOT, trader),
            1_000_000_000 + amounts_out[1] // 1_000_000_996
        );
    })
}

#[test]
fn swap_should_work_quote_to_base() {
    new_test_ext().execute_with(|| {
        let trader = EVE;

        // create pool and add liquidity
        assert_ok!(DefaultStableSwap::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (SDOT, DOT),                     // Currency pool, in which liquidity will be added
            (50_000_000, 100_000_000),       // Liquidity amounts to be added in pool
            CHARLIE,                         // LPToken receiver
            SAMPLE_LP_TOKEN,                 // Liquidity pool share representative token
        ));

        // SDOT is base_asset 1001
        // DOT is quote_asset 101

        // check that pool was funded correctly
        assert_eq!(
            DefaultStableSwap::pools(SDOT, DOT).unwrap().base_amount,
            50_000_000
        ); // SDOT
        assert_eq!(
            DefaultStableSwap::pools(SDOT, DOT).unwrap().quote_amount,
            100_000_000
        ); // DOT

        let path = vec![DOT, SDOT];

        let amount_in = 1_000;

        let amounts_out = DefaultStableSwap::get_amounts_out(amount_in, path).unwrap();

        // check balances before swap
        assert_eq!(Assets::balance(DOT, trader), 1_000_000_000);
        assert_eq!(Assets::balance(SDOT, trader), 1_000_000_000);

        assert_ok!(DefaultStableSwap::swap(
            &trader,
            (DOT, SDOT),
            amounts_out[0],
        ));

        assert_eq!(
            Assets::balance(DOT, trader),
            1_000_000_000 - amount_in // 999_999_000
        );

        assert_eq!(
            Assets::balance(SDOT, trader),
            1_000_000_000 + amounts_out[1] // 1_000_000_996
        );
    })
}
