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
use std::collections::BTreeMap;

use frame_support::{assert_noop, assert_ok};

#[test]
fn test_add_stake() {
    new_test_ext().execute_with(|| {
        assert_ok!(Doracle::register_repeater(Origin::signed(ALICE), HKO));
        assert_ok!(Doracle::stake(Origin::signed(ALICE), HKO, 100_000));

        let rep = Doracle::repeaters(ALICE, HKO).unwrap();

        assert_eq!(rep.staked_balance, 100_000);

        assert_ok!(Doracle::stake(Origin::signed(ALICE), HKO, 200_000));
        let rep = Doracle::repeaters(ALICE, HKO).unwrap();
        assert_eq!(rep.staked_balance, 300_000);

        assert_ok!(Doracle::stake(Origin::signed(ALICE), HKO, 200_000));
        assert_ok!(Doracle::stake(Origin::signed(ALICE), HKO, 200_000));
        assert_ok!(Doracle::stake(Origin::signed(ALICE), HKO, 200_000));
        let rep = Doracle::repeaters(ALICE, HKO).unwrap();
        assert_eq!(rep.staked_balance, 900_000);
    });
}

#[test]
fn test_stake_with_invalid_asset() {
    // Tries to stake with non a native token
    new_test_ext().execute_with(|| {
        assert_ok!(Doracle::register_repeater(Origin::signed(ALICE), HKO));
        assert_noop!(
            Doracle::stake(Origin::signed(ALICE), 10, 100_000),
            Error::<Test>::InvalidStakingCurrency
        );
    });
}

#[test]
fn test_stake_with_amount_less_than_minimum_amount() {
    new_test_ext().execute_with(|| {
        assert_ok!(Doracle::register_repeater(Origin::signed(ALICE), HKO));
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
        assert_ok!(Doracle::register_repeater(Origin::signed(ALICE), HKO));
        // Alice nicely staked 100_000
        assert_ok!(Doracle::stake(Origin::signed(ALICE), HKO, 100_000));

        // NOTE: we should have a cool down period
        // this should be invaild
        assert_ok!(Doracle::unstake(Origin::signed(ALICE), HKO, 100_000));
    });
}

#[test]
// TODO: Check this scenario
fn test_unstake() {
    new_test_ext().execute_with(|| {
        assert_ok!(Doracle::register_repeater(Origin::signed(ALICE), HKO));
        // Alice nicely staked 100_000
        assert_ok!(Doracle::stake(Origin::signed(ALICE), HKO, 100_000));

        // Unstake 99_999
        // Remains 100_000 - 90_000 = 10_000
        assert_ok!(Doracle::unstake(Origin::signed(ALICE), HKO, 90_000));

        let rep = Doracle::repeaters(ALICE, HKO).unwrap();

        assert_eq!(rep.staked_balance, 10_000);

        // Stakes again
        // balance -> 10_000 + 500 = 10_500
        // balance after unstake -> 10_500 - 6_000 = 4500
        assert_ok!(Doracle::stake(Origin::signed(ALICE), HKO, 500));
        assert_ok!(Doracle::unstake(Origin::signed(ALICE), HKO, 6_000));

        let rep = Doracle::repeaters(ALICE, HKO).unwrap();

        assert_eq!(rep.staked_balance, 4_500);

        assert_ok!(Doracle::unstake(Origin::signed(ALICE), HKO, 11));
        assert_ok!(Doracle::unstake(Origin::signed(ALICE), HKO, 11));

        let rep = Doracle::repeaters(ALICE, HKO).unwrap();

        assert_eq!(rep.staked_balance, 4478);
    });
}

#[test]
fn test_unstake_stake_erroneous_scenarios() {
    new_test_ext().execute_with(|| {
        assert_ok!(Doracle::register_repeater(Origin::signed(ALICE), HKO));
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
        assert_ok!(Doracle::register_repeater(Origin::signed(BOB), HKO));
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
        // TODO: Flip This -> Stake and register~
        // NOTE: we might want to flip this? stake then register

        // Register a staking account as a repeater
        assert_ok!(Doracle::register_repeater(Origin::signed(ALICE), HKO));

        // Tries to register the same account as the repeater
        assert_noop!(
            Doracle::register_repeater(Origin::signed(ALICE), HKO),
            Error::<Test>::RepeaterExists
        );
    });
}

#[test]
fn test_stake_as_non_repeater() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Doracle::stake(Origin::signed(ALICE), HKO, 100_000),
            Error::<Test>::InvalidRepeater
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

