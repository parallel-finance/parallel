// Copyright 2021 Parallel Finance Developer.
// This file is part of Parallel Finance.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
// http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Unit tests for the router pallet.

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::*;

#[test]
fn too_many_routes_should_not_work() {
    new_test_ext().execute_with(|| {
        let routes_11 = core::iter::repeat(DOT)
            .take(MaxLengthRoute::get() as usize + 1)
            .collect::<Vec<CurrencyId>>();

        // User cannot input empty route.
        assert_noop!(
            AMMRoute::swap_exact_tokens_for_tokens(Origin::signed(ALICE), routes_11, 1, 2),
            Error::<Runtime>::ExceedMaxLengthRoute
        );
    });
}
#[test]
fn empty_routes_should_not_work() {
    new_test_ext().execute_with(|| {
        // User cannot input empty route.
        assert_noop!(
            AMMRoute::swap_exact_tokens_for_tokens(Origin::signed(ALICE), Vec::new(), 1, 2),
            Error::<Runtime>::EmptyRoute
        );
    });
}

#[test]
fn duplicated_routes_should_not_work() {
    new_test_ext().execute_with(|| {
        let dup_routes = vec![DOT, SDOT, DOT];

        assert_noop!(
            AMMRoute::swap_exact_tokens_for_tokens(Origin::signed(ALICE), dup_routes, 1, 2),
            Error::<Runtime>::DuplicatedRoute
        );
    });
}

#[test]
fn too_low_balance_should_not_work() {
    new_test_ext().execute_with(|| {
        let route = vec![DOT, SDOT];
        assert_noop!(
            AMMRoute::swap_exact_tokens_for_tokens(Origin::signed(ALICE), route, 0, 0),
            Error::<Runtime>::ZeroBalance
        );
    });
}

#[test]
fn swap_exact_tokens_for_tokens_should_work() {
    new_test_ext().execute_with(|| {
        let trader = ALICE;

        // create pool and add liquidity
        assert_ok!(DefaultAMM::create_pool(
            Origin::signed(ALICE),
            (DOT, SDOT),
            (100_000_000, 100_000_000),
            DAVE,
            SAMPLE_LP_TOKEN
        ));

        let route = vec![DOT, SDOT];

        // check balances before swap
        assert_eq!(Assets::balance(DOT, trader), 10_000);
        assert_eq!(Assets::balance(SDOT, trader), 10_000);

        AMMRoute::swap_exact_tokens_for_tokens(
            Origin::signed(ALICE),
            route,
            1_000, // amount_in
            900,   // min_amount_out
        )
        .unwrap();

        assert_eq!(Assets::balance(DOT, trader), 10_000 - 1_000);

        assert_eq!(Assets::balance(SDOT, trader), 10_000 + 994);
    });
}

#[test]
fn swap_tokens_for_exact_tokens_should_work() {
    new_test_ext().execute_with(|| {
        let trader = ALICE;

        // create pool and add liquidity
        assert_ok!(DefaultAMM::create_pool(
            Origin::signed(ALICE),
            (DOT, SDOT),
            (100_000_000, 100_000_000),
            DAVE,
            SAMPLE_LP_TOKEN
        ));

        let route = vec![DOT, SDOT];

        // check balances before swap
        assert_eq!(Assets::balance(DOT, trader), 10_000);
        assert_eq!(Assets::balance(SDOT, trader), 10_000);

        AMMRoute::swap_tokens_for_exact_tokens(
            Origin::signed(ALICE),
            route,
            1_000, // amount_out
            1_010, // max_amount_in
        )
        .unwrap();

        // check balances after swap
        assert_eq!(Assets::balance(DOT, trader), 10_000 - 1_006);
        assert_eq!(Assets::balance(SDOT, trader), 10_000 + 1_000);
    });
}

