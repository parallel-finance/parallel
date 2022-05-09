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
use frame_support::{assert_err, assert_ok};
use mock::*;
use sp_runtime::traits::Zero;

#[test]
fn create_works() {
    new_test_ext().execute_with(|| {
        // Alice creates stream 100 DOT to Bob
        assert_ok!(Streaming::create(
            Origin::signed(ALICE),
            BOB,
            dollar(100),
            DOT,
            6000,
            12000
        ));
        // Dave cannot access
        assert_err!(
            Streaming::withdraw(Origin::signed(DAVE), 0, 1),
            Error::<Test>::NotTheRecipient
        );

        // Alice creates stream 100 DOT to Bob
        assert_err!(
            Streaming::create(
                Origin::signed(ALICE),
                BOB,
                dollar(100),
                DOT,
                6,
                922337203685477580
            ),
            Error::<Test>::InvalidRatePerSecond
        );
    });
}

#[test]
fn cancel_works_without_withdrawal() {
    new_test_ext().execute_with(|| {
        // Alice creates stream 100 DOT to Bob
        assert_ok!(Streaming::create(
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
        assert_ok!(Streaming::cancel(Origin::signed(ALICE), 0));
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
            Streaming::withdraw(Origin::signed(BOB), 0, 1),
            Error::<Test>::StreamHasFinished
        );
    });
}

#[test]
fn withdraw_works() {
    new_test_ext().execute_with(|| {
        let before_bob = <Test as Config>::Assets::balance(DOT, &BOB);
        // Alice creates stream 100 DOT to Bob
        assert_ok!(Streaming::create(
            Origin::signed(ALICE),
            BOB,
            dollar(100),
            DOT,
            6,
            18
        ));
        // Dave cannot access
        assert_err!(
            Streaming::withdraw(Origin::signed(DAVE), 0, 1),
            Error::<Test>::NotTheRecipient
        );

        // Time passes for 1 second
        assert_eq!(TimestampPallet::now(), 6000);
        // 6000(init) + 1000(ms)
        TimestampPallet::set_timestamp(7000);

        let stream = Streams::<Test>::get(0).unwrap();
        assert_eq!(stream.delta_of(), Ok(1));
        // Bob withdraws some
        assert_ok!(Streaming::withdraw(Origin::signed(BOB), 0, dollar(1)));
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
        assert_ok!(Streaming::withdraw(Origin::signed(BOB), 0, dollar(99)));
        assert_eq!(Streams::<Test>::get(&0).unwrap().remaining_balance, 0);
        assert_eq!(
            Streams::<Test>::get(&0).unwrap().status,
            StreamStatus::Completed
        );
    });
}

#[test]
fn withdraw_fwith_slower_rate_works() {
    new_test_ext().execute_with(|| {
        let before_bob = <Test as Config>::Assets::balance(DOT, &BOB);
        // Alice creates stream 100 DOT to Bob
        assert_eq!(TimestampPallet::now(), 6000);
        assert_ok!(Streaming::create(
            Origin::signed(ALICE),
            BOB,
            dollar(100),
            DOT,
            6,
            18
        ));
        // Dave cannot access
        assert_err!(
            Streaming::withdraw(Origin::signed(DAVE), 0, 1),
            Error::<Test>::NotTheRecipient
        );

        // passed 12 seconds
        TimestampPallet::set_timestamp(18000);

        let stream = Streams::<Test>::get(0).unwrap();
        // delta of should only increase until end_time
        assert_eq!(stream.delta_of(), Ok(12));
        // Bob withdraws some
        assert_ok!(Streaming::withdraw(Origin::signed(BOB), 0, dollar(100)));
        // Bob is received with 100 DOT as stream end time has passed
        // Stream is removed as balance goes zero
        assert_eq!(
            <Test as Config>::Assets::balance(DOT, &BOB) - before_bob,
            dollar(100)
        );
    });
}

#[test]
fn cancel_works_with_withdrawal() {
    new_test_ext().execute_with(|| {
        // Alice creates stream 100 DOT to Bob
        assert_ok!(Streaming::create(
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
        assert_eq!(stream.delta_of(), Ok(1));
        // Bob withdraws some
        assert_ok!(Streaming::withdraw(Origin::signed(BOB), 0, dollar(25)));
        stream = Streams::<Test>::get(0).unwrap();
        assert_eq!(stream.balance_of(&BOB).unwrap(), dollar(0));
        // Time passes for 1 second
        TimestampPallet::set_timestamp(8000); // 7000(before) + 1000(second)
                                              // Alice cancels existing stream sent to bob
        assert_ok!(Streaming::cancel(Origin::signed(ALICE), 0));
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
            Streaming::withdraw(Origin::signed(BOB), 0, 1),
            Error::<Test>::StreamHasFinished,
        );
    });
}

#[test]
fn streams_library_should_works() {
    new_test_ext().execute_with(|| {
        let stream_id = NextStreamId::<Test>::get();
        assert_ok!(Streaming::create(
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
        assert_ok!(Streaming::withdraw(
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
fn max_finished_streams_count_should_work() {
    new_test_ext().execute_with(|| {
        let stream_id_0 = NextStreamId::<Test>::get();
        assert_ok!(Streaming::create(
            Origin::signed(ALICE),
            BOB,
            dollar(10),
            DOT,
            6,
            10,
        ));
        TimestampPallet::set_timestamp(10000);
        assert_ok!(Streaming::withdraw(
            Origin::signed(BOB),
            stream_id_0,
            dollar(10)
        ));

        // StreamLibrary should contains stream_id_0
        assert_ok!(StreamLibrary::<Test>::get(ALICE, StreamKind::Finish)
            .unwrap()
            .binary_search(&stream_id_0));

        let stream_id_1 = NextStreamId::<Test>::get();
        assert_ok!(Streaming::create(
            Origin::signed(ALICE),
            BOB,
            dollar(10),
            DOT,
            11,
            20,
        ));
        TimestampPallet::set_timestamp(15000);
        assert_ok!(Streaming::withdraw(
            Origin::signed(BOB),
            stream_id_1,
            dollar(2)
        ));
        assert_ok!(Streaming::cancel(Origin::signed(ALICE), stream_id_1));

        // StreamLibrary should contains stream_id_1
        assert_ok!(StreamLibrary::<Test>::get(ALICE, StreamKind::Finish)
            .unwrap()
            .binary_search(&stream_id_1));

        // storage should be removed due to MaxFinishedStreamsCount = 2
        assert_ok!(Streaming::create(
            Origin::signed(ALICE),
            BOB,
            dollar(10),
            DOT,
            16,
            30,
        ));
        assert_eq!(
            StreamLibrary::<Test>::get(ALICE, StreamKind::Finish)
                .unwrap()
                .contains(&stream_id_0),
            false
        );
        assert_eq!(
            StreamLibrary::<Test>::get(BOB, StreamKind::Finish)
                .unwrap()
                .contains(&stream_id_0),
            false
        );

        assert_eq!(
            StreamLibrary::<Test>::get(ALICE, StreamKind::Send)
                .unwrap()
                .contains(&stream_id_0),
            false
        );
        assert_eq!(
            StreamLibrary::<Test>::get(BOB, StreamKind::Receive)
                .unwrap()
                .contains(&stream_id_0),
            false
        );

        assert_eq!(
            StreamLibrary::<Test>::get(ALICE, StreamKind::Finish)
                .unwrap()
                .contains(&stream_id_1),
            true
        );
        assert_eq!(
            StreamLibrary::<Test>::get(BOB, StreamKind::Finish)
                .unwrap()
                .contains(&stream_id_1),
            true
        );
        assert_eq!(
            StreamLibrary::<Test>::get(ALICE, StreamKind::Send)
                .unwrap()
                .contains(&stream_id_1),
            true
        );
        assert_eq!(
            StreamLibrary::<Test>::get(BOB, StreamKind::Receive)
                .unwrap()
                .contains(&stream_id_1),
            true
        );
    })
}

#[test]
fn create_with_minimum_deposit_works() {
    new_test_ext().execute_with(|| {
        // Set minimum deposit for DOT
        assert_ok!(Streaming::set_minimum_deposit(
            Origin::root(),
            DOT,
            dollar(100)
        ));

        // Alice creates stream 100 DOT to Bob, which is equal to minimum deposit
        assert_err!(
            Streaming::create(Origin::signed(ALICE), BOB, dollar(99), DOT, 6, 10),
            Error::<Test>::DepositLowerThanMinimum
        );

        // Check with default option
        assert_err!(
            Streaming::create(Origin::signed(ALICE), BOB, 0, KSM, 6, 10),
            Error::<Test>::DepositLowerThanMinimum
        );
    })
}
