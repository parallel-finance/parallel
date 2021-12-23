use super::*;
use crate::mock::*;

use frame_support::{assert_noop, assert_ok};
use primitives::ump::*;
use sp_runtime::traits::{One, Zero};

#[test]
fn update_xcm_fees_should_work() {
    new_test_ext().execute_with(|| {
        // check error code
        assert_noop!(
            XcmHelpers::update_xcm_fees(
                frame_system::RawOrigin::Root.into(), // origin
                Zero::zero()                          // fees
            ),
            Error::<Test>::ZeroXcmFees
        );

        assert_ok!(XcmHelpers::update_xcm_fees(
            frame_system::RawOrigin::Root.into(), // origin
            One::one()                            // fees
        ));

        assert_eq!(XcmFees::<Test>::get(), One::one());
    });
}

#[test]
fn update_xcm_weight_should_work() {
    new_test_ext().execute_with(|| {
        // check error code
        let zero_xcm_weight_misc = XcmWeightMisc {
            bond_weight: Zero::zero(),
            bond_extra_weight: Zero::zero(),
            unbond_weight: Zero::zero(),
            rebond_weight: Zero::zero(),
            withdraw_unbonded_weight: Zero::zero(),
            nominate_weight: Zero::zero(),
            contribute_weight: Zero::zero(),
            withdraw_weight: Zero::zero(),
            add_memo_weight: Zero::zero(),
        };
        assert_noop!(
            XcmHelpers::update_xcm_weight(
                frame_system::RawOrigin::Root.into(), // origin
                zero_xcm_weight_misc                  // xcm_weight_misc
            ),
            Error::<Test>::ZeroXcmWeightMisc
        );

        assert_ok!(XcmHelpers::update_xcm_weight(
            frame_system::RawOrigin::Root.into(), // origin
            XcmWeightMisc::default()              // xcm_weight_misc
        ));

        assert_eq!(XcmWeight::<Test>::get(), XcmWeightMisc::default());
    });
}
