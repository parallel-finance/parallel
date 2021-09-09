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
use pallet_loans::{JumpModel, Market, MarketState};
use pallet_prices::{Config as PriceConfig, Pallet as Prices};
use primitives::{CurrencyId, Price, PriceWithDecimal, Rate, Ratio, TokenSymbol};
use sp_runtime::traits::{One, StaticLookup};
use sp_runtime::{ArithmeticError, FixedPointNumber, FixedU128};
use sp_std::prelude::*;
use sp_std::vec;

pub struct Pallet<T: Config>(Loans<T>);
pub trait Config:
    LoansConfig
    + ORMLOracleConfig<Instance1>
    + CurrencyIdConvert<Self>
    + FixedU128Convert<Self>
    + PriceConfig
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

const DOT: CurrencyId = CurrencyId::Token(TokenSymbol::DOT);
const KSM: CurrencyId = CurrencyId::Token(TokenSymbol::KSM);
const INITIAL_AMOUNT: u128 = 100_000_000_000;
const SEED: u32 = 0;
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

fn initial_set_up<T: Config>() {
    let account_id = Loans::<T>::account_id();
    pallet_loans::ExchangeRate::<T>::insert(DOT, Rate::saturating_from_rational(2, 100));
    pallet_loans::ExchangeRate::<T>::insert(KSM, Rate::saturating_from_rational(2, 100));
    pallet_loans::BorrowIndex::<T>::insert(DOT, Rate::one());
    pallet_loans::BorrowIndex::<T>::insert(KSM, Rate::one());

    <T as LoansConfig>::Currency::deposit(DOT, &account_id, INITIAL_AMOUNT).unwrap();
    <T as LoansConfig>::Currency::deposit(KSM, &account_id, INITIAL_AMOUNT).unwrap();

    pallet_loans::Markets::<T>::insert(DOT, MARKET_MOCK);
    pallet_loans::Markets::<T>::insert(KSM, MARKET_MOCK);
}

fn transfer_initial_balance<T: Config>(caller: T::AccountId) {
    <T as LoansConfig>::Currency::deposit(DOT, &caller, INITIAL_AMOUNT).unwrap();
    <T as LoansConfig>::Currency::deposit(KSM, &caller, INITIAL_AMOUNT).unwrap();
}

benchmarks! {
    active_market {
    }: {
        let _ = Loans::<T>::active_market(
            SystemOrigin::Root.into(),
            DOT,
        );
    }

    add_market {
    }: {
        let _ = Loans::<T>::add_market(
            SystemOrigin::Root.into(),
            CurrencyId::Token(TokenSymbol::DOT),
            PENDING_MARKET_MOCK
        );
    }

    update_market {
    }: {
        let _ = Loans::<T>::update_market(
            SystemOrigin::Root.into(),
            DOT,
            PENDING_MARKET_MOCK
        );
    }

    mint {
        let caller: T::AccountId = whitelisted_caller();
        initial_set_up::<T>();
        transfer_initial_balance::<T>(caller.clone());
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
        initial_set_up::<T>();
        transfer_initial_balance::<T>(caller.clone());
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
        initial_set_up::<T>();
        transfer_initial_balance::<T>(caller.clone());
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
        initial_set_up::<T>();
        transfer_initial_balance::<T>(caller.clone());
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
        initial_set_up::<T>();
        transfer_initial_balance::<T>(caller.clone());
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
        initial_set_up::<T>();
        transfer_initial_balance::<T>(caller.clone());
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
        initial_set_up::<T>();
        transfer_initial_balance::<T>(caller.clone());
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
        initial_set_up::<T>();
        transfer_initial_balance::<T>(caller.clone());
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
        let alice: T::AccountId = account("Sample", 100, SEED);
        initial_set_up::<T>();
        let bob: T::AccountId = account("Sample", 101, SEED);
        transfer_initial_balance::<T>(alice.clone());
        transfer_initial_balance::<T>(bob.clone());
        assert_ok!(Loans::<T>::mint(SystemOrigin::Signed(bob.clone()).into(), KSM, 200_000_000_00));
        assert_ok!(Loans::<T>::mint(SystemOrigin::Signed(alice.clone()).into(), DOT, 200_000_000_00));
        assert_ok!(Loans::<T>::collateral_asset(SystemOrigin::Signed(alice.clone()).into(), DOT, true));
        assert_ok!(Loans::<T>::borrow(SystemOrigin::Signed(alice.clone()).into(), KSM, 100_000_000_00));
                let price = PriceWithDecimal {
                    price: Price::saturating_from_integer(2),
                    decimal: 8
                };
        assert_ok!(Prices::<T>::set_price(SystemOrigin::Root.into(), KSM, price));
    }: {
         let _ = Loans::<T>::liquidate_borrow(SystemOrigin::Signed(bob.clone()).into(), alice.clone(), KSM, 50_000_000_00, DOT);
    }
    verify {
         assert_eq!(
            Loans::<T>::account_borrows(KSM, alice).principal,
            50_000_000_00
        );
        assert_eq!(
            Loans::<T>::exchange_rate(DOT)
                .saturating_mul_int(Loans::<T>::account_deposits(DOT, bob).voucher_balance),
            11_000_000_000,
        );
    }

    add_reserves {
        let caller: T::AccountId = whitelisted_caller();
        let payer = T::Lookup::unlookup(caller.clone());
        initial_set_up::<T>();
        transfer_initial_balance::<T>(caller.clone());
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
        initial_set_up::<T>();
        transfer_initial_balance::<T>(caller.clone());
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
}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test,);
