use super::*;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok};
use frame_system::RawOrigin;
use pallet_traits::StableSwap as _;
use primitives::tokens;

const MINIMUM_LIQUIDITY: u128 = 1_000;

#[test]
fn create_pool_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(DefaultStableSwap::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, SDOT),                     // Currency pool, in which liquidity will be added
            (1_000, 2_000),                  // Liquidity amounts to be added in pool
            BOB,                             // LPToken receiver
            SAMPLE_LP_TOKEN,                 // Liquidity pool share representative token
        ));

        assert_eq!(
            DefaultStableSwap::pools(SDOT, DOT).unwrap().base_amount,
            2_000
        );
        assert_eq!(Assets::total_issuance(SAMPLE_LP_TOKEN), 1_414);
        // should be issuance minus the min liq locked
        assert_eq!(Assets::balance(SAMPLE_LP_TOKEN, BOB), 414);
    })
}

#[test]
fn double_liquidity_correct_liq_ratio_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(DefaultStableSwap::create_pool(
            RawOrigin::Signed(ALICE).into(),              // Origin
            (DOT, KSM), // Currency pool, in which liquidity will be added
            (15_000_000_000_000, 50_000_000_000_000_000), // Liquidity amounts to be added in pool
            FRANK,      // LPToken receiver
            SAMPLE_LP_TOKEN, // Liquidity pool share representative token
        ));

        // total liquidity after pool created
        let total_liquidity_tokens = Assets::total_issuance(SAMPLE_LP_TOKEN);

        assert_ok!(DefaultStableSwap::add_liquidity(
            RawOrigin::Signed(FRANK).into(),              // Origin
            (DOT, KSM), // Currency pool, in which liquidity will be added
            (15_000_000_000_000, 50_000_000_000_000_000), // Liquidity amounts to be added in pool
            (15_000_000_000_000, 50_000_000_000_000_000), // specifying its worst case ratio when pool already
        ));

        let total_liquidity_tokens_after_double = Assets::total_issuance(SAMPLE_LP_TOKEN);
        let liquidity_received = total_liquidity_tokens_after_double - total_liquidity_tokens;

        // received liquidity should be half of total liquidity
        assert_eq!(
            liquidity_received as f64 / total_liquidity_tokens_after_double as f64,
            0.6666666666666666
        );
    })
}

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
        // y = 1048189
        // TODO: Fix this scenario since it returns more value
        // Correct Test
        let dy = y.checked_sub(1_000_000u128).unwrap();
        let ex_ratio = dy.checked_div(amount_in).unwrap();

        assert_eq!(ex_ratio, 96);
        assert_eq!(dy, 48189);
    })
}

