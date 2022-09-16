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
use sp_runtime::{traits::Zero, ArithmeticError};

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
            dollar(101),
            DOT,
            6,
            19,
            true,
        ));
        // rate_per_secs: 101 / (19-6) = 7769230769230
        let stream = Streams::<Test>::get(stream_id_0).unwrap();
        assert_eq!(
            stream,
            Stream::new(dollar(101), DOT, 7769230769230, ALICE, BOB, 6, 19, true,)
        );
        // Get before bob and alice balance
        let before_alice = <Test as Config>::Assets::balance(DOT, &ALICE);
        let before_bob = <Test as Config>::Assets::balance(DOT, &BOB);
        // Time passes for 1 seconds
        TimestampPallet::set_timestamp(7000);
        assert_ok!(Streaming::cancel(Origin::signed(ALICE), stream_id_0));
        // Alice and Bob is received with 100 DOT and 0 DOT respectively as deposit == remaining_balance
        assert_eq!(
            <Test as Config>::Assets::balance(DOT, &ALICE) - before_alice,
            93230769230770
        );
        assert_eq!(
            <Test as Config>::Assets::balance(DOT, &BOB) - before_bob,
            7769230769230
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
        assert_eq!(TimestampPallet::now(), 6000);
        // Alice creates stream 101 dollars to Bob
        let stream_id_0 = NextStreamId::<Test>::get();
        assert_ok!(Streaming::create(
            Origin::signed(ALICE),
            BOB,
            dollar(101),
            DOT,
            6,
            19,
            true,
        ));
        // rate_per_secs: 101 / (19-6) = 7769230769230
        let stream = Streams::<Test>::get(stream_id_0).unwrap();
        assert_eq!(
            stream,
            Stream::new(dollar(101), DOT, 7769230769230, ALICE, BOB, 6, 19, true,)
        );

        // Dave cannot access
        assert_err!(
            Streaming::withdraw(Origin::signed(DAVE), 0, 1),
            Error::<Test>::NotTheRecipient
        );

        // passed 11 seconds
        TimestampPallet::set_timestamp(17000);
        assert_eq!(stream.delta_of(), Ok(11));
        // Should be 15538461538460, but add 10(amount) dut to accuracy loss
        assert_eq!(stream.sender_balance().unwrap(), 15538461538470);
        // per_rate_secs * 11 = 85461538461530
        assert_eq!(stream.recipient_balance().unwrap(), 85461538461530);

        // passed 12 seconds
        TimestampPallet::set_timestamp(18000);
        let mut stream = Streams::<Test>::get(stream_id_0).unwrap();
        // delta of should only increase until end_time
        assert_eq!(stream.delta_of(), Ok(12));
        // Should be 7769230769230, but add 10(amount) dut to accuracy loss
        assert_eq!(stream.sender_balance().unwrap(), 7769230769240);
        assert_eq!(stream.recipient_balance().unwrap(), 93230769230760);

        // Bob withdraw all available balance (93230769229759 + 1 + 1000 = 93230769230760)
        assert_ok!(Streaming::withdraw(
            Origin::signed(BOB),
            stream_id_0,
            93230769229759
        ));
        // withdraw a small value should be ok
        assert_ok!(Streaming::withdraw(Origin::signed(BOB), stream_id_0, 1001));

        stream = Streams::<Test>::get(stream_id_0).unwrap();
        assert_eq!(stream.sender_balance().unwrap(), 7769230769240);
        assert_eq!(stream.recipient_balance().unwrap(), 0);

        // passed 14 seconds
        TimestampPallet::set_timestamp(20000);
        stream = Streams::<Test>::get(stream_id_0).unwrap();
        assert_eq!(stream.delta_of(), Ok(13));
        assert_eq!(stream.sender_balance().unwrap(), 0);
        // Reaches the end_time, returned amount should contains the accuracy loss(10)
        // recipient_balance = 7769230769230 + 10
        assert_eq!(stream.recipient_balance().unwrap(), 7769230769240);

        // Bob withdraw remaining_balance
        assert_ok!(Streaming::withdraw(
            Origin::signed(BOB),
            stream_id_0,
            7769230768239
        ));

        assert_ok!(Streaming::withdraw(Origin::signed(BOB), stream_id_0, 1001));

        // Stream is removed as balance goes zero
        assert_eq!(
            <Test as Config>::Assets::balance(DOT, &BOB) - before_bob,
            dollar(101)
        );
    });
}