#[test]
fn pool_as_bridge_swap_tokens_for_exact_tokens_should_work() {
    new_test_ext().execute_with(|| {
        let trader = ALICE;

        // create pool and add liquidity
        assert_ok!(DefaultAMM::create_pool(
            Origin::signed(ALICE),
            (USDT, SDOT),
            (40_000_000, 1_000_000),
            DAVE,
            SAMPLE_LP_TOKEN
        ));
        // 1 SDOT ~= 40 USDT

        // create pool and add liquidity
        assert_ok!(DefaultAMM::create_pool(
            Origin::signed(ALICE),
            (DOT, SDOT),
            (50_000_000, 50_000_000),
            DAVE,
            SAMPLE_LP_TOKEN_2
        ));
        // 1 DOT == 1 SDOT

        let route = vec![DOT, SDOT, USDT];

        // check balances before swap
        assert_eq!(Assets::balance(DOT, trader), 10_000);
        assert_eq!(Assets::balance(SDOT, trader), 10_000);
        assert_eq!(Assets::balance(USDT, trader), 0);

        let exact_amount_we_want_out = 20_000;
        // 20_000 / 40 ~= 500
        // however we need to cover fees
        let max_input_token_willing_to_spend = 510;

        AMMRoute::swap_tokens_for_exact_tokens(
            Origin::signed(ALICE),
            route,
            exact_amount_we_want_out,         // want 1_000 USDT
            max_input_token_willing_to_spend, // dont want to spend more than 4_000 DOT
        )
        .unwrap();

        // check balances after swap
        assert_eq!(Assets::balance(DOT, trader), 10_000 - 508);
        assert_eq!(Assets::balance(SDOT, trader), 10_000 + 1);
        assert_eq!(Assets::balance(USDT, trader), 0 + 20_000 + 69);
    });
}

#[test]
fn swap_exact_tokens_for_tokens_should_not_work_if_amount_less_than_min_amount_out() {
    new_test_ext().execute_with(|| {
        // create pool and add liquidity
        assert_ok!(DefaultAMM::create_pool(
            Origin::signed(ALICE),
            (DOT, SDOT),
            (100_000_000, 100_000_000),
            DAVE,
            SAMPLE_LP_TOKEN
        ));

        // check that pool was funded correctly
        assert_eq!(
            DefaultAMM::pools(SDOT, DOT).unwrap().base_amount,
            100_000_000
        ); // SDOT

        assert_eq!(
            DefaultAMM::pools(SDOT, DOT).unwrap().quote_amount,
            100_000_000
        ); // DOT

        // calculate amount out
        let min_amount_out = 999;
        let routes = vec![DOT, SDOT];
        assert_noop!(
            AMMRoute::swap_exact_tokens_for_tokens(
                Origin::signed(ALICE),
                routes,
                1_000,
                min_amount_out
            ),
            Error::<Runtime>::MinimumAmountOutViolated
        );
    })
}

#[test]
fn swap_tokens_for_exact_tokens_should_not_work_if_amount_more_than_max_amount_in() {
    new_test_ext().execute_with(|| {
        // create pool and add liquidity
        assert_ok!(DefaultAMM::create_pool(
            Origin::signed(ALICE),
            (DOT, SDOT),
            (100_000_000, 100_000_000),
            DAVE,
            SAMPLE_LP_TOKEN
        ));

        // check that pool was funded correctly
        assert_eq!(
            DefaultAMM::pools(SDOT, DOT).unwrap().base_amount,
            100_000_000
        ); // SDOT

        assert_eq!(
            DefaultAMM::pools(SDOT, DOT).unwrap().quote_amount,
            100_000_000
        ); // DOT

        // calculate amount out
        let max_amount_in = 999;
        let routes = vec![DOT, SDOT];
        assert_noop!(
            AMMRoute::swap_tokens_for_exact_tokens(
                Origin::signed(ALICE),
                routes,
                1_000,
                max_amount_in
            ),
            Error::<Runtime>::MaximumAmountInViolated
        );
    })
}