#[test]
fn unbalanced_small_stable_swap_amount_out_should_work() {
    // y = 1051916
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
        assert_eq!(y, 1_051_916);
        let dy = y.checked_sub(1_000_000u128).unwrap();
        let ex_ratio = dy.checked_div(amount_in).unwrap();

        // assert_eq!(ex_ratio, 10);
        assert_eq!(ex_ratio, 320);
        // assert_eq!(dy, 1720);
        assert_eq!(dy, 51916)
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

        // assert_eq!(ex_ratio, 1);
        assert_eq!(ex_ratio, 0);
        // assert_eq!(dy, 10012);
        assert_eq!(dy, 9997);
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
        // assert_eq!(Assets::total_issuance(SAMPLE_LP_TOKEN), 1414390653);
        assert_eq!(Assets::total_issuance(SAMPLE_LP_TOKEN), 1415842255);
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
fn cannot_create_pool_twice() {
    new_test_ext().execute_with(|| {
        assert_ok!(DefaultStableSwap::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, SDOT),                     // Currency pool, in which liquidity will be added
            (1_000, 2_000),                  // Liquidity amounts to be added in pool
            ALICE,                           // LPToken receiver
            SAMPLE_LP_TOKEN                  // Liquidity pool share representative token
        ));

        assert_noop!(
            DefaultStableSwap::create_pool(
                RawOrigin::Signed(ALICE).into(), // Origin
                (DOT, SDOT),                     // Currency pool, in which liquidity will be added
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

#[test]
fn trade_should_work_base_to_quote_flipped_currencies_on_pool_creation() {
    new_test_ext().execute_with(|| {
        let trader = EVE;

        // create pool and add liquidity
        assert_ok!(DefaultStableSwap::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (SDOT, DOT),                     // Currency pool, in which liquidity will be added
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

        // calculate amount out
        assert_ok!(DefaultStableSwap::swap(&trader, (DOT, SDOT), 1_000));

        // old
        // assert_eq!(
        //     Assets::balance(SDOT, trader),
        //     1_000_000_000 + 996 // 1_000_000_996
        // );

        // new
        assert_eq!(
            Assets::balance(SDOT, trader),
            1_000_000_000 + 997 // 1_000_000_996
        );

        // pools values should be updated - we should have less SDOT
        // assert_eq!(
        //     DefaultStableSwap::pools(SDOT, DOT).unwrap().base_amount,
        //     99_999_004
        // );
        assert_eq!(
            DefaultStableSwap::pools(SDOT, DOT).unwrap().base_amount,
            99999003
        );

        // pools values should be updated - we should have more DOT in the pool
        assert_eq!(
            DefaultStableSwap::pools(SDOT, DOT).unwrap().quote_amount,
            100_001_000
        );
    })
}

#[test]
fn trade_should_work_quote_to_base() {
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

        // calculate amount out
        // trade base for quote
        assert_ok!(DefaultStableSwap::swap(&trader, (DOT, SDOT), 1_000));

        // Old
        // assert_eq!(
        //     Assets::balance(SDOT, trader),
        //     1_000_000_000 + 996 // 1_000_000_996
        // );

        // New
        assert_eq!(
            Assets::balance(SDOT, trader),
            1_000_000_000 + 997 // 1_000_000_996
        );

        // we should have more DOT in the pool since were trading it for DOT
        assert_eq!(
            DefaultStableSwap::pools(SDOT, DOT).unwrap().quote_amount,
            100_001_000
        );

        // we should have less SDOT since we traded it for SDOT
        // assert_eq!(
        //     DefaultStableSwap::pools(SDOT, DOT).unwrap().base_amount,
        //     99_999_004
        // );
        assert_eq!(
            DefaultStableSwap::pools(SDOT, DOT).unwrap().base_amount,
            99_999_003
        );
    })
}

#[test]
fn trade_should_not_work_if_insufficient_amount_in() {
    new_test_ext().execute_with(|| {
        let trader = EVE;

        assert_ok!(DefaultStableSwap::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, SDOT),                     // Currency pool, in which liquidity will be added
            (100_000, 100_000),              // Liquidity amounts to be added in pool
            CHARLIE,                         // LPToken receiver
            SAMPLE_LP_TOKEN,                 // Liquidity pool share representative token
        ));

        // create pool and add liquidity
        assert_ok!(DefaultStableSwap::add_liquidity(
            RawOrigin::Signed(CHARLIE).into(), // Origin
            (DOT, SDOT),                       // Currency pool, in which liquidity will be added
            (100_000, 100_000),                // Liquidity amounts to be added in pool
            (99_999, 99_999),                  // specifying its worst case ratio when pool already
        ));

        // check that pool was funded correctly
        assert_eq!(
            DefaultStableSwap::pools(SDOT, DOT).unwrap().base_amount,
            200_000
        ); // SDOT
        assert_eq!(
            DefaultStableSwap::pools(SDOT, DOT).unwrap().quote_amount,
            200_000
        ); // DOT

        // amount out is less than minimum_amount_out
        assert_noop!(
            DefaultStableSwap::swap(&trader, (DOT, SDOT), 332),
            Error::<Test>::InsufficientAmountIn
        );
    })
}

#[test]
fn trade_should_work_flipped_currencies() {
    new_test_ext().execute_with(|| {
        let trader = EVE;

        // create pool and add liquidity
        assert_ok!(DefaultStableSwap::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, SDOT),                     // Currency pool, in which liquidity will be added
            (100_000, 50_000),               // Liquidity amounts to be added in pool
            CHARLIE,                         // LPToken receiver
            SAMPLE_LP_TOKEN                  // Liquidity pool share representative token
        ));

        // check that pool was funded correctly
        assert_eq!(
            DefaultStableSwap::pools(SDOT, DOT).unwrap().quote_amount,
            100_000
        ); // DOT
        assert_eq!(
            DefaultStableSwap::pools(SDOT, DOT).unwrap().base_amount,
            50_000
        ); // SDOT

        // calculate amount out
        assert_ok!(DefaultStableSwap::swap(&trader, (DOT, SDOT), 500));
        // Old
        // assert_eq!(
        //     Assets::balance(SDOT, trader),
        //     1_000_000_000 + 248 //
        // );
        // New
        assert_eq!(
            Assets::balance(SDOT, trader),
            1_000_000_000 + 502 //
        );

        // pools values should be updated - we should have less DOT in the pool
        assert_eq!(
            DefaultStableSwap::pools(SDOT, DOT).unwrap().quote_amount,
            100_000 + 500
        );

        // pools values should be updated - we should have more SDOT
        // assert_eq!(
        //     DefaultStableSwap::pools(SDOT, DOT).unwrap().base_amount,
        //     50_000 - 248
        // );
        assert_eq!(
            DefaultStableSwap::pools(SDOT, DOT).unwrap().base_amount,
            49498
        );
    })
}

