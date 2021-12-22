//! Liquid staking pallet benchmarking.
#![cfg(feature = "runtime-benchmarks")]
use super::*;

use crate::Pallet as LiquidStaking;

use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite};
use frame_support::{
    traits::{fungibles::Mutate, OnIdle},
    weights::Weight,
};
use frame_system::{self, RawOrigin as SystemOrigin};
use primitives::{
    tokens::{KSM, XKSM},
    ump::{RewardDestination, XcmWeightMisc},
    Balance, CurrencyId, Rate, Ratio,
};
use sp_runtime::traits::{One, StaticLookup};
use sp_std::{prelude::*, vec};

const SEED: u32 = 0;
const MARKET_CAP: u128 = 10000000000000000u128;
const XCM_FEES: u128 = 50000000000u128;
const RESERVE_FACTOR: Ratio = Ratio::from_perthousand(5);
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
const INITIAL_INSURANCE: u128 = 1000000000000u128;
const INITIAL_AMOUNT: u128 = 1000000000000000u128;

const STAKE_AMOUNT: u128 = 20000000000000u128;
const STAKED_AMOUNT: u128 = 19900000000000u128; // 20000000000000 * (1 - 5/1000)
const UNSTAKE_AMOUNT: u128 = 10000000000000u128;
const SLASHES: u128 = 1000000000u128;
const BOND_AMOUNT: u128 = 10000000000000u128;
const UNBOND_AMOUNT: u128 = 5000000000000u128;
const REBOND_AMOUNT: u128 = 5000000000000u128;
const WITHDRAW_AMOUNT: u128 = 5000000000000u128;
const INSURANCE_AMOUNT: u128 = 5000000000000u128;
const UNBONDING_AMOUNT: u128 = 0u128;
const REMAINING_WEIGHT: Weight = 100000000000u64;

fn initial_set_up<
    T: Config
        + pallet_assets::Config<AssetId = CurrencyId, Balance = Balance>
        + pallet_xcm_helper::Config,
>(
    caller: T::AccountId,
) {
    let account_id = T::Lookup::unlookup(caller.clone());
    let staking_pool_account = LiquidStaking::<T>::account_id();

    pallet_assets::Pallet::<T>::force_create(
        SystemOrigin::Root.into(),
        KSM,
        account_id.clone(),
        true,
        1,
    )
    .ok();
    pallet_assets::Pallet::<T>::force_set_metadata(
        SystemOrigin::Root.into(),
        KSM,
        b"Kusama".to_vec(),
        b"KSM".to_vec(),
        12,
        false,
    )
    .unwrap();

    pallet_assets::Pallet::<T>::force_create(SystemOrigin::Root.into(), XKSM, account_id, true, 1)
        .ok();

    pallet_assets::Pallet::<T>::force_set_metadata(
        SystemOrigin::Root.into(),
        XKSM,
        b"Parallel Kusama".to_vec(),
        b"XKSM".to_vec(),
        12,
        false,
    )
    .unwrap();

    <T as pallet_xcm_helper::Config>::Assets::mint_into(KSM, &caller, INITIAL_AMOUNT).unwrap();

    LiquidStaking::<T>::update_staking_pool_capacity(SystemOrigin::Root.into(), MARKET_CAP)
        .unwrap();
    LiquidStaking::<T>::update_xcm_fees(SystemOrigin::Root.into(), XCM_FEES).unwrap();

    <T as pallet_xcm_helper::Config>::Assets::mint_into(
        KSM,
        &staking_pool_account,
        INITIAL_INSURANCE,
    )
    .unwrap();
    <T as pallet_xcm_helper::Config>::Assets::mint_into(
        KSM,
        &pallet_xcm_helper::Pallet::<T>::account_id(),
        INITIAL_INSURANCE,
    )
    .unwrap();
    ExchangeRate::<T>::mutate(|b| *b = Rate::one());
    InsurancePool::<T>::mutate(|b| *b = INITIAL_INSURANCE);
}

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
    frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

