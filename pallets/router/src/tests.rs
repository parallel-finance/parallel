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
use core::convert::TryFrom;
use frame_support::{assert_noop, assert_ok};
use mock::*;
use primitives::CurrencyId;

#[test]
fn too_many_or_too_less_routes_should_not_work() {
    new_test_ext().execute_with(|| {
        let routes_11 = Route::<Runtime, ()>::try_from(
            core::iter::repeat((DOT, XDOT))
                .take(MaxLengthRoute::get() as usize + 1)
                .collect::<Vec<(CurrencyId, CurrencyId)>>(),
        );
        assert!(routes_11.is_err());

        // User cannot input empty route.
        assert_noop!(
            AMMRoute::trade(Origin::signed(ALICE), Route::<Runtime, ()>::default(), 1, 2),
            Error::<Runtime>::EmptyRoute
        );
    });
}

#[test]
fn duplicated_routes_should_not_work() {
    new_test_ext().execute_with(|| {
        let dup_routes = Route::<Runtime, ()>::try_from(vec![(DOT, XDOT), (DOT, XDOT)])
            .expect("Failed to create route list.");
        assert_noop!(
            AMMRoute::trade(Origin::signed(ALICE), dup_routes, 1, 2),
            Error::<Runtime>::DuplicatedRoute
        );
    });
}

#[test]
fn too_low_balance_should_not_work() {
    new_test_ext().execute_with(|| {
        let dup_routes = Route::<Runtime, ()>::try_from(vec![(DOT, XDOT)])
            .expect("Failed to create route list.");
        assert_noop!(
            AMMRoute::trade(Origin::signed(ALICE), dup_routes, 0, 0),
            Error::<Runtime>::ZeroBalance
        );
    });
}

#[test]
fn trade_should_work() {
    new_test_ext().execute_with(|| {
        // create pool and add liquidity
        assert_ok!(DefaultAMM::create_pool(
            Origin::signed(ALICE),
            (DOT, XDOT),
            (100_000_000, 100_000_000),
            DAVE,
            10
        ));

        // check that pool was funded correctly
        assert_eq!(
            DefaultAMM::pools(XDOT, DOT).unwrap().base_amount,
            100_000_000
        ); // XDOT
        assert_eq!(
            DefaultAMM::pools(XDOT, DOT).unwrap().quote_amount,
            100_000_000
        ); // DOT

        // calculate amount out
        let routes = Route::<Runtime, ()>::try_from(vec![(DOT, XDOT)])
            .expect("Failed to create route list.");
        assert_ok!(AMMRoute::trade(Origin::signed(ALICE), routes, 1_000, 980));

        // Check Alice should get 994
        assert_eq!(Assets::balance(tokens::XDOT, &ALICE), 10_000 + 994);

        // we should have less XDOT in the pool
        assert_eq!(
            DefaultAMM::pools(XDOT, DOT).unwrap().base_amount,
            99_999_006
        );

        // we should have more DOT
        assert_eq!(
            DefaultAMM::pools(XDOT, DOT).unwrap().quote_amount,
            100_001_000
        );
    })
}

#[test]
fn trade_should_not_work_if_amount_less_than_min_amount_out() {
    new_test_ext().execute_with(|| {
        // create pool and add liquidity
        assert_ok!(DefaultAMM::create_pool(
            Origin::signed(ALICE),
            (DOT, XDOT),
            (100_000_000, 100_000_000),
            DAVE,
            10
        ));

        // check that pool was funded correctly
        assert_eq!(
            DefaultAMM::pools(XDOT, DOT).unwrap().base_amount,
            100_000_000
        ); // XDOT
        assert_eq!(
            DefaultAMM::pools(XDOT, DOT).unwrap().quote_amount,
            100_000_000
        ); // DOT

        // calculate amount out
        let min_amount_out = 999;
        let routes = Route::<Runtime, ()>::try_from(vec![(DOT, XDOT)])
            .expect("Failed to create route list.");
        assert_noop!(
            AMMRoute::trade(Origin::signed(ALICE), routes, 1_000, min_amount_out),
            Error::<Runtime>::UnexpectedSlippage
        );
    })
}

