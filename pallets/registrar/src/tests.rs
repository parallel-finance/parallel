use super::*;
use crate::mock::*;
use crate::AssetMultiLocations;
use frame_support::{assert_noop, assert_ok, dispatch::*};
use xcm::VersionedMultiLocation;

#[test]
fn do_register_asset_works() {
    new_test_ext().execute_with(|| {
        let location = VersionedMultiLocation::V0(xcm::v0::MultiLocation::X1(
            xcm::v0::Junction::Parachain(1000),
        ));
        let asset_id = 32;

        assert!(!AssetMultiLocations::<Test>::contains_key(asset_id.clone()));
        assert_ok!(Registrar::register_asset(
            Origin::root(),
            asset_id.clone(),
            Box::new(location)
        ));
        assert!(AssetMultiLocations::<Test>::contains_key(asset_id));
    });
}

#[test]
fn do_register_asset_does_not_work_if_already_exist() {
    new_test_ext().execute_with(|| {
        let location = VersionedMultiLocation::V0(xcm::v0::MultiLocation::X1(
            xcm::v0::Junction::Parachain(1000),
        ));
        let asset_id = 32;

        assert!(!AssetMultiLocations::<Test>::contains_key(asset_id.clone()));
        assert_ok!(Registrar::register_asset(
            Origin::root(),
            asset_id.clone(),
            Box::new(location.clone())
        ));
        assert!(AssetMultiLocations::<Test>::contains_key(asset_id));

        assert_noop!(
            Registrar::register_asset(Origin::root(), asset_id.clone(), Box::new(location)),
            Error::<Test>::AssetAlreadyExists,
        );
    });
}
