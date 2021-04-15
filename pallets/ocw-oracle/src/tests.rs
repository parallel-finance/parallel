// Copyright 2021 Parallel Finance Developer.
// This file is part of Parallel Finance.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
// http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate as ocw_oracle;
use crate::*;

use frame_support::{assert_noop, assert_ok, construct_runtime, parameter_types};
use sp_core::{
    offchain::{testing, OffchainWorkerExt, TransactionPoolExt},
    sr25519::{self, Signature},
    Pair, Public, H256,
};
use sp_keystore::{
    testing::KeyStore,
    {KeystoreExt, SyncCryptoStore},
};
use sp_runtime::{
    testing::{Header, TestXt},
    traits::{BlakeTwo256, Extrinsic as ExtrinsicT, IdentifyAccount, IdentityLookup, Verify},
    RuntimeAppPublic,
};
use sp_std::vec::Vec;
use std::sync::Arc;
type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Storage, Event<T>},
        OcwOracle: ocw_oracle::{Pallet, Call, Storage, Event<T>},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Runtime {
    type BaseCallFilter = ();
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = sp_core::sr25519::Public;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
}

parameter_types! {
    pub const PricePrecision: u8 = 3;
}

impl Config for Runtime {
    type AuthorityId = ocw_oracle::crypto::TestAuthId;
    type Call = Call;
    type Event = Event;
    type Time = Self;
    type PricePrecision = PricePrecision;
}

impl Time for Runtime {
    type Moment = u64;
    fn now() -> Self::Moment {
        0
    }
}

type Extrinsic = TestXt<Call, ()>;
type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
type AccountPublic = <Signature as Verify>::Signer;

fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}
/// Generate an account ID from seed.
fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

impl frame_system::offchain::SigningTypes for Runtime {
    type Public = <Signature as Verify>::Signer;
    type Signature = Signature;
}

impl<LocalCall> frame_system::offchain::SendTransactionTypes<LocalCall> for Runtime
where
    Call: From<LocalCall>,
{
    type OverarchingCall = Call;
    type Extrinsic = Extrinsic;
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Runtime
where
    Call: From<LocalCall>,
{
    fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
        call: Call,
        _public: <Signature as Verify>::Signer,
        _account: AccountId,
        index: u64,
    ) -> Option<(Call, <Extrinsic as ExtrinsicT>::SignaturePayload)> {
        Some((call, (index, ())))
    }
}

#[test]
fn get_price_works() {
    sp_io::TestExternalities::default().execute_with(|| {
        assert_eq!(OcwOracle::get_price(CurrencyId::BTC), None);
    });
}

#[test]
fn make_http_call_and_parse_result_works() {
    let (offchain, state) = testing::TestOffchainExt::new();
    let mut t = sp_io::TestExternalities::default();
    t.register_extension(OffchainWorkerExt::new(offchain));
    let url = "https://api.binance.com/api/v3/ticker/price?symbol=DOTUSDT";
    let response =br#"{"data": {"id": "polkadot","symbol": "DOT", "priceUsd":"45.17530000"},"timestamp":1616844616682}"#.to_vec();
    price_oracle_response(&mut state.write(), url, response);

    t.execute_with(|| {
        // when
        let resp_bytes = OcwOracle::fetch_from_remote(url).unwrap();
        let payload_detail: TickerPayloadDetail =
            get_ticker::<Runtime>(CurrencyId::DOT, DataSourceEnum::COINCAP, resp_bytes).unwrap();
        // then
        let right = TickerPayloadDetail {
            symbol: CurrencyId::DOT,
            data_source_enum: DataSourceEnum::COINCAP,
            price: 45175,
            timestamp: 1616844616682,
        };
        assert_eq!(payload_detail, right);
    });
}

