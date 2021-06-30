//! Loans pallet benchmarking.

#![cfg_attr(not(feature = "std"), no_std)]

mod mock;

use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::assert_ok;
use frame_system::RawOrigin as SystemOrigin;
use orml_oracle::Instance1;
use orml_oracle::{Config as ORMLOracleConfig, Pallet as ORMLOracle};
use orml_traits::MultiCurrency;
use pallet_loans::{Config as LoansConfig, InterestRateModel, Pallet as Loans};
use primitives::{CurrencyId, PriceWithDecimal, Rate, Ratio};
use sp_runtime::traits::{Bounded, One, StaticLookup};
use sp_runtime::{ArithmeticError, FixedPointNumber, FixedU128};
use sp_std::prelude::*;
use sp_std::vec;

pub struct Pallet<T: Config>(Loans<T>);
pub trait Config:
    LoansConfig + ORMLOracleConfig<Instance1> + CurrencyIdConvert<Self> + FixedU128Convert<Self>
{
}

pub trait CurrencyIdConvert<T: ORMLOracleConfig<Instance1>> {
    fn convert(currency_id: CurrencyId) -> <T as ORMLOracleConfig<Instance1>>::OracleKey;
}

impl<T: ORMLOracleConfig<Instance1>> CurrencyIdConvert<T> for T
where
    <T as ORMLOracleConfig<Instance1>>::OracleKey: From<CurrencyId>,
{
    fn convert(currency_id: CurrencyId) -> <T as ORMLOracleConfig<Instance1>>::OracleKey {
        currency_id.into()
    }
}

pub trait FixedU128Convert<T: ORMLOracleConfig<Instance1>> {
    fn convert_price(price: PriceWithDecimal) -> <T as ORMLOracleConfig<Instance1>>::OracleValue;
}

impl<T: ORMLOracleConfig<Instance1>> FixedU128Convert<T> for T
where
    <T as ORMLOracleConfig<Instance1>>::OracleValue: From<PriceWithDecimal>,
{
    fn convert_price(price: PriceWithDecimal) -> <T as ORMLOracleConfig<Instance1>>::OracleValue {
        price.into()
    }
}

const DOT: CurrencyId = CurrencyId::DOT;
const INITIAL_AMOUNT: u128 = 100_000_000_000;
const SEED: u32 = 0;

fn initial_set_up<T: Config>(caller: T::AccountId) {
    let account_id = Loans::<T>::account_id();
    pallet_loans::ExchangeRate::<T>::insert(DOT, Rate::saturating_from_rational(2, 100));
    pallet_loans::BorrowIndex::<T>::insert(DOT, Rate::one());
    pallet_loans::CollateralFactor::<T>::insert(DOT, Ratio::from_percent(50));
    pallet_loans::CloseFactor::<T>::insert(DOT, Ratio::from_percent(50));
    pallet_loans::LiquidationIncentive::<T>::insert(DOT, Rate::saturating_from_rational(110, 100));
    <T as LoansConfig>::Currency::deposit(DOT, &caller, INITIAL_AMOUNT).unwrap();
    <T as LoansConfig>::Currency::deposit(DOT, &account_id, INITIAL_AMOUNT).unwrap();
}

