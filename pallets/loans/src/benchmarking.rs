//! Loans pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]
use super::*;
use crate::Pallet as Loans;

use crate::AccountBorrows;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::assert_ok;
use frame_system::{self, RawOrigin as SystemOrigin};
use primitives::{Balance, CurrencyId};
use sp_std::prelude::*;

const SEED: u32 = 0;
const DOT: CurrencyId = 101;
const KSM: CurrencyId = 100;
const UNKNOWN: CurrencyId = 5;
const PKSM: CurrencyId = 1000;
const PDOT: CurrencyId = 1001;
const PUNKNOWN: CurrencyId = 1005;

fn market_mock<T: Config>() -> Market<BalanceOf<T>> {
    Market {
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
        cap: 1_000_000_000_000_000_000_000u128, // set to $1B
        ptoken_id: 1200,
    }
}

fn pending_market_mock<T: Config>(ptoken_id: CurrencyId) -> Market<BalanceOf<T>>
where
    BalanceOf<T>: From<u128>,
{
    let mut market = market_mock::<T>();
    market.state = MarketState::Pending;
    market.ptoken_id = ptoken_id;
    market
}

const INITIAL_AMOUNT: u32 = 500_000_000;

fn transfer_initial_balance<
    T: Config + pallet_assets::Config<AssetId = CurrencyId, Balance = Balance> + pallet_prices::Config,
>(
    caller: T::AccountId,
) {
    let account_id = T::Lookup::unlookup(caller.clone());

    pallet_assets::Pallet::<T>::force_create(
        SystemOrigin::Root.into(),
        KSM,
        account_id.clone(),
        true,
        1,
    )
    .ok();

    pallet_assets::Pallet::<T>::force_create(SystemOrigin::Root.into(), DOT, account_id, true, 1)
        .ok();

    T::Assets::mint_into(DOT, &caller, INITIAL_AMOUNT.into()).unwrap();
    T::Assets::mint_into(KSM, &caller, INITIAL_AMOUNT.into()).unwrap();
    pallet_prices::Pallet::<T>::set_price(SystemOrigin::Root.into(), DOT, 1.into()).ok();
    pallet_prices::Pallet::<T>::set_price(SystemOrigin::Root.into(), KSM, 1.into()).ok();
}

fn set_account_borrows<T: Config>(
    who: T::AccountId,
    asset_id: AssetIdOf<T>,
    borrow_balance: BalanceOf<T>,
) {
    AccountBorrows::<T>::insert(
        asset_id,
        &who,
        BorrowSnapshot {
            principal: borrow_balance,
            borrow_index: Rate::one(),
        },
    );
    TotalBorrows::<T>::insert(asset_id, borrow_balance);
    T::Assets::burn_from(asset_id, &who, borrow_balance).unwrap();
}

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
    frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

