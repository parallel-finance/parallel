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

const KSM: CurrencyId = CurrencyId::KSM;
const INITIAL_AMOUNT: u128 = 100_000_000_000;
const SEED: u32 = 0;

fn initial_set_up<T: Config>(caller: T::AccountId) {
    let account_id = Loans::<T>::account_id();
    pallet_loans::ExchangeRate::<T>::insert(KSM, Rate::saturating_from_rational(2, 100));
    pallet_loans::BorrowIndex::<T>::insert(KSM, Rate::one());
    pallet_loans::CollateralFactor::<T>::insert(KSM, Ratio::from_percent(50));
    pallet_loans::CloseFactor::<T>::insert(KSM, Ratio::from_percent(50));
    pallet_loans::LiquidationIncentive::<T>::insert(KSM, Rate::saturating_from_rational(110, 100));
    <T as LoansConfig>::Currency::deposit(KSM, &caller, INITIAL_AMOUNT).unwrap();
    <T as LoansConfig>::Currency::deposit(KSM, &account_id, INITIAL_AMOUNT).unwrap();
}

benchmarks! {
    set_liquidation_incentive {
        let caller: T::AccountId = whitelisted_caller();
    }: {
        let _ = Loans::<T>::set_liquidation_incentive(
            SystemOrigin::Root.into(),
            KSM,
            Rate::max_value()
        );
    }

    mint {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T>(caller.clone());
        let amount = 100_000;
    }: {
        let _ = Loans::<T>::mint(SystemOrigin::Signed(caller.clone()).into(), KSM, amount);
        }
    verify {
        assert_eq!(
            <T as LoansConfig>::Currency::free_balance(KSM, &caller),
            INITIAL_AMOUNT - amount,
        );
    }

    borrow {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T>(caller.clone());
        let amount = 200_000_000;
        let borrowed_amount = 100_000_000;
        let currency_id: <T as ORMLOracleConfig<Instance1>>::OracleKey = T::convert(KSM);
        let price: <T as ORMLOracleConfig<Instance1>>::OracleValue = T::convert_price(PriceWithDecimal{ price: FixedU128::from(100_000), decimal: 12 });
        assert_ok!(ORMLOracle::<T, _>::feed_values(SystemOrigin::Root.into(),
            vec![(currency_id, price)]));
        assert_ok!(Loans::<T>::mint(SystemOrigin::Signed(caller.clone()).into(), KSM, amount));
        assert_ok!(Loans::<T>::collateral_asset(SystemOrigin::Signed(caller.clone()).into(), KSM, true));
    }: {
         let _ = Loans::<T>::borrow(SystemOrigin::Signed(caller.clone()).into(), KSM, borrowed_amount);
    }
    verify {
        assert_eq!(
            <T as LoansConfig>::Currency::free_balance(KSM, &caller),
            INITIAL_AMOUNT - amount + borrowed_amount,
        );
        assert_eq!(Loans::<T>::account_borrows(KSM, caller).principal, borrowed_amount);
    }

    redeem {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T>(caller.clone());
        assert_ok!(Loans::<T>::mint(SystemOrigin::Signed(caller.clone()).into(), KSM, 100_000_000));
        let amount = 100_000;
        let initial_balance = <T as LoansConfig>::Currency::free_balance(KSM, &Loans::<T>::account_id());
    }: {
         let _ = Loans::<T>::redeem(SystemOrigin::Signed(caller.clone()).into(), KSM, amount);
    }
    verify {
        assert_eq!(
            <T as LoansConfig>::Currency::free_balance(KSM, &Loans::<T>::account_id()),
            initial_balance - amount,
        );
    }

    redeem_all {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T>(caller.clone());
        assert_ok!(Loans::<T>::mint(SystemOrigin::Signed(caller.clone()).into(), KSM, 100_000_000));
        let deposits = Loans::<T>::account_deposits(KSM, caller.clone());
        let exchange_rate = Loans::<T>::exchange_rate(KSM);
        let redeem_amount = exchange_rate
            .checked_mul_int(deposits.voucher_balance)
            .ok_or(ArithmeticError::Overflow)?;
        let initial_balance = <T as LoansConfig>::Currency::free_balance(KSM, &Loans::<T>::account_id());
    }: {
         let _ = Loans::<T>::redeem_all(SystemOrigin::Signed(caller.clone()).into(), KSM);
    }
    verify {
        assert_eq!(
            <T as LoansConfig>::Currency::free_balance(KSM, &Loans::<T>::account_id()),
            initial_balance - redeem_amount,
        );
    }

    repay_borrow {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T>(caller.clone());
        let amount = 200_000_000;
        let borrowed_amount = 100_000_000;
        let repay_amount = 100;
        let currency_id: <T as ORMLOracleConfig<Instance1>>::OracleKey = T::convert(KSM);
        let price: <T as ORMLOracleConfig<Instance1>>::OracleValue = T::convert_price(PriceWithDecimal{ price: FixedU128::from(100_000), decimal: 12 });
        assert_ok!(ORMLOracle::<T, _>::feed_values(SystemOrigin::Root.into(),
            vec![(currency_id, price)]));
        assert_ok!(Loans::<T>::mint(SystemOrigin::Signed(caller.clone()).into(), KSM, INITIAL_AMOUNT));
        assert_ok!(Loans::<T>::collateral_asset(SystemOrigin::Signed(caller.clone()).into(), KSM, true));
        assert_ok!(Loans::<T>::borrow(SystemOrigin::Signed(caller.clone()).into(), KSM, borrowed_amount));
        let total_borrows = Loans::<T>::total_borrows(KSM);
    }: {
         let _ = Loans::<T>::repay_borrow(SystemOrigin::Signed(caller.clone()).into(), KSM, repay_amount);
    }
    verify {
        assert_eq!(
            Loans::<T>::total_borrows(KSM),
            total_borrows - repay_amount,
        );
    }

    repay_borrow_all {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T>(caller.clone());
        let borrowed_amount = 100_000_000;
        let currency_id: <T as ORMLOracleConfig<Instance1>>::OracleKey = T::convert(KSM);
        let price: <T as ORMLOracleConfig<Instance1>>::OracleValue = T::convert_price(PriceWithDecimal{ price: FixedU128::from(100_000), decimal: 12 });
        assert_ok!(ORMLOracle::<T, _>::feed_values(SystemOrigin::Root.into(),
            vec![(currency_id, price)]));
        assert_ok!(Loans::<T>::mint(SystemOrigin::Signed(caller.clone()).into(), KSM, INITIAL_AMOUNT));
        assert_ok!(Loans::<T>::collateral_asset(SystemOrigin::Signed(caller.clone()).into(), KSM, true));
        assert_ok!(Loans::<T>::borrow(SystemOrigin::Signed(caller.clone()).into(), KSM, borrowed_amount));
        let repay_amount = Loans::<T>::current_borrow_balance(&caller.clone(), &KSM)?;
        let total_borrows = Loans::<T>::total_borrows(KSM);
    }: {
         let _ = Loans::<T>::repay_borrow_all(SystemOrigin::Signed(caller.clone()).into(), KSM);
    }
    verify {
        assert_eq!(
            Loans::<T>::total_borrows(KSM),
            total_borrows - repay_amount,
        );
    }

    transfer_token {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T>(caller.clone());
        let to: T::AccountId = account("Sample", 100, SEED);
        let amount = 200_000_000;
        let initial_balance = <T as LoansConfig>::Currency::free_balance(KSM, &caller.clone());
    }: {
         let _ = Loans::<T>::transfer_token(SystemOrigin::Signed(caller.clone()).into(), to, KSM, amount);
    }
    verify {
        assert_eq!(
            <T as LoansConfig>::Currency::free_balance(KSM, &caller),
            initial_balance - amount,
        );
    }

    collateral_asset {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T>(caller.clone());
        assert_ok!(Loans::<T>::mint(SystemOrigin::Signed(caller.clone()).into(), KSM, INITIAL_AMOUNT));
    }: {
         let _ = Loans::<T>::collateral_asset(SystemOrigin::Signed(caller.clone()).into(), KSM, true);
    }
    verify {
        assert_eq!(
            pallet_loans::AccountDeposits::<T>::get(KSM, &caller.clone()).is_collateral,
            true,
        );
    }

    liquidate_borrow {
        let borrower: T::AccountId = whitelisted_caller();
        initial_set_up::<T>(borrower.clone());
        let caller: T::AccountId = account("Sample", 100, SEED);
        <T as LoansConfig>::Currency::deposit(KSM, &caller.clone(), INITIAL_AMOUNT).unwrap();
        let repay_amount = 2000;
        let borrowed_amount = 100_000_000;
        let currency_id: <T as ORMLOracleConfig<Instance1>>::OracleKey = T::convert(KSM);
        let price: <T as ORMLOracleConfig<Instance1>>::OracleValue = T::convert_price(PriceWithDecimal{ price: FixedU128::from(100_000), decimal: 12 });
        assert_ok!(ORMLOracle::<T, _>::feed_values(SystemOrigin::Root.into(),
            vec![(currency_id, price)]));
        assert_ok!(Loans::<T>::mint(SystemOrigin::Signed(borrower.clone()).into(), KSM, INITIAL_AMOUNT));
        assert_ok!(Loans::<T>::collateral_asset(SystemOrigin::Signed(borrower.clone()).into(), KSM, true));
        assert_ok!(Loans::<T>::borrow(SystemOrigin::Signed(borrower.clone()).into(), KSM, borrowed_amount));
        let total_borrows = pallet_loans::TotalBorrows::<T>::get(KSM);
    }: {
         let _ = Loans::<T>::liquidate_borrow(SystemOrigin::Signed(caller.clone()).into(), borrower, KSM, repay_amount, KSM);
    }
    verify {
        assert_eq!(
            pallet_loans::TotalBorrows::<T>::get(KSM),
            total_borrows - repay_amount,
        );
    }

    add_reserves {
        let caller: T::AccountId = whitelisted_caller();
        let payer = T::Lookup::unlookup(caller.clone());
        initial_set_up::<T>(caller.clone());
        let amount = 2000;
        let total_reserves = Loans::<T>::total_reserves(KSM);
    }: {
         let _ = Loans::<T>::add_reserves(SystemOrigin::Root.into(), payer, KSM, amount);
    }
    verify {
        assert_eq!(
            Loans::<T>::total_reserves(KSM),
            total_reserves + amount,
        );
    }

    reduce_reserves {
        let caller: T::AccountId = whitelisted_caller();
        let payer = T::Lookup::unlookup(caller.clone());
        initial_set_up::<T>(caller.clone());
        let amount = 2000;
        let amount1 = 1000;
        assert_ok!(Loans::<T>::add_reserves(SystemOrigin::Root.into(), payer.clone(), KSM, amount));
        let total_reserves = Loans::<T>::total_reserves(KSM);
    }: {
         let _ = Loans::<T>::reduce_reserves(SystemOrigin::Root.into(), payer, KSM, amount1);
    }
    verify {
        assert_eq!(
            Loans::<T>::total_reserves(KSM),
            total_reserves - amount1,
        );
    }

    set_rate_model {
        let caller: T::AccountId = whitelisted_caller();
    }: {
         let _ = Loans::<T>::set_rate_model(
            SystemOrigin::Root.into(),
            KSM,
            InterestRateModel::new_jump_model(
                Rate::saturating_from_rational(5, 100),
                Rate::saturating_from_rational(15, 100),
                Rate::saturating_from_rational(35, 100),
                Ratio::from_percent(80))
         );
    }
    verify {
        assert_eq!(
            Loans::<T>::currency_interest_model(KSM),
            InterestRateModel::new_jump_model(
                Rate::saturating_from_rational(5, 100),
                Rate::saturating_from_rational(15, 100),
                Rate::saturating_from_rational(35, 100),
                Ratio::from_percent(80))
        );
    }
}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test,);
