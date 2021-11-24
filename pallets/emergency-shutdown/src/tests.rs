use crate::mock::*;
use frame_support::assert_ok;

#[test]
fn toggle_shutdown_flag_works() {
    new_test_ext().execute_with(|| {
        assert_eq!(EmergencyShutdown::is_shut_down_flag(), false);
        assert_ok!(EmergencyShutdown::toggle_shutdown_flag(Origin::root()));
        assert_eq!(EmergencyShutdown::is_shut_down_flag(), true);
        assert_ok!(EmergencyShutdown::toggle_shutdown_flag(Origin::root()));
        assert_eq!(EmergencyShutdown::is_shut_down_flag(), false);
    });
}