benchmarks! {
    where_clause {
        where
            T: pallet_assets::Config<AssetId = CurrencyId, Balance = Balance> + pallet_prices::Config
    }

    add_market {
    }: _(SystemOrigin::Root, UNKNOWN, pending_market_mock::<T>(PUNKNOWN))
    verify {
        assert_last_event::<T>(Event::<T>::NewMarket(pending_market_mock::<T>(PUNKNOWN)).into());
    }

    active_market {
        assert_ok!(Loans::<T>::add_market(SystemOrigin::Root.into(), UNKNOWN, pending_market_mock::<T>(PUNKNOWN)));
    }: _(SystemOrigin::Root,UNKNOWN)
    verify {
        assert_last_event::<T>(Event::<T>::ActivatedMarket(UNKNOWN).into());
    }

    update_market {
        assert_ok!(Loans::<T>::add_market(SystemOrigin::Root.into(), DOT, pending_market_mock::<T>(PDOT)));
    }: _(SystemOrigin::Root,DOT, pending_market_mock::<T>(PDOT))
    verify {
        assert_last_event::<T>(Event::<T>::UpdatedMarket(pending_market_mock::<T>(PDOT)).into());
    }

    mint {
        let caller: T::AccountId = whitelisted_caller();
        transfer_initial_balance::<T>(caller.clone());
        assert_ok!(Loans::<T>::add_market(SystemOrigin::Root.into(), DOT, pending_market_mock::<T>(DOT)));
        assert_ok!(Loans::<T>::active_market(SystemOrigin::Root.into(), DOT));
        let amount: u32 = 100_000;
    }: _(SystemOrigin::Signed(caller.clone()), DOT, amount.into())
    verify {
        assert_last_event::<T>(Event::<T>::Deposited(caller, DOT, amount.into()).into());
    }

    borrow {
        let caller: T::AccountId = whitelisted_caller();
        transfer_initial_balance::<T>(caller.clone());
        let deposit_amount: u32 = 200_000_000;
        let borrowed_amount: u32 = 100_000_000;
        assert_ok!(Loans::<T>::add_market(SystemOrigin::Root.into(), DOT, pending_market_mock::<T>(PDOT)));
        assert_ok!(Loans::<T>::active_market(SystemOrigin::Root.into(), DOT));
        assert_ok!(Loans::<T>::mint(SystemOrigin::Signed(caller.clone()).into(), DOT, deposit_amount.into()));
        assert_ok!(Loans::<T>::collateral_asset(SystemOrigin::Signed(caller.clone()).into(), DOT, true));
    }: _(SystemOrigin::Signed(caller.clone()), DOT, borrowed_amount.into())
    verify {
        assert_last_event::<T>(Event::<T>::Borrowed(caller, DOT, borrowed_amount.into()).into());
    }

    redeem {
        let caller: T::AccountId = whitelisted_caller();
        transfer_initial_balance::<T>(caller.clone());
        let deposit_amount: u32 = 100_000_000;
        let redeem_amount: u32 = 100_000;
        assert_ok!(Loans::<T>::add_market(SystemOrigin::Root.into(), DOT, pending_market_mock::<T>(PDOT)));
        assert_ok!(Loans::<T>::active_market(SystemOrigin::Root.into(), DOT));
        assert_ok!(Loans::<T>::mint(SystemOrigin::Signed(caller.clone()).into(), DOT, deposit_amount.into()));
    }: _(SystemOrigin::Signed(caller.clone()), DOT, redeem_amount.into())
    verify {
        assert_last_event::<T>(Event::<T>::Redeemed(caller, DOT, redeem_amount.into()).into());
    }

    redeem_all {
        let caller: T::AccountId = whitelisted_caller();
        transfer_initial_balance::<T>(caller.clone());
        let deposit_amount: u32 = 100_000_000;
        assert_ok!(Loans::<T>::add_market(SystemOrigin::Root.into(), DOT, pending_market_mock::<T>(PDOT)));
        assert_ok!(Loans::<T>::active_market(SystemOrigin::Root.into(), DOT));
        assert_ok!(Loans::<T>::mint(SystemOrigin::Signed(caller.clone()).into(), DOT, deposit_amount.into()));
    }: _(SystemOrigin::Signed(caller.clone()), DOT)
    verify {
        assert_last_event::<T>(Event::<T>::Redeemed(caller, DOT, deposit_amount.into()).into());
    }

    repay_borrow {
        let caller: T::AccountId = whitelisted_caller();
        transfer_initial_balance::<T>(caller.clone());
        let deposit_amount: u32 = 200_000_000;
        let borrowed_amount: u32 = 100_000_000;
        let repay_amount: u32 = 100;
        assert_ok!(Loans::<T>::add_market(SystemOrigin::Root.into(), DOT, pending_market_mock::<T>(PDOT)));
        assert_ok!(Loans::<T>::active_market(SystemOrigin::Root.into(), DOT));
        assert_ok!(Loans::<T>::mint(SystemOrigin::Signed(caller.clone()).into(), DOT, deposit_amount.into()));
        assert_ok!(Loans::<T>::collateral_asset(SystemOrigin::Signed(caller.clone()).into(), DOT, true));
        assert_ok!(Loans::<T>::borrow(SystemOrigin::Signed(caller.clone()).into(), DOT, borrowed_amount.into()));
    }: _(SystemOrigin::Signed(caller.clone()), DOT, repay_amount.into())
    verify {
        assert_last_event::<T>(Event::<T>::RepaidBorrow(caller, DOT, repay_amount.into()).into());
    }

    repay_borrow_all {
        let caller: T::AccountId = whitelisted_caller();
        transfer_initial_balance::<T>(caller.clone());
        let deposit_amount: u32 = 200_000_000;
        let borrowed_amount: u32 = 100_000_000;
        assert_ok!(Loans::<T>::add_market(SystemOrigin::Root.into(), DOT, pending_market_mock::<T>(PDOT)));
        assert_ok!(Loans::<T>::active_market(SystemOrigin::Root.into(), DOT));
        assert_ok!(Loans::<T>::mint(SystemOrigin::Signed(caller.clone()).into(), DOT, deposit_amount.into()));
        assert_ok!(Loans::<T>::collateral_asset(SystemOrigin::Signed(caller.clone()).into(), DOT, true));
        assert_ok!(Loans::<T>::borrow(SystemOrigin::Signed(caller.clone()).into(), DOT, borrowed_amount.into()));
    }: _(SystemOrigin::Signed(caller.clone()), DOT)
    verify {
        assert_last_event::<T>(Event::<T>::RepaidBorrow(caller, DOT, borrowed_amount.into()).into());
    }

    collateral_asset {
        let caller: T::AccountId = whitelisted_caller();
        transfer_initial_balance::<T>(caller.clone());
        let deposit_amount: u32 = 200_000_000;
        assert_ok!(Loans::<T>::add_market(SystemOrigin::Root.into(), DOT, pending_market_mock::<T>(PDOT)));
        assert_ok!(Loans::<T>::active_market(SystemOrigin::Root.into(), DOT));
        assert_ok!(Loans::<T>::mint(SystemOrigin::Signed(caller.clone()).into(), DOT, deposit_amount.into()));
    }: _(SystemOrigin::Signed(caller.clone()), DOT, true)
    verify {
        assert_last_event::<T>(Event::<T>::CollateralAssetAdded(caller, DOT).into());
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
        assert_ok!(Loans::<T>::add_market(SystemOrigin::Root.into(), DOT, pending_market_mock::<T>(PDOT)));
        assert_ok!(Loans::<T>::active_market(SystemOrigin::Root.into(), DOT));
        assert_ok!(Loans::<T>::add_market(SystemOrigin::Root.into(), KSM, pending_market_mock::<T>(PKSM)));
        assert_ok!(Loans::<T>::active_market(SystemOrigin::Root.into(), KSM));
        assert_ok!(Loans::<T>::mint(SystemOrigin::Signed(bob.clone()).into(), KSM, deposit_amount.into()));
        assert_ok!(Loans::<T>::mint(SystemOrigin::Signed(alice.clone()).into(), DOT, deposit_amount.into()));
        assert_ok!(Loans::<T>::collateral_asset(SystemOrigin::Signed(alice.clone()).into(), DOT, true));
        set_account_borrows::<T>(alice.clone(), KSM, borrowed_amount.into());
    }: _(SystemOrigin::Signed(bob.clone()), alice.clone(), KSM, liquidate_amount.into(), DOT)
    verify {
        assert_last_event::<T>(Event::<T>::LiquidatedBorrow(bob.clone(), alice.clone(), KSM, DOT, liquidate_amount.into(), incentive_amount.into()).into());
    }

    add_reserves {
        let caller: T::AccountId = whitelisted_caller();
        let payer = T::Lookup::unlookup(caller.clone());
        transfer_initial_balance::<T>(caller.clone());
        let amount: u32 = 2000;
        assert_ok!(Loans::<T>::add_market(SystemOrigin::Root.into(), DOT, pending_market_mock::<T>(PDOT)));
        assert_ok!(Loans::<T>::active_market(SystemOrigin::Root.into(), DOT));
    }: _(SystemOrigin::Root, payer, DOT, amount.into())
    verify {
        assert_last_event::<T>(Event::<T>::ReservesAdded(caller, DOT, amount.into(), amount.into()).into());
    }

    reduce_reserves {
        let caller: T::AccountId = whitelisted_caller();
        let payer = T::Lookup::unlookup(caller.clone());
        transfer_initial_balance::<T>(caller.clone());
        let add_amount: u32 = 2000;
        let reduce_amount: u32 = 1000;
        assert_ok!(Loans::<T>::add_market(SystemOrigin::Root.into(), DOT, pending_market_mock::<T>(PDOT)));
        assert_ok!(Loans::<T>::active_market(SystemOrigin::Root.into(), DOT));
        assert_ok!(Loans::<T>::add_reserves(SystemOrigin::Root.into(), payer.clone(), DOT, add_amount.into()));
    }: _(SystemOrigin::Root, payer, DOT, reduce_amount.into())
    verify {
        assert_last_event::<T>(Event::<T>::ReservesReduced(caller, DOT, reduce_amount.into(), (add_amount-reduce_amount).into()).into());
    }

    accrue_interest {
        let alice: T::AccountId = account("Sample", 100, SEED);
        transfer_initial_balance::<T>(alice.clone());
        let deposit_amount: u32 = 200_000_000;
        let borrow_amount: u32 = 100_000_000;
        assert_ok!(Loans::<T>::add_market(SystemOrigin::Root.into(), DOT, pending_market_mock::<T>(PDOT)));
        assert_ok!(Loans::<T>::active_market(SystemOrigin::Root.into(), DOT));
        assert_ok!(Loans::<T>::mint(SystemOrigin::Signed(alice.clone()).into(), DOT, deposit_amount.into()));
        assert_ok!(Loans::<T>::collateral_asset(SystemOrigin::Signed(alice.clone()).into(), DOT, true));
        assert_ok!(Loans::<T>::borrow(SystemOrigin::Signed(alice).into(), DOT, borrow_amount.into()));
    }: {
        Loans::<T>::accrue_interest(6)?;
    }
    verify {
        assert_eq!(Loans::<T>::borrow_index(DOT), Rate::from_inner(1000000013318112633));
    }
}

impl_benchmark_test_suite!(Loans, crate::mock::new_test_ext(), crate::mock::Test);
