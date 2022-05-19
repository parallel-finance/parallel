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
            Error::<Test>::InvalidUnstaker
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
            Error::<Test>::UnstakeAmoutExceedsStakedBalance
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
fn test_unstake_as_non_repeater() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Doracle::unstake(Origin::signed(ALICE), HKO, 10),
            Error::<Test>::InvalidUnstaker
        );
    });
}

#[test]
fn test_initial_round() {
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
            Origin::signed(ALICE),      // origin
            HKO,                        // asset_id
            Price::from_inner(100_000), // price
            round_id                    // round_id
        ));

        assert_ok!(Doracle::set_price_for_round(
            Origin::signed(BOB),
            HKO,
            Price::from_inner(100_000),
            round_id
        ));

        assert_ok!(Doracle::set_price_for_round(
            Origin::signed(CHARLIE),
            HKO,
            Price::from_inner(100_000),
            round_id
        ));

        let mut expected_participated = BTreeMap::from([(ALICE, 6), (BOB, 6), (CHARLIE, 6)]);

        let mut expected_submitters = BTreeMap::from([
            (ALICE, (FixedU128::from_inner(100_000), 6)),
            (BOB, (FixedU128::from_inner(100_000), 6)),
            (CHARLIE, (FixedU128::from_inner(100_000), 6)),
        ]);

        let mut expected_people_to_reward = BTreeMap::from([(ALICE, 6), (BOB, 6), (CHARLIE, 6)]);

        let manager = Doracle::manager().unwrap();

        assert_eq!(manager.participated, expected_participated);
        assert_eq!(manager.people_to_slash, BTreeMap::new());
        assert_eq!(manager.people_to_reward, expected_people_to_reward);

        let current_round = Doracle::get_current_round(HKO, round_id).unwrap();

        assert_eq!(current_round.agg_price, FixedU128::from_inner(300_000));
        assert_eq!(current_round.mean_price, FixedU128::from_inner(100_000));
        assert_eq!(current_round.submitters, expected_submitters);
        assert_eq!(current_round.submitter_count, 3);

        // Alice tries to submit again in the same round should throw an error
        assert_noop!(
            Doracle::set_price_for_round(
                Origin::signed(ALICE),
                HKO,
                Price::from_inner(100_000),
                round_id
            ),
            Error::<Test>::AccountAlreadySubmittedPrice
        );
    });
}

#[test]
fn test_flow_slashing_after_round_one() {
    new_test_ext().execute_with(|| {
        assert_ok!(Doracle::populate_treasury(Origin::signed(ALICE)));

        assert_ok!(Doracle::register_repeater(Origin::signed(ALICE), HKO));
        assert_ok!(Doracle::stake(Origin::signed(ALICE), HKO, 10_000));

        assert_ok!(Doracle::register_repeater(Origin::signed(BOB), HKO));
        assert_ok!(Doracle::stake(Origin::signed(BOB), HKO, 100_000));

        assert_ok!(Doracle::register_repeater(Origin::signed(CHARLIE), HKO));
        assert_ok!(Doracle::stake(Origin::signed(CHARLIE), HKO, 100_000));

        // First Round -> 3 submitted prices
        let round_id = 1u128;

        assert_ok!(Doracle::set_price_for_round(
            Origin::signed(ALICE),
            HKO,
            Price::from_inner(100_000),
            round_id
        ));

        assert_ok!(Doracle::set_price_for_round(
            Origin::signed(BOB),
            HKO,
            Price::from_inner(100_000),
            round_id
        ));

        assert_ok!(Doracle::set_price_for_round(
            Origin::signed(CHARLIE),
            HKO,
            Price::from_inner(100_000),
            round_id
        ));

        // Check who has participated, needs to slash and rewarded after round 1
        let round_manager = Doracle::manager().unwrap();

        // ALICE, BOB and CHARLIE participated
        assert_eq!(
            round_manager.participated,
            BTreeMap::from([(ALICE, 6), (BOB, 6), (CHARLIE, 6)])
        );
        // No one to slash
        assert_eq!(round_manager.people_to_slash, BTreeMap::from([]));

        // All participants should get rewards
        assert_eq!(
            round_manager.people_to_reward,
            BTreeMap::from([(ALICE, 6), (BOB, 6), (CHARLIE, 6)])
        );

        // Send round stars BOB didn't submit a price
        let round_id = 2u128;

        assert_ok!(Doracle::set_price_for_round(
            Origin::signed(ALICE),
            HKO,
            Price::from_inner(55_000),
            round_id
        ));

        assert_ok!(Doracle::set_price_for_round(
            Origin::signed(CHARLIE),
            HKO,
            Price::from_inner(65_000),
            round_id
        ));

        // Only ALICE and CHARLIE should get rewards
        // Check repeater's balances after round 2
        let rep_alice = Doracle::repeaters(ALICE, HKO).unwrap();
        let rep_bob = Doracle::repeaters(BOB, HKO).unwrap();
        let rep_charlie = Doracle::repeaters(CHARLIE, HKO).unwrap();

        // At the end of second round Alice and Bob gets rewards
        assert_eq!(rep_alice.staked_balance, 10_001);
        assert_eq!(rep_alice.reward, 1);

        assert_eq!(rep_charlie.staked_balance, 100_001);
        assert_eq!(rep_charlie.reward, 1);

        // repeater BOB's staked balance should not changed and should not get any rewards
        assert_eq!(rep_bob.staked_balance, 100_000);
        assert_eq!(rep_bob.reward, 0);

        // At the end of second round treasury value must decreased by 2 HKO since rewarded for 2 accounts
        let treasury = Doracle::get_treasury().unwrap();
        assert_eq!(treasury, 99_999_999_998);

        // 3rd round starts BOB submits a price
        let round_id = 3u128;

        assert_ok!(Doracle::set_price_for_round(
            Origin::signed(ALICE),
            HKO,
            Price::from_inner(80_000),
            round_id
        ));

        assert_ok!(Doracle::set_price_for_round(
            Origin::signed(BOB),
            HKO,
            Price::from_inner(60_000),
            round_id
        ));

        assert_ok!(Doracle::set_price_for_round(
            Origin::signed(CHARLIE),
            HKO,
            Price::from_inner(70_000),
            round_id
        ));

        // Only ALICE and CHARLIE should get rewards
        // Check repeater's balances after round 2
        let rep_alice = Doracle::repeaters(ALICE, HKO).unwrap();
        let rep_bob = Doracle::repeaters(BOB, HKO).unwrap();
        let rep_charlie = Doracle::repeaters(CHARLIE, HKO).unwrap();

        // At the end of second round Alice and Bob gets rewards
        // Bot BOB since not submitted a price in the previous round
        assert_eq!(rep_alice.staked_balance, 10_002);
        assert_eq!(rep_alice.reward, 2);

        assert_eq!(rep_charlie.staked_balance, 100_002);
        assert_eq!(rep_charlie.reward, 2);

        // repeater BOB should get slashed by 1
        assert_eq!(rep_bob.staked_balance, 99_999);
        assert_eq!(rep_bob.reward, 0);

        let treasury = Doracle::get_treasury().unwrap();

        // Treasury at the end of round 2 = 99_999_999_998
        // After round 3 gave rewards 1 each (Alice and Charlie), Treasury =  99_999_999_996
        assert_eq!(treasury, 99_999_999_997);
    });
}