benchmarks! {
    where_clause {
        where
            T: pallet_assets::Config<AssetId = CurrencyId, Balance = Balance> + pallet_xcm_helper::Config
    }

    stake {
        let alice: T::AccountId = account("Sample", 100, SEED);
        initial_set_up::<T>(alice.clone());
    }: _(SystemOrigin::Signed(alice.clone()), STAKE_AMOUNT)
    verify {
        assert_last_event::<T>(Event::<T>::Staked(alice, STAKED_AMOUNT).into());
    }

    unstake {
        let alice: T::AccountId = account("Sample", 100, SEED);
        initial_set_up::<T>(alice.clone());
        LiquidStaking::<T>::stake(SystemOrigin::Signed(alice.clone()).into(), STAKE_AMOUNT).unwrap();
    }: _(SystemOrigin::Signed(alice.clone()), UNSTAKE_AMOUNT)
    verify {
        assert_last_event::<T>(Event::<T>::Unstaked(alice, UNSTAKE_AMOUNT, UNSTAKE_AMOUNT).into());
    }

    bond {
        let alice: T::AccountId = account("Sample", 100, SEED);
        initial_set_up::<T>(alice.clone());
        LiquidStaking::<T>::stake(SystemOrigin::Signed(alice).into(), STAKE_AMOUNT).unwrap();
    }: _(SystemOrigin::Root, BOND_AMOUNT,  RewardDestination::Staked)
    verify {
        assert_last_event::<T>(Event::<T>::Bonding(LiquidStaking::<T>::derivative_para_account_id(), BOND_AMOUNT, RewardDestination::Staked).into());
    }

    nominate {
        let alice: T::AccountId = account("Sample", 100, SEED);
        let val1: T::AccountId = account("Sample", 101, SEED);
        let val2: T::AccountId = account("Sample", 102, SEED);
        initial_set_up::<T>(alice.clone());
        LiquidStaking::<T>::stake(SystemOrigin::Signed(alice).into(), STAKE_AMOUNT).unwrap();
    }: _(SystemOrigin::Root, vec![val1.clone(), val2.clone()])
    verify {
        assert_last_event::<T>(Event::<T>::Nominating(vec![val1, val2]).into());
    }

    bond_extra {
        let alice: T::AccountId = account("Sample", 100, SEED);
        initial_set_up::<T>(alice.clone());
        LiquidStaking::<T>::stake(SystemOrigin::Signed(alice).into(), STAKE_AMOUNT).unwrap();
        LiquidStaking::<T>::bond(SystemOrigin::Root.into(), BOND_AMOUNT, RewardDestination::Staked).unwrap();
    }: _(SystemOrigin::Root, BOND_AMOUNT)
    verify {
        assert_last_event::<T>(Event::<T>::BondingExtra(BOND_AMOUNT).into());
    }

    settlement {
        let alice: T::AccountId = account("Sample", 100, SEED);
        initial_set_up::<T>(alice.clone());
        LiquidStaking::<T>::stake(SystemOrigin::Signed(alice.clone()).into(), STAKE_AMOUNT).unwrap();
        LiquidStaking::<T>::unstake(SystemOrigin::Signed(alice.clone()).into(), UNSTAKE_AMOUNT).unwrap();
        LiquidStaking::<T>::stake(SystemOrigin::Signed(alice.clone()).into(), STAKE_AMOUNT).unwrap();
        LiquidStaking::<T>::unstake(SystemOrigin::Signed(alice).into(), UNSTAKE_AMOUNT).unwrap();
    }: _(SystemOrigin::Root, 0u128,  UNBONDING_AMOUNT)
    verify {
        assert_last_event::<T>(Event::<T>::Settlement(2 * STAKED_AMOUNT - 2 * UNSTAKE_AMOUNT, 0u128, 0u128).into());
    }

    unbond {
        let alice: T::AccountId = account("Sample", 100, SEED);
        initial_set_up::<T>(alice.clone());
        LiquidStaking::<T>::stake(SystemOrigin::Signed(alice).into(), STAKE_AMOUNT).unwrap();
        LiquidStaking::<T>::bond(SystemOrigin::Root.into(), BOND_AMOUNT, RewardDestination::Staked).unwrap();
    }: _(SystemOrigin::Root, UNBOND_AMOUNT)
    verify {
        assert_last_event::<T>(Event::<T>::Unbonding(UNBOND_AMOUNT).into());
    }

    rebond {
        let alice: T::AccountId = account("Sample", 100, SEED);
        initial_set_up::<T>(alice.clone());
        LiquidStaking::<T>::stake(SystemOrigin::Signed(alice).into(), STAKE_AMOUNT).unwrap();
        LiquidStaking::<T>::bond(SystemOrigin::Root.into(), BOND_AMOUNT, RewardDestination::Staked).unwrap();
        LiquidStaking::<T>::unbond(SystemOrigin::Root.into(), UNBOND_AMOUNT).unwrap();
    }: _(SystemOrigin::Root, REBOND_AMOUNT)
    verify {
        assert_last_event::<T>(Event::<T>::Rebonding(REBOND_AMOUNT).into());
    }

    withdraw_unbonded {
        let alice: T::AccountId = account("Sample", 100, SEED);
        initial_set_up::<T>(alice.clone());
        LiquidStaking::<T>::stake(SystemOrigin::Signed(alice).into(), STAKE_AMOUNT).unwrap();
        LiquidStaking::<T>::bond(SystemOrigin::Root.into(), BOND_AMOUNT, RewardDestination::Staked).unwrap();
        LiquidStaking::<T>::unbond(SystemOrigin::Root.into(), UNBOND_AMOUNT).unwrap();
    }: _(SystemOrigin::Root, 0, WITHDRAW_AMOUNT)
    verify {
        assert_last_event::<T>(Event::<T>::WithdrawingUnbonded(0).into());
    }

    update_reserve_factor {
    }: _(SystemOrigin::Root, RESERVE_FACTOR)
    verify {
        assert_eq!(ReserveFactor::<T>::get(), RESERVE_FACTOR);
    }

    update_staking_pool_capacity {
    }: _(SystemOrigin::Root, MARKET_CAP)
    verify {
    }

    update_xcm_fees {
    }: _(SystemOrigin::Root, XCM_FEES)
    verify {
        assert_last_event::<T>(Event::XcmFeesUpdated(XCM_FEES).into())
    }

    update_xcm_weight {
    }: _(SystemOrigin::Root, XCM_WEIGHT)
    verify {
        assert_eq!(XcmWeight::<T>::get(), XCM_WEIGHT);
    }

    add_insurances {
        let alice: T::AccountId = account("Sample", 100, SEED);
        initial_set_up::<T>(alice.clone());
    }: _(SystemOrigin::Signed(alice.clone()), INSURANCE_AMOUNT)
    verify {
        assert_eq!(InsurancePool::<T>::get(), INSURANCE_AMOUNT + INITIAL_INSURANCE);
    }

    payout_slashed {
        let alice: T::AccountId = account("Sample", 100, SEED);
        initial_set_up::<T>(alice);
    }: _(SystemOrigin::Root, SLASHES)
    verify {
        assert_eq!(InsurancePool::<T>::get(), INITIAL_INSURANCE - SLASHES);
    }

    on_idle {
        let alice: T::AccountId = account("Sample", 100, SEED);
        let bob: T::AccountId = account("Sample", 101, SEED);
        let charlie: T::AccountId = account("Sample", 102, SEED);
        initial_set_up::<T>(alice.clone());
        LiquidStaking::<T>::stake(SystemOrigin::Signed(alice).into(), STAKE_AMOUNT).unwrap();
        <T as pallet_xcm_helper::Config>::Assets::mint_into(XKSM, &bob, STAKED_AMOUNT).unwrap();
        <T as pallet_xcm_helper::Config>::Assets::mint_into(XKSM, &charlie, STAKED_AMOUNT).unwrap();
        LiquidStaking::<T>::unstake(SystemOrigin::Signed(bob).into(), STAKED_AMOUNT).unwrap();
        LiquidStaking::<T>::unstake(SystemOrigin::Signed(charlie).into(), STAKED_AMOUNT).unwrap();

        // Simulate withdraw_unbonded
        <T as pallet_xcm_helper::Config>::Assets::mint_into(KSM, &LiquidStaking::<T>::account_id(), 10 * STAKED_AMOUNT).unwrap();
    }: {
        LiquidStaking::<T>::on_idle(0u32.into(), REMAINING_WEIGHT)
    }
    verify {
        assert_eq!(UnstakeQueue::<T>::get().len(), 0);
    }
}

impl_benchmark_test_suite!(LiquidStaking, crate::mock::para_ext(1), crate::mock::Test);
