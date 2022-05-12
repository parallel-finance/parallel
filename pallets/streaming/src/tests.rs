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

use crate::types::StreamStatus;

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
            12000,
            true,
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
                922337203685477580,
                true,
            ),
            Error::<Test>::InvalidRatePerSecond
        );
    });
}

#[test]
fn cancel_works_without_withdrawal() {
    new_test_ext().execute_with(|| {
        // Alice creates stream 100 DOT to Bob
        let stream_id_0 = NextStreamId::<Test>::get();
        assert_ok!(Streaming::create(
            Origin::signed(ALICE),
            BOB,
            dollar(100),
            DOT,
            6,
            18,
            true,
        ));
        // Get before bob and alice balance
        let before_alice = <Test as Config>::Assets::balance(DOT, &ALICE);
        let before_bob = <Test as Config>::Assets::balance(DOT, &BOB);
        // Time passes for 10 seconds
        TimestampPallet::set_timestamp(6010); // 6000(init) + 10
                                              // Alice cancels existing stream sent to bob
        assert_ok!(Streaming::cancel(Origin::signed(ALICE), stream_id_0));
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
            Streaming::withdraw(Origin::signed(BOB), stream_id_0, 1),
            Error::<Test>::HasFinished
        );

        // If steam is as collateral, it cannot be cancelled
        let stream_id_1 = NextStreamId::<Test>::get();
        assert_ok!(Streaming::create(
            Origin::signed(ALICE),
            BOB,
            dollar(100),
            DOT,
            60,
            180,
            true,
        ));
        let mut stream = Streams::<Test>::get(stream_id_1).unwrap();
        stream.as_collateral().unwrap();
        Streams::<Test>::insert(stream_id_1, stream);
        assert_eq!(
            Streams::<Test>::get(&stream_id_1).unwrap().status,
            StreamStatus::Ongoing {
                as_collateral: true
            },
        );
        assert_err!(
            Streaming::cancel(Origin::signed(ALICE), stream_id_1),
            Error::<Test>::CannotBeCancelled,
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
            16,
            true,
        ));
        // Dave cannot access
        assert_err!(
            Streaming::withdraw(Origin::signed(DAVE), 0, 1),
            Error::<Test>::NotTheRecipient
        );

        // Stream not started
        assert_err!(
            Streaming::withdraw(Origin::signed(BOB), 0, 1),
            Error::<Test>::NotStarted
        );
        // Time passes for 1 second
        assert_eq!(TimestampPallet::now(), 6000);
        // 6000(init) + 2000(ms)
        TimestampPallet::set_timestamp(8000);

        let stream = Streams::<Test>::get(0).unwrap();
        assert_eq!(stream.delta_of(), Ok(2));
        // Bob withdraws some
        assert_ok!(Streaming::withdraw(Origin::signed(BOB), 0, dollar(20)));
        // Bob is received with 20 dollars
        assert_eq!(
            <Test as Config>::Assets::balance(DOT, &BOB) - before_bob,
            dollar(20)
        );
        // balance is updated in the existing stream
        assert_eq!(
            Streams::<Test>::get(0).unwrap().remaining_balance,
            dollar(80),
        );

        TimestampPallet::set_timestamp(16000);
        assert_ok!(Streaming::withdraw(Origin::signed(BOB), 0, dollar(80)));
        assert_eq!(Streams::<Test>::get(&0).unwrap().remaining_balance, 0);
        assert_eq!(
            Streams::<Test>::get(&0).unwrap().status,
            StreamStatus::Completed { cancelled: false },
        );
    });
}

