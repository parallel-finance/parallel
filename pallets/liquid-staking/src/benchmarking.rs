//! Liquid staking pallet benchmarking.
#![cfg(feature = "runtime-benchmarks")]

use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite};
use frame_support::{
    assert_ok,
    dispatch::DispatchResult,
    storage::with_transaction,
    traits::{fungibles::Mutate, Hooks},
};
use frame_system::{self, RawOrigin as SystemOrigin};
use sp_runtime::{
    traits::{One, Saturating, StaticLookup},
    TransactionOutcome,
};
use sp_std::{prelude::*, vec};
use xcm::latest::prelude::*;

use pallet_traits::ump::RewardDestination;
use primitives::{Balance, CurrencyId, Rate, Ratio};

use crate::{
    types::{StakingLedger, UnstakeProvider},
    Pallet as LiquidStaking,
};

use super::*;

const SEED: u32 = 0;
const STAKING_LEDGER_CAP: u128 = 10000000000000000u128;
const RESERVE_FACTOR: Ratio = Ratio::from_perthousand(5);
const INITIAL_XCM_FEES: u128 = 1000000000000u128;
const INITIAL_AMOUNT: u128 = 1000000000000000u128;

const STAKE_AMOUNT: u128 = 25000000000000u128;
const UNSTAKE_AMOUNT: u128 = 10000000000000u128;
const BOND_AMOUNT: u128 = 10000000000000u128;
const UNBOND_AMOUNT: u128 = 5000000000000u128;
const REBOND_AMOUNT: u128 = 5000000000000u128;

const COMMISSION_RATE: Rate = Rate::from_rational(1, 100);

fn initial_set_up<
    T: Config
        + pallet_assets::Config<AssetId = CurrencyId, Balance = Balance>
        + pallet_xcm_helper::Config,
>(
    caller: T::AccountId,
) {
    let account_id = T::Lookup::unlookup(caller.clone());

    pallet_assets::Pallet::<T>::force_create(
        SystemOrigin::Root.into(),
        T::StakingCurrency::get(),
        account_id.clone(),
        true,
        1,
    )
    .ok();
    pallet_assets::Pallet::<T>::force_set_metadata(
        SystemOrigin::Root.into(),
        T::StakingCurrency::get(),
        b"Staking Currency".to_vec(),
        b"Staking Currency".to_vec(),
        12,
        false,
    )
    .unwrap();

    pallet_assets::Pallet::<T>::force_create(
        SystemOrigin::Root.into(),
        T::LiquidCurrency::get(),
        account_id,
        true,
        1,
    )
    .ok();

    pallet_assets::Pallet::<T>::force_set_metadata(
        SystemOrigin::Root.into(),
        T::LiquidCurrency::get(),
        b"Liquid Currency".to_vec(),
        b"Liquid Currency".to_vec(),
        12,
        false,
    )
    .unwrap();

    <T as pallet_xcm_helper::Config>::Assets::mint_into(
        T::StakingCurrency::get(),
        &caller,
        INITIAL_AMOUNT,
    )
    .unwrap();

    LiquidStaking::<T>::update_staking_ledger_cap(SystemOrigin::Root.into(), STAKING_LEDGER_CAP)
        .unwrap();

    LiquidStaking::<T>::update_reserve_factor(SystemOrigin::Root.into(), RESERVE_FACTOR).unwrap();

    <T as pallet_xcm_helper::Config>::Assets::mint_into(
        T::StakingCurrency::get(),
        &pallet_xcm_helper::Pallet::<T>::account_id(),
        INITIAL_XCM_FEES,
    )
    .unwrap();
    ExchangeRate::<T>::mutate(|b| *b = Rate::one());
}

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
    frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

