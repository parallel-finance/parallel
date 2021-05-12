//! Loans pallet benchmarking.

#![cfg_attr(not(feature = "std"), no_std)]

mod mock;

use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::assert_ok;
use frame_system::RawOrigin as SystemOrigin;
use orml_oracle::Instance1;
use orml_oracle::{Config as ORMOracleConfig, Pallet as ORMOracle};
use orml_traits::MultiCurrency;
use pallet_loans::{Config as LoansConfig, Pallet as Loans};
use primitives::{CurrencyId, Rate, Ratio};
use sp_runtime::{FixedPointNumber, FixedU128};
use sp_std::prelude::*;
use sp_std::vec;

pub struct Pallet<T: Config>(Loans<T>);
pub trait Config:
    LoansConfig + ORMOracleConfig<Instance1> + CurrencyIdConvert<Self> + FixedU128Convert<Self>
{
}

pub trait CurrencyIdConvert<T: ORMOracleConfig<Instance1>> {
    fn convert(currency_id: CurrencyId) -> <T as ORMOracleConfig<Instance1>>::OracleKey;
}

impl<T: ORMOracleConfig<Instance1>> CurrencyIdConvert<T> for T
where
    <T as ORMOracleConfig<Instance1>>::OracleKey: From<CurrencyId>,
{
    fn convert(currency_id: CurrencyId) -> <T as ORMOracleConfig<Instance1>>::OracleKey {
        currency_id.into()
    }
}

pub trait FixedU128Convert<T: ORMOracleConfig<Instance1>> {
    fn convert_price(price: FixedU128) -> <T as ORMOracleConfig<Instance1>>::OracleValue;
}

impl<T: ORMOracleConfig<Instance1>> FixedU128Convert<T> for T
where
    <T as ORMOracleConfig<Instance1>>::OracleValue: From<FixedU128>,
{
    fn convert_price(price: FixedU128) -> <T as ORMOracleConfig<Instance1>>::OracleValue {
        price.into()
    }
}

const DOT: CurrencyId = CurrencyId::DOT;
const INITIAL_AMOUNT: u128 = 100_000_000_000;

fn initial_set_up<T: Config>(caller: T::AccountId) {
    let account_id = Loans::<T>::account_id();
    pallet_loans::ExchangeRate::<T>::insert(DOT, Rate::saturating_from_rational(2, 100));
    pallet_loans::CollateralFactor::<T>::insert(DOT, Ratio::from_percent(50));
    <T as LoansConfig>::Currency::deposit(DOT, &caller, INITIAL_AMOUNT).unwrap();
    <T as LoansConfig>::Currency::deposit(DOT, &account_id, INITIAL_AMOUNT).unwrap();
}

benchmarks! {
    mint {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T>(caller.clone());
        let amount = 100_000;
    }: {
        let _ = Loans::<T>::mint(SystemOrigin::Signed(caller.clone()).into(), DOT, amount);
        }
    verify {
        assert_eq!(
            <T as LoansConfig>::Currency::free_balance(DOT, &caller),
            INITIAL_AMOUNT - amount,
        );
    }

    borrow {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T>(caller.clone());
        let amount = 200_000_000;
        let borrowed_amount = 100_000_000;
        let currency_id: <T as ORMOracleConfig<Instance1>>::OracleKey = T::convert(DOT);
        let price: <T as ORMOracleConfig<Instance1>>::OracleValue = T::convert_price(FixedU128::from(100_000));
        assert_ok!(ORMOracle::<T, _>::feed_values(SystemOrigin::Root.into(),
            vec![(currency_id, price)]));
        assert_ok!(Loans::<T>::mint(SystemOrigin::Signed(caller.clone()).into(), DOT, amount));
        assert_ok!(Loans::<T>::collateral_asset(SystemOrigin::Signed(caller.clone()).into(), DOT, true));
    }: {
         let _ = Loans::<T>::borrow(SystemOrigin::Signed(caller.clone()).into(), DOT, borrowed_amount);
    }
    verify {
        assert_eq!(
            <T as LoansConfig>::Currency::free_balance(DOT, &caller),
            INITIAL_AMOUNT - amount + borrowed_amount,
        );
        assert_eq!(Loans::<T>::account_borrows(DOT, caller).principal, borrowed_amount);
    }
}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test,);
