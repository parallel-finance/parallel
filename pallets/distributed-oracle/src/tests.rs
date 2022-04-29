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

use super::*;
use mock::*;

use frame_support::{assert_noop, assert_ok};

#[test]
fn test_add_stake() {
    new_test_ext().execute_with(|| {
        assert_ok!(Doracle::stake(Origin::signed(ALICE), HKO, 100_000));
        let oracle_deposit = Doracle::staking_pool(ALICE, HKO).unwrap();
        assert_eq!(oracle_deposit.total, 100_000);

        assert_ok!(Doracle::stake(Origin::signed(ALICE), HKO, 200_000));
        let oracle_deposit = Doracle::staking_pool(ALICE, HKO).unwrap();
        assert_eq!(oracle_deposit.total, 300_000);
    });
}

#[test]
fn test_stake_with_invalid_asset() {
    // Tries to stake with non a native token
    new_test_ext().execute_with(|| {
        assert_noop!(
            Doracle::stake(Origin::signed(ALICE), 10, 100_000),
            Error::<Test>::InvalidStakingAsset
        );
    });
}

#[test]
fn test_stake_with_amount_less_than_minimum_amount() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Doracle::stake(Origin::signed(ALICE), HKO, 10),
            Error::<Test>::InsufficientStakeAmount
        );
    });
}
//
//
// #[test]
// fn test_add_stake() {
//     new_test_ext().execute_with(|| {
//
//         assert_ok!(Doracle::create_something(Origin::signed(ALICE),));
//
//         // let staked = staking_pool(Origin::signed(ALICE));
//
//         assert_eq!(1, 1);
//     });
// }