#[test]
fn withdraw_under_ed_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Streaming::set_minimum_deposit(
            Origin::root(),
            HKO,
            dollar(10)
        ));
        let before_bob = <Test as Config>::Assets::balance(HKO, &BOB);
        assert_eq!(TimestampPallet::now(), 6000);
        // Alice creates stream 101 dollars to Bob
        let stream_id_0 = NextStreamId::<Test>::get();
        assert_ok!(Streaming::create(
            Origin::signed(ALICE),
            BOB,
            dollar(101),
            HKO,
            6,
            19,
            true,
        ));
        // rate_per_secs: 101 / (19-6) = 7769230769230
        let stream = Streams::<Test>::get(stream_id_0).unwrap();
        assert_eq!(
            stream,
            Stream::new(dollar(101), HKO, 7769230769230, ALICE, BOB, 6, 19, true,)
        );

        // Dave cannot access
        assert_err!(
            Streaming::withdraw(Origin::signed(DAVE), 0, 1),
            Error::<Test>::NotTheRecipient
        );

        // passed 11 seconds
        TimestampPallet::set_timestamp(17000);
        assert_eq!(stream.delta_of(), Ok(11));
        // Should be 15538461538460, but add 10(amount) dut to accuracy loss
        assert_eq!(stream.sender_balance().unwrap(), 15538461538470);
        // per_rate_secs * 11 = 85461538461530
        assert_eq!(stream.recipient_balance().unwrap(), 85461538461530);

        // passed 12 seconds
        TimestampPallet::set_timestamp(18000);
        let mut stream = Streams::<Test>::get(stream_id_0).unwrap();
        // delta of should only increase until end_time
        assert_eq!(stream.delta_of(), Ok(12));
        // Should be 7769230769230, but add 10(amount) dut to accuracy loss
        assert_eq!(stream.sender_balance().unwrap(), 7769230769240);
        assert_eq!(stream.recipient_balance().unwrap(), 93230769230760);

        // Bob withdraw balance
        let ed = <Test as Config>::NativeExistentialDeposit::get();
        assert_ok!(Streaming::withdraw(
            Origin::signed(BOB),
            stream_id_0,
            93230769230760 - ed
        ));

        stream = Streams::<Test>::get(stream_id_0).unwrap();
        assert_eq!(stream.sender_balance().unwrap(), 7769230769240);
        assert_eq!(stream.recipient_balance().unwrap(), ed);

        // passed 14 seconds
        TimestampPallet::set_timestamp(20000);
        stream = Streams::<Test>::get(stream_id_0).unwrap();
        assert_eq!(stream.delta_of(), Ok(13));
        assert_eq!(stream.sender_balance().unwrap(), 0);
        // Reaches the end_time, returned amount should contains the accuracy loss(10)
        // recipient_balance = 7769230769230 + 10
        assert_eq!(stream.recipient_balance().unwrap(), 7769230769240 + ed);

        // Bob withdraw remaining_balance
        assert_ok!(Streaming::withdraw(
            Origin::signed(BOB),
            stream_id_0,
            7769230769240 + 1
        ));
        stream = Streams::<Test>::get(stream_id_0).unwrap();
        assert_eq!(stream.recipient_balance().unwrap(), 0);
        // Stream is removed as balance goes zero
        assert_eq!(
            <Test as Config>::Assets::balance(HKO, &BOB) - before_bob,
            dollar(101)
        );
    });
}

