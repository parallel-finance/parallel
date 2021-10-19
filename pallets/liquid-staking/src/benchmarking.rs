//! Liquid staking pallet benchmarking.
#![cfg(feature = "runtime-benchmarks")]
use super::{
    types::{RewardDestination, StakingSettlementKind, XcmWeightMisc},
    *,
};

use crate::Pallet as LiquidStaking;

use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite};
use frame_support::{
    traits::{
        fungibles::{Inspect, Mutate},
        OnIdle,
    },
    weights::Weight,
};
use frame_system::{self, RawOrigin as SystemOrigin};
use primitives::{
    tokens::{DOT, XDOT},
    Balance, CurrencyId, Rate, Ratio,
};
use sp_runtime::{
    traits::{AtLeast32BitUnsigned, One, StaticLookup},
    FixedPointOperand,
};
use sp_std::{prelude::*, vec};

const SEED: u32 = 0;
const MARKET_CAP: u128 = 10000000000000000u128;
const XCM_FEES_COMPENSATION: u128 = 50000000000u128;
const RESERVE_FACTOR: Ratio = Ratio::from_perthousand(5);
const XCM_WEIGHT: XcmWeightMisc<Weight> = XcmWeightMisc {
    bond_weight: 2_000_000_000,
    bond_extra_weight: 2_000_000_000,
    unbond_weight: 2_000_000_000,
    rebond_weight: 2_000_000_000,
    withdraw_unbonded_weight: 2_000_000_000,
    nominate_weight: 2_000_000_000,
};
const INITIAL_INSURANCE: u128 = 1000000000000u128;
const INITIAL_AMOUNT: u128 = 1000000000000000u128;

const STAKE_AMOUNT: u128 = 20000000000000u128;
const STAKED_AMOUNT: u128 = 19900000000000u128; // 20000000000000 * (1 - 5/1000)
const UNSTAKE_AMOUNT: u128 = 10000000000000u128;
const REWARDS: u128 = 10000000000000u128;
const BOND_AMOUNT: u128 = 10000000000000u128;
const UNBOND_AMOUNT: u128 = 5000000000000u128;
const REBOND_AMOUNT: u128 = 5000000000000u128;
const WITHDRAW_AMOUNT: u128 = 5000000000000u128;
const INSURANCE_AMOUNT: u128 = 5000000000000u128;
const UNBONDING_AMOUNT: u128 = 0u128;
const REMAINING_WEIGHT: Weight = 100000000000u64;

fn initial_set_up<T: Config + pallet_assets::Config<AssetId = CurrencyId, Balance = Balance>>(
    caller: T::AccountId,
) where
    [u8; 32]: From<<T as frame_system::Config>::AccountId>,
    u128: From<<<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance>,
    BalanceOf<T>: FixedPointOperand + From<u128>,
    AssetIdOf<T>: AtLeast32BitUnsigned,
{
    let account_id = T::Lookup::unlookup(caller.clone());
    let staking_pool_account = LiquidStaking::<T>::account_id();

    pallet_assets::Pallet::<T>::force_create(
        SystemOrigin::Root.into(),
        DOT,
        account_id.clone(),
        true,
        1,
    )
    .ok();

    pallet_assets::Pallet::<T>::force_create(SystemOrigin::Root.into(), XDOT, account_id, true, 1)
        .ok();

    T::Assets::mint_into(DOT.into(), &caller, INITIAL_AMOUNT.into()).unwrap();

    LiquidStaking::<T>::set_liquid_currency(SystemOrigin::Root.into(), XDOT.into()).unwrap();
    LiquidStaking::<T>::set_staking_currency(SystemOrigin::Root.into(), DOT.into()).unwrap();
    LiquidStaking::<T>::update_staking_pool_capacity(SystemOrigin::Root.into(), MARKET_CAP.into())
        .unwrap();
    LiquidStaking::<T>::update_xcm_fees_compensation(
        SystemOrigin::Root.into(),
        XCM_FEES_COMPENSATION.into(),
    )
    .unwrap();

    T::Assets::mint_into(DOT.into(), &staking_pool_account, INITIAL_INSURANCE.into()).unwrap();
    ExchangeRate::<T>::mutate(|b| *b = Rate::one());
    InsurancePool::<T>::mutate(|b| *b = INITIAL_INSURANCE.into());
}

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
    frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