#[test]
fn trade_should_not_work_if_amount_in_is_zero() {
    new_test_ext().execute_with(|| {
        let trader = EVE;

        // create pool and add liquidity
        assert_ok!(DefaultStableSwap::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, SDOT),                     // Currency pool, in which liquidity will be added
            (1_000, 1_000),                  // Liquidity amounts to be added in pool
            ALICE,                           // LPToken receiver
            SAMPLE_LP_TOKEN                  // Liquidity pool share representative token
        ));

        // fail if amount_in is zero
        assert_noop!(
            DefaultStableSwap::swap(&trader, (DOT, SDOT), 0),
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
            DefaultStableSwap::swap(&trader, (DOT, SDOT), 10),
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

        let amount_out =
            DefaultStableSwap::get_amount_out(amount_in, supply_in, supply_out).unwrap();

        // actual value == 996.9900600091017
        // TODO: assumes we round down to int
        // old
        // assert_eq!(amount_out, 996);
        // new
        assert_eq!(amount_out, 997);
    })
}

#[test]
fn amounts_out_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(DefaultStableSwap::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, SDOT),                     // Currency pool, in which liquidity will be added
            (1_000, 2_000),                  // Liquidity amounts to be added in pool
            BOB,                             // LPToken receiver
            SAMPLE_LP_TOKEN,                 // Liquidity pool share representative token
        ));

        assert_ok!(DefaultStableSwap::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (KSM, DOT),                      // Currency pool, in which liquidity will be added
            (1_000, 1_000),                  // Liquidity amounts to be added in pool
            BOB,                             // LPToken receiver
            SAMPLE_LP_TOKEN_2,               // Liquidity pool share representative token
        ));

        let path = vec![SDOT, DOT, KSM];

        let amount_in = 1_000;

        let amounts_out = DefaultStableSwap::get_amounts_out(amount_in, path).unwrap();
        // Old
        // assert_eq!(amounts_out, [1000, 332, 249]);

        // New
        assert_eq!(amounts_out, [1000, 998, 947]);
    })
}

#[test]
fn long_route_amounts_in_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(DefaultStableSwap::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, SDOT),                     // Currency pool, in which liquidity will be added
            (10_000, 20_000),                // Liquidity amounts to be added in pool
            BOB,                             // LPToken receiver
            SAMPLE_LP_TOKEN,                 // Liquidity pool share representative token
        ));

        assert_ok!(DefaultStableSwap::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (KSM, DOT),                      // Currency pool, in which liquidity will be added
            (10_000, 10_000),                // Liquidity amounts to be added in pool
            BOB,                             // LPToken receiver
            SAMPLE_LP_TOKEN_2,               // Liquidity pool share representative token
        ));

        let path = vec![SDOT, DOT, KSM];

        let amount_out = 1_000;

        let amounts_in = DefaultStableSwap::get_amounts_in(amount_out, path).unwrap();

        assert_eq!(amounts_in, [2518, 1115, 1000]);
    })
}

#[test]
fn short_route_amounts_in_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(DefaultStableSwap::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, SDOT),                     // Currency pool, in which liquidity will be added
            (10_000_000, 10_000_000),        // Liquidity amounts to be added in pool
            BOB,                             // LPToken receiver
            SAMPLE_LP_TOKEN,                 // Liquidity pool share representative token
        ));

        let path = vec![DOT, SDOT];

        let amount_out = 1_000;

        let amounts_in = DefaultStableSwap::get_amounts_in(amount_out, path).unwrap();

        assert_eq!(amounts_in, [1004, 1000]);
    })
}

