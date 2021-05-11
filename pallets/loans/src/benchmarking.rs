//! Benchmarking setup for pallet-template
use super::*;

use crate::Pallet as Loans;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::assert_ok;
use frame_system::RawOrigin as SystemOrigin;

pub const DOT: CurrencyId = CurrencyId::DOT;
pub const INITIAL_AMOUNT: u128 = 10000;

fn initial_set_up<T: Config>(caller: T::AccountId) {
    let account_id = Loans::<T>::account_id();
    T::Currency::deposit(DOT, &caller, INITIAL_AMOUNT).unwrap();
    T::Currency::deposit(DOT, &account_id, INITIAL_AMOUNT).unwrap();
}

benchmarks! {
    mint {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T>(caller.clone());
        let amount = 100;
    }: _(SystemOrigin::Signed(caller.clone()), DOT, amount)
    verify {
        assert_eq!(
            <T as Config>::Currency::free_balance(DOT, &caller),
            INITIAL_AMOUNT - amount,
        );
    }

    borrow {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T>(caller.clone());
        let amount = 200;
        let borrowed_amount = 100;
        assert_ok!(Loans::<T>::mint(SystemOrigin::Signed(caller.clone()).into(), DOT, amount));
        assert_ok!(Loans::<T>::collateral_asset(SystemOrigin::Signed(caller.clone()).into(), DOT, true));
    }: _(SystemOrigin::Signed(caller.clone()), DOT, borrowed_amount)
    verify {
        assert_eq!(
            <T as Config>::Currency::free_balance(DOT, &caller),
            INITIAL_AMOUNT - borrowed_amount,
        );
        assert_eq!(Loans::<T>::account_borrows(DOT, caller).principal, borrowed_amount);
    }

    redeem {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T>(caller.clone());
        <AccountCollateral<T>>::insert(DOT, caller.clone(), 100_000);
        <TotalSupply<T>>::insert(DOT, 100_000);
        let amount = 100;
    }: _(SystemOrigin::Signed(caller.clone()), DOT, amount)
    verify {
        assert_eq!(
            <T as Config>::Currency::free_balance(DOT, &Loans::<T>::account_id()),
            INITIAL_AMOUNT - amount,
        );
    }
}

impl_benchmark_test_suite!(
    Loans,
    crate::mock::ExtBuilder::default().build(),
    crate::mock::Runtime,
);
