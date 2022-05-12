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

        // NOTE: we should have a cool down period
        // this should be invaild
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
        // TODO: Flip This -> Stake and register~
        // NOTE: we might want to flip this? stake then register

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

// NOTE - unclear
#[test]
fn test_manager() {
    // Repeater stakes
    // Manager's coffer increased at each round
    new_test_ext().execute_with(|| {
        // assert_ok!(Doracle::register_repeater(Origin::signed(ALICE)));
        //
        // assert_ok!(Doracle::stake(Origin::signed(ALICE), HKO, 100_000));
        // let managers_coffer = Doracle::get_round_manager::get().unwrap();
        // assert_eq!(managers_coffer.balance, 100_000);
        //
        // assert_ok!(Doracle::stake(Origin::signed(ALICE), HKO, 100_000));
        // let managers_coffer = Doracle::get_round_manager::get().unwrap();
        // assert_eq!(managers_coffer.balance, 200_000);
    });
}

// NOTE - reward should be based on actions
#[test]
fn test_rewards() {
    /*
    Test Rewards
    Current formula

    reward = (repeater.staked_balance / current_timestamp_in_seconds as unix time) / 100_000_000

    Within the time if a repeater has more staked balance it can get a higher reward
    TODO: We may need to change the final divisor
    -------------------------------------------------------------------------
    | staked_balance                        | reward amount 10 to the pow   |
    -------------------------------------------------------------------------
    | Under 100_000_000_0                   | No                            |
    | 100_000_000_0 -  100_000_000_00       | 1                             |
    | 100_000_000_00 - 100_000_000_000      | 2                             |
    | 100_000_000_000 - above               | 3                             |
    -------------------------------------------------------------------------
    * Please note that the time is also increasing
    * More the stake balance more the reward with time
    */
    new_test_ext().execute_with(|| {
        assert_ok!(Doracle::register_repeater(Origin::signed(ALICE)));

        assert_ok!(Doracle::stake(Origin::signed(ALICE), HKO, 100_00));
        let repeater = Doracle::repeaters(ALICE).unwrap();
        assert_eq!(repeater.reward, 0);

        assert_ok!(Doracle::stake(Origin::signed(ALICE), HKO, 100_000_000_0));
        let repeater = Doracle::repeaters(ALICE).unwrap();
        assert_eq!(repeater.reward, 1);

        assert_ok!(Doracle::stake(Origin::signed(ALICE), HKO, 100_000_000_00));
        let repeater = Doracle::repeaters(ALICE).unwrap();
        assert_eq!(repeater.reward, 19);

        assert_ok!(Doracle::stake(Origin::signed(ALICE), HKO, 100_000_000_000));
        let repeater = Doracle::repeaters(ALICE).unwrap();
        assert_eq!(repeater.reward, 204);
    });
}

#[test]
fn test_slashing_for_no_response() {
    new_test_ext().execute_with(|| {
        // Checks the functionality Set Price for Rounds
        // we want to setup a couple of repeater
        assert_ok!(Doracle::register_repeater(Origin::signed(ALICE)));
        assert_ok!(Doracle::stake(Origin::signed(ALICE), HKO, 100_00));

        assert_ok!(Doracle::register_repeater(Origin::signed(BOB)));
        assert_ok!(Doracle::stake(Origin::signed(BOB), HKO, 100_000));

        assert_ok!(Doracle::register_repeater(Origin::signed(CHARLIE)));
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
        // assert_ok!(Doracle::set_price_for_round(
        //     Origin::signed(ALICE),                 // origin
        //     HKO,                                   // asset_id
        //     Price::from_inner(10_000_000_000 * 1), // price
        //     round_id                               // round_id
        // ));
        //
        // assert_ok!(Doracle::set_price_for_round(
        //     Origin::signed(BOB),                   // origin
        //     HKO,                                   // asset_id
        //     Price::from_inner(10_000_000_000 * 1), // price
        //     round_id                               // round_id
        // ));
        //
        // // round 3 (charlie is slashed for being offline)
        // // alice, bob, charlie
        // let round_id = 3;
        // assert_ok!(Doracle::set_price_for_round(
        //     Origin::signed(ALICE),                 // origin
        //     HKO,                                   // asset_id
        //     Price::from_inner(10_000_000_000 * 1), // price
        //     round_id                               // round_id
        // ));
        //
        // assert_ok!(Doracle::set_price_for_round(
        //     Origin::signed(BOB),                   // origin
        //     HKO,                                   // asset_id
        //     Price::from_inner(10_000_000_000 * 1), // price
        //     round_id                               // round_id
        // ));
        //
        // assert_ok!(Doracle::set_price_for_round(
        //     Origin::signed(CHARLIE),               // origin
        //     HKO,                                   // asset_id
        //     Price::from_inner(10_000_000_000 * 1), // price
        //     round_id                               // round_id
        // ));
        //
        // assert_eq!(0, 1);
    })
}

// #[test]
// fn test_that_repeater_are_rewarded_after_n_rounds() {
//     new_test_ext().execute_with(|| {

