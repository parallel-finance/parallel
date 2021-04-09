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
use primitives::{Amount, Balance, CurrencyId, PriceDetail, PriceFeeder, RATE_DECIMAL};
use sp_core::H256;
use sp_runtime::{testing::Header, traits::IdentityLookup, ModuleId};
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
        Balances: pallet_balances::{Pallet, Call, Storage, Event<T>},
        Currencies: orml_currencies::{Pallet, Call, Event<T>},
        Loans: loans::{Pallet, Storage, Call, Config, Event<T>},
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

pub type AccountId = u128;
pub type BlockNumber = u64;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;

pub const DOT: CurrencyId = CurrencyId::DOT;
pub const KSM: CurrencyId = CurrencyId::KSM;
pub const BTC: CurrencyId = CurrencyId::BTC;
pub const USDT: CurrencyId = CurrencyId::USDT;
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

impl Config for Runtime {
    type Event = Event;
    type Currency = Currencies;
    type ModuleId = LoansModuleId;
    type PriceFeeder = Self;
}

impl PriceFeeder for Runtime {
    fn get(_currency_id: &CurrencyId) -> Option<PriceDetail> {
        Some((1, 1))
    }
}

parameter_types! {
    pub const LoansModuleId: ModuleId = ModuleId(*b"par/loan");
}

pub struct ExtBuilder {
    endowed_accounts: Vec<(AccountId, CurrencyId, Balance)>,
}

impl Default for ExtBuilder {
    fn default() -> Self {
        Self {
            endowed_accounts: vec![
                (ALICE, DOT, 1000),
                (ALICE, BTC, 1000),
                (BOB, DOT, 1000),
                (BOB, BTC, 1000),
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
                CurrencyId::USDT,
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
                (CurrencyId::USDT, 5 * RATE_DECIMAL / 10),
                (CurrencyId::xDOT, 5 * RATE_DECIMAL / 10),
            ],
            liquidation_incentive: vec![
                (CurrencyId::DOT, 9 * RATE_DECIMAL / 10),
                (CurrencyId::KSM, 9 * RATE_DECIMAL / 10),
                (CurrencyId::BTC, 9 * RATE_DECIMAL / 10),
                (CurrencyId::USDT, 9 * RATE_DECIMAL / 10),
                (CurrencyId::xDOT, 9 * RATE_DECIMAL / 10),
            ],
            liquidation_threshold: vec![
                (CurrencyId::DOT, 8 * RATE_DECIMAL / 10),
                (CurrencyId::KSM, 8 * RATE_DECIMAL / 10),
                (CurrencyId::BTC, 8 * RATE_DECIMAL / 10),
                (CurrencyId::USDT, 9 * RATE_DECIMAL / 10),
                (CurrencyId::xDOT, 8 * RATE_DECIMAL / 10),
            ],
            close_factor: vec![
                (CurrencyId::DOT, 5 * RATE_DECIMAL / 10),
                (CurrencyId::KSM, 5 * RATE_DECIMAL / 10),
                (CurrencyId::BTC, 5 * RATE_DECIMAL / 10),
                (CurrencyId::USDT, 5 * RATE_DECIMAL / 10),
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