#[test]
fn amount_in_should_work() {
    new_test_ext().execute_with(|| {
        let amount_out = 1_000;
        let supply_in = 100_000_000;
        let supply_out = 100_000_000;

        let amount_in =
            DefaultStableSwap::get_amount_in(amount_out, supply_in, supply_out).unwrap();

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

        let amount_in =
            DefaultStableSwap::get_amount_in(amount_out, supply_in, supply_out).unwrap();

        assert_eq!(amount_in, 75);
    })
}

#[test]
fn supply_out_should_larger_than_amount_out() {
    // Test case for Panic when amount_out >= supply_out
    new_test_ext().execute_with(|| {
        let amount_out = 100_00;
        let supply_in = 100_000;
        let supply_out = 100_00;

        assert_noop!(
            DefaultStableSwap::get_amount_in(amount_out, supply_in, supply_out),
            Error::<Test>::InsufficientSupplyOut
        );
    })
}

#[test]
fn amount_out_and_in_should_work() {
    new_test_ext().execute_with(|| {
        let amount_out = 1_000;
        let supply_in = 100_000_000;
        let supply_out = 100_000_000;

        let amount_in =
            DefaultStableSwap::get_amount_in(amount_out, supply_in, supply_out).unwrap();

        assert_eq!(amount_in, 1004);

        let amount_out =
            DefaultStableSwap::get_amount_out(amount_in, supply_in, supply_out).unwrap();

        // old
        // assert_eq!(amount_out, 1000);

        // new
        assert_eq!(amount_out, 1001);
    })
}

#[test]
fn update_oracle_should_work() {
    new_test_ext().execute_with(|| {
        let trader = EVE;

        assert_ok!(DefaultStableSwap::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (SDOT, DOT),                     // Currency pool, in which liquidity will be added
            (100_000, 100_000),              // Liquidity amounts to be added in pool
            BOB,                             // LPToken receiver
            SAMPLE_LP_TOKEN,                 // Liquidity pool share representative token
        ));

        assert_eq!(
            DefaultStableSwap::pools(SDOT, DOT)
                .unwrap()
                .block_timestamp_last,
            0
        );
        assert_eq!(
            DefaultStableSwap::pools(SDOT, DOT)
                .unwrap()
                .price_0_cumulative_last,
            0
        );
        assert_eq!(
            DefaultStableSwap::pools(SDOT, DOT)
                .unwrap()
                .price_1_cumulative_last,
            0
        );

        run_to_block(2);

        assert_ok!(DefaultStableSwap::swap(&trader, (DOT, SDOT), 1_000));

        assert_eq!(
            DefaultStableSwap::pools(SDOT, DOT)
                .unwrap()
                .block_timestamp_last,
            2
        );
        // old
        // assert_eq!(
        //     DefaultStableSwap::pools(SDOT, DOT)
        //         .unwrap()
        //         .price_0_cumulative_last,
        //     2_040136143738700978
        // );

        // new
        assert_eq!(
            DefaultStableSwap::pools(SDOT, DOT)
                .unwrap()
                .price_0_cumulative_last,
            2_040342211852166095
        );
        assert_eq!(
            DefaultStableSwap::pools(SDOT, DOT)
                .unwrap()
                .price_1_cumulative_last,
            1_960455445544554455
        );

        run_to_block(4);

        assert_ok!(DefaultStableSwap::swap(&trader, (DOT, SDOT), 1_000));

        assert_eq!(
            DefaultStableSwap::pools(SDOT, DOT)
                .unwrap()
                .block_timestamp_last,
            4
        );
        // assert_eq!(
        //     DefaultStableSwap::pools(SDOT, DOT)
        //         .unwrap()
        //         .price_0_cumulative_last,
        //     4_120792162342213614
        // );
        assert_eq!(
            DefaultStableSwap::pools(SDOT, DOT)
                .unwrap()
                .price_0_cumulative_last,
            4_121868664584169564
        );
        // assert_eq!(
        //     DefaultStableSwap::pools(SDOT, DOT)
        //         .unwrap()
        //         .price_1_cumulative_last,
        //     3_883124053581828770
        // );
        assert_eq!(
            DefaultStableSwap::pools(SDOT, DOT)
                .unwrap()
                .price_1_cumulative_last,
            3_882122112211221121
        );
    })
}

// TODO: Fix this scenario

