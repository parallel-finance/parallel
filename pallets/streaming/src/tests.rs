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

use frame_support::{assert_err, assert_ok};

#[test]
fn create_stream_works() {
    new_test_ext().execute_with(|| {
        // Alice creates stream 100 DOT to Bob
        assert_ok!(Streaming::create_stream(
            Origin::signed(ALICE),
            BOB,
            dollar(100),
            DOT,
            6000,
            12000
        ));
        // Dave cannot access
        assert_err!(
            Streaming::withdraw_from_stream(Origin::signed(DAVE), 0, 1),
            Error::<Test>::NotTheRecipient
        );
    });
}

#[test]
fn cancel_stream_works_without_withdrawal() {
    new_test_ext().execute_with(|| {
        // Alice creates stream 100 DOT to Bob
        assert_ok!(Streaming::create_stream(
            Origin::signed(ALICE),
            BOB,
            dollar(100),
            DOT,
            6,
            18
        ));
        // Get before bob and alice balance
        let before_alice = <Test as Config>::Assets::balance(DOT, &ALICE);
        let before_bob = <Test as Config>::Assets::balance(DOT, &BOB);
        // Time passes for 10 seconds
        TimestampPallet::set_timestamp(6010); // 6000(init) + 10
                                              // Alice cancels existing stream sent to bob
        assert_ok!(Streaming::cancel_stream(Origin::signed(ALICE), 0));
        // Alice and Bob is received with 100 DOT and 0 DOT respectively as deposit == remaining_balance
        assert_eq!(
            <Test as Config>::Assets::balance(DOT, &ALICE) - before_alice,
            dollar(100)
        );
        assert_eq!(
            <Test as Config>::Assets::balance(DOT, &BOB) - before_bob,
            dollar(0)
        );
        // Bob cannot access to previous stream
        assert_err!(
            Streaming::withdraw_from_stream(Origin::signed(BOB), 0, 1),
            Error::<Test>::StreamCompleted
        );
    });
}

#[test]
fn withdraw_from_stream_works() {
    new_test_ext().execute_with(|| {
        let before_bob = <Test as Config>::Assets::balance(DOT, &BOB);
        // Alice creates stream 100 DOT to Bob
        assert_ok!(Streaming::create_stream(
            Origin::signed(ALICE),
            BOB,
            dollar(100),
            DOT,
            6,
            18
        ));
        // Dave cannot access
        assert_err!(
            Streaming::withdraw_from_stream(Origin::signed(DAVE), 0, 1),
            Error::<Test>::NotTheRecipient
        );

        // Time passes for 1 second
        assert_eq!(TimestampPallet::now(), 6000);
        // 6000(init) + 1000(ms)
        TimestampPallet::set_timestamp(7000);

        let stream = Streams::<Test>::get(0).unwrap();
        assert_eq!(Streaming::delta_of(&stream), Ok(1));
        // Bob withdraws some
        assert_ok!(Streaming::withdraw_from_stream(
            Origin::signed(BOB),
            0,
            dollar(1)
        ));
        // Bob is received with 100 DOT
        assert_eq!(
            <Test as Config>::Assets::balance(DOT, &BOB) - before_bob,
            dollar(1)
        );
        // balance is updated in the existing stream
        assert_eq!(
            Streams::<Test>::get(0).unwrap().remaining_balance,
            dollar(99),
        );

        TimestampPallet::set_timestamp(18000);
        assert_ok!(Streaming::withdraw_from_stream(
            Origin::signed(BOB),
            0,
            dollar(99)
        ));
        assert_eq!(Streams::<Test>::get(&0).unwrap().remaining_balance, 0);
        assert_eq!(
            Streams::<Test>::get(&0).unwrap().status,
            StreamStatus::Completed
        );
    });
}

#[test]
fn withdraw_from_with_slower_rate_works() {
    new_test_ext().execute_with(|| {
        let before_bob = <Test as Config>::Assets::balance(DOT, &BOB);
        // Alice creates stream 100 DOT to Bob
        assert_ok!(Streaming::create_stream(
            Origin::signed(ALICE),
            BOB,
            dollar(100),
            DOT,
            6,
            18
        ));
        // Dave cannot access
        assert_err!(
            Streaming::withdraw_from_stream(Origin::signed(DAVE), 0, 1),
            Error::<Test>::NotTheRecipient
        );
        // Time passes after stop time
        TimestampPallet::set_timestamp(20000); // after stop timestamp in milliseconds
                                               // check if 12 second has passed
        let stream = Streams::<Test>::get(0).unwrap();
        // delta of should only increase until stop_time
        assert_eq!(Streaming::delta_of(&stream), Ok(12));
        // Bob withdraws some
        assert_ok!(Streaming::withdraw_from_stream(
            Origin::signed(BOB),
            0,
            dollar(100)
        ));
        // Bob is received with 100 DOT as stream stop time has passed
        // Stream is removed as balance goes zero
        assert_eq!(
            <Test as Config>::Assets::balance(DOT, &BOB) - before_bob,
            dollar(100)
        );
    });
}