benchmarks! {
    where_clause {
        where
            T: Config + pallet_assets::Config<AssetId = CurrencyId, Balance = Balance> + pallet_xcm_helper::Config,
            <T as frame_system::Config>::RuntimeOrigin: From<pallet_xcm::Origin>
    }

    stake {
        let alice: T::AccountId = account("Sample", 100, SEED);
        initial_set_up::<T>(alice.clone());
    }: _(SystemOrigin::Signed(alice.clone()), STAKE_AMOUNT)
    verify {
        let xcm_fee = T::XcmFees::get();
        let reserve = ReserveFactor::<T>::get().mul_floor(STAKE_AMOUNT);
        assert_last_event::<T>(Event::<T>::Staked(alice, STAKE_AMOUNT - xcm_fee - reserve).into());
    }

    unstake {
        let alice: T::AccountId = account("Sample", 100, SEED);
        initial_set_up::<T>(alice.clone());
        LiquidStaking::<T>::stake(SystemOrigin::Signed(alice.clone()).into(), STAKE_AMOUNT).unwrap();
    }: _(SystemOrigin::Signed(alice.clone()), UNSTAKE_AMOUNT, Default::default())
    verify {
        assert_last_event::<T>(Event::<T>::Unstaked(alice, UNSTAKE_AMOUNT, UNSTAKE_AMOUNT).into());
    }

    bond {
        let alice: T::AccountId = account("Sample", 100, SEED);
        initial_set_up::<T>(alice.clone());
        LiquidStaking::<T>::stake(SystemOrigin::Signed(alice).into(), STAKE_AMOUNT).unwrap();
    }: _(SystemOrigin::Root, 0, BOND_AMOUNT,  RewardDestination::Staked)
    verify {
        assert_last_event::<T>(Event::<T>::Bonding(0, LiquidStaking::<T>::derivative_sovereign_account_id(0), BOND_AMOUNT, RewardDestination::Staked).into());
    }

    nominate {
        let alice: T::AccountId = account("Sample", 100, SEED);
        let val1: T::AccountId = account("Sample", 101, SEED);
        let val2: T::AccountId = account("Sample", 102, SEED);
        initial_set_up::<T>(alice.clone());
        LiquidStaking::<T>::stake(SystemOrigin::Signed(alice).into(), STAKE_AMOUNT).unwrap();
        LiquidStaking::<T>::bond(SystemOrigin::Root.into(), 0, BOND_AMOUNT, RewardDestination::Staked).unwrap();
        LiquidStaking::<T>::notification_received(
            pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
            0u64,
            Response::ExecutionResult(None)
        ).unwrap();
    }: _(SystemOrigin::Root, 0, vec![val1.clone(), val2.clone()])
    verify {
        assert_last_event::<T>(Event::<T>::Nominating(0, vec![val1, val2]).into());
    }

    bond_extra {
        let alice: T::AccountId = account("Sample", 100, SEED);
        initial_set_up::<T>(alice.clone());
        LiquidStaking::<T>::stake(SystemOrigin::Signed(alice).into(), STAKE_AMOUNT).unwrap();
        LiquidStaking::<T>::bond(SystemOrigin::Root.into(), 0, BOND_AMOUNT, RewardDestination::Staked).unwrap();
        LiquidStaking::<T>::notification_received(
            pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
            0u64,
            Response::ExecutionResult(None)
        ).unwrap();
    }: _(SystemOrigin::Root, 0, BOND_AMOUNT)
    verify {
        assert_last_event::<T>(Event::<T>::BondingExtra(0, BOND_AMOUNT).into());
    }

    force_set_staking_ledger {
        let alice: T::AccountId = account("Sample", 100, SEED);
        initial_set_up::<T>(alice.clone());
        LiquidStaking::<T>::stake(SystemOrigin::Signed(alice).into(), STAKE_AMOUNT).unwrap();
        LiquidStaking::<T>::bond(SystemOrigin::Root.into(), 0, BOND_AMOUNT, RewardDestination::Staked).unwrap();
        LiquidStaking::<T>::notification_received(
            pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
            0u64,
            Response::ExecutionResult(None)
        ).unwrap();
        let staking_ledger = StakingLedgers::<T>::get(0).unwrap();
    }: _(SystemOrigin::Root, 0u16,  staking_ledger.clone())
    verify {
        assert_last_event::<T>(Event::<T>::StakingLedgerUpdated(0, staking_ledger).into());
    }

    unbond {
        let alice: T::AccountId = account("Sample", 100, SEED);
        initial_set_up::<T>(alice.clone());
        LiquidStaking::<T>::stake(SystemOrigin::Signed(alice.clone()).into(), STAKE_AMOUNT).unwrap();
        LiquidStaking::<T>::unstake(SystemOrigin::Signed(alice).into(), UNBOND_AMOUNT, Default::default()).unwrap();
        LiquidStaking::<T>::bond(SystemOrigin::Root.into(), 0, BOND_AMOUNT, RewardDestination::Staked).unwrap();
        LiquidStaking::<T>::notification_received(
            pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
            0u64,
            Response::ExecutionResult(None)
        ).unwrap();
    }: _(SystemOrigin::Root, 0, UNBOND_AMOUNT)
    verify {
        assert_last_event::<T>(Event::<T>::Unbonding(0, UNBOND_AMOUNT).into());
    }

    rebond {
        let alice: T::AccountId = account("Sample", 100, SEED);
        initial_set_up::<T>(alice.clone());
        LiquidStaking::<T>::stake(SystemOrigin::Signed(alice.clone()).into(), STAKE_AMOUNT).unwrap();
        LiquidStaking::<T>::unstake(SystemOrigin::Signed(alice).into(), UNBOND_AMOUNT, Default::default()).unwrap();
        LiquidStaking::<T>::bond(SystemOrigin::Root.into(), 0, BOND_AMOUNT, RewardDestination::Staked).unwrap();
        LiquidStaking::<T>::notification_received(
            pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
            0u64,
            Response::ExecutionResult(None)
        ).unwrap();
        LiquidStaking::<T>::unbond(SystemOrigin::Root.into(), 0, UNBOND_AMOUNT).unwrap();
        LiquidStaking::<T>::notification_received(
            pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
            1u64,
            Response::ExecutionResult(None)
        ).unwrap();
    }: _(SystemOrigin::Root, 0, REBOND_AMOUNT)
    verify {
        assert_last_event::<T>(Event::<T>::Rebonding(0, REBOND_AMOUNT).into());
    }

    withdraw_unbonded {
        let alice: T::AccountId = account("Sample", 100, SEED);
        initial_set_up::<T>(alice.clone());
        LiquidStaking::<T>::stake(SystemOrigin::Signed(alice.clone()).into(), STAKE_AMOUNT).unwrap();
        LiquidStaking::<T>::unstake(SystemOrigin::Signed(alice).into(), UNBOND_AMOUNT, Default::default()).unwrap();
        LiquidStaking::<T>::bond(SystemOrigin::Root.into(), 0, BOND_AMOUNT, RewardDestination::Staked).unwrap();
        LiquidStaking::<T>::notification_received(
            pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
            0u64,
            Response::ExecutionResult(None)
        ).unwrap();
        LiquidStaking::<T>::unbond(SystemOrigin::Root.into(), 0, UNBOND_AMOUNT).unwrap();
        LiquidStaking::<T>::notification_received(
            pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
            1u64,
            Response::ExecutionResult(None)
        ).unwrap();
        LiquidStaking::<T>::force_set_current_era(SystemOrigin::Root.into(), T::BondingDuration::get() + 1).unwrap();
    }: _(SystemOrigin::Root, 0, 0)
    verify {
        assert_last_event::<T>(Event::<T>::WithdrawingUnbonded(0, 0).into());
    }

    update_reserve_factor {
    }: _(SystemOrigin::Root, RESERVE_FACTOR)
    verify {
        assert_eq!(ReserveFactor::<T>::get(), RESERVE_FACTOR);
    }

    update_commission_rate {
    }: _(SystemOrigin::Root, COMMISSION_RATE)
    verify {
        assert_eq!(CommissionRate::<T>::get(), COMMISSION_RATE);
    }

    update_incentive {
    }: _(SystemOrigin::Root, BalanceOf::<T>::one())
    verify {
        assert_eq!(Incentive::<T>::get(), BalanceOf::<T>::one());
    }

    update_staking_ledger_cap {
    }: _(SystemOrigin::Root, STAKING_LEDGER_CAP)
    verify {
    }

    notification_received {
        let alice: T::AccountId = account("Sample", 100, SEED);
        initial_set_up::<T>(alice.clone());
        LiquidStaking::<T>::stake(SystemOrigin::Signed(alice).into(), STAKE_AMOUNT).unwrap();
        LiquidStaking::<T>::bond(SystemOrigin::Root.into(), 0, BOND_AMOUNT, RewardDestination::Staked).unwrap();
    }:  _(
        pallet_xcm::Origin::Response(MultiLocation::parent()),
        0u64,
        Response::ExecutionResult(None)
    )
    verify {
        assert_last_event::<T>(Event::<T>::NotificationReceived(Box::new(MultiLocation::parent()), 0u64, None).into());
    }

    claim_for {
        let alice: T::AccountId = account("Sample", 100, SEED);
        let account_id = T::Lookup::unlookup(alice.clone());
        initial_set_up::<T>(alice.clone());
        LiquidStaking::<T>::stake(SystemOrigin::Signed(alice.clone()).into(), STAKE_AMOUNT).unwrap();
        LiquidStaking::<T>::unstake(SystemOrigin::Signed(alice.clone()).into(), UNSTAKE_AMOUNT, Default::default()).unwrap();
        assert_ok!(with_transaction(|| -> TransactionOutcome<DispatchResult>{
            LiquidStaking::<T>::do_advance_era(T::BondingDuration::get() + 1).unwrap();
            LiquidStaking::<T>::do_matching().unwrap();
            TransactionOutcome::Commit(Ok(()))
        }));
        LiquidStaking::<T>::notification_received(
            pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
            0u64,
            Response::ExecutionResult(None)
        ).unwrap();
    }: _(SystemOrigin::Root, account_id)
    verify {
        assert_last_event::<T>(Event::<T>::ClaimedFor(alice, UNSTAKE_AMOUNT).into());
    }

    force_set_era_start_block {
    }: _(SystemOrigin::Root, 11u32.into())
    verify {
        assert_eq!(EraStartBlock::<T>::get(), 11u32.into());
    }

    force_set_current_era {
    }: _(SystemOrigin::Root, 12)
    verify {
        assert_eq!(CurrentEra::<T>::get(), 12);
    }

    on_initialize {
    }: {
        LiquidStaking::<T>::on_initialize(11u32.into())
    }
    verify {
        assert_eq!(EraStartBlock::<T>::get(), 0u32.into());
        assert_eq!(CurrentEra::<T>::get(), 0);
    }

    force_advance_era {
        let alice: T::AccountId = account("Sample", 100, SEED);
        initial_set_up::<T>(alice.clone());
        // Insert a ledger, let `on_initialize` process three xcm:
        // do_withdraw_unbonded/do_bond_extra/do_rebond
        let mut staking_ledger = <StakingLedger<T::AccountId, BalanceOf<T>>>::new(
            LiquidStaking::<T>::derivative_sovereign_account_id(0u16),
            BOND_AMOUNT,
        );
        staking_ledger.unbond(UNBOND_AMOUNT,10);
        StakingLedgers::<T>::insert(0u16,staking_ledger);
        LiquidStaking::<T>::stake(SystemOrigin::Signed(alice).into(), STAKE_AMOUNT).unwrap();
    }: {
        assert_ok!(with_transaction(|| -> TransactionOutcome<DispatchResult> {
            LiquidStaking::<T>::do_matching().unwrap();
            LiquidStaking::<T>::do_advance_era(1).unwrap();
            TransactionOutcome::Commit(Ok(()))
        }));
    }
    verify {
        assert_eq!(EraStartBlock::<T>::get(), 0u32.into());
        assert_eq!(CurrentEra::<T>::get(), 1);
        assert_last_event::<T>(Event::<T>::NewEra(1).into());
    }

    force_matching {
        let alice: T::AccountId = account("Sample", 100, SEED);
        initial_set_up::<T>(alice.clone());
        // Insert a ledger, let `on_initialize` process three xcm:
        // do_withdraw_unbonded/do_bond_extra/do_rebond
        let mut staking_ledger = <StakingLedger<T::AccountId, BalanceOf<T>>>::new(
            LiquidStaking::<T>::derivative_sovereign_account_id(0u16),
            BOND_AMOUNT,
        );
        staking_ledger.unbond(UNBOND_AMOUNT, 10);
        StakingLedgers::<T>::insert(0u16,staking_ledger);
        LiquidStaking::<T>::stake(SystemOrigin::Signed(alice).into(), STAKE_AMOUNT).unwrap();
    }: {
        assert_ok!(with_transaction(|| -> TransactionOutcome<DispatchResult> {
            LiquidStaking::<T>::do_matching().unwrap();
            TransactionOutcome::Commit(Ok(()))
        }));
    }
    verify {
        let xcm_fee = T::XcmFees::get();
        let reserve = ReserveFactor::<T>::get().mul_floor(STAKE_AMOUNT);
        let bond_amount = STAKE_AMOUNT - xcm_fee - reserve - UNBOND_AMOUNT;
        assert_last_event::<T>(Event::<T>::Matching(bond_amount, UNBOND_AMOUNT, 0).into());
    }

    reduce_reserves {
        let alice: T::AccountId = account("Sample", 100, SEED);
        let account_id = T::Lookup::unlookup(alice.clone());
        let reduce_amount: u128 = 1000;
        initial_set_up::<T>(alice.clone());
        LiquidStaking::<T>::stake(SystemOrigin::Signed(alice.clone()).into(), STAKE_AMOUNT).unwrap();
    }: _(SystemOrigin::Root, account_id, reduce_amount)
    verify {
        let reserve = ReserveFactor::<T>::get().mul_floor(STAKE_AMOUNT) - reduce_amount;
        assert_eq!(TotalReserves::<T>::get(), reserve);
        assert_last_event::<T>(Event::<T>::ReservesReduced(alice, reduce_amount).into());
    }

    cancel_unstake {
        let alice: T::AccountId = account("Sample", 100, SEED);
        initial_set_up::<T>(alice.clone());
        LiquidStaking::<T>::stake(SystemOrigin::Signed(alice.clone()).into(), STAKE_AMOUNT).unwrap();
        LiquidStaking::<T>::unstake(SystemOrigin::Signed(alice.clone()).into(), UNSTAKE_AMOUNT, UnstakeProvider::MatchingPool).unwrap();
    }: _(SystemOrigin::Signed(alice.clone()), UNSTAKE_AMOUNT)
    verify {
        assert_last_event::<T>(Event::<T>::UnstakeCancelled(alice, UNSTAKE_AMOUNT, UNSTAKE_AMOUNT).into());
    }

    fast_match_unstake {
        let n in 1 .. 50;
        let alice: T::AccountId = account("Sample", 100, SEED);
        initial_set_up::<T>(alice.clone());
        LiquidStaking::<T>::stake(SystemOrigin::Signed(alice).into(), STAKE_AMOUNT).unwrap();

        let mut unstaker_list: Vec<T::AccountId> = vec![];
        let fast_unstake_amount = 50_000_000_000;
        for i in 0 .. n {
            let unstaker = account("unstaker", i, SEED);
            <T as pallet_xcm_helper::Config>::Assets::mint_into(
                T::LiquidCurrency::get(),
                &unstaker,
                INITIAL_AMOUNT,
            )
            .unwrap();

            LiquidStaking::<T>::unstake(SystemOrigin::Signed(unstaker.clone()).into(), fast_unstake_amount, UnstakeProvider::MatchingPool).unwrap();
            assert_eq!(FastUnstakeRequests::<T>::get(&unstaker), fast_unstake_amount);
            unstaker_list.push(unstaker);
        }
    }: _(SystemOrigin::Root, unstaker_list)
    verify {
        let xcm_fee = T::XcmFees::get();
        let reserve = ReserveFactor::<T>::get().mul_floor(STAKE_AMOUNT);
        let total_matched_amount = Rate::one()
            .saturating_sub(T::MatchingPoolFastUnstakeFee::get())
            .saturating_mul_int(fast_unstake_amount) * (n as u128) ;
        assert_eq!(
            MatchingPool::<T>::get().total_stake_amount.total,
            STAKE_AMOUNT - xcm_fee - reserve - total_matched_amount
        );
    }
}

impl_benchmark_test_suite!(LiquidStaking, crate::mock::para_ext(1), crate::mock::Test);