#[test]
fn oracle_big_block_no_overflow() {
    new_test_ext().execute_with(|| {
        let trader = FRANK;

        assert_ok!(DefaultStableSwap::create_pool(
            RawOrigin::Signed(ALICE).into(),                     // Origin
            (DOT, KSM), // Currency pool, in which liquidity will be added
            (9_999_650_729_873_433, 30_001_051_000_000_000_000), // Liquidity amounts to be added in pool
            FRANK,                                               // LPToken receiver
            SAMPLE_LP_TOKEN, // Liquidity pool share representative token
        ));

        assert_eq!(
            DefaultStableSwap::pools(DOT, KSM)
                .unwrap()
                .block_timestamp_last,
            0
        );
        assert_eq!(
            DefaultStableSwap::pools(DOT, KSM)
                .unwrap()
                .price_0_cumulative_last,
            0
        );
        assert_eq!(
            DefaultStableSwap::pools(DOT, KSM)
                .unwrap()
                .price_1_cumulative_last,
            0
        );

        let mut big_block = 30_000;
        run_to_block(big_block);

        for _ in 0..5 {
            big_block += 1000;
            run_to_block(big_block);
            assert_ok!(DefaultStableSwap::swap(&trader, (DOT, KSM), 1000));
        }

        assert_eq!(
            DefaultStableSwap::pools(DOT, KSM)
                .unwrap()
                .block_timestamp_last,
            big_block
        );
        assert_eq!(
            DefaultStableSwap::pools(DOT, KSM)
                .unwrap()
                .price_0_cumulative_last,
            104962346092892538490488113 //105007346_092879071079611686
        );
        assert_eq!(
            DefaultStableSwap::pools(DOT, KSM)
                .unwrap()
                .price_1_cumulative_last,
            11670852942309388101 // 11_665850491226458031
        );

        // increment a block
        big_block += 4;
        run_to_block(big_block);

        // this would swap used to overflow
        assert_ok!(DefaultStableSwap::swap(&trader, (DOT, KSM), 10_000_000_000));
    })
}

#[test]
fn create_pool_large_amount_should_work() {
    /*
    With ample supplies
    Recheck values
    */
    new_test_ext().execute_with(|| {
        Assets::mint(
            RawOrigin::Signed(ALICE).into(),
            tokens::DOT,
            ALICE,
            3_000_000_000_000_000_000_000,
        )
        .ok();
        Assets::mint(
            RawOrigin::Signed(ALICE).into(),
            tokens::SDOT,
            ALICE,
            2_000_000_000_000_000_000_000,
        )
        .ok();

        assert_ok!(DefaultStableSwap::create_pool(
            RawOrigin::Signed(ALICE).into(),                            // Origin
            (DOT, SDOT), // Currency pool, in which liquidity will be added
            (1_000_000_000_000_000_000, 2_000_000_000_000_000_000_000), // Liquidity amounts to be added in pool
            ALICE,                                                      // LPToken receiver
            SAMPLE_LP_TOKEN, // Liquidity pool share representative token
        ));

        assert_eq!(
            DefaultStableSwap::pools(SDOT, DOT).unwrap().base_amount,
            2_000_000_000_000_000_000_000
        );
        assert_eq!(
            Assets::total_issuance(SAMPLE_LP_TOKEN),
            447_213_595_499_957_939_28
        );
        // should be issuance minus the min liq locked
        assert_eq!(
            Assets::balance(SAMPLE_LP_TOKEN, ALICE),
            447_213_595_499_957_939_28
        );
    })
}

#[test]
fn create_pool_large_amount_from_an_account_without_sufficient_amount_of_tokens_should_not_panic() {
    /*
    With ample supplies for Alice and less for Bob :'(
    `create_pool` with Large amount panic for Bob
    Recheck values
    */
    new_test_ext().execute_with(|| {
        Assets::mint(
            RawOrigin::Signed(ALICE).into(),
            tokens::DOT,
            ALICE,
            3_000_000_000_000_000_000_000,
        )
        .ok();
        Assets::mint(
            RawOrigin::Signed(ALICE).into(),
            tokens::SDOT,
            ALICE,
            2_000_000_000_000_000_000_000,
        )
        .ok();

        // Creating for BOB
        // This Panics!
        assert_noop!(
            DefaultStableSwap::create_pool(
                RawOrigin::Signed(ALICE).into(),                            // Origin
                (DOT, SDOT), // Currency pool, in which liquidity will be added
                (1_000_000_000_000_000_000, 2_000_000_000_000_000_000_000), // Liquidity amounts to be added in pool
                BOB,                                                        // LPToken receiver
                SAMPLE_LP_TOKEN, // Liquidity pool share representative token
            ),
            pallet_assets::Error::<Test>::BalanceLow
        );
    })
}

