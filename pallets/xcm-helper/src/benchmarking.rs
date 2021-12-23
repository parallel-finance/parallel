//! Crowdloans pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]
use super::*;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite};
use frame_support::pallet_prelude::*;
use frame_system::{self, RawOrigin as SystemOrigin};

const XCM_FEES: u128 = 50000000000u128;
const XCM_WEIGHT: XcmWeightMisc<Weight> = XcmWeightMisc {
    bond_weight: 3_000_000_000,
    bond_extra_weight: 3_000_000_000,
    unbond_weight: 3_000_000_000,
    rebond_weight: 3_000_000_000,
    withdraw_unbonded_weight: 3_000_000_000,
    nominate_weight: 3_000_000_000,
    contribute_weight: 3_000_000_000,
    withdraw_weight: 3_000_000_000,
    add_memo_weight: 3_000_000_000,
};
fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
    frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

benchmarks! {
    where_clause {
        where
            <T as frame_system::Config>::Origin: From<pallet_xcm::Origin>
    }

    update_xcm_fees {
    }: _(SystemOrigin::Root, XCM_FEES)
    verify {
        assert_last_event::<T>(Event::XcmFeesUpdated(XCM_FEES).into())
    }

    update_xcm_weight {
    }: _(SystemOrigin::Root, XCM_WEIGHT)
    verify {
        assert_last_event::<T>(Event::XcmWeightUpdated(XCM_WEIGHT).into())
    }

}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test,);
