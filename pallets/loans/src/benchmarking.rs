//! Loans pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]
use super::*;

use crate::Pallet as Loans;

use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::assert_ok;
use frame_system::{self, RawOrigin as SystemOrigin};
use sp_std::prelude::*;

const SEED: u32 = 0;
const DOT: AssetId = 0;
const KSM: AssetId = 1;

const MARKET_MOCK: Market = Market {
    close_factor: Ratio::from_percent(50),
    collateral_factor: Ratio::from_percent(50),
    liquidate_incentive: Rate::from_inner(Rate::DIV / 100 * 110),
    state: MarketState::Active,
    rate_model: InterestRateModel::Jump(JumpModel {
        base_rate: Rate::from_inner(Rate::DIV / 100 * 2),
        jump_rate: Rate::from_inner(Rate::DIV / 100 * 10),
        full_rate: Rate::from_inner(Rate::DIV / 100 * 32),
        jump_utilization: Ratio::from_percent(80),
    }),
    reserve_factor: Ratio::from_percent(15),
};
const PENDING_MARKET_MOCK: Market = {
    let mut market = MARKET_MOCK;
    market.state = MarketState::Pending;
    market
};

const INITIAL_AMOUNT: u32 = 500_000_000;

fn transfer_initial_balance<T: Config>(caller: T::AccountId)
where
    BalanceOf<T>: FixedPointOperand,
    AssetIdOf<T>: AtLeast32BitUnsigned,
{
    T::Assets::mint_into(DOT.into(), &caller, INITIAL_AMOUNT.into()).unwrap();
    T::Assets::mint_into(KSM.into(), &caller, INITIAL_AMOUNT.into()).unwrap();
}

fn set_account_borrows<T: Config>(
    who: T::AccountId,
    asset_id: AssetIdOf<T>,
    borrow_balance: BalanceOf<T>,
) where
    BalanceOf<T>: FixedPointOperand,
    AssetIdOf<T>: AtLeast32BitUnsigned,
{
    AccountBorrows::<T>::insert(
        asset_id,
        who,
        BorrowSnapshot {
            principal: borrow_balance,
            borrow_index: Rate::one(),
        },
    );
}

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
    frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