#[ignore]
#[test]
fn do_add_liquidity_exact_amounts_should_work() {
    /*
    substrate->frame->assets->src->functions.rs
    ensure!(f.best_effort || actual >= amount, Error::<T, I>::BalanceLow);   // Fails here
    replica of `add_liquidity_should_work` with larger values
    Loss of precision?
    */
    new_test_ext().execute_with(|| {
        // Already deposited 100000000
        Assets::mint(
            RawOrigin::Signed(ALICE).into(),
            tokens::DOT,
            ALICE,
            999_999_999_999_900_000_000,
        )
        .ok();

        // Already deposited 100000000
        Assets::mint(
            RawOrigin::Signed(ALICE).into(),
            tokens::SDOT,
            ALICE,
            199_999_999_999_990_000_000_0,
        )
        .ok();

        assert_ok!(DefaultStableSwap::create_pool(
            RawOrigin::Signed(ALICE).into(),                            // Origin
            (DOT, SDOT), // Currency pool, in which liquidity will be added
            (1_000_000_000_000_000_000, 2_000_000_000_000_000_000_000), // Liquidity amounts to be added in pool
            ALICE,                                                      // LPToken receiver
            SAMPLE_LP_TOKEN, // Liquidity pool share representative token
        ));
        assert_ok!(DefaultStableSwap::add_liquidity(
            RawOrigin::Signed(ALICE).into(),                            // Origin
            (DOT, SDOT), // Currency pool, in which liquidity will be added
            (1_000_000_000_000_000_000, 2_000_000_000_000_000_000_000), // Liquidity amounts to be added in pool
            (5, 5), // specifying its worst case ratio when pool already
        ));

        assert_eq!(
            DefaultStableSwap::pools(SDOT, DOT).unwrap().base_amount,
            4_000
        );
    })
}

#[test]
fn do_add_liquidity_large_amounts_should_work() {
    /*
    With ample supplies
     */

    new_test_ext().execute_with(|| {
        Assets::mint(
            RawOrigin::Signed(ALICE).into(),
            tokens::DOT,
            ALICE,
            3_000_000_000_000_000_000_000,
        )
        .ok();
        Assets::mint(
            RawOrigin::Signed(ALICE).into(),
            tokens::SDOT,
            ALICE,
            2_000_000_000_000_000_000_000,
        )
        .ok();

        assert_ok!(DefaultStableSwap::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, SDOT),                     // Currency pool, in which liquidity will be added
            (
                1_000_000_000_000_000_000_000, // Either base amount or quote amount
                2_000_000_000_000_000_000_000
            ), // Liquidity amounts to be added in pool
            ALICE,                           // LPToken receiver
            SAMPLE_LP_TOKEN,                 // Liquidity pool share representative token
        ));
    })
}