#[test]
fn withdraw_with_slower_rate_works() {
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
            18,
            true,
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
        let stream_id_0 = NextStreamId::<Test>::get();
        assert_ok!(Streaming::create(
            Origin::signed(ALICE),
            BOB,
            dollar(100),
            DOT,
            6,
            11,
            true,
        ));
        // Get before bob and alice balance
        let before_alice = <Test as Config>::Assets::balance(DOT, &ALICE);
        let before_bob = <Test as Config>::Assets::balance(DOT, &BOB);

        // Time passes for 1 second
        TimestampPallet::set_timestamp(7000);
        let mut stream = Streams::<Test>::get(stream_id_0).unwrap();
        assert_eq!(stream.delta_of(), Ok(1));
        // Bob withdraws some
        assert_ok!(Streaming::withdraw(
            Origin::signed(BOB),
            stream_id_0,
            dollar(20)
        ));
        stream = Streams::<Test>::get(stream_id_0).unwrap();
        assert_eq!(stream.balance_of(&BOB).unwrap(), dollar(0));

        // Time passes for 1 second
        TimestampPallet::set_timestamp(8000);
        assert_ok!(Streaming::cancel(Origin::signed(ALICE), stream_id_0));
        // Alice and Bob is received with 60 DOT and 40 DOT respectively as deposit == remaining_balance
        assert_eq!(
            <Test as Config>::Assets::balance(DOT, &ALICE) - before_alice,
            dollar(60)
        );
        assert_eq!(
            <Test as Config>::Assets::balance(DOT, &BOB) - before_bob,
            dollar(40)
        );
        // Bob cannot access to previous stream
        assert_err!(
            Streaming::withdraw(Origin::signed(BOB), 0, 1),
            Error::<Test>::HasFinished,
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
            true,
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

        assert!(
            Streams::<Test>::get(stream_id).unwrap().status
                == StreamStatus::Ongoing {
                    as_collateral: false
                }
        );
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
        assert!(stream.status == StreamStatus::Completed { cancelled: false });

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
            true,
        ));
        TimestampPallet::set_timestamp(10000);
        assert_ok!(Streaming::withdraw(
            Origin::signed(BOB),
            stream_id_0,
            dollar(10)
        ));

        let stream_id_1 = NextStreamId::<Test>::get();
        assert_ok!(Streaming::create(
            Origin::signed(ALICE),
            DAVE,
            dollar(10),
            DOT,
            11,
            20,
            true,
        ));
        TimestampPallet::set_timestamp(15000);
        assert_ok!(Streaming::withdraw(
            Origin::signed(DAVE),
            stream_id_1,
            dollar(2)
        ));
        assert_ok!(Streaming::cancel(Origin::signed(ALICE), stream_id_1));

        // StreamLibrary should contains stream_id_1, stream_id_0
        assert_ok!(Streaming::create(
            Origin::signed(ALICE),
            BOB,
            dollar(10),
            DOT,
            16,
            30,
            true,
        ));

        // storage should be removed due to MaxFinishedStreamsCount = 2
        assert_eq!(
            StreamLibrary::<Test>::get(ALICE, StreamKind::Finish)
                .unwrap()
                .to_vec(),
            vec![1, 0]
        );
        assert_eq!(
            StreamLibrary::<Test>::get(ALICE, StreamKind::Send)
                .unwrap()
                .to_vec(),
            vec![2, 1, 0]
        );
        assert_eq!(
            StreamLibrary::<Test>::get(ALICE, StreamKind::Receive)
                .unwrap_or_default()
                .to_vec(),
            vec![]
        );

        assert_eq!(
            StreamLibrary::<Test>::get(BOB, StreamKind::Finish)
                .unwrap_or_default()
                .to_vec(),
            vec![0]
        );
        assert_eq!(
            StreamLibrary::<Test>::get(BOB, StreamKind::Send)
                .unwrap_or_default()
                .to_vec(),
            vec![]
        );
        assert_eq!(
            StreamLibrary::<Test>::get(BOB, StreamKind::Receive)
                .unwrap()
                .to_vec(),
            vec![2, 0]
        );

        assert_eq!(
            StreamLibrary::<Test>::get(DAVE, StreamKind::Finish)
                .unwrap()
                .to_vec(),
            vec![1]
        );
        assert_eq!(
            StreamLibrary::<Test>::get(DAVE, StreamKind::Send)
                .unwrap_or_default()
                .to_vec(),
            vec![]
        );
        assert_eq!(
            StreamLibrary::<Test>::get(DAVE, StreamKind::Receive)
                .unwrap()
                .to_vec(),
            vec![1]
        );

        // Alice create many streams
        let stream_id_3 = NextStreamId::<Test>::get();
        assert_ok!(Streaming::create(
            Origin::signed(ALICE),
            BOB,
            dollar(10),
            DOT,
            16,
            30,
            true,
        ));
        let stream_id_4 = NextStreamId::<Test>::get();
        assert_ok!(Streaming::create(
            Origin::signed(ALICE),
            BOB,
            dollar(10),
            DOT,
            16,
            30,
            true,
        ));
        assert_ok!(Streaming::cancel(Origin::signed(ALICE), stream_id_3));
        assert_eq!(
            StreamLibrary::<Test>::get(ALICE, StreamKind::Finish)
                .unwrap()
                .to_vec(),
            vec![3, 1]
        );
        assert_eq!(
            StreamLibrary::<Test>::get(ALICE, StreamKind::Send)
                .unwrap()
                .to_vec(),
            vec![4, 3, 2, 1]
        );
        assert_ok!(Streaming::cancel(Origin::signed(ALICE), stream_id_4));
        assert_eq!(
            StreamLibrary::<Test>::get(ALICE, StreamKind::Finish)
                .unwrap()
                .to_vec(),
            vec![4, 3]
        );
        assert_eq!(
            StreamLibrary::<Test>::get(ALICE, StreamKind::Send)
                .unwrap()
                .to_vec(),
            vec![4, 3, 2]
        );
        // BOB create some streams
        let stream_id_5 = NextStreamId::<Test>::get();
        assert_ok!(Streaming::create(
            Origin::signed(BOB),
            DAVE,
            dollar(3),
            DOT,
            16,
            30,
            true,
        ));
        // let stream_id_6= NextStreamId::<Test>::get();
        assert_ok!(Streaming::create(
            Origin::signed(BOB),
            ALICE,
            dollar(3),
            DOT,
            16,
            30,
            true,
        ));
        assert_ok!(Streaming::cancel(Origin::signed(BOB), stream_id_5));
        // assert_ok!(Streaming::cancel(Origin::signed(BOB), stream_id_6));

        // storage should be removed due to MaxFinishedStreamsCount = 2
        assert_eq!(
            StreamLibrary::<Test>::get(ALICE, StreamKind::Finish)
                .unwrap()
                .to_vec(),
            vec![4]
        );
        assert_eq!(
            StreamLibrary::<Test>::get(ALICE, StreamKind::Send)
                .unwrap()
                .to_vec(),
            vec![4, 2]
        );
        assert_eq!(
            StreamLibrary::<Test>::get(ALICE, StreamKind::Receive)
                .unwrap()
                .to_vec(),
            vec![6]
        );

        assert_eq!(
            StreamLibrary::<Test>::get(BOB, StreamKind::Finish)
                .unwrap()
                .to_vec(),
            vec![5, 4]
        );
        assert_eq!(
            StreamLibrary::<Test>::get(BOB, StreamKind::Send)
                .unwrap()
                .to_vec(),
            vec![6, 5]
        );
        assert_eq!(
            StreamLibrary::<Test>::get(BOB, StreamKind::Receive)
                .unwrap()
                .to_vec(),
            vec![4, 2]
        );

        assert_eq!(
            StreamLibrary::<Test>::get(DAVE, StreamKind::Finish)
                .unwrap()
                .to_vec(),
            vec![5]
        );
        assert_eq!(
            StreamLibrary::<Test>::get(DAVE, StreamKind::Send)
                .unwrap_or_default()
                .to_vec(),
            vec![]
        );
        assert_eq!(
            StreamLibrary::<Test>::get(DAVE, StreamKind::Receive)
                .unwrap()
                .to_vec(),
            vec![5]
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
            Streaming::create(Origin::signed(ALICE), BOB, dollar(99), DOT, 6, 10, true),
            Error::<Test>::DepositLowerThanMinimum
        );

        // Check with default option
        assert_err!(
            Streaming::create(Origin::signed(ALICE), BOB, 0, KSM, 6, 10, true),
            Error::<Test>::DepositLowerThanMinimum
        );
    })
}
