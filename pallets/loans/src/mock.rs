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

mod loans {
    pub use super::super::*;
}
use loans::*;

use frame_support::{construct_runtime, parameter_types};

use orml_traits::parameter_type_with_key;
use primitives::{Amount, Balance, CurrencyId, RATE_DECIMAL};
use sp_core::{
    sr25519::{self, Signature},
    Pair, Public, H256,
};
use sp_runtime::{
    testing::{Header, TestXt},
    traits::{Extrinsic as ExtrinsicT, IdentifyAccount, IdentityLookup, Verify},
    ModuleId,
};
use sp_std::vec::Vec;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Storage, Config, Event<T>},
        Tokens: orml_tokens::{Pallet, Storage, Event<T>, Config<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        Currencies: orml_currencies::{Pallet, Call, Event<T>},
        Loans: loans::{Pallet, Storage, Call, Config, Event<T>},
        OcwOracle: pallet_ocw_oracle::{Pallet, Call, Storage, Event<T>, ValidateUnsigned},
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
    type Hashing = ::sp_runtime::traits::BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
}

pub type BlockNumber = u64;

pub const DOT: CurrencyId = CurrencyId::DOT;
pub const KSM: CurrencyId = CurrencyId::KSM;
pub const BTC: CurrencyId = CurrencyId::BTC;
pub const USDC: CurrencyId = CurrencyId::USDC;
pub const NATIVE: CurrencyId = CurrencyId::Native;

parameter_type_with_key! {
    pub ExistentialDeposits: |_currency_id: CurrencyId| -> Balance {
        Default::default()
    };
}

impl orml_tokens::Config for Runtime {
    type Event = Event;
    type Balance = Balance;
    type Amount = Amount;
    type CurrencyId = CurrencyId;
    type WeightInfo = ();
    type OnDust = ();
    type ExistentialDeposits = ExistentialDeposits;
}

parameter_types! {
    pub const GetNativeCurrencyId: CurrencyId = NATIVE;
}

impl orml_currencies::Config for Runtime {
    type Event = Event;
    type MultiCurrency = Tokens;
    type NativeCurrency =
        orml_currencies::BasicCurrencyAdapter<Runtime, Balances, Amount, BlockNumber>;
    type GetNativeCurrencyId = GetNativeCurrencyId;
    type WeightInfo = ();
}

parameter_types! {
    pub const ExistentialDeposit: Balance = 1;
    pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for Runtime {
    type MaxLocks = MaxLocks;
    type Balance = Balance;
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub const PricePrecision: u8 = 3;
}

impl pallet_ocw_oracle::Config for Runtime {
    type AuthorityId = pallet_ocw_oracle::crypto::TestAuthId;
    type Call = Call;
    type Event = Event;
    type PricePrecision = PricePrecision;
}

type Extrinsic = TestXt<Call, ()>;
type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
type AccountPublic = <Signature as Verify>::Signer;

pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}
/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
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

impl Config for Runtime {
    type Event = Event;
    type Currency = Currencies;
    type ModuleId = LoansModuleId;
}

parameter_types! {
    pub const LoansModuleId: ModuleId = ModuleId(*b"par/loan");
}

pub struct ExtBuilder {
    endowed_accounts: Vec<(AccountId, CurrencyId, Balance)>,
}

impl Default for ExtBuilder {
    fn default() -> Self {
        let alice: AccountId = get_account_id_from_seed::<sr25519::Public>("Alice");
        let bob: AccountId = get_account_id_from_seed::<sr25519::Public>("Bob");
        Self {
            endowed_accounts: vec![
                (alice, DOT, 1000),
                (alice, BTC, 1000),
                (bob, DOT, 1000),
                (bob, BTC, 1000),
            ],
        }
    }
}

impl ExtBuilder {
    pub fn build(self) -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::default()
            .build_storage::<Runtime>()
            .unwrap();

        orml_tokens::GenesisConfig::<Runtime> {
            endowed_accounts: self.endowed_accounts.clone(),
        }
        .assimilate_storage(&mut t)
        .unwrap();

        loans::GenesisConfig {
            currencies: vec![
                CurrencyId::DOT,
                CurrencyId::KSM,
                CurrencyId::BTC,
                CurrencyId::USDC,
                CurrencyId::xDOT,
            ],
            // total_supply: 100 * TOKEN_DECIMAL, // 100
            // total_borrows: 50 * TOKEN_DECIMAL, // 50
            borrow_index: RATE_DECIMAL,                 // 1
            exchange_rate: 2 * RATE_DECIMAL / 100,      // 0.02
            base_rate: 2 * RATE_DECIMAL / 100,          // 0.02
            multiplier_per_year: 1 * RATE_DECIMAL / 10, // 0.1
            jump_muiltiplier: 11 * RATE_DECIMAL / 10,   // 1.1
            kink: 8 * RATE_DECIMAL / 10,                // 0.8
            collateral_rate: vec![
                (CurrencyId::DOT, 5 * RATE_DECIMAL / 10),
                (CurrencyId::KSM, 5 * RATE_DECIMAL / 10),
                (CurrencyId::BTC, 5 * RATE_DECIMAL / 10),
                (CurrencyId::USDC, 5 * RATE_DECIMAL / 10),
                (CurrencyId::xDOT, 5 * RATE_DECIMAL / 10),
            ],
            liquidation_incentive: vec![
                (CurrencyId::DOT, 9 * RATE_DECIMAL / 10),
                (CurrencyId::KSM, 9 * RATE_DECIMAL / 10),
                (CurrencyId::BTC, 9 * RATE_DECIMAL / 10),
                (CurrencyId::USDC, 9 * RATE_DECIMAL / 10),
                (CurrencyId::xDOT, 9 * RATE_DECIMAL / 10),
            ],
            liquidation_threshold: vec![
                (CurrencyId::DOT, 8 * RATE_DECIMAL / 10),
                (CurrencyId::KSM, 8 * RATE_DECIMAL / 10),
                (CurrencyId::BTC, 8 * RATE_DECIMAL / 10),
                (CurrencyId::USDC, 9 * RATE_DECIMAL / 10),
                (CurrencyId::xDOT, 8 * RATE_DECIMAL / 10),
            ],
            close_factor: vec![
                (CurrencyId::DOT, 5 * RATE_DECIMAL / 10),
                (CurrencyId::KSM, 5 * RATE_DECIMAL / 10),
                (CurrencyId::BTC, 5 * RATE_DECIMAL / 10),
                (CurrencyId::USDC, 5 * RATE_DECIMAL / 10),
                (CurrencyId::xDOT, 5 * RATE_DECIMAL / 10),
            ],
        }
        .assimilate_storage::<Runtime>(&mut t)
        .unwrap();

        // t.into()
        let mut ext = sp_io::TestExternalities::new(t);
        ext.execute_with(|| System::set_block_number(1));
        ext
    }
}