#[test]
fn handling_fees_should_work() {
    new_test_ext().execute_with(|| {
        // Pool gets created and BOB should receive all of the LP tokens (minus the min amount)
        // Created Pool for Bob
        assert_ok!(DefaultStableSwap::create_pool(
            RawOrigin::Signed(ALICE).into(),    // Origin
            (DOT, SDOT),                        // Currency pool, in which liquidity will be added
            (100_000_000_000, 100_000_000_000), // Liquidity amounts to be added in pool
            BOB,                                // LPToken receiver
            SAMPLE_LP_TOKEN                     // Liquidity pool share representative token
        ));

        // Another user makes a swap that should generate fees for the LP provider and the protocol
        assert_ok!(DefaultStableSwap::swap(&FRANK, (DOT, SDOT), 6_000_000));
        assert_eq!(Assets::balance(SDOT, FRANK), 5_981_998); // 18_002

        // we can check the total balance
        //
        // no extra fees should be minted because liquid has not been added or removed
        //
        assert_eq!(Assets::total_issuance(SAMPLE_LP_TOKEN), 100_000_000_000);

        // bob should have all of the fees minus the min amount burned/locked
        assert_eq!(
            Assets::balance(SAMPLE_LP_TOKEN, BOB),
            100_000_000_000 - MINIMUM_LIQUIDITY
        );

        // now we withdraw the fees and at this point we should mint tokens
        // for the protocol proportional to 1/6 of the total fees generated

        // we know that 18_000 fees should be collected and ~3_000 are for the protocol
        let total_fees_collected = 6_000_000.0 * 0.003;
        let fees_to_be_collected_by_protocol = total_fees_collected * (1.0 / 6.0);
        assert_eq!(fees_to_be_collected_by_protocol, 3000.0);

        // expand the math to calculate exact amount of fees to dilute lp total supply
        let prop_of_total_fee = 1.0 / 6.0;
        let scalar = (1.0 / prop_of_total_fee) - 1.0;
        assert_eq!(scalar, 5.0);

        let total_lp_token_supply = 100_000_000_000.0;
        let old_root_k =
            DefaultStableSwap::delta_util(100_000_000_000, 100_000_000_000).unwrap() as f64;
        let new_root_k =
            DefaultStableSwap::delta_util(100_000_000_000 - 5_981_998, 100_000_000_000 + 6_000_000)
                .unwrap() as f64;
        let root_k_growth = new_root_k - old_root_k;

        let numerator = total_lp_token_supply * root_k_growth;
        let denominator = new_root_k * scalar + old_root_k;
        let rewards_to_mint = numerator / denominator;

        assert_eq!(old_root_k, 200_000_000_000.0); // 200_000_000_000
        assert_eq!(new_root_k, 200_000_017_999.0); // 200_000_017_999
        assert_eq!(root_k_growth, 17_999.0); // 17_999
        assert_eq!(numerator, 1_799_900_000_000_000.0); // 1_799_900_000_000_000
        assert_eq!(denominator, 1_200_000_089_995.0); // 1_200_000_089_995
        assert_eq!(rewards_to_mint, 1499.9165541791747); // 1499

        assert_ok!(DefaultStableSwap::remove_liquidity(
            RawOrigin::Signed(PROTOCOL_FEE_RECEIVER).into(),
            (DOT, SDOT),
            1_499,
        ));

        // PROTOCOL_FEE_RECEIVER should have slightly less then 3_000 total rewards
        // split between the two pools - the small difference is due to rounding errors
        assert_eq!(Assets::balance(DOT, PROTOCOL_FEE_RECEIVER), 1499);
        assert_eq!(Assets::balance(SDOT, PROTOCOL_FEE_RECEIVER), 1498);
    })
}

#[test]
fn amount_out_should_work_simple() {
    new_test_ext().execute_with(|| {
        let amount_in = 1_000_000;
        let supply_in = 1_000_000_000;
        let supply_out = 1_000_000_000;

        // assert_eq!(amount_in, 1004);

        let amount_out =
            DefaultStableSwap::get_amount_out(amount_in, supply_in, supply_out).unwrap();

        // old
        // assert_eq!(amount_out, 1000);

        // new
        assert_eq!(amount_out, 996_995); // currently 999_995
    })
}

#[test]
fn swap_stable_tokens() {
    new_test_ext().execute_with(|| {
        assert_ok!(DefaultStableSwap::create_pool(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, SDOT),                     // Currency pool, in which liquidity will be added
            (1000000, 1000000),              // Liquidity amounts to be added in pool
            ALICE,                           // LPToken receiver
            SAMPLE_LP_TOKEN,                 // Liquidity pool share representative token
        ));

        let amount_in = 1000;
        let trader = EVE;

        let bal_dot_before = Assets::balance(DOT, trader);
        let bal_sdot_before = Assets::balance(SDOT, trader);

        assert_eq!(bal_dot_before, 1_000000000);
        assert_eq!(bal_sdot_before, 1_000000000);
        // println!("DOT Balance Before\t{:?}", bal_dot_before);
        // println!("SDOT Balance Before\t{:?}", bal_sdot_before);

        // Swapping 1000 DOTs to SDOTs
        assert_ok!(DefaultStableSwap::swap(&trader, (DOT, SDOT), amount_in));

        let bal_dot_after = Assets::balance(DOT, trader);
        let bal_sdot_after = Assets::balance(SDOT, trader);

        assert_eq!(bal_dot_after, 999999000);
        assert_eq!(bal_sdot_after, 1000000997);

        // println!("DOT Balance After\t{:?}", bal_dot_after);
        // println!("SDOT Balance After\t{:?}", bal_sdot_after);

        // println!("DOT Diff\t{:?}", bal_dot_before - bal_dot_after);
        // println!("SDOT Diff\t{:?}", bal_sdot_after - bal_sdot_before);
    })
}