#[test]
fn trade_should_work_more_than_one_route() {
    new_test_ext().execute_with(|| {
        // create pool and add liquidity
        assert_ok!(DefaultAMM::create_pool(
            Origin::signed(ALICE),
            (DOT, SDOT),
            (100_000_000, 100_000_000),
            DAVE,
            SAMPLE_LP_TOKEN
        ));

        // create pool and add liquidity
        assert_ok!(DefaultAMM::create_pool(
            Origin::signed(ALICE),
            (SDOT, KSM),
            (100_000_000, 100_000_000),
            DAVE,
            SAMPLE_LP_TOKEN_2
        ));

        // create pool and add liquidity
        assert_ok!(DefaultAMM::create_pool(
            Origin::signed(ALICE),
            (USDT, KSM),
            (100_000_000, 100_000_000),
            DAVE,
            SAMPLE_LP_TOKEN_3
        ));

        // CHECK POOLS
        // check that pool was funded correctly
        assert_eq!(
            DefaultAMM::pools(SDOT, DOT).unwrap().base_amount,
            100_000_000
        ); // SDOT
        assert_eq!(
            DefaultAMM::pools(SDOT, DOT).unwrap().quote_amount,
            100_000_000
        ); // DOT

        // check that pool was funded correctly
        assert_eq!(
            DefaultAMM::pools(SDOT, KSM).unwrap().base_amount,
            100_000_000
        ); // KSM
        assert_eq!(
            DefaultAMM::pools(SDOT, KSM).unwrap().quote_amount,
            100_000_000
        ); // SDOT

        // check that pool was funded correctly
        assert_eq!(
            DefaultAMM::pools(USDT, KSM).unwrap().base_amount,
            100_000_000
        ); // KSM

        assert_eq!(
            DefaultAMM::pools(USDT, KSM).unwrap().quote_amount,
            100_000_000
        ); // USDT

        // Alice should have original amount ofDOT
        assert_eq!(Assets::balance(tokens::DOT, &ALICE), 10_000);

        // Alice should have original amount of SDOT
        assert_eq!(Assets::balance(tokens::SDOT, &ALICE), 10_000);

        // Alice should have original amount of KSM
        assert_eq!(Assets::balance(tokens::KSM, &ALICE), 10_000);

        // Alice should have no USDT
        assert_eq!(Assets::balance(tokens::USDT, &ALICE), 0);

        // DO TRADE
        // calculate amount out
        let routes = vec![DOT, SDOT, KSM, USDT];
        assert_ok!(AMMRoute::swap_exact_tokens_for_tokens(
            Origin::signed(ALICE),
            routes,
            1_000,
            980
        ));

        // CHECK TRADER AFTER TRADES

        // Alice should now have less DOT
        assert_eq!(Assets::balance(tokens::DOT, &ALICE), 9_000);

        // Alice should have original amount of SDOT
        // (temp transfer) were made within swap
        assert_eq!(Assets::balance(tokens::SDOT, &ALICE), 10_000);

        // Alice should have original amount of KSM
        // (temp transfer) were made within swap
        assert_eq!(Assets::balance(tokens::KSM, &ALICE), 10_000);

        // Alice should now have some USDT!
        assert_eq!(Assets::balance(tokens::USDT, &ALICE), 984);

        // First Pool

        // we should have more DOT in the pool since the trader sent DOT
        assert_eq!(
            DefaultAMM::pools(SDOT, DOT).unwrap().quote_amount,
            100_000_000 + 1_000
        );

        // we should have less SDOT since we traded for DOT
        assert_eq!(
            DefaultAMM::pools(SDOT, DOT).unwrap().base_amount,
            100_000_000 - 994
        );

        // Second Pool

        // we should have more SDOT since were trading it for KSM
        assert_eq!(
            DefaultAMM::pools(SDOT, KSM).unwrap().base_amount,
            100_000_000 + 994
        );

        // we should have less KSM
        assert_eq!(
            DefaultAMM::pools(SDOT, KSM).unwrap().quote_amount,
            100_000_000 - 989
        );

        // Third Pool

        // we should have more KSM since were trading it for USDT
        assert_eq!(
            DefaultAMM::pools(USDT, KSM).unwrap().quote_amount,
            100_000_000 + 989
        );

        // we should have less USDT since its the token the trader is recieving
        assert_eq!(
            DefaultAMM::pools(USDT, KSM).unwrap().base_amount,
            100_000_000 - 984
        );
    })
}

#[test]
fn get_all_routes_should_work() {
    new_test_ext().execute_with(|| {
        let input_amount = 1_000;
        // create pool and add liquidity
        assert_ok!(DefaultAMM::create_pool(
            Origin::signed(ALICE),
            (DOT, SDOT),
            (100_000_000, 90_000_000),
            DAVE,
            SAMPLE_LP_TOKEN
        ));

        // create pool and add liquidity
        assert_ok!(DefaultAMM::create_pool(
            Origin::signed(ALICE),
            (SDOT, KSM),
            (100_000_000, 100_000_000),
            DAVE,
            SAMPLE_LP_TOKEN_2
        ));

        // create pool and add liquidity
        assert_ok!(DefaultAMM::create_pool(
            Origin::signed(ALICE),
            (DOT, KSM),
            (100_000_000, 70_000_000),
            DAVE,
            SAMPLE_LP_TOKEN_3
        ));

        let routes = AMMRoute::get_all_routes(
            input_amount, // input amount
            DOT,          // input token
            KSM,          // output token
        )
        .unwrap();

        // Returns descending order `highest` value first.
        assert_eq!(
            routes,
            vec![(vec![101, 1001, 100], 890), (vec![101, 100], 696)]
        );
    })
}

