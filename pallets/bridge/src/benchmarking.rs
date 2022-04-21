//! Benchmarks for Bridge Pallet

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Pallet as Bridge;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::assert_ok;
use frame_system::RawOrigin as SystemOrigin;
use primitives::{ChainId, CurrencyId};
use sp_runtime::traits::StaticLookup;

const ETHEREUM: ChainId = 3;

const USDT: CurrencyId = 1;
const EUSDT: CurrencyId = 1;

const EUSDT_CURRENCY: BridgeToken = BridgeToken {
    id: EUSDT,
    external: false,
    fee: 0,
    enable: true,
    out_cap: 1000000000000000,
    in_cap: 1000000000000000,
    out_amount: 0,
    in_amount: 0,
};

fn transfer_initial_balance<
    T: Config + pallet_assets::Config<AssetId = CurrencyId, Balance = Balance>,
>(
    caller: T::AccountId,
) {
    let account_id = T::Lookup::unlookup(caller.clone());
    pallet_assets::Pallet::<T>::force_create(SystemOrigin::Root.into(), USDT, account_id, true, 1)
        .ok();
    pallet_assets::Pallet::<T>::force_set_metadata(
        SystemOrigin::Root.into(),
        USDT,
        b"tether".to_vec(),
        b"USDT".to_vec(),
        6,
        true,
    )
    .ok();
    T::Assets::mint_into(USDT, &caller, dollar(100)).unwrap();
}

pub fn dollar(d: u128) -> u128 {
    d.saturating_mul(10_u128.pow(12))
}

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
    frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

benchmarks! {
    where_clause {
        where
            T: pallet_assets::Config<AssetId = CurrencyId, Balance = Balance>
        + pallet_balances::Config<Balance = Balance>
    }

    register_chain {
        let caller: T::AccountId = whitelisted_caller();
    }: _(SystemOrigin::Root, ETHEREUM)
    verify {
        assert_last_event::<T>(Event::ChainRegistered(ETHEREUM).into())
    }

    unregister_chain {
        let caller: T::AccountId = whitelisted_caller();
        assert_ok!(Bridge::<T>::register_chain(SystemOrigin::Root.into(), ETHEREUM));
    }: _(SystemOrigin::Root, ETHEREUM)
    verify {
        assert_last_event::<T>(Event::ChainRemoved(ETHEREUM).into())
    }

    register_bridge_token {
        let caller: T::AccountId = whitelisted_caller();
    }: _(SystemOrigin::Root, USDT, EUSDT_CURRENCY)
    verify {
        assert_last_event::<T>(
            Event::BridgeTokenRegistered(
                USDT,
                EUSDT,
                false,
                0,
                true,
                EUSDT_CURRENCY.out_cap,
                EUSDT_CURRENCY.out_amount,
                EUSDT_CURRENCY.in_cap,
                EUSDT_CURRENCY.in_amount,
            ).into()
        )
    }

    unregister_bridge_token {
        let caller: T::AccountId = whitelisted_caller();
        assert_ok!(Bridge::<T>::register_bridge_token(SystemOrigin::Root.into(), USDT, EUSDT_CURRENCY));
    }: _(SystemOrigin::Root, EUSDT)
    verify {
        assert_last_event::<T>(Event::BridgeTokenRemoved(USDT, EUSDT).into())
    }

    set_bridge_token_fee {
        let caller: T::AccountId = whitelisted_caller();
        assert_ok!(Bridge::<T>::register_bridge_token(SystemOrigin::Root.into(), USDT, EUSDT_CURRENCY));
    }: _(SystemOrigin::Root, EUSDT, dollar(1))
    verify {
        assert_last_event::<T>(Event::BridgeTokenFeeUpdated(EUSDT, dollar(1)).into())
    }

    set_bridge_token_status {
        let caller: T::AccountId = whitelisted_caller();
        assert_ok!(Bridge::<T>::register_bridge_token(SystemOrigin::Root.into(), USDT, EUSDT_CURRENCY));
    }: _(SystemOrigin::Root, EUSDT, false)
    verify {
        assert_last_event::<T>(Event::BridgeTokenStatusUpdated(EUSDT, false).into())
    }

    set_bridge_token_cap {
        let caller: T::AccountId = whitelisted_caller();
        assert_ok!(Bridge::<T>::register_bridge_token(SystemOrigin::Root.into(), USDT, EUSDT_CURRENCY));
    }: _(SystemOrigin::Root, EUSDT, BridgeType::BridgeIn, dollar(200))
    verify {
        assert_last_event::<T>(Event::BridgeTokenCapUpdated(EUSDT, BridgeType::BridgeIn, dollar(200)).into())
    }

    clean_cap_accumulated_value {
        let caller: T::AccountId = whitelisted_caller();
        assert_ok!(Bridge::<T>::register_bridge_token(SystemOrigin::Root.into(), USDT, EUSDT_CURRENCY));
    }: _(SystemOrigin::Root, EUSDT, BridgeType::BridgeIn)
    verify {
        assert_last_event::<T>(Event::BridgeTokenAccumulatedValueCleaned(EUSDT, BridgeType::BridgeIn).into())
    }

    teleport {
        let caller: T::AccountId = whitelisted_caller();
        assert_ok!(Bridge::<T>::register_chain(SystemOrigin::Root.into(), ETHEREUM));
        assert_ok!(Bridge::<T>::register_bridge_token(SystemOrigin::Root.into(), USDT, EUSDT_CURRENCY));
        transfer_initial_balance::<T>(caller.clone());
        let tele: TeleAccount = whitelisted_caller();
    }: _(SystemOrigin::Signed(caller.clone()), ETHEREUM, EUSDT, tele.clone(), dollar(50))
    verify {
        assert_last_event::<T>(Event::TeleportBurned(caller, ETHEREUM, 1, EUSDT, tele, dollar(50), dollar(0)).into())
    }

    materialize {
        let caller: T::AccountId = whitelisted_caller();
        assert_ok!(Bridge::<T>::register_chain(SystemOrigin::Root.into(), ETHEREUM));
        assert_ok!(Bridge::<T>::register_bridge_token(SystemOrigin::Root.into(), USDT, EUSDT_CURRENCY));
        transfer_initial_balance::<T>(caller.clone());
        let tele: TeleAccount = whitelisted_caller();
        assert_ok!(
            Bridge::<T>::teleport(
                SystemOrigin::Signed(caller).into(),
                ETHEREUM,
                EUSDT,
                tele,
                dollar(50)
            )
        );
        let recipient: T::AccountId = whitelisted_caller();
    }: _(SystemOrigin::Root, ETHEREUM, 0, EUSDT, recipient.clone(), dollar(10), true)
    verify {
        assert_last_event::<T>(Event::MaterializeMinted(ETHEREUM, 0, EUSDT, recipient, dollar(10)).into())
    }
}

impl_benchmark_test_suite!(Bridge, crate::mock::new_test_ext(), crate::mock::Test,);
