//! Registrar pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]
use super::*;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite};
use frame_system::{self, RawOrigin as SystemOrigin};

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
    frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

benchmarks! {
    register_asset {
        let asset_id = 32;
        let location = VersionedMultiLocation::V0(xcm::v0::MultiLocation::X1(
            xcm::v0::Junction::Parachain(1000),
        ));
    }: _(SystemOrigin::Root, asset_id, Box::new(location))
    verify {
        assert_last_event::<T>(Event::AssetRegistered(asset_id).into())
    }
}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test,);
