use crate::mock::*;
use frame_support::traits::PalletInfoAccess;
use frame_support::{assert_noop, assert_ok, dispatch::*};

#[test]
fn toggle_call_works() {
    new_test_ext().execute_with(|| {
        let remark = "test".as_bytes().to_vec();
        let call = Call::System(frame_system::Call::remark { remark });

        let (pallet_idx, call_idx): (u8, u8) = call
            .using_encoded(|mut bytes| Decode::decode(&mut bytes))
            .expect(
                "decode input is output of Call encode; Call guaranteed to have two enums; qed",
            );
        assert_eq!(
            EmergencyShutdown::disabled_calls(pallet_idx.clone(), call_idx.clone()),
            false
        );
        assert_ok!(EmergencyShutdown::toggle_call(
            Origin::root(),
            pallet_idx.clone(),
            call_idx.clone()
        ));
        assert_eq!(
            EmergencyShutdown::disabled_calls(pallet_idx.clone(), call_idx.clone()),
            true
        );
        assert_ok!(EmergencyShutdown::toggle_call(
            Origin::root(),
            pallet_idx.clone(),
            call_idx.clone()
        ));
        assert_eq!(
            EmergencyShutdown::disabled_calls(pallet_idx, call_idx),
            false
        );
    });
}

#[test]
fn toggle_pallet_works() {
    new_test_ext().execute_with(|| {
        let pallet_idx = System::index() as u8;

        assert_eq!(
            EmergencyShutdown::disabled_pallets(pallet_idx.clone()),
            false
        );
        assert_ok!(EmergencyShutdown::toggle_pallet(
            Origin::root(),
            pallet_idx.clone()
        ));
        assert_eq!(
            EmergencyShutdown::disabled_pallets(pallet_idx.clone()),
            true
        );
        assert_ok!(EmergencyShutdown::toggle_pallet(
            Origin::root(),
            pallet_idx.clone()
        ));
        assert_eq!(EmergencyShutdown::disabled_pallets(pallet_idx), false);
    });
}

#[test]
fn call_filter_works() {
    new_test_ext().execute_with(|| {
        let pallet_idx = System::index() as u8;

        assert_eq!(
            EmergencyShutdown::disabled_pallets(pallet_idx.clone()),
            false
        );
        assert_ok!(EmergencyShutdown::toggle_pallet(
            Origin::root(),
            pallet_idx.clone()
        ));

        let remark = "test".as_bytes().to_vec();
        let call = Call::System(frame_system::Call::remark { remark });
        // When emergency shutdown toggle is on
        assert_eq!(
            EmergencyShutdown::disabled_pallets(pallet_idx.clone()),
            true
        );
        assert_noop!(
            call.clone().dispatch(Origin::signed(1)),
            frame_system::Error::<Test>::CallFiltered,
        );

        // When emergency shutdown toggle is off
        assert_ok!(EmergencyShutdown::toggle_pallet(Origin::root(), pallet_idx));

        assert_ok!(call.dispatch(Origin::signed(1)));
    });
}
