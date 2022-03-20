use super::*;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok};
use frame_system::RawOrigin;
use primitives::{tokens, StableSwap as _};

const MINIMUM_LIQUIDITY: u128 = 1_000;

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

        let dy = 1_000_000u128.checked_sub(y).unwrap();

        assert_eq!(dy, 9999);
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

        assert_eq!(ex_ratio, 1); // 33202
        assert_eq!(dy, 10); // 332027
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

        let amount_in = 999_999;
        let y = AMM::get_alternative_var(amount_in, (DOT, XDOT)).unwrap();

        let dy = 1_000_000u128.checked_sub(y).unwrap();
        let ex_ratio = dy.checked_div(amount_in).unwrap();

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

        let dy = 1_000_000u128.checked_sub(y).unwrap();
        let ex_ratio = dy.checked_div(amount_in).unwrap();

        assert_eq!(ex_ratio, 10);
        assert_eq!(dy, 5167);
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

        let dy = 1_000_000u128.checked_sub(y).unwrap();
        let ex_ratio = dy.checked_div(amount_in).unwrap();

        assert_eq!(ex_ratio, 10);
        assert_eq!(dy, 1720);
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

        let dy = 1_000_000u128.checked_sub(y).unwrap();
        let ex_ratio = dy.checked_div(amount_in).unwrap();

        assert_eq!(ex_ratio, 1);
        assert_eq!(dy, 10012);
    })
}