benchmarks! {
    where_clause {
        where
            [u8; 32]: From<<T as frame_system::Config>::AccountId>,
            u128: From<
                <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance,
            >,
            BalanceOf<T>: FixedPointOperand + From<u128>,
            AssetIdOf<T>: AtLeast32BitUnsigned,
            T: pallet_assets::Config<AssetId = CurrencyId, Balance = Balance>
    }

    stake {
        let alice: T::AccountId = account("Sample", 100, SEED);
        initial_set_up::<T>(alice.clone());
    }: _(SystemOrigin::Signed(alice.clone()), STAKE_AMOUNT.into())
    verify {
        assert_last_event::<T>(Event::<T>::Staked(alice, STAKED_AMOUNT.into()).into());
    }

    unstake {
        let alice: T::AccountId = account("Sample", 100, SEED);
        initial_set_up::<T>(alice.clone());
        LiquidStaking::<T>::stake(SystemOrigin::Signed(alice.clone()).into(), STAKE_AMOUNT.into()).unwrap();
    }: _(SystemOrigin::Signed(alice.clone()), UNSTAKE_AMOUNT.into())
    verify {
        assert_last_event::<T>(Event::<T>::Unstaked(alice, UNSTAKE_AMOUNT.into(), UNSTAKE_AMOUNT.into()).into());
    }

    bond {
        let alice: T::AccountId = account("Sample", 100, SEED);
        initial_set_up::<T>(alice.clone());
        LiquidStaking::<T>::stake(SystemOrigin::Signed(alice.clone()).into(), STAKE_AMOUNT.into()).unwrap();
    }: _(SystemOrigin::Root, BOND_AMOUNT.into(),  RewardDestination::Staked)
    verify {
        assert_last_event::<T>(Event::<T>::BondCallSent(LiquidStaking::<T>::derivative_para_account_id(), BOND_AMOUNT.into(), RewardDestination::Staked).into());
    }

    nominate {
        let alice: T::AccountId = account("Sample", 100, SEED);
        let val1: T::AccountId = account("Sample", 101, SEED);
        let val2: T::AccountId = account("Sample", 102, SEED);
        initial_set_up::<T>(alice.clone());
        LiquidStaking::<T>::stake(SystemOrigin::Signed(alice).into(), STAKE_AMOUNT.into()).unwrap();
    }: _(SystemOrigin::Root, vec![val1.clone(), val2.clone()])
    verify {
        assert_last_event::<T>(Event::<T>::NominateCallSent(vec![val1.clone(), val2.clone()]).into());
    }

    bond_extra {
        let alice: T::AccountId = account("Sample", 100, SEED);
        initial_set_up::<T>(alice.clone());
        LiquidStaking::<T>::stake(SystemOrigin::Signed(alice).into(), STAKE_AMOUNT.into()).unwrap();
        LiquidStaking::<T>::bond(SystemOrigin::Root.into(), BOND_AMOUNT.into(), RewardDestination::Staked).unwrap();
    }: _(SystemOrigin::Root, BOND_AMOUNT.into())
    verify {
        assert_last_event::<T>(Event::<T>::BondExtraCallSent(BOND_AMOUNT.into()).into());
    }

    settlement {
        let alice: T::AccountId = account("Sample", 100, SEED);
        initial_set_up::<T>(alice.clone());
        LiquidStaking::<T>::stake(SystemOrigin::Signed(alice.clone()).into(), STAKE_AMOUNT.into()).unwrap();
        LiquidStaking::<T>::unstake(SystemOrigin::Signed(alice.clone()).into(), UNSTAKE_AMOUNT.into()).unwrap();
        LiquidStaking::<T>::stake(SystemOrigin::Signed(alice.clone()).into(), STAKE_AMOUNT.into()).unwrap();
        LiquidStaking::<T>::unstake(SystemOrigin::Signed(alice.clone()).into(), UNSTAKE_AMOUNT.into()).unwrap();
    }: _(SystemOrigin::Root, false,  UNBONDING_AMOUNT.into())
    verify {
        assert_last_event::<T>(Event::<T>::Settlement((2 * STAKED_AMOUNT - 2 * UNSTAKE_AMOUNT).into(), 0u128.into(), 0u128.into()).into());
    }

    unbond {
        let alice: T::AccountId = account("Sample", 100, SEED);
        initial_set_up::<T>(alice.clone());
        LiquidStaking::<T>::stake(SystemOrigin::Signed(alice).into(), STAKE_AMOUNT.into()).unwrap();
        LiquidStaking::<T>::bond(SystemOrigin::Root.into(), BOND_AMOUNT.into(), RewardDestination::Staked).unwrap();
    }: _(SystemOrigin::Root, UNBOND_AMOUNT.into())
    verify {
        assert_last_event::<T>(Event::<T>::UnbondCallSent(UNBOND_AMOUNT.into()).into());
    }

    rebond {
        let alice: T::AccountId = account("Sample", 100, SEED);
        initial_set_up::<T>(alice.clone());
        LiquidStaking::<T>::stake(SystemOrigin::Signed(alice).into(), STAKE_AMOUNT.into()).unwrap();
        LiquidStaking::<T>::bond(SystemOrigin::Root.into(), BOND_AMOUNT.into(), RewardDestination::Staked).unwrap();
        LiquidStaking::<T>::unbond(SystemOrigin::Root.into(), UNBOND_AMOUNT.into()).unwrap();
    }: _(SystemOrigin::Root, REBOND_AMOUNT.into())
    verify {
        assert_last_event::<T>(Event::<T>::RebondCallSent(REBOND_AMOUNT.into()).into());
    }

    withdraw_unbonded {
        let alice: T::AccountId = account("Sample", 100, SEED);
        initial_set_up::<T>(alice.clone());
        LiquidStaking::<T>::stake(SystemOrigin::Signed(alice).into(), STAKE_AMOUNT.into()).unwrap();
        LiquidStaking::<T>::bond(SystemOrigin::Root.into(), BOND_AMOUNT.into(), RewardDestination::Staked).unwrap();
        LiquidStaking::<T>::unbond(SystemOrigin::Root.into(), UNBOND_AMOUNT.into()).unwrap();
    }: _(SystemOrigin::Root, 0, WITHDRAW_AMOUNT.into())
    verify {
        assert_last_event::<T>(Event::<T>::WithdrawUnbondedCallSent(0).into());
    }

    record_staking_settlement {
        let alice: T::AccountId = account("Sample", 100, SEED);
        initial_set_up::<T>(alice.clone());
        LiquidStaking::<T>::stake(SystemOrigin::Signed(alice).into(), STAKE_AMOUNT.into()).unwrap();
    }: _(SystemOrigin::Root, REWARDS.into(), StakingSettlementKind::Reward)
    verify {
        assert_last_event::<T>(Event::<T>::StakingSettlementRecorded(StakingSettlementKind::Reward, REWARDS.into()).into());
    }

    set_liquid_currency {
    }: _(SystemOrigin::Root, XDOT.into())
    verify {
        assert_eq!(LiquidCurrency::<T>::get(), Some(XDOT.into()));
    }

    set_staking_currency {
    }: _(SystemOrigin::Root, DOT.into())
    verify {
        assert_eq!(StakingCurrency::<T>::get(), Some(DOT.into()));
    }

    update_reserve_factor {
    }: _(SystemOrigin::Root, RESERVE_FACTOR)
    verify {
        assert_eq!(ReserveFactor::<T>::get(), RESERVE_FACTOR);
    }

    update_staking_pool_capacity {
    }: _(SystemOrigin::Root, MARKET_CAP.into())
    verify {
        assert_eq!(StakingPoolCapacity::<T>::get(), MARKET_CAP.into());
    }

    update_xcm_fees_compensation {
    }: _(SystemOrigin::Root, XCM_FEES_COMPENSATION.into())
    verify {
        assert_eq!(XcmFeesCompensation::<T>::get(), XCM_FEES_COMPENSATION.into());
    }

    update_xcm_weight {
    }: _(SystemOrigin::Root, XCM_WEIGHT)
    verify {
        assert_eq!(XcmWeight::<T>::get(), XCM_WEIGHT);
    }

    add_insurances {
        let alice: T::AccountId = account("Sample", 100, SEED);
        initial_set_up::<T>(alice.clone());
    }: _(SystemOrigin::Signed(alice.clone()), INSURANCE_AMOUNT.into())
    verify {
        assert_eq!(InsurancePool::<T>::get(), (INSURANCE_AMOUNT + INITIAL_INSURANCE).into());
    }

    on_idle {
        let alice: T::AccountId = account("Sample", 100, SEED);
        let bob: T::AccountId = account("Sample", 101, SEED);
        let charlie: T::AccountId = account("Sample", 102, SEED);
        let eve: T::AccountId = account("Sample", 103, SEED);
        initial_set_up::<T>(alice.clone());
        LiquidStaking::<T>::stake(SystemOrigin::Signed(alice.clone()).into(), STAKE_AMOUNT.into()).unwrap();
        StakingPool::<T>::mutate(|b| *b = *b + (2 * STAKED_AMOUNT).into());
        T::Assets::mint_into(XDOT.into(), &bob.clone(), STAKED_AMOUNT.into()).unwrap();
        T::Assets::mint_into(XDOT.into(), &charlie.clone(), STAKED_AMOUNT.into()).unwrap();
        T::Assets::mint_into(XDOT.into(), &eve.clone(), STAKED_AMOUNT.into()).unwrap();
        LiquidStaking::<T>::unstake(SystemOrigin::Signed(bob.clone()).into(), STAKED_AMOUNT.into()).unwrap();
        LiquidStaking::<T>::unstake(SystemOrigin::Signed(charlie.clone()).into(), STAKED_AMOUNT.into()).unwrap();
        LiquidStaking::<T>::unstake(SystemOrigin::Signed(eve.clone()).into(), STAKED_AMOUNT.into()).unwrap();

        // Simulate withdraw_unbonded
        T::Assets::mint_into(DOT.into(), &LiquidStaking::<T>::account_id(), (10 * STAKED_AMOUNT).into()).unwrap();
    }: {
        LiquidStaking::<T>::on_idle(0u32.into(), REMAINING_WEIGHT)
    }
    verify {
        assert_eq!(UnstakeQueue::<T>::get().len(), 0);
    }
}

impl_benchmark_test_suite!(LiquidStaking, crate::mock::para_ext(1), crate::mock::Test);