#[test]
fn create_ed_and_withdraw_all_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Streaming::set_minimum_deposit(Origin::root(), HKO, 0));
        let before_bob = <Test as Config>::Assets::balance(HKO, &BOB);
        assert_eq!(TimestampPallet::now(), 6000);
        let ed = <Test as Config>::NativeExistentialDeposit::get();
        // Alice creates stream 101 dollars to Bob
        let stream_id_0 = NextStreamId::<Test>::get();
        assert_ok!(Streaming::create(
            Origin::signed(ALICE),
            BOB,
            ed,
            HKO,
            7,
            12,
            true,
        ));
        let stream = Streams::<Test>::get(stream_id_0).unwrap();
        let new_stream = Stream::new(ed, HKO, 2000, ALICE, BOB, 7, 12, true);
        assert_eq!(stream, new_stream);
        TimestampPallet::set_timestamp(8000);
        // Bob withdraw balance
        assert_ok!(Streaming::withdraw(Origin::signed(BOB), stream_id_0, 0));

        let stream = Streams::<Test>::get(stream_id_0).unwrap();
        let mut new_stream = new_stream;
        new_stream.try_deduct(ed).unwrap();
        new_stream.try_complete().unwrap();
        assert_eq!(stream, new_stream);
        assert_err!(stream.sender_balance(), ArithmeticError::Underflow);
        assert_err!(stream.recipient_balance(), ArithmeticError::Underflow);

        // Stream is removed as balance goes zero
        assert_eq!(
            <Test as Config>::Assets::balance(HKO, &BOB) - before_bob,
            ed
        );
    });
}

#[test]
fn cancel_works_with_withdrawal() {
    new_test_ext().execute_with(|| {
        // Alice creates stream 101 dollars to Bob
        let stream_id_0 = NextStreamId::<Test>::get();
        assert_ok!(Streaming::create(
            Origin::signed(ALICE),
            BOB,
            dollar(101),
            DOT,
            6,
            19,
            true,
        ));
        // rate_per_secs: 101 / (19-6) = 7769230769230
        let stream = Streams::<Test>::get(stream_id_0).unwrap();
        assert_eq!(
            stream,
            Stream::new(dollar(101), DOT, 7769230769230, ALICE, BOB, 6, 19, true,)
        );

        // Get before bob and alice balance
        let before_alice = <Test as Config>::Assets::balance(DOT, &ALICE);
        let before_bob = <Test as Config>::Assets::balance(DOT, &BOB);

        // Time passes for 11 second
        TimestampPallet::set_timestamp(17000);
        let mut stream = Streams::<Test>::get(stream_id_0).unwrap();
        assert_eq!(stream.delta_of(), Ok(11));
        // Bob withdraws some
        assert_ok!(Streaming::withdraw(
            Origin::signed(BOB),
            stream_id_0,
            dollar(20)
        ));
        stream = Streams::<Test>::get(stream_id_0).unwrap();

        // Should be 15538461538460, but lost 10(amount) dut to accuracy loss
        assert_eq!(stream.sender_balance().unwrap(), 15538461538470);
        // per_rate_secs * 11 - dollar(20) = 65461538461530
        assert_eq!(stream.recipient_balance().unwrap(), 65461538461530);

        // Time passes for 1 second
        TimestampPallet::set_timestamp(18000);

        // seconds * per_rate_sec + Accuracy loss
        // 1 * 7769230769230 + 10 = 7769230769240
        assert_eq!(stream.sender_balance().unwrap(), 7769230769240);
        assert_eq!(stream.recipient_balance().unwrap(), 73230769230760);

        assert_ok!(Streaming::cancel(Origin::signed(ALICE), stream_id_0));
        // Cannot cancel multiple times
        assert_err!(
            Streaming::cancel(Origin::signed(ALICE), stream_id_0),
            Error::<Test>::HasFinished
        );
        stream = Streams::<Test>::get(stream_id_0).unwrap();
        assert_eq!(stream.remaining_balance, 7769230769240);
        assert_eq!(stream.has_finished(), true);
        assert_eq!(stream.recipient_balance().unwrap(), 0);

        // Alice and Bob is received DOT respectively
        assert_eq!(
            <Test as Config>::Assets::balance(DOT, &ALICE) - before_alice,
            7769230769240
        );
        assert_eq!(
            <Test as Config>::Assets::balance(DOT, &BOB) - before_bob,
            73230769230760 + dollar(20)
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

        // Asset is not supported to create stream
        Assets::force_create(Origin::root(), USDT, ALICE, true, 1).unwrap();
        Assets::mint(Origin::signed(ALICE), USDT, ALICE, dollar(10000)).unwrap();
        assert_err!(
            Streaming::create(Origin::signed(ALICE), BOB, dollar(99), USDT, 6, 10, true),
            Error::<Test>::InvalidAssetId
        );
    })
}
