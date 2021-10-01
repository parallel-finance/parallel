//! Router pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

extern crate alloc;

use super::*;
use crate::pallet::AssetIdOf;
use crate::pallet::BalanceOf;
#[allow(unused_imports)]
use crate::Pallet as AMMRoute;
use core::convert::TryFrom;
use frame_benchmarking::{
    account, benchmarks_instance_pallet, impl_benchmark_test_suite, whitelisted_caller,
};
use frame_support::{
    assert_ok,
    traits::fungibles::{Inspect, Mutate},
    BoundedVec,
};
use frame_system::{self, RawOrigin as SystemOrigin};
use primitives::{tokens, CurrencyId};
use sp_runtime::traits::AtLeast32BitUnsigned;
use sp_runtime::traits::One;
use sp_runtime::traits::StaticLookup;
use sp_runtime::FixedPointOperand;

const DOT: CurrencyId = tokens::DOT;
const XDOT: CurrencyId = tokens::XDOT;
const INITIAL_AMOUNT: u128 = 1000_000_000_000_000;
const ASSET_ID: u32 = 10;

fn assert_last_event<T: Config<I>, I: 'static>(generic_event: <T as Config<I>>::Event) {
    frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

fn initial_set_up<T: Config<I>, I: 'static>(caller: T::AccountId)
where
    <<T as crate::Config<I>>::Assets as Inspect<T::AccountId>>::Balance: From<u128>,
    <<T as crate::Config<I>>::Assets as Inspect<T::AccountId>>::AssetId: From<u32>,
    <<T as pallet_amm::Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance:
        FixedPointOperand,
    <<T as pallet_amm::Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::AssetId:
        AtLeast32BitUnsigned,
    <<T as pallet_amm::Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance:
        From<u128>,
    <<T as pallet_amm::Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::AssetId:
        From<u32>,
{
    let account_id = T::Lookup::unlookup(caller.clone());

    pallet_assets::Pallet::<T>::force_create(
        SystemOrigin::Root.into(),
        tokens::XDOT.into(),
        account_id.clone(),
        true,
        One::one(),
    )
    .ok();

    pallet_assets::Pallet::<T>::force_create(
        SystemOrigin::Root.into(),
        tokens::DOT.into(),
        account_id.clone(),
        true,
        One::one(),
    )
    .ok();

    <T as crate::Config<I>>::Assets::mint_into(DOT.into(), &caller, INITIAL_AMOUNT.into()).ok();

    let pool_creator = account("pool_creator", 1, 0);
    <T as crate::Config<I>>::Assets::mint_into(DOT.into(), &pool_creator, INITIAL_AMOUNT.into())
        .ok();
    <T as crate::Config<I>>::Assets::mint_into(XDOT.into(), &pool_creator, INITIAL_AMOUNT.into())
        .ok();

    assert_ok!(pallet_amm::Pallet::<T>::add_liquidity(
        SystemOrigin::Signed(pool_creator).into(),
        (DOT.into(), XDOT.into()),
        (100_000_000u128.into(), 100_000_000u128.into()),
        (99_999.into(), 99_999.into()),
        ASSET_ID.into()
    ));
}

benchmarks_instance_pallet! {
     where_clause {
        where
            BalanceOf<T, I>: FixedPointOperand,
            AssetIdOf<T, I>: AtLeast32BitUnsigned,
            <<T as crate::Config<I>>::Assets as Inspect<T::AccountId>>::Balance: From<u128>,
            <<T as crate::Config<I>>::Assets as Inspect<T::AccountId>>::AssetId: From<u32>,
            <<T as pallet_amm::Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance:
                FixedPointOperand,
            <<T as pallet_amm::Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::AssetId:
                AtLeast32BitUnsigned,
            <<T as pallet_amm::Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance:
                From<u128>,
            <<T as pallet_amm::Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::AssetId:
            From<u32>,
    }
    trade {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T, I>(caller.clone());
        let amount_in = 1_000;
        let original_amount_in = amount_in;
        let min_amount_out = 980;
        let expiry = u32::MAX;
        let routes: BoundedVec<_, <T as Config<I>>::MaxLengthRoute> = Route::<T, I>::try_from(alloc::vec![(DOT.into(), XDOT.into())]).unwrap();
    }: trade(SystemOrigin::Signed(caller.clone()), routes.clone(), amount_in.into(), min_amount_out.into(), expiry.into())

    verify {
        let amount_out: BalanceOf<T, I> = <T as crate::Config<I>>::Assets::balance(XDOT.into(), &caller);

        assert_eq!(amount_out, 994.into());
        assert_last_event::<T, I>(Event::TradedSuccessfully(caller, original_amount_in.into(), routes, amount_out.into()).into());
    }
}

impl_benchmark_test_suite!(AMMRoute, crate::mock::new_test_ext(), crate::mock::Runtime,);
