use super::*;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok};
use frame_system::RawOrigin;
use primitives::{tokens, StableSwap as _};

const MINIMUM_LIQUIDITY: u128 = 1_000;

#[test]
fn test_dex_demo() {
    new_test_ext().execute_with(|| {
        assert_ok!(DefaultStableSwap::create_pool(
            RawOrigin::Signed(ALICE).into(),              // Origin
            (DOT, KSM), // Currency pool, in which liquidity will be added
            (10_000_000_000_000, 50_000_000_000_000_000), // Liquidity amounts to be added in pool
            FRANK,      // LPToken receiver
            SAMPLE_LP_TOKEN, // Liquidity pool share representative token
        ));

        // let unit = 1_000_000_000_000_u128;
        let unit = 1_000_000_u128;
        let usdc_price = 1 * unit;

        let nb_of_usdc = 1_000_000_000;
        let usdt_price = 1 * unit;

        let nb_of_usdt = 1_000_000_000;

        // TODO: Change this to DOT/KSM
        // 10^9 USDC/10^9 USDT
        let initial_usdc = nb_of_usdc * usdc_price;
        let initial_usdt = nb_of_usdt * usdt_price;
        //
        //
        //
        // assert_ok!(DefaultStableSwap::create_pool(
        //     RawOrigin::Signed(ALICE).into(), // Origin
        //     (DOT, KSM),
        //     (10_000_000_000_000, 10_000_000_000_000_000), // Currency pool, in which liquidity will be added
        //     // (initial_usdc, initial_usdt),    // Liquidity amounts to be added in pool
        //     ALICE,                           // LPToken receiver
        //     SAMPLE_LP_TOKEN                  // Liquidity pool share representative token
        // ));

        //
        assert_ok!(DefaultStableSwap::add_liquidity(
            RawOrigin::Signed(ALICE).into(), // Origin
            (DOT, KSM),                      // Currency pool, in which liquidity will be added
            (100_000, 100_000),              // Liquidity amounts to be added in pool
            (0, 0),                          // specifying its worst case ratio when pool already
        ));
        //
        // let precision = 100;
        // let epsilon = 1;
        // // 1 unit of usdc == 1 unit of usdt
        let ratio = DefaultStableSwap::get_exchange_value((DOT, KSM), USDC, unit)
            .expect("get_exchange_value failed");
        /*assert_ok!(acceptable_computation_error(ratio, unit, precision, epsilon));

        let swap_usdc = 100_u128 * unit;
        assert_ok!(Tokens::mint_into(USDC, &BOB, swap_usdc));
        // mint 1 USDT, after selling 100 USDC we get 99 USDT so to buy 100 USDC we need 100 USDT
        assert_ok!(Tokens::mint_into(USDT, &BOB, unit));

        StableSwap::sell(Origin::signed(BOB), pool_id, USDC, swap_usdc, false)
            .expect("sell failed");

        StableSwap::buy(Origin::signed(BOB), pool_id, USDC, swap_usdc, false).expect("buy failed");

        let bob_usdc = Tokens::balance(USDC, &BOB);

        assert_ok!(acceptable_computation_error(
            bob_usdc.into(),
            swap_usdc.into(),
            precision,
            epsilon
        ));
        let lp = Tokens::balance(pool.lp_token, &ALICE);
        assert_ok!(StableSwap::remove_liquidity(Origin::signed(ALICE), pool_id, lp, 0, 0));

        // Alice should get back a different amount of tokens.
        let alice_usdc = Tokens::balance(USDC, &ALICE);
        let alice_usdt = Tokens::balance(USDT, &ALICE);
        assert_ok!(default_acceptable_computation_error(alice_usdc.into(), initial_usdc.into()));
        assert_ok!(default_acceptable_computation_error(alice_usdt.into(), initial_usdt.into()));*/
    });
}
