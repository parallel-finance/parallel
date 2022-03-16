use crate::mock::*;
use frame_support::assert_ok;
use frame_system::RawOrigin;

#[test]
fn create_stream_should_work() {
    new_test_ext().execute_with(|| {
        let recipient = BOB;
        let deposit = 1_000_000;
        let currency_id = 1;
        let rate_per_sec = 1;
        let start_time = 6;
        let stop_time = 1647465601;

        assert_ok!(Payroll::create_stream(
            RawOrigin::Signed(ALICE).into(),
            recipient,
            deposit,
            currency_id,
            rate_per_sec,
            start_time,
            stop_time,
        ));
    })
}

#[test]
fn create_and_cancel_stream_should_work() {
    new_test_ext().execute_with(|| {
        let recipient = BOB;
        let deposit = 1_000_000;
        let currency_id = 1;
        let rate_per_sec = 1;
        let start_time = 6;
        let stop_time = 1647465601;

        assert_ok!(Payroll::create_stream(
            RawOrigin::Signed(ALICE).into(),
            recipient,
            deposit,
            currency_id,
            rate_per_sec,
            start_time,
            stop_time,
        ));

        let stream = Payroll::get_stream(0).unwrap();
        assert_eq!(stream.deposit, deposit);
        assert_eq!(stream.start_time, start_time);

        assert_ok!(Payroll::cancel_stream(RawOrigin::Signed(ALICE).into(), 0,));

        let stream = Payroll::get_stream(0);
        assert_eq!(stream, None);
    })
}

#[test]
fn create_and_withdraw_from_stream_should_work() {
    new_test_ext().execute_with(|| {
        let recipient = BOB;
        let deposit = 1_000_000;
        let currency_id = 1;
        let rate_per_sec = 1;
        let start_time = 6;
        let stop_time = 1647465601;

        assert_ok!(Payroll::create_stream(
            RawOrigin::Signed(ALICE).into(),
            recipient,
            deposit,
            currency_id,
            rate_per_sec,
            start_time,
            stop_time,
        ));

        let stream = Payroll::get_stream(0).unwrap();
        assert_eq!(stream.deposit, deposit);
        assert_eq!(stream.start_time, start_time);

        // time is 6 seconds
        run_to_block(100);
        // time is 600 seconds

        let stream = Payroll::get_stream(0).unwrap();
        println!("{:?}", stream);

        assert_ok!(Payroll::withdraw_from_stream(
            RawOrigin::Signed(BOB).into(),
            0,
            (600 - start_time).into(),
        ));
    })
}