benchmarks! {
    set_liquidation_incentive {
        let caller: T::AccountId = whitelisted_caller();
    }: {
        let _ = Loans::<T>::set_liquidation_incentive(
            SystemOrigin::Root.into(),
            DOT,
            Rate::max_value()
        );
    }

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
        let currency_id: <T as ORMLOracleConfig<Instance1>>::OracleKey = T::convert(DOT);
        let price: <T as ORMLOracleConfig<Instance1>>::OracleValue = T::convert_price(PriceWithDecimal{ price: FixedU128::from(100_000), decimal: 12 });
        assert_ok!(ORMLOracle::<T, _>::feed_values(SystemOrigin::Root.into(),
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

    redeem {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T>(caller.clone());
        assert_ok!(Loans::<T>::mint(SystemOrigin::Signed(caller.clone()).into(), DOT, 100_000_000));
        let amount = 100_000;
        let initial_balance = <T as LoansConfig>::Currency::free_balance(DOT, &Loans::<T>::account_id());
    }: {
         let _ = Loans::<T>::redeem(SystemOrigin::Signed(caller.clone()).into(), DOT, amount);
    }
    verify {
        assert_eq!(
            <T as LoansConfig>::Currency::free_balance(DOT, &Loans::<T>::account_id()),
            initial_balance - amount,
        );
    }

    redeem_all {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T>(caller.clone());
        assert_ok!(Loans::<T>::mint(SystemOrigin::Signed(caller.clone()).into(), DOT, 100_000_000));
        let deposits = Loans::<T>::account_deposits(DOT, caller.clone());
        let exchange_rate = Loans::<T>::exchange_rate(DOT);
        let redeem_amount = exchange_rate
            .checked_mul_int(deposits.voucher_balance)
            .ok_or(ArithmeticError::Overflow)?;
        let initial_balance = <T as LoansConfig>::Currency::free_balance(DOT, &Loans::<T>::account_id());
    }: {
         let _ = Loans::<T>::redeem_all(SystemOrigin::Signed(caller.clone()).into(), DOT);
    }
    verify {
        assert_eq!(
            <T as LoansConfig>::Currency::free_balance(DOT, &Loans::<T>::account_id()),
            initial_balance - redeem_amount,
        );
    }

    repay_borrow {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T>(caller.clone());
        let amount = 200_000_000;
        let borrowed_amount = 100_000_000;
        let repay_amount = 100;
        let currency_id: <T as ORMLOracleConfig<Instance1>>::OracleKey = T::convert(DOT);
        let price: <T as ORMLOracleConfig<Instance1>>::OracleValue = T::convert_price(PriceWithDecimal{ price: FixedU128::from(100_000), decimal: 12 });
        assert_ok!(ORMLOracle::<T, _>::feed_values(SystemOrigin::Root.into(),
            vec![(currency_id, price)]));
        assert_ok!(Loans::<T>::mint(SystemOrigin::Signed(caller.clone()).into(), DOT, INITIAL_AMOUNT));
        assert_ok!(Loans::<T>::collateral_asset(SystemOrigin::Signed(caller.clone()).into(), DOT, true));
        assert_ok!(Loans::<T>::borrow(SystemOrigin::Signed(caller.clone()).into(), DOT, borrowed_amount));
        let total_borrows = Loans::<T>::total_borrows(DOT);
    }: {
         let _ = Loans::<T>::repay_borrow(SystemOrigin::Signed(caller.clone()).into(), DOT, repay_amount);
    }
    verify {
        assert_eq!(
            Loans::<T>::total_borrows(DOT),
            total_borrows - repay_amount,
        );
    }

    repay_borrow_all {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T>(caller.clone());
        let borrowed_amount = 100_000_000;
        let currency_id: <T as ORMLOracleConfig<Instance1>>::OracleKey = T::convert(DOT);
        let price: <T as ORMLOracleConfig<Instance1>>::OracleValue = T::convert_price(PriceWithDecimal{ price: FixedU128::from(100_000), decimal: 12 });
        assert_ok!(ORMLOracle::<T, _>::feed_values(SystemOrigin::Root.into(),
            vec![(currency_id, price)]));
        assert_ok!(Loans::<T>::mint(SystemOrigin::Signed(caller.clone()).into(), DOT, INITIAL_AMOUNT));
        assert_ok!(Loans::<T>::collateral_asset(SystemOrigin::Signed(caller.clone()).into(), DOT, true));
        assert_ok!(Loans::<T>::borrow(SystemOrigin::Signed(caller.clone()).into(), DOT, borrowed_amount));
        let repay_amount = Loans::<T>::current_borrow_balance(&caller.clone(), &DOT)?;
        let total_borrows = Loans::<T>::total_borrows(DOT);
    }: {
         let _ = Loans::<T>::repay_borrow_all(SystemOrigin::Signed(caller.clone()).into(), DOT);
    }
    verify {
        assert_eq!(
            Loans::<T>::total_borrows(DOT),
            total_borrows - repay_amount,
        );
    }

    transfer_token {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T>(caller.clone());
        let to: T::AccountId = account("Sample", 100, SEED);
        let amount = 200_000_000;
        let initial_balance = <T as LoansConfig>::Currency::free_balance(DOT, &caller.clone());
    }: {
         let _ = Loans::<T>::transfer_token(SystemOrigin::Signed(caller.clone()).into(), to, DOT, amount);
    }
    verify {
        assert_eq!(
            <T as LoansConfig>::Currency::free_balance(DOT, &caller),
            initial_balance - amount,
        );
    }

    collateral_asset {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T>(caller.clone());
        assert_ok!(Loans::<T>::mint(SystemOrigin::Signed(caller.clone()).into(), DOT, INITIAL_AMOUNT));
    }: {
         let _ = Loans::<T>::collateral_asset(SystemOrigin::Signed(caller.clone()).into(), DOT, true);
    }
    verify {
        assert_eq!(
            pallet_loans::AccountDeposits::<T>::get(DOT, &caller.clone()).is_collateral,
            true,
        );
    }

    liquidate_borrow {
        let borrower: T::AccountId = whitelisted_caller();
        initial_set_up::<T>(borrower.clone());
        let caller: T::AccountId = account("Sample", 100, SEED);
        <T as LoansConfig>::Currency::deposit(DOT, &caller.clone(), INITIAL_AMOUNT).unwrap();
        let repay_amount = 2000;
        let borrowed_amount = 100_000_000;
        let currency_id: <T as ORMLOracleConfig<Instance1>>::OracleKey = T::convert(DOT);
        let price: <T as ORMLOracleConfig<Instance1>>::OracleValue = T::convert_price(PriceWithDecimal{ price: FixedU128::from(100_000), decimal: 12 });
        assert_ok!(ORMLOracle::<T, _>::feed_values(SystemOrigin::Root.into(),
            vec![(currency_id, price)]));
        assert_ok!(Loans::<T>::mint(SystemOrigin::Signed(borrower.clone()).into(), DOT, INITIAL_AMOUNT));
        assert_ok!(Loans::<T>::collateral_asset(SystemOrigin::Signed(borrower.clone()).into(), DOT, true));
        assert_ok!(Loans::<T>::borrow(SystemOrigin::Signed(borrower.clone()).into(), DOT, borrowed_amount));
        let total_borrows = pallet_loans::TotalBorrows::<T>::get(DOT);
    }: {
         let _ = Loans::<T>::liquidate_borrow(SystemOrigin::Signed(caller.clone()).into(), borrower, DOT, repay_amount, DOT);
    }
    verify {
        assert_eq!(
            pallet_loans::TotalBorrows::<T>::get(DOT),
            total_borrows - repay_amount,
        );
    }

    add_reserves {
        let caller: T::AccountId = whitelisted_caller();
        let payer = T::Lookup::unlookup(caller.clone());
        initial_set_up::<T>(caller.clone());
        let amount = 2000;
        let total_reserves = Loans::<T>::total_reserves(DOT);
    }: {
         let _ = Loans::<T>::add_reserves(SystemOrigin::Root.into(), payer, DOT, amount);
    }
    verify {
        assert_eq!(
            Loans::<T>::total_reserves(DOT),
            total_reserves + amount,
        );
    }

    reduce_reserves {
        let caller: T::AccountId = whitelisted_caller();
        let payer = T::Lookup::unlookup(caller.clone());
        initial_set_up::<T>(caller.clone());
        let amount = 2000;
        let amount1 = 1000;
        assert_ok!(Loans::<T>::add_reserves(SystemOrigin::Root.into(), payer.clone(), DOT, amount));
        let total_reserves = Loans::<T>::total_reserves(DOT);
    }: {
         let _ = Loans::<T>::reduce_reserves(SystemOrigin::Root.into(), payer, DOT, amount1);
    }
    verify {
        assert_eq!(
            Loans::<T>::total_reserves(DOT),
            total_reserves - amount1,
        );
    }

    set_rate_model {
        let caller: T::AccountId = whitelisted_caller();
    }: {
         let _ = Loans::<T>::set_rate_model(
            SystemOrigin::Root.into(),
            DOT,
            InterestRateModel::new_jump_model(
                Rate::saturating_from_rational(5, 100),
                Rate::saturating_from_rational(15, 100),
                Rate::saturating_from_rational(35, 100),
                Ratio::from_percent(80))
         );
    }
    verify {
        assert_eq!(
            Loans::<T>::currency_interest_model(DOT),
            InterestRateModel::new_jump_model(
                Rate::saturating_from_rational(5, 100),
                Rate::saturating_from_rational(15, 100),
                Rate::saturating_from_rational(35, 100),
                Ratio::from_percent(80))
        );
    }
}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test,);