benchmarks! {
    where_clause {
        where
            BalanceOf<T>: FixedPointOperand,
            AssetIdOf<T>: AtLeast32BitUnsigned,
    }

    add_market {
    }: _(SystemOrigin::Root, DOT.into(),PENDING_MARKET_MOCK)
    verify {
        assert_last_event::<T>(Event::<T>::NewMarket(PENDING_MARKET_MOCK).into());
    }

    active_market {
        assert_ok!(Loans::<T>::add_market(SystemOrigin::Root.into(), DOT.into(), PENDING_MARKET_MOCK));
    }: _(SystemOrigin::Root,DOT.into())
    verify {
        assert_last_event::<T>(Event::<T>::ActivatedMarket(DOT.into()).into());
    }

    update_market {
        assert_ok!(Loans::<T>::add_market(SystemOrigin::Root.into(), DOT.into(), PENDING_MARKET_MOCK));
    }: _(SystemOrigin::Root,DOT.into(), PENDING_MARKET_MOCK)
    verify {

    }

    mint {
        let caller: T::AccountId = whitelisted_caller();
        transfer_initial_balance::<T>(caller.clone());
        let amount: u32 = 100_000;
    }: _(SystemOrigin::Signed(caller.clone()), DOT.into(), amount.into())
    verify {
        assert_last_event::<T>(Event::<T>::Deposited(caller, DOT.into(), amount.into()).into());
    }

    borrow {
        let caller: T::AccountId = whitelisted_caller();
        transfer_initial_balance::<T>(caller.clone());
        let deposit_amount: u32 = 200_000_000;
        let borrowed_amount: u32 = 100_000_000;
        assert_ok!(Loans::<T>::mint(SystemOrigin::Signed(caller.clone()).into(), DOT.into(), deposit_amount.into()));
        assert_ok!(Loans::<T>::collateral_asset(SystemOrigin::Signed(caller.clone()).into(), DOT.into(), true));
    }: _(SystemOrigin::Signed(caller.clone()), DOT.into(), borrowed_amount.into())
    verify {
        assert_last_event::<T>(Event::<T>::Borrowed(caller, DOT.into(), borrowed_amount.into()).into());
    }

    redeem {
        let caller: T::AccountId = whitelisted_caller();
        transfer_initial_balance::<T>(caller.clone());
        let deposit_amount: u32 = 100_000_000;
        let redeem_amount: u32 = 100_000;
        assert_ok!(Loans::<T>::mint(SystemOrigin::Signed(caller.clone()).into(), DOT.into(), deposit_amount.into()));
    }: _(SystemOrigin::Signed(caller.clone()), DOT.into(), redeem_amount.into())
    verify {
        assert_last_event::<T>(Event::<T>::Redeemed(caller, DOT.into(), redeem_amount.into()).into());
    }

    redeem_all {
        let caller: T::AccountId = whitelisted_caller();
        transfer_initial_balance::<T>(caller.clone());
        let deposit_amount: u32 = 100_000_000;
        assert_ok!(Loans::<T>::mint(SystemOrigin::Signed(caller.clone()).into(), DOT.into(), deposit_amount.into()));
    }: _(SystemOrigin::Signed(caller.clone()), DOT.into())
    verify {
        assert_last_event::<T>(Event::<T>::Redeemed(caller, DOT.into(), deposit_amount.into()).into());
    }

    repay_borrow {
        let caller: T::AccountId = whitelisted_caller();
        transfer_initial_balance::<T>(caller.clone());
        let deposit_amount: u32 = 200_000_000;
        let borrowed_amount: u32 = 100_000_000;
        let repay_amount: u32 = 100;
        assert_ok!(Loans::<T>::mint(SystemOrigin::Signed(caller.clone()).into(), DOT.into(), deposit_amount.into()));
        assert_ok!(Loans::<T>::collateral_asset(SystemOrigin::Signed(caller.clone()).into(), DOT.into(), true));
        assert_ok!(Loans::<T>::borrow(SystemOrigin::Signed(caller.clone()).into(), DOT.into(), borrowed_amount.into()));
    }: _(SystemOrigin::Signed(caller.clone()), DOT.into(), repay_amount.into())
    verify {
        assert_last_event::<T>(Event::<T>::RepaidBorrow(caller, DOT.into(), repay_amount.into()).into());
    }

    repay_borrow_all {
        let caller: T::AccountId = whitelisted_caller();
        transfer_initial_balance::<T>(caller.clone());
        let deposit_amount: u32 = 200_000_000;
        let borrowed_amount: u32 = 100_000_000;
        assert_ok!(Loans::<T>::mint(SystemOrigin::Signed(caller.clone()).into(), DOT.into(), deposit_amount.into()));
        assert_ok!(Loans::<T>::collateral_asset(SystemOrigin::Signed(caller.clone()).into(), DOT.into(), true));
        assert_ok!(Loans::<T>::borrow(SystemOrigin::Signed(caller.clone()).into(), DOT.into(), borrowed_amount.into()));
    }: _(SystemOrigin::Signed(caller.clone()), DOT.into())
    verify {
        assert_last_event::<T>(Event::<T>::RepaidBorrow(caller, DOT.into(), borrowed_amount.into()).into());
    }

    collateral_asset {
        let caller: T::AccountId = whitelisted_caller();
        transfer_initial_balance::<T>(caller.clone());
        let deposit_amount: u32 = 200_000_000;
        assert_ok!(Loans::<T>::mint(SystemOrigin::Signed(caller.clone()).into(), DOT.into(), deposit_amount.into()));
    }: _(SystemOrigin::Signed(caller.clone()), DOT.into(), true)
    verify {
        assert_last_event::<T>(Event::<T>::CollateralAssetAdded(caller, DOT.into()).into());
    }

    liquidate_borrow {
        let alice: T::AccountId = account("Sample", 100, SEED);
        let bob: T::AccountId = account("Sample", 101, SEED);
        transfer_initial_balance::<T>(alice.clone());
        transfer_initial_balance::<T>(bob.clone());
        let deposit_amount: u32 = 200_000_000;
        let borrowed_amount: u32 = 200_000_000;
        let liquidate_amount: u32 = 100_000_000;
        let incentive_amount: u32 = 110_000_000;
        assert_ok!(Loans::<T>::mint(SystemOrigin::Signed(bob.clone()).into(), KSM.into(), deposit_amount.into()));
        assert_ok!(Loans::<T>::mint(SystemOrigin::Signed(alice.clone()).into(), DOT.into(), deposit_amount.into()));
        assert_ok!(Loans::<T>::collateral_asset(SystemOrigin::Signed(alice.clone()).into(), DOT.into(), true));
        // assert_ok!(Loans::<T>::borrow(SystemOrigin::Signed(alice.clone()).into(), KSM.into(), borrowed_amount.into()));
        set_account_borrows::<T>(alice.clone(), KSM.into(), borrowed_amount.into());
    }: _(SystemOrigin::Signed(bob.clone()), alice.clone(), KSM.into(), liquidate_amount.into(), DOT.into())
    verify {
        assert_last_event::<T>(Event::<T>::LiquidatedBorrow(bob.clone(), alice.clone(), KSM.into(), DOT.into(), liquidate_amount.into(), incentive_amount.into()).into());
    }

    add_reserves {
        let caller: T::AccountId = whitelisted_caller();
        let payer = T::Lookup::unlookup(caller.clone());
        transfer_initial_balance::<T>(caller.clone());
        let amount: u32 = 2000;
    }: _(SystemOrigin::Root, payer, DOT.into(), amount.into())
    verify {
        assert_last_event::<T>(Event::<T>::ReservesAdded(caller, DOT.into(), amount.into(), amount.into()).into());
    }

    reduce_reserves {
        let caller: T::AccountId = whitelisted_caller();
        let payer = T::Lookup::unlookup(caller.clone());
        transfer_initial_balance::<T>(caller.clone());
        let add_amount: u32 = 2000;
        let reduce_amount: u32 = 1000;
        assert_ok!(Loans::<T>::add_reserves(SystemOrigin::Root.into(), payer.clone(), DOT.into(), add_amount.into()));
    }: _(SystemOrigin::Root, payer, DOT.into(), reduce_amount.into())
    verify {
        assert_last_event::<T>(Event::<T>::ReservesReduced(caller, DOT.into(), reduce_amount.into(), (add_amount-reduce_amount).into()).into());
    }
}

impl_benchmark_test_suite!(Loans, crate::mock::new_test_ext(), crate::mock::Test);
