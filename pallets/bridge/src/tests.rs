#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::*;

#[test]
fn set_relayer_threshold_works() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Bridge::set_threshold(Origin::signed(ALICE), 3),
            Error::<Test>::OriginNoPermission,
        );
        assert_noop!(
            Bridge::set_threshold(Origin::signed(0u128), 0),
            Error::<Test>::InvalidThreshold,
        );
        assert_ok!(Bridge::set_threshold(Origin::signed(0u128), 3,));
    });
}