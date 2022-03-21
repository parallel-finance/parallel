use crate::mock::*;
use frame_support::{assert_noop, assert_ok, dispatch::*};

#[test]
fn toggle_shutdown_flag_works() {
    new_test_ext().execute_with(|| {
        let remark = "test".as_bytes().to_vec();
        let boxed_call = Box::new(Call::System(frame_system::Call::remark { remark }));

        assert_eq!(EmergencyShutdown::disable_calls(boxed_call.clone()), false);
        assert_ok!(EmergencyShutdown::toggle_call(
            Origin::root(),
            boxed_call.clone()
        ));
        assert_eq!(EmergencyShutdown::disable_calls(boxed_call.clone()), true);
        assert_ok!(EmergencyShutdown::toggle_call(
            Origin::root(),
            boxed_call.clone()
        ));
        assert_eq!(EmergencyShutdown::disable_calls(boxed_call.clone()), false);
        assert_ok!(EmergencyShutdown::toggle_call(Origin::root(), boxed_call));
    });
}

#[test]
fn call_filter_works() {
    new_test_ext().execute_with(|| {
        let remark = "test".as_bytes().to_vec();
        let call = Call::System(frame_system::Call::remark { remark });
        let boxed_call = Box::new(call.clone());
        assert_eq!(EmergencyShutdown::disable_calls(boxed_call.clone()), false);
        assert_ok!(EmergencyShutdown::toggle_call(
            Origin::root(),
            boxed_call.clone()
        ));

        // When emergency shutdown toggle is on
        assert_eq!(EmergencyShutdown::disable_calls(boxed_call.clone()), true);
        assert_noop!(
            call.clone().dispatch(Origin::signed(1)),
            frame_system::Error::<Test>::CallFiltered,
        );

        // When emergency shutdown toggle is off
        assert_ok!(EmergencyShutdown::toggle_call(Origin::root(), boxed_call));
        assert_ok!(call.dispatch(Origin::signed(1)));
    });
}