#[test]
fn get_best_route_should_work() {
    new_test_ext().execute_with(|| {
        let input_amount = 1_000;
        // create pool and add liquidity
        assert_ok!(DefaultAMM::create_pool(
            Origin::signed(ALICE),
            (DOT, SDOT),
            (100_000_000, 90_000_000),
            DAVE,
            SAMPLE_LP_TOKEN
        ));

        // create pool and add liquidity
        assert_ok!(DefaultAMM::create_pool(
            Origin::signed(ALICE),
            (SDOT, KSM),
            (100_000_000, 100_000_000),
            DAVE,
            SAMPLE_LP_TOKEN_2
        ));

        // create pool and add liquidity
        assert_ok!(DefaultAMM::create_pool(
            Origin::signed(ALICE),
            (DOT, KSM),
            (100_000_000, 70_000_000),
            DAVE,
            SAMPLE_LP_TOKEN_3
        ));

        let best_route = AMMRoute::get_best_route(
            input_amount, // input amount
            DOT,          // input token
            KSM,          // output token
        )
        .unwrap();

        // Returns descending order `highest` value first.
        assert_eq!(best_route, (vec![101, 1001, 100], 890));
    })
}

#[test]
fn get_route_for_tokens_not_in_graph_should_not_work() {
    new_test_ext().execute_with(|| {
        let input_amount = 1_000;
        // create pool and add liquidity
        assert_ok!(DefaultAMM::create_pool(
            Origin::signed(ALICE),
            (DOT, SDOT),
            (100_000_000, 90_000_000),
            DAVE,
            SAMPLE_LP_TOKEN
        ));

        // create pool and add liquidity
        assert_ok!(DefaultAMM::create_pool(
            Origin::signed(ALICE),
            (SDOT, KSM),
            (100_000_000, 100_000_000),
            DAVE,
            SAMPLE_LP_TOKEN_2
        ));

        // create pool and add liquidity
        assert_ok!(DefaultAMM::create_pool(
            Origin::signed(ALICE),
            (DOT, KSM),
            (100_000_000, 70_000_000),
            DAVE,
            SAMPLE_LP_TOKEN_3
        ));

        assert_noop!(
            AMMRoute::get_best_route(
                input_amount, // input amount
                SDOT,         // input token
                USDT,         // output token (not in any pool)
            ),
            Error::<Runtime>::TokenDoesNotExists
        );
    })
}

#[test]
fn get_route_for_tokens_not_possible_should_not_work() {
    new_test_ext().execute_with(|| {
        let input_amount = 1_000;
        // create pool and add liquidity
        assert_ok!(DefaultAMM::create_pool(
            Origin::signed(ALICE),
            (DOT, SDOT),
            (100_000_000, 90_000_000),
            DAVE,
            SAMPLE_LP_TOKEN
        ));

        // create pool and add liquidity
        assert_ok!(DefaultAMM::create_pool(
            Origin::signed(ALICE),
            (KSM, USDT),
            (100_000_000, 70_000_000),
            DAVE,
            SAMPLE_LP_TOKEN_3
        ));

        assert_noop!(
            AMMRoute::get_best_route(
                input_amount, // input amount
                SDOT,         // input token
                USDT,         // output token
            ),
            Error::<Runtime>::NoPossibleRoute
        );
    })
}

#[test]
fn get_routes_for_non_existing_pair_should_not_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(DefaultAMM::create_pool(
            Origin::signed(ALICE),
            (DOT, KSM),
            (100_000_000, 70_000_000),
            DAVE,
            SAMPLE_LP_TOKEN_3
        ));
        assert_noop!(
            AMMRoute::get_best_route(
                10000, // input amount
                USDT,  // input token
                SDOT,  // output token
            ),
            Error::<Runtime>::TokenDoesNotExists
        );
    });
}

#[test]
fn get_best_route_same_tokens_should_work() {
    new_test_ext().execute_with(|| {
        let input_amount = 1_000;
        // create pool and add liquidity
        assert_ok!(DefaultAMM::create_pool(
            Origin::signed(ALICE),
            (DOT, SDOT),
            (100_000_000, 90_000_000),
            DAVE,
            SAMPLE_LP_TOKEN
        ));

        // create pool and add liquidity
        assert_ok!(DefaultAMM::create_pool(
            Origin::signed(ALICE),
            (SDOT, KSM),
            (100_000_000, 100_000_000),
            DAVE,
            SAMPLE_LP_TOKEN_2
        ));

        // create pool and add liquidity
        assert_ok!(DefaultAMM::create_pool(
            Origin::signed(ALICE),
            (DOT, KSM),
            (100_000_000, 70_000_000),
            DAVE,
            SAMPLE_LP_TOKEN_3
        ));

        let best_route = AMMRoute::get_best_route(
            input_amount, // input amount
            DOT,          // input token
            DOT,          // output token
        )
        .unwrap();

        assert_eq!(best_route, (vec![101], 1000));
    })
}