#[test]
fn trade_should_work_more_than_one_route() {
    new_test_ext().execute_with(|| {
        // create pool and add liquidity
        assert_ok!(DefaultAMM::create_pool(
            Origin::signed(ALICE),
            (DOT, XDOT),
            (100_000_000, 100_000_000),
            DAVE,
            10
        ));

        // create pool and add liquidity
        assert_ok!(DefaultAMM::create_pool(
            Origin::signed(ALICE),
            (XDOT, KSM),
            (100_000_000, 100_000_000),
            DAVE,
            11
        ));

        // create pool and add liquidity
        assert_ok!(DefaultAMM::create_pool(
            Origin::signed(ALICE),
            (USDT, KSM),
            (100_000_000, 100_000_000),
            DAVE,
            12
        ));

        // CHECK POOLS
        // check that pool was funded correctly
        assert_eq!(
            DefaultAMM::pools(XDOT, DOT).unwrap().base_amount,
            100_000_000
        ); // XDOT
        assert_eq!(
            DefaultAMM::pools(XDOT, DOT).unwrap().quote_amount,
            100_000_000
        ); // DOT

        // check that pool was funded correctly
        assert_eq!(
            DefaultAMM::pools(XDOT, KSM).unwrap().base_amount,
            100_000_000
        ); // KSM
        assert_eq!(
            DefaultAMM::pools(XDOT, KSM).unwrap().quote_amount,
            100_000_000
        ); // XDOT

        // check that pool was funded correctly
        assert_eq!(
            DefaultAMM::pools(USDT, KSM).unwrap().base_amount,
            100_000_000
        ); // KSM

        assert_eq!(
            DefaultAMM::pools(USDT, KSM).unwrap().quote_amount,
            100_000_000
        ); // USDT

        // Alice should have no USDT
        assert_eq!(Assets::balance(tokens::USDT, &ALICE), 0);

        // DO TRADE
        // calculate amount out
        let routes = Route::<Runtime, ()>::try_from(vec![(DOT, XDOT), (XDOT, KSM), (KSM, USDT)])
            .expect("Failed to create route list.");
        assert_ok!(AMMRoute::trade(Origin::signed(ALICE), routes, 1_000, 980));

        // CHECK TRADER
        // Alice should have no XDOT (it was only a temp transfer)
        assert_eq!(Assets::balance(tokens::XDOT, &ALICE), 10_000);

        // Alice should have no KSM (it was only a temp transfer)
        assert_eq!(Assets::balance(tokens::KSM, &ALICE), 10_000);

        // Alice should now have some USDT!
        assert_eq!(Assets::balance(tokens::USDT, &ALICE), 984);

        // Alice should now have less DOT
        assert_eq!(Assets::balance(tokens::DOT, &ALICE), 9000);

        ////// First Route

        // we should have less XDOT since we traded for DOT
        assert_eq!(
            DefaultAMM::pools(XDOT, DOT).unwrap().base_amount,
            99_999_006
        );

        // we should have more DOT in the pool since the trader sent DOT
        assert_eq!(
            DefaultAMM::pools(XDOT, DOT).unwrap().quote_amount,
            100_001_000
        );

        ////// Second Route

        // we should have more XDOT since were trading it for KSM
        assert_eq!(
            DefaultAMM::pools(XDOT, KSM).unwrap().base_amount,
            100_000_994
        );

        // we should have less KSM
        assert_eq!(
            DefaultAMM::pools(XDOT, KSM).unwrap().quote_amount,
            99_999_011
        );

        ////// Third Route

        // we should have less USDT since its the token the trader is recieving
        assert_eq!(
            DefaultAMM::pools(USDT, KSM).unwrap().base_amount,
            99_999_016
        );

        // we should have more KSM since were trading it for USDT
        assert_eq!(
            DefaultAMM::pools(USDT, KSM).unwrap().quote_amount,
            100_000_989
        );
    })
}
