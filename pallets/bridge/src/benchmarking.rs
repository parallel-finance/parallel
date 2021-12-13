//! Benchmarks for Bridge Pallet

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Pallet as Bridge;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_support::assert_ok;
use frame_support::dispatch::UnfilteredDispatchable;
use frame_system::RawOrigin as SystemOrigin;
use primitives::{ChainId, CurrencyId};
use sp_runtime::traits::StaticLookup;

const ETH: ChainId = 1;

const HKO: CurrencyId = 0;
const EHKO: CurrencyId = 0;

const EHKO_CURRENCY: BridgeToken = BridgeToken {
    id: EHKO,
    external: false,
    fee: 0,
};

fn transfer_initial_balance<T: Config + pallet_balances::Config<Balance = Balance>>(
    caller: T::AccountId,
) {
    let account_id = T::Lookup::unlookup(caller);
    // T::Assets::mint_into(USDT, &caller, INITIAL_AMOUNT.into()).unwrap();
    pallet_balances::Pallet::<T>::set_balance(
        SystemOrigin::Root.into(),
        account_id,
        dollar(100),
        dollar(0),
    )
    .unwrap();
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

    set_vote_threshold {
        let caller: T::AccountId = whitelisted_caller();
    }: _(SystemOrigin::Root, 1)
    verify {
        assert_last_event::<T>(Event::VoteThresholdChanged(1).into())
    }

    register_chain {
        let caller: T::AccountId = whitelisted_caller();
    }: _(SystemOrigin::Root, ETH)
    verify {
        assert_last_event::<T>(Event::ChainRegistered(ETH).into())
    }

    unregister_chain {
        let caller: T::AccountId = whitelisted_caller();
        assert_ok!(Bridge::<T>::register_chain(SystemOrigin::Root.into(), ETH));
    }: _(SystemOrigin::Root, ETH)
    verify {
        assert_last_event::<T>(Event::ChainRemoved(ETH).into())
    }

    register_bridge_token {
        let caller: T::AccountId = whitelisted_caller();
    }: _(SystemOrigin::Root, HKO, EHKO_CURRENCY)
    verify {
        assert_last_event::<T>(Event::BridgeTokenRegistered(HKO, EHKO).into())
    }

    unregister_bridge_token {
        let caller: T::AccountId = whitelisted_caller();
        assert_ok!(Bridge::<T>::register_bridge_token(SystemOrigin::Root.into(), HKO, EHKO_CURRENCY));
    }: _(SystemOrigin::Root, EHKO)
    verify {
        assert_last_event::<T>(Event::BridgeTokenRemoved(HKO, EHKO).into())
    }

    set_bridge_token_fee {
        let caller: T::AccountId = whitelisted_caller();
        assert_ok!(Bridge::<T>::register_bridge_token(SystemOrigin::Root.into(), HKO, EHKO_CURRENCY));
    }: _(SystemOrigin::Root, EHKO, dollar(1))
    verify {
        assert_last_event::<T>(Event::BridgeTokenFeeChanged(EHKO, dollar(1)).into())
    }

    teleport {
        let caller: T::AccountId = whitelisted_caller();
        assert_ok!(Bridge::<T>::register_chain(SystemOrigin::Root.into(), ETH));
        assert_ok!(Bridge::<T>::register_bridge_token(SystemOrigin::Root.into(), HKO, EHKO_CURRENCY));
        transfer_initial_balance::<T>(caller.clone());
        let tele: TeleAccount = whitelisted_caller();
    }: _(SystemOrigin::Signed(caller), ETH, EHKO, tele, dollar(50))

    materialize {
        let caller: T::AccountId = whitelisted_caller();
        assert_ok!(Bridge::<T>::register_chain(T::RootOperatorOrigin::successful_origin(), ETH));
        assert_ok!(Bridge::<T>::register_bridge_token(T::RootOperatorOrigin::successful_origin(), HKO, EHKO_CURRENCY));
        transfer_initial_balance::<T>(caller.clone());
        let tele: TeleAccount = whitelisted_caller();
        assert_ok!(
            Bridge::<T>::teleport(
                SystemOrigin::Signed(caller).into(),
                ETH,
                0,
                tele,
                dollar(50)
            )
        );
        let recipient: T::AccountId = whitelisted_caller();
        let call = Call::<T>::materialize {
            src_id: ETH,
            src_nonce: 1,
            bridge_token_id: EHKO,
            to: recipient,
            amount: dollar(10),
            favour: true,
        };
        let caller2: T::AccountId = whitelisted_caller();
    }: { call.dispatch_bypass_filter(SystemOrigin::Signed(caller2).into())? }
}