#[test]
fn test_first_round() {
    new_test_ext().execute_with(|| {
        assert_ok!(Doracle::populate_treasury(Origin::signed(ALICE)));

        assert_ok!(Doracle::register_repeater(Origin::signed(ALICE), HKO));
        assert_ok!(Doracle::stake(Origin::signed(ALICE), HKO, 100_00));

        assert_ok!(Doracle::register_repeater(Origin::signed(BOB), HKO));
        assert_ok!(Doracle::stake(Origin::signed(BOB), HKO, 100_000));

        assert_ok!(Doracle::register_repeater(Origin::signed(CHARLIE), HKO));
        assert_ok!(Doracle::stake(Origin::signed(CHARLIE), HKO, 100_000));

        let round_id = 1u128;

        assert_ok!(Doracle::set_price_for_round(
            Origin::signed(ALICE),          // origin
            HKO,                            // asset_id
            Price::from_inner(100_000 * 1), // price
            round_id                        // round_id
        ));

        // assert_ok!(Doracle::set_price_for_round(
        //     Origin::signed(BOB),
        //     HKO,
        //     Price::from_inner(100_000 * 1),
        //     round_id
        // ));
        //
        // assert_ok!(Doracle::set_price_for_round(
        //     Origin::signed(CHARLIE),
        //     HKO,
        //     Price::from_inner(100_000 * 1),
        //     round_id
        // ));

        let expected_participated = BTreeMap::new().insert(ALICE, 6);

        let manager = Doracle::get_round_manager().unwrap();
        // let p = manager.participated;
        assert_eq!(manager.participated, BTreeMap::new());
        assert_eq!(manager.people_to_slash, BTreeMap::new());
        assert_eq!(manager.people_to_reward, BTreeMap::new());
    });
}

#[test]
fn test_slashing_for_no_response() {
    new_test_ext().execute_with(|| {
        // Checks the functionality Set Price for Rounds
        // we want to setup a couple of repeater

        assert_ok!(Doracle::populate_treasury(Origin::signed(ALICE)));

        assert_ok!(Doracle::register_repeater(Origin::signed(ALICE), HKO));
        assert_ok!(Doracle::stake(Origin::signed(ALICE), HKO, 100_00));

        assert_ok!(Doracle::register_repeater(Origin::signed(BOB), HKO));
        assert_ok!(Doracle::stake(Origin::signed(BOB), HKO, 100_000));

        assert_ok!(Doracle::register_repeater(Origin::signed(CHARLIE), HKO));
        assert_ok!(Doracle::stake(Origin::signed(CHARLIE), HKO, 100_000));

        // notes
        // implement the ability to add round to update
        // add this function -> set_price_for_round

        // round 1
        // alice, bob, charlie
        let round_id = 1;
        assert_ok!(Doracle::set_price_for_round(
            Origin::signed(ALICE),                 // origin
            HKO,                                   // asset_id
            Price::from_inner(10_000_000_000 * 1), // price
            round_id                               // round_id
        ));

        // let price_holder = Doracle::get_currency_price(HKO).unwrap();

        // assert_eq!(price_holder.round, 1);
        // assert_eq!(price_holder.price, Price::from_inner(10_000_000_000 * 1));

        assert_ok!(Doracle::set_price_for_round(
            Origin::signed(BOB),                   // origin
            HKO,                                   // asset_id
            Price::from_inner(20_000_000_000 * 1), // price
            round_id                               // round_id
        ));

        // assert_eq!(price_holder.round, 1);
        // assert_eq!(price_holder.price, Price::from_inner(10_000_000_000 * 1));

        assert_ok!(Doracle::set_price_for_round(
            Origin::signed(CHARLIE),               // origin
            HKO,                                   // asset_id
            Price::from_inner(10_000_000_000 * 1), // price
            round_id                               // round_id
        ));

        // assert_eq!(price_holder.round, 1);
        // assert_eq!(price_holder.price, Price::from_inner(10_000_000_000 * 1));
        // //
        // round 2
        // alice, bob
        // let round_id = 2;
        assert_ok!(Doracle::set_price_for_round(
            Origin::signed(ALICE),                 // origin
            HKO,                                   // asset_id
            Price::from_inner(10_000_000_000 * 1), // price
            round_id                               // round_id
        ));
        //
        assert_ok!(Doracle::set_price_for_round(
            Origin::signed(BOB),                   // origin
            HKO,                                   // asset_id
            Price::from_inner(10_000_000_000 * 1), // price
            round_id                               // round_id
        ));
        //
        // // round 3 (charlie is slashed for being offline)
        // alice, bob, charlie
        let round_id = 3;
        assert_ok!(Doracle::set_price_for_round(
            Origin::signed(ALICE),                 // origin
            HKO,                                   // asset_id
            Price::from_inner(10_000_000_000 * 1), // price
            round_id                               // round_id
        ));
        //
        assert_ok!(Doracle::set_price_for_round(
            Origin::signed(BOB),                   // origin
            HKO,                                   // asset_id
            Price::from_inner(10_000_000_000 * 1), // price
            round_id                               // round_id
        ));
        //
        assert_ok!(Doracle::set_price_for_round(
            Origin::signed(CHARLIE),               // origin
            HKO,                                   // asset_id
            Price::from_inner(10_000_000_000 * 1), // price
            round_id                               // round_id
        ));

        assert_eq!(0, 1);
    })
}