#[test]
fn cancel_stream_works_with_withdrawal() {
    new_test_ext().execute_with(|| {
        // Alice creates stream 100 DOT to Bob
        assert_ok!(Streaming::create_stream(
            Origin::signed(ALICE),
            BOB,
            dollar(100),
            DOT,
            6,
            10
        ));
        // Get before bob and alice balance
        let before_alice = <Test as Config>::Assets::balance(DOT, &ALICE);
        let before_bob = <Test as Config>::Assets::balance(DOT, &BOB);
        // Time passes for 1 second
        TimestampPallet::set_timestamp(7000); // 6000(init) + 1000(second)
                                              // check if 1 second has passed
        let mut stream = Streams::<Test>::get(0).unwrap();
        assert_eq!(Streaming::delta_of(&stream), Ok(1));
        // Bob withdraws some
        assert_ok!(Streaming::withdraw_from_stream(
            Origin::signed(BOB),
            0,
            dollar(25)
        ));
        stream = Streams::<Test>::get(0).unwrap();
        assert_eq!(Streaming::balance_of(&stream, &BOB).unwrap(), dollar(0));
        // Time passes for 1 second
        TimestampPallet::set_timestamp(8000); // 7000(before) + 1000(second)
                                              // Alice cancels existing stream sent to bob
        assert_ok!(Streaming::cancel_stream(Origin::signed(ALICE), 0));
        // Alice and Bob is received with 98 DOT and 2 DOT respectively as deposit == remaining_balance
        assert_eq!(
            <Test as Config>::Assets::balance(DOT, &ALICE) - before_alice,
            dollar(50)
        );
        assert_eq!(
            <Test as Config>::Assets::balance(DOT, &BOB) - before_bob,
            dollar(50)
        );
        // Bob cannot access to previous stream
        assert_err!(
            Streaming::withdraw_from_stream(Origin::signed(BOB), 0, 1),
            Error::<Test>::StreamCompleted,
        );
    });
}

#[test]
fn streams_library_should_works() {
    new_test_ext().execute_with(|| {
        let stream_id = NextStreamId::<Test>::get();
        assert_ok!(Streaming::create_stream(
            Origin::signed(ALICE),
            BOB,
            dollar(100),
            DOT,
            6,
            10,
        ));

        // StreamLibrary should contains stream_id = 0
        assert_ok!(StreamLibrary::<Test>::get(ALICE, StreamKind::Send)
            .unwrap()
            .binary_search(&stream_id));
        assert_ok!(StreamLibrary::<Test>::get(BOB, StreamKind::Receive)
            .unwrap()
            .binary_search(&stream_id));

        // 6000(init) + 4000(ms)
        TimestampPallet::set_timestamp(10000);

        assert!(Streams::<Test>::get(stream_id).unwrap().status == StreamStatus::Ongoing);
        assert_eq!(
            Streams::<Test>::get(stream_id).unwrap().remaining_balance,
            dollar(100),
        );
        assert_ok!(Streaming::withdraw_from_stream(
            Origin::signed(BOB),
            stream_id,
            dollar(100)
        ));

        let stream = Streams::<Test>::get(stream_id).unwrap();
        assert!(stream.remaining_balance == Zero::zero());
        assert!(stream.status == StreamStatus::Completed);

        // storage shouldn't be removed though stream completed
        assert_ok!(StreamLibrary::<Test>::get(ALICE, StreamKind::Send)
            .unwrap()
            .binary_search(&stream_id));
        assert_ok!(StreamLibrary::<Test>::get(BOB, StreamKind::Receive)
            .unwrap()
            .binary_search(&stream_id));
    })
}

#[test]
fn create_stream_with_minimum_deposit_works() {
    new_test_ext().execute_with(|| {
        // Set minimum deposit for DOT
        assert_ok!(Streaming::set_minimum_deposit(
            Origin::root(),
            DOT,
            dollar(100)
        ));

        // Alice creates stream 100 DOT to Bob, which is equal to minimum deposit
        assert_err!(
            Streaming::create_stream(Origin::signed(ALICE), BOB, dollar(99), DOT, 6, 10),
            Error::<Test>::DepositLowerThanMinimum
        );

        // Check with default option
        assert_err!(
            Streaming::create_stream(Origin::signed(ALICE), BOB, 0, KSM, 6, 10),
            Error::<Test>::DepositLowerThanMinimum
        );
    })
}
