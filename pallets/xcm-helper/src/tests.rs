use super::*;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok};

use sp_runtime::traits::{One, Zero};

#[test]
fn update_xcm_fees_should_work() {
    new_test_ext().execute_with(|| {
        // check error code
        assert_noop!(
            XcmHelpers::update_xcm_weight_fee(
                frame_system::RawOrigin::Root.into(), // origin
                XcmCall::Bond,
                XcmWeightFeeMisc {
                    weight: Weight::from_ref_time(One::one()),
                    fee: Zero::zero()
                }
            ),
            Error::<Test>::ZeroXcmFees
        );

        assert_noop!(
            XcmHelpers::update_xcm_weight_fee(
                frame_system::RawOrigin::Root.into(), // origin
                XcmCall::Bond,
                XcmWeightFeeMisc {
                    weight: Zero::zero(),
                    fee: One::one()
                }
            ),
            Error::<Test>::ZeroXcmWeightMisc
        );

        assert_ok!(XcmHelpers::update_xcm_weight_fee(
            frame_system::RawOrigin::Root.into(), // origin
            XcmCall::Bond,
            XcmWeightFeeMisc::default()
        ));

        assert_eq!(
            XcmWeightFee::<Test>::get(XcmCall::Bond),
            XcmWeightFeeMisc::default()
        );
    });
}