#[test]
fn test_flow_treasury_and_rewards_good_submitters() {
    new_test_ext().execute_with(|| {
        assert_ok!(Doracle::populate_treasury(Origin::signed(ALICE)));

        assert_ok!(Doracle::register_repeater(Origin::signed(ALICE), HKO));
        assert_ok!(Doracle::stake(Origin::signed(ALICE), HKO, 10_000));

        assert_ok!(Doracle::register_repeater(Origin::signed(BOB), HKO));
        assert_ok!(Doracle::stake(Origin::signed(BOB), HKO, 100_000));

        // First Round
        let round_id = 1u128;

        assert_ok!(Doracle::set_price_for_round(
            Origin::signed(ALICE),
            HKO,
            Price::from_inner(100_000),
            round_id
        ));

        assert_ok!(Doracle::set_price_for_round(
            Origin::signed(BOB),
            HKO,
            Price::from_inner(100_000),
            round_id
        ));

        // Check repeater's before the second round
        let rep_alice = Doracle::repeaters(ALICE, HKO).unwrap();
        let rep_bob = Doracle::repeaters(BOB, HKO).unwrap();
        let treasury = Doracle::get_treasury().unwrap();

        // At the end of round one repeaters didn't get any rewards
        assert_eq!(rep_alice.staked_balance, 10_000);
        assert_eq!(rep_alice.reward, 0);
        assert_eq!(rep_alice.last_submission, 0);

        assert_eq!(rep_bob.staked_balance, 100_000);
        assert_eq!(rep_bob.reward, 0);
        assert_eq!(rep_bob.last_submission, 0);

        assert_eq!(treasury, 100_000_000_000);

        // Second round starts
        // Participants form the first round submits in the 2nd round
        let round_id = 2u128;

        assert_ok!(Doracle::set_price_for_round(
            Origin::signed(ALICE),
            HKO,
            Price::from_inner(60_000),
            round_id
        ));

        assert_ok!(Doracle::set_price_for_round(
            Origin::signed(BOB),
            HKO,
            Price::from_inner(70_000),
            round_id
        ));

        let mut expected_participated = BTreeMap::from([(ALICE, 6), (BOB, 6)]);

        let mut expected_submitters = BTreeMap::from([
            (ALICE, (FixedU128::from_inner(60_000), 6)),
            (BOB, (FixedU128::from_inner(70_000), 6)),
        ]);

        let mut expected_people_to_reward = BTreeMap::from([(ALICE, 6), (BOB, 6)]);

        let manager = Doracle::manager().unwrap();

        assert_eq!(manager.participated, expected_participated);
        assert_eq!(manager.people_to_slash, BTreeMap::new());
        assert_eq!(manager.people_to_reward, expected_people_to_reward);

        let current_round = Doracle::get_current_round(HKO, round_id).unwrap();

        assert_eq!(current_round.agg_price, FixedU128::from_inner(130_000));
        assert_eq!(current_round.mean_price, FixedU128::from_inner(65_000));
        assert_eq!(current_round.submitters, expected_submitters);
        assert_eq!(current_round.submitter_count, 2);

        // Check repeater's balances after round 1
        let rep_alice = Doracle::repeaters(ALICE, HKO).unwrap();
        let rep_bob = Doracle::repeaters(BOB, HKO).unwrap();

        // At the end of second round Alice and Bob gets rewards
        assert_eq!(rep_alice.staked_balance, 10_001);
        assert_eq!(rep_alice.reward, 1);

        assert_eq!(rep_bob.staked_balance, 100_001);
        assert_eq!(rep_bob.reward, 1);

        // At the end of second round treasury value must decreased by 2 HKO since rewarded for 2 accounts
        let treasury = Doracle::get_treasury().unwrap();
        assert_eq!(treasury, 99_999_999_998);
    });
}
