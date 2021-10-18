//! Liquid staking pallet benchmarking.
#![cfg(feature = "runtime-benchmarks")]
use super::*;

use crate::Pallet as LiquidStaking;

use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::traits::fungibles::Inspect;
use frame_system::{self, RawOrigin as SystemOrigin};
use primitives::{
    tokens::{DOT, XDOT},
    Balance, CurrencyId,
};
use sp_runtime::{traits::AtLeast32BitUnsigned, FixedPointOperand};
use sp_std::prelude::*;

// stake
// unstake
// record_staking_settlement
// update_xcm_fees_compensation
// update_reserve_factor
// update_xcm_weight
// update_staking_pool_capacity
// set_liquid_currency
// set_staking_currency
//
// settlement
// bond
// bond_extra
// unbond
// rebond
// withdraw_unbonded
// nominate

fn initial_set_up<T: Config + pallet_assets::Config<AssetId = CurrencyId, Balance = Balance>>(
    caller: T::AccountId,
) where
    [u8; 32]: From<<T as frame_system::Config>::AccountId>,
    u128: From<<<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance>,
    BalanceOf<T>: FixedPointOperand,
    AssetIdOf<T>: AtLeast32BitUnsigned,
{
    // pallet_assets::Pallet::<T>::force_create(SystemOrigin::Rot.into(), DOT, caller, true, 1)
    //     .unwrap();
    // pallet_assets::Pallet::<T>::force_create(SystemOrigin::Root.into(), XDOT, caller, true, 1)
    //     .unwrap();
    // pallet_assets::Pallet::<T>::mint(SystemOrigin::Root.into(), DOT, caller, 10000).unwrap();
    //
    // LiquidStaking::set_liquid_currency(Origin::signed(BOB), XDOT).unwrap();
    // LiquidStaking::set_staking_currency(Origin::signed(BOB), DOT).unwrap();
    // LiquidStaking::update_staking_pool_capacity(Origin::signed(ALICE), dot(10000f64)).unwrap();
    // LiquidStaking::update_xcm_fees_compensation(Origin::signed(ALICE), dot(10f64)).unwrap();
}

benchmarks! {
    where_clause {
        where
            [u8; 32]: From<<T as frame_system::Config>::AccountId>,
            u128: From<
                <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance,
            >,
            BalanceOf<T>: FixedPointOperand,
            AssetIdOf<T>: AtLeast32BitUnsigned,
            T: pallet_assets::Config<AssetId = CurrencyId, Balance = Balance>
    }

    set_liquid_currency {
    }: _(SystemOrigin::Root, XDOT.into())
    verify {
        assert_eq!(LiquidCurrency::<T>::get(), Some(XDOT.into()));
    }
}

impl_benchmark_test_suite!(LiquidStaking, crate::mock::para_ext(1), crate::mock::Test);