//         // in this case we want to pay repeaters X amount at the end
//         // of 3 (n) rounds
//         assert_ok!(Doracle::set_rounds_before_rewards(
//             Origin::signed(ALICE), // should should update
//             3 // number of rounds
//         ));

//         // we want to setup a couple of repeater
//         assert_ok!(Doracle::register_repeater(Origin::signed(ALICE)));
//         assert_ok!(Doracle::stake(Origin::signed(ALICE), HKO, 100_00));

//         assert_ok!(Doracle::register_repeater(Origin::signed(BOB)));
//         assert_ok!(Doracle::stake(Origin::signed(BOB), HKO, 100_000));

//         assert_ok!(Doracle::register_repeater(Origin::signed(CHARLIE)));
//         assert_ok!(Doracle::stake(Origin::signed(CHARLIE), HKO, 100_000));

//         // round 1
//         // alice, bob, charlie
//         let round_id = 1;
//         assert_ok!(Doracle::set_price_for_round(
//             Origin::signed(ALICE),               // origin
//             HKO,                                   // asset_id
//             Price::from_inner(10_000_000_000 * 1),  // price
//             round_id // round_id
//         ));

//         assert_ok!(Doracle::set_price_for_round(
//             Origin::signed(BOB),               // origin
//             HKO,                                   // asset_id
//             Price::from_inner(10_000_000_000 * 1),  // price
//             round_id // round_id
//         ));

//         assert_ok!(Doracle::set_price_for_round(
//             Origin::signed(CHARLIE),               // origin
//             HKO,                                   // asset_id
//             Price::from_inner(10_000_000_000 * 1),  // price
//             round_id // round_id
//         ));

//         // round 2
//         // alice, bob, charlie
//         let round_id = 2;
//         assert_ok!(Doracle::set_price_for_round(
//             Origin::signed(ALICE),               // origin
//             HKO,                                   // asset_id
//             Price::from_inner(10_000_000_000 * 1),  // price
//             round_id // round_id
//         ));

//         assert_ok!(Doracle::set_price_for_round(
//             Origin::signed(BOB),               // origin
//             HKO,                                   // asset_id
//             Price::from_inner(10_000_000_000 * 1),  // price
//             round_id // round_id
//         ));

//         assert_ok!(Doracle::set_price_for_round(
//             Origin::signed(CHARLIE),               // origin
//             HKO,                                   // asset_id
//             Price::from_inner(10_000_000_000 * 1),  // price
//             round_id // round_id
//         ));

//         // round 3
//         // alice, bob, charlie
//         let round_id = 3;
//         assert_ok!(Doracle::set_price_for_round(
//             Origin::signed(ALICE),               // origin
//             HKO,                                   // asset_id
//             Price::from_inner(10_000_000_000 * 1),  // price
//             round_id // round_id
//         ));

//         assert_ok!(Doracle::set_price_for_round(
//             Origin::signed(BOB),               // origin
//             HKO,                                   // asset_id
//             Price::from_inner(10_000_000_000 * 1),  // price
//             round_id // round_id
//         ));

//         assert_ok!(Doracle::set_price_for_round(
//             Origin::signed(CHARLIE),               // origin
//             HKO,                                   // asset_id
//             Price::from_inner(10_000_000_000 * 1),  // price
//             round_id // round_id
//         ));

//         // round 4
//         // alice, bob, charlie
//         let round_id = 3;
//         assert_ok!(Doracle::set_price_for_round(
//             Origin::signed(ALICE),               // origin
//             HKO,                                   // asset_id
//             Price::from_inner(10_000_000_000 * 1),  // price
//             round_id // round_id
//         ));

//         assert_ok!(Doracle::set_price_for_round(
//             Origin::signed(BOB),               // origin
//             HKO,                                   // asset_id
//             Price::from_inner(10_000_000_000 * 1),  // price
//             round_id // round_id
//         ));

//         assert_ok!(Doracle::set_price_for_round(
//             Origin::signed(CHARLIE),               // origin
//             HKO,                                   // asset_id
//             Price::from_inner(10_000_000_000 * 1),  // price
//             round_id // round_id
//         ));

//         // check the balances (internal balance [rewards])
//         // of A, B and C
//         // > the starting balance (because of rewards)

//         // let current_balance = Assets::balance(ALICE, HKO);
//         let current_balance = Doracle::pending_reward_balance(ALICE, HKO);
//         let current_balance = Doracle::pending_reward_balance(BOB, HKO);
//         let current_balance = Doracle::pending_reward_balance(CHARLIE, HKO);

//         let bal_diff = current_balance - older_balance;
//         let reward_amount_for_n_rounds = 1000; // <--- some number

//         assert_eq!(bal_diff, reward_amount_for_n_rounds);
//     })
// }

// // // dave cant set a price because he is not a repeater
// // assert_noop!(
// //     Doracle::set_price(
// //         Origin::signed(DAVE),                  // origin
// //         HKO,                                   // asset_id
// //         Price::from_inner(10_000_000_000 * 1)  // price

// //         // NOTE: prices should be set for a round?
// //     ),
// //     Error::<Test>::StakedAmountIsLessThanMinStakeAmount
// // );
