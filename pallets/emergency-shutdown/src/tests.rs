use crate::mock::*;
use frame_support::{assert_noop, assert_ok, dispatch::*};

#[test]
fn toggle_shutdown_flag_works() {
    new_test_ext().execute_with(|| {
        assert_eq!(EmergencyShutdown::is_shut_down_flag(), false);
        assert_ok!(EmergencyShutdown::toggle_shutdown_flag(Origin::root()));
        assert_eq!(EmergencyShutdown::is_shut_down_flag(), true);
        assert_ok!(EmergencyShutdown::toggle_shutdown_flag(Origin::root()));
        assert_eq!(EmergencyShutdown::is_shut_down_flag(), false);
        assert_ok!(EmergencyShutdown::toggle_shutdown_flag(Origin::root()));
    });
}

#[test]
fn call_filter_works() {
    new_test_ext().execute_with(|| {
        assert_eq!(EmergencyShutdown::is_shut_down_flag(), false);
        assert_ok!(EmergencyShutdown::toggle_shutdown_flag(Origin::root()));
        let call = Call::System(frame_system::Call::remark { remark: vec![] });

        // When emergency shutdown toggle is on
        assert_eq!(EmergencyShutdown::is_shut_down_flag(), true);
        assert_noop!(
            call.clone().dispatch(Origin::signed(1)),
            frame_system::Error::<Test>::CallFiltered,
        );

        // When emergency shutdown toggle is off
        assert_ok!(EmergencyShutdown::toggle_shutdown_flag(Origin::root()));
        assert_ok!(call.dispatch(Origin::signed(1)));
    });
}