#[test]
fn test_that_repeater_are_rewarded_after_n_rounds() {
    new_test_ext().execute_with(|| {
        // // in this case we want to pay repeaters X amount at the end
        // // of 3 (n) rounds
        // assert_ok!(Doracle::set_rounds_before_rewards(
        //     Origin::signed(ALICE), // should should update
        //     3 // number of rounds
        // ));

        // we want to setup a couple of repeater
        // assert_ok!(Doracle::register_repeater(Origin::signed(ALICE), HKO));
        // assert_ok!(Doracle::register_repeater(Origin::signed(BOB), HKO));
        // assert_ok!(Doracle::register_repeater(Origin::signed(CHARLIE), HKO));

        assert_ok!(Doracle::stake(Origin::signed(ALICE), HKO, 100_00));
        assert_ok!(Doracle::stake(Origin::signed(BOB), HKO, 100_000));
        assert_ok!(Doracle::stake(Origin::signed(CHARLIE), HKO, 100_000));

        // round 1
        // alice, bob, charlie
        let round_id = 1;
        assert_ok!(Doracle::set_price_for_round(
            Origin::signed(ALICE),                 // origin
            HKO,                                   // asset_id
            Price::from_inner(10_000_000_000 * 1), // price
            round_id                               // round_id
        ));

        assert_ok!(Doracle::set_price_for_round(
            Origin::signed(BOB),                   // origin
            HKO,                                   // asset_id
            Price::from_inner(10_000_000_000 * 1), // price
            round_id                               // round_id
        ));

        assert_ok!(Doracle::set_price_for_round(
            Origin::signed(CHARLIE),               // origin
            HKO,                                   // asset_id
            Price::from_inner(10_000_000_000 * 1), // price
            round_id                               // round_id
        ));

        // round 2
        // alice, bob, charlie
        let round_id = 2;
        assert_ok!(Doracle::set_price_for_round(
            Origin::signed(ALICE),                 // origin
            HKO,                                   // asset_id
            Price::from_inner(10_000_000_000 * 1), // price
            round_id                               // round_id
        ));

        assert_ok!(Doracle::set_price_for_round(
            Origin::signed(BOB),                   // origin
            HKO,                                   // asset_id
            Price::from_inner(10_000_000_000 * 1), // price
            round_id                               // round_id
        ));

        assert_ok!(Doracle::set_price_for_round(
            Origin::signed(CHARLIE),               // origin
            HKO,                                   // asset_id
            Price::from_inner(10_000_000_000 * 1), // price
            round_id                               // round_id
        ));

        // round 3
        // alice, bob, charlie
        let round_id = 3;
        assert_ok!(Doracle::set_price_for_round(
            Origin::signed(ALICE),                 // origin
            HKO,                                   // asset_id
            Price::from_inner(10_000_000_000 * 1), // price
            round_id                               // round_id
        ));

        assert_ok!(Doracle::set_price_for_round(
            Origin::signed(BOB),                   // origin
            HKO,                                   // asset_id
            Price::from_inner(10_000_000_000 * 1), // price
            round_id                               // round_id
        ));

        assert_ok!(Doracle::set_price_for_round(
            Origin::signed(CHARLIE),               // origin
            HKO,                                   // asset_id
            Price::from_inner(10_000_000_000 * 1), // price
            round_id                               // round_id
        ));

        // round 4
        // alice, bob, charlie
        let round_id = 3;
        assert_ok!(Doracle::set_price_for_round(
            Origin::signed(ALICE),                 // origin
            HKO,                                   // asset_id
            Price::from_inner(10_000_000_000 * 1), // price
            round_id                               // round_id
        ));

        assert_ok!(Doracle::set_price_for_round(
            Origin::signed(BOB),                   // origin
            HKO,                                   // asset_id
            Price::from_inner(10_000_000_000 * 1), // price
            round_id                               // round_id
        ));

        assert_ok!(Doracle::set_price_for_round(
            Origin::signed(CHARLIE),               // origin
            HKO,                                   // asset_id
            Price::from_inner(10_000_000_000 * 1), // price
            round_id                               // round_id
        ));

        // check the balances (internal balance [rewards])
        // of A, B and C
        // > the starting balance (because of rewards)

        // let current_balance = Assets::balance(ALICE, HKO);
        // let current_balance = Doracle::pending_reward_balance(ALICE, HKO);
        // let current_balance = Doracle::pending_reward_balance(BOB, HKO);
        // let current_balance = Doracle::pending_reward_balance(CHARLIE, HKO);

        // let bal_diff = current_balance - older_balance;
        // let reward_amount_for_n_rounds = 1000; // <--- some number
        //
        // assert_eq!(bal_diff, reward_amount_for_n_rounds);
    })
}

#[test]
fn test_slashes() {
    new_test_ext().execute_with(|| assert_eq!(1, 2));
}

#[test]
fn test_rewards_after_n_minutes() {
    new_test_ext().execute_with(|| assert_eq!(1, 2));
}

#[test]
fn treasury_increment_decrement() {
    new_test_ext().execute_with(|| assert_eq!(1, 2));
}