#[test]
fn submit_signed_transaction_on_chain_works() {
    const PHRASE: &str =
        "news slush supreme milk chapter athlete soap sausage put clutch what kitten";
    let (offchain, offchain_state) = testing::TestOffchainExt::new();
    let (pool, pool_state) = testing::TestTransactionPoolExt::new();
    let keystore = KeyStore::new();
    SyncCryptoStore::sr25519_generate_new(
        &keystore,
        crate::crypto::Public::ID,
        Some(&format!("{}/hunter1", PHRASE)),
    )
    .unwrap();

    let mut t = sp_io::TestExternalities::default();
    t.register_extension(OffchainWorkerExt::new(offchain));
    t.register_extension(TransactionPoolExt::new(pool));
    t.register_extension(KeystoreExt(Arc::new(keystore)));

    let url = "https://api.binance.com/api/v3/ticker/price?symbol=DOTUSDT";
    let response =br#"{"data": {"id": "polkadot","symbol": "DOT", "priceUsd":"45.17530000"},"timestamp":1616844616682}"#.to_vec();
    price_oracle_response(&mut offchain_state.write(), url, response);

    t.execute_with(|| {
        // when
        let currency_id = CurrencyId::DOT;
        let resp_bytes = OcwOracle::fetch_from_remote(url).unwrap();
        let payload_detail: TickerPayloadDetail =
            get_ticker::<Runtime>(currency_id.clone(), DataSourceEnum::COINCAP, resp_bytes)
                .unwrap();

        let payload_list = vec![payload_detail];
        let blocknumber = 10u64;
        OcwOracle::offchain_signed_tx(payload_list.clone(), blocknumber).unwrap();
        // then
        let payload = Payload {
            index: blocknumber,
            list: payload_list,
        };

        let tx = pool_state.write().transactions.pop().unwrap();
        assert!(pool_state.read().transactions.is_empty());
        let tx = Extrinsic::decode(&mut &*tx).unwrap();
        assert_eq!(tx.signature.unwrap().0, 0);
        assert_eq!(tx.call, Call::OcwOracle(crate::Call::submit_price(payload)));
    });
}

#[test]
fn insert_initial_data_works() {
    let secret_alice = "//Alice";
    let secret_bob = "//Bob";

    let alice = get_from_seed::<sr25519::Public>(secret_alice);
    let bob = get_from_seed::<sr25519::Public>(secret_bob);
    let keystore = KeyStore::new();
    keystore
        .insert_unknown(crate::crypto::Public::ID, secret_alice, alice.as_ref())
        .expect("Insert key should succeed");
    keystore
        .insert_unknown(crate::crypto::Public::ID, secret_bob, bob.as_ref())
        .expect("Insert key should succeed");

    let (offchain, _offchain_state) = testing::TestOffchainExt::new();
    let (pool, _pool_state) = testing::TestTransactionPoolExt::new();
    let mut t = sp_io::TestExternalities::default();
    t.register_extension(OffchainWorkerExt::new(offchain));
    t.register_extension(TransactionPoolExt::new(pool));
    t.register_extension(KeystoreExt(Arc::new(keystore)));
    t.execute_with(|| {
        assert_eq!(OcwOracle::get_price(CurrencyId::BTC), None);
        assert_noop!(
            OcwOracle::insert_initial_data(Origin::signed(Default::default())),
            DispatchError::BadOrigin
        );
        assert_eq!(OcwOracle::ocw_oracle_aggregation_strategy(), None);
        assert_eq!(OcwOracle::ocw_oracle_currencies(), vec![]);
        assert_eq!(OcwOracle::ocw_oracle_data_source(), vec![]);
        assert_eq!(OcwOracle::members(), BTreeSet::new());
        assert_ok!(OcwOracle::insert_initial_data(Origin::root()));
        assert_eq!(
            OcwOracle::ocw_oracle_aggregation_strategy(),
            Some(AggregationStrategyEnum::MEDIAN)
        );
        assert_eq!(
            OcwOracle::ocw_oracle_data_source(),
            vec![
                DataSourceEnum::BINANCE,
                DataSourceEnum::COINBASE,
                DataSourceEnum::COINCAP,
            ]
        );
        assert_eq!(
            OcwOracle::ocw_oracle_currencies(),
            vec![
                CurrencyId::DOT,
                CurrencyId::KSM,
                CurrencyId::BTC,
                CurrencyId::USDT,
                CurrencyId::xDOT,
            ]
        );
        let alice: AccountId = get_account_id_from_seed::<sr25519::Public>(secret_alice);
        let bob: AccountId = get_account_id_from_seed::<sr25519::Public>(secret_bob);
        assert_ok!(OcwOracle::change_members(Origin::root(), vec![alice, bob]));
        assert_eq!(OcwOracle::members().len(), 2);
    });
}

