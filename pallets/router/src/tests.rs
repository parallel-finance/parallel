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
use orml_traits::MultiCurrency;

#[test]
fn too_many_or_too_less_routes_should_not_work() {
    ExtBuilder::default().build().execute_with(|| {
        let routes_11 = core::iter::repeat((0, DOT, XDOT)).take(11).collect();
        assert_noop!(
            AMMRoute::trade(Origin::signed(ALICE), routes_11, 1, 2, 3),
            Error::<Runtime>::ExceedMaxLengthRoute
        );

        assert_noop!(
            AMMRoute::trade(Origin::signed(ALICE), vec![], 1, 2, 3),
            Error::<Runtime>::EmptyRoute
        );
    });
}

#[test]
fn duplicated_routes_should_not_work() {
    ExtBuilder::default().build().execute_with(|| {
        let dup_routes = vec![(0, DOT, XDOT), (0, DOT, XDOT)];
        assert_noop!(
            AMMRoute::trade(Origin::signed(ALICE), dup_routes, 1, 2, 3),
            Error::<Runtime>::DuplicatedRoute
        );
    });
}

#[test]
fn too_low_balance_should_not_work() {
    ExtBuilder::default().build().execute_with(|| {
        let dup_routes = vec![(0, DOT, XDOT)];
        assert_noop!(
            AMMRoute::trade(Origin::signed(ALICE), dup_routes, 0, 0, 3),
            Error::<Runtime>::ZeroBalance
        );
    });
}

#[test]
fn too_small_expiry_should_not_work() {
    ExtBuilder::default().build().execute_with(|| {
        let dup_routes = vec![(0, DOT, XDOT), (1, DOT, XDOT)];
        let current_block_num = 4;
        run_to_block(current_block_num);

        assert_noop!(
            AMMRoute::trade(
                Origin::signed(ALICE),
                dup_routes,
                1,
                2,
                current_block_num - 1
            ),
            Error::<Runtime>::TooSmallExpiry
        );
    });
}

#[test]
fn trade_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        // create pool and add liquidity
        assert_ok!(DOT2XDOT::add_liquidity(
            Origin::signed(DAVE),
            (DOT, XDOT),
            (100_000_000, 100_000_000),
            (99_999, 99_999),
        ));

        // check that pool was funded correctly
        assert_eq!(DOT2XDOT::pools(XDOT, DOT).unwrap().base_amount, 100_000_000); // XDOT
        assert_eq!(
            DOT2XDOT::pools(XDOT, DOT).unwrap().quote_amount,
            100_000_000
        ); // DOT

        // calculate amount out
        assert_ok!(AMMRoute::trade(
            Origin::signed(ALICE),
            vec![(0, DOT, XDOT)],
            1_000,
            980,
            1
        ));

        // Check Alice should get 994
        assert_eq!(Currencies::free_balance(XDOT, &ALICE), 994);

        // pools values should be updated - we should have less XDOT
        assert_eq!(DOT2XDOT::pools(XDOT, DOT).unwrap().base_amount, 99_999_006);

        // pools values should be updated - we should have more DOT in the pool
        assert_eq!(
            DOT2XDOT::pools(XDOT, DOT).unwrap().quote_amount,
            100_000_998
        );
    })
}
