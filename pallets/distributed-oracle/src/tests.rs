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
        assert_ok!(Doracle::register_repeater(Origin::signed(ALICE)));
        assert_ok!(Doracle::stake(Origin::signed(ALICE), HKO, 100_000));
        let oracle_stake_deposit = Doracle::staking_pool(ALICE, HKO).unwrap();
        assert_eq!(oracle_stake_deposit.total, 100_000);

        assert_ok!(Doracle::stake(Origin::signed(ALICE), HKO, 200_000));
        let oracle_stake_deposit = Doracle::staking_pool(ALICE, HKO).unwrap();
        assert_eq!(oracle_stake_deposit.total, 300_000);

        assert_ok!(Doracle::stake(Origin::signed(ALICE), HKO, 200_000));
        assert_ok!(Doracle::stake(Origin::signed(ALICE), HKO, 200_000));
        assert_ok!(Doracle::stake(Origin::signed(ALICE), HKO, 200_000));
        let oracle_stake_deposit = Doracle::staking_pool(ALICE, HKO).unwrap();
        assert_eq!(oracle_stake_deposit.total, 900_000);
    });
}

#[test]
fn test_stake_with_invalid_asset() {
    // Tries to stake with non a native token
    new_test_ext().execute_with(|| {
        assert_ok!(Doracle::register_repeater(Origin::signed(ALICE)));
        assert_noop!(
            Doracle::stake(Origin::signed(ALICE), 10, 100_000),
            Error::<Test>::InvalidStakingCurrency
        );
    });
}

#[test]
fn test_stake_with_amount_less_than_minimum_amount() {
    new_test_ext().execute_with(|| {
        assert_ok!(Doracle::register_repeater(Origin::signed(ALICE)));
        assert_noop!(
            Doracle::stake(Origin::signed(ALICE), HKO, 10),
            Error::<Test>::InsufficientStakeAmount
        );
    });
}

#[test]
#[ignore]
// TODO: Check this scenario
fn test_unstake_stake_amount() {
    new_test_ext().execute_with(|| {
        assert_ok!(Doracle::register_repeater(Origin::signed(ALICE)));
        // Alice nicely staked 100_000
        assert_ok!(Doracle::stake(Origin::signed(ALICE), HKO, 100_000));
        assert_ok!(Doracle::unstake(Origin::signed(ALICE), HKO, 100_000));
    });
}

#[test]
// TODO: Check this scenario
fn test_unstake() {
    new_test_ext().execute_with(|| {
        assert_ok!(Doracle::register_repeater(Origin::signed(ALICE)));
        // Alice nicely staked 100_000
        assert_ok!(Doracle::stake(Origin::signed(ALICE), HKO, 100_000));

        // Unstake 99_999
        // Remains 100_000 - 90_000 = 10_000
        assert_ok!(Doracle::unstake(Origin::signed(ALICE), HKO, 90_000));
        let oracle_stake_deposit = Doracle::staking_pool(ALICE, HKO).unwrap();
        assert_eq!(oracle_stake_deposit.total, 10_000);

        // Stakes again
        // balance -> 10_000 + 500 = 10_500
        // balance after unstake -> 10_500 - 6_000 = 4500
        assert_ok!(Doracle::stake(Origin::signed(ALICE), HKO, 500));
        assert_ok!(Doracle::unstake(Origin::signed(ALICE), HKO, 6_000));
        let oracle_stake_deposit = Doracle::staking_pool(ALICE, HKO).unwrap();
        assert_eq!(oracle_stake_deposit.total, 4_500);

        assert_ok!(Doracle::unstake(Origin::signed(ALICE), HKO, 11));
        assert_ok!(Doracle::unstake(Origin::signed(ALICE), HKO, 11));

        let oracle_stake_deposit = Doracle::staking_pool(ALICE, HKO).unwrap();
        assert_eq!(oracle_stake_deposit.total, 4478);
    });
}

#[test]
fn test_unstake_stake_erroneous_scenarios() {
    new_test_ext().execute_with(|| {
        assert_ok!(Doracle::register_repeater(Origin::signed(ALICE)));
        // Alice nicely staked 100_000
        assert_ok!(Doracle::stake(Origin::signed(ALICE), HKO, 100_000));

        // Trying to unstake non native currency
        assert_noop!(
            Doracle::unstake(Origin::signed(ALICE), 10, 100_000),
            Error::<Test>::InvalidStakingCurrency
        );

        // Unstake an insufficient amount
        assert_noop!(
            Doracle::unstake(Origin::signed(ALICE), HKO, 10),
            Error::<Test>::InsufficientUnStakeAmount
        );

        // Unstake more than staked amount
        assert_noop!(
            Doracle::unstake(Origin::signed(ALICE), HKO, 10),
            Error::<Test>::InsufficientUnStakeAmount
        );

        // Unstake from an account without a stake though a repeater
        assert_ok!(Doracle::register_repeater(Origin::signed(BOB)));
        assert_noop!(
            Doracle::unstake(Origin::signed(BOB), HKO, 11),
            Error::<Test>::StakingAccountNotFound
        );

        // Unstake amount isn larger than staked amount
        assert_noop!(
            Doracle::unstake(Origin::signed(ALICE), HKO, 100_001),
            Error::<Test>::UnstakeAmoutExceedsStakedBalance
        );
    });
}

#[test]
fn test_register_repeater() {
    new_test_ext().execute_with(|| {
        // Register a staking account as a repeater
        assert_ok!(Doracle::register_repeater(Origin::signed(ALICE)));

        // Tries to register the same account as the repeater
        assert_noop!(
            Doracle::register_repeater(Origin::signed(ALICE)),
            Error::<Test>::RepeaterExists
        );
    });
}

#[test]
fn test_stake_as_non_repeater() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Doracle::stake(Origin::signed(ALICE), HKO, 100_000),
            Error::<Test>::InvalidStaker
        );
    });
}

#[test]
fn test_unstake_as_non_repeater() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Doracle::unstake(Origin::signed(ALICE), HKO, 10),
            Error::<Test>::InvalidUnstaker
        );
    });
}

// TODO: Implement the followings
#[test]
fn test_slashing() {}

#[test]
fn test_slashing_errors() {}

#[test]
fn test_manager() {}

#[test]
fn test_reweard_scenarions() {
    // Test for data contribution

    // Test for reposting slashable  activities
}