#[test]
fn aggregate_price_works() {
    let keystore = KeyStore::new();
    let (offchain, _offchain_state) = testing::TestOffchainExt::new();
    let (pool, _pool_state) = testing::TestTransactionPoolExt::new();
    let mut t = sp_io::TestExternalities::default();
    t.register_extension(OffchainWorkerExt::new(offchain));
    t.register_extension(TransactionPoolExt::new(pool));
    t.register_extension(KeystoreExt(Arc::new(keystore)));
    t.execute_with(|| {
        let secret_alice = "//Alice";
        let secret_bob = "//Bob";
        let secret_charlie = "//Charlie";
        assert_ok!(OcwOracle::insert_initial_data(Origin::root()));
        let alice: AccountId = get_account_id_from_seed::<sr25519::Public>(secret_alice);
        let bob: AccountId = get_account_id_from_seed::<sr25519::Public>(secret_bob);
        let charlie: AccountId = get_account_id_from_seed::<sr25519::Public>(secret_charlie);
        assert_ok!(OcwOracle::change_members(Origin::root(), vec![alice, bob]));
        assert_eq!(System::block_number(), 0);
        System::set_block_number(11);
        let blocknumber = 10u64;
        //test alice submit price
        let alice_price1 = TickerPayloadDetail {
            symbol: CurrencyId::DOT,
            data_source_enum: DataSourceEnum::COINCAP,
            price: 45175,
            timestamp: 1616844616682,
        };
        let alice_price2 = TickerPayloadDetail {
            symbol: CurrencyId::DOT,
            data_source_enum: DataSourceEnum::BINANCE,
            price: 45176,
            timestamp: 1616844616682,
        };
        let alice_payload_list = vec![alice_price1, alice_price2];
        let alice_payload = Payload {
            index: blocknumber,
            list: alice_payload_list,
        };
        OcwOracle::submit_price(Origin::signed(alice), alice_payload)
            .expect("Insert payload should succeed");
        //test bob submit price
        let bob_price1 = TickerPayloadDetail {
            symbol: CurrencyId::DOT,
            data_source_enum: DataSourceEnum::COINCAP,
            price: 45177,
            timestamp: 1616844616682,
        };
        let bob_price2 = TickerPayloadDetail {
            symbol: CurrencyId::DOT,
            data_source_enum: DataSourceEnum::BINANCE,
            price: 45178,
            timestamp: 1616844616682,
        };
        let bob_payload_list = vec![bob_price1, bob_price2];
        let bob_payload = Payload {
            index: blocknumber,
            list: bob_payload_list,
        };
        OcwOracle::submit_price(Origin::signed(bob), bob_payload)
            .expect("Insert payload should succeed");
        //test charlie submit price
        let charlie_price1 = TickerPayloadDetail {
            symbol: CurrencyId::DOT,
            data_source_enum: DataSourceEnum::COINCAP,
            price: 45179,
            timestamp: 1616844616682,
        };
        let charlie_price2 = TickerPayloadDetail {
            symbol: CurrencyId::DOT,
            data_source_enum: DataSourceEnum::BINANCE,
            price: 45180,
            timestamp: 1616844616682,
        };
        let charlie_payload_list = vec![charlie_price1, charlie_price2];
        let charlie_payload = Payload {
            index: blocknumber,
            list: charlie_payload_list,
        };
        assert_noop!(
            OcwOracle::submit_price(Origin::signed(charlie), charlie_payload.clone()),
            Error::<Runtime>::NoPermission
        );
        assert_ok!(OcwOracle::change_members(
            Origin::root(),
            vec![alice, bob, charlie]
        ));
        OcwOracle::submit_price(Origin::signed(charlie), charlie_payload)
            .expect("Insert payload should succeed");

        // check on-chain round data
        let onchain_round = OcwOracle::ocw_oracle_round(CurrencyId::DOT);
        let right_round = Round {
            index: 10,
            provider: vec![alice, bob, charlie],
            combined: false,
            last_combined: 10,
        };
        assert_eq!(onchain_round, Some(right_round));

        //check median aggregation strategy
        OcwOracle::on_finalize(11);
        let mut price = 0;
        if let Some((p, _)) = OcwOracle::get_price(CurrencyId::DOT) {
            price = p;
        }
        // according to the price we mock submit above, the median price should be 45178
        assert_eq!(price, 45178);
    });
}

fn price_oracle_response(state: &mut testing::OffchainState, url: &str, response: Vec<u8>) {
    state.expect_request(testing::PendingRequest {
        method: "GET".into(),
        uri: url.into(),
        response: Some(response),
        sent: true,
        headers: vec![("User-Agent".into(), HTTP_HEADER_USER_AGENT.into())],
        ..Default::default()
    });
}
