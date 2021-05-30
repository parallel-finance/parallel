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

use frame_support::{construct_runtime, parameter_types, PalletId};
use frame_system::EnsureRoot;
use lazy_static::lazy_static;
use orml_traits::parameter_type_with_key;
use primitives::{
    Amount, Balance, CurrencyId, Price, PriceDetail, PriceFeeder, Rate, Ratio, TOKEN_DECIMAL,
};
use sp_core::H256;
use sp_runtime::traits::One;
use sp_runtime::{testing::Header, traits::IdentityLookup};
use sp_std::vec::Vec;
use std::{collections::HashMap, sync::Mutex};

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
pub const USDT: CurrencyId = CurrencyId::USDT;
pub const XDOT: CurrencyId = CurrencyId::xDOT;
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
    type MaxLocks = MaxLocks;
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

lazy_static! {
    pub static ref MOCK_PRICE_FEEDER: Mutex<HashMap<CurrencyId, Option<PriceDetail>>> = {
        Mutex::new(
            vec![DOT, KSM, USDT, XDOT]
                .iter()
                .map(|&x| (x, Some((Price::saturating_from_integer(1), 1))))
                .collect(),
        )
    };
}

impl MOCK_PRICE_FEEDER {
    pub fn set_price(currency_id: CurrencyId, price: Price) {
        MOCK_PRICE_FEEDER
            .lock()
            .unwrap()
            .insert(currency_id, Some((price, 1u64)));
    }

    pub fn reset() {
        for (_, val) in MOCK_PRICE_FEEDER.lock().unwrap().iter_mut() {
            *val = Some((Price::saturating_from_integer(1), 1u64));
        }
    }
}

impl PriceFeeder for MOCK_PRICE_FEEDER {
    fn get_price(currency_id: &CurrencyId) -> Option<PriceDetail> {
        *MOCK_PRICE_FEEDER.lock().unwrap().get(currency_id).unwrap()
    }
}

impl Config for Runtime {
    type Event = Event;
    type Currency = Currencies;
    type PalletId = LoansPalletId;
    type BlockPerYear = BlockPerYear;
    type PriceFeeder = MOCK_PRICE_FEEDER;
    type ReserveOrigin = EnsureRoot<AccountId>;
    type UpdateOrigin = EnsureRoot<AccountId>;
    type WeightInfo = ();
}

parameter_types! {
    pub const LoansPalletId: PalletId = PalletId(*b"par/loan");
    pub const BlockPerYear: u128 = 5256000;
}

pub struct ExtBuilder {
    endowed_accounts: Vec<(AccountId, CurrencyId, Balance)>,
}

impl Default for ExtBuilder {
    fn default() -> Self {
        Self {
            endowed_accounts: vec![
                (ALICE, DOT, dollar(1000)),
                (ALICE, KSM, dollar(1000)),
                (ALICE, USDT, dollar(1000)),
                (BOB, DOT, dollar(1000)),
                (BOB, KSM, dollar(1000)),
                (BOB, USDT, dollar(1000)),
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
                CurrencyId::USDT,
                CurrencyId::xDOT,
            ],
            borrow_index: Rate::one(),                             // 1
            exchange_rate: Rate::saturating_from_rational(2, 100), // 0.02
            base_rate: Rate::saturating_from_rational(2, 100),     // 2%
            kink_rate: Rate::saturating_from_rational(10, 100),    // 10%
            full_rate: Rate::saturating_from_rational(32, 100),    // 32%
            kink_utilization: Ratio::from_percent(80),             // 80%
            collateral_factor: vec![
                (CurrencyId::DOT, Ratio::from_percent(50)),
                (CurrencyId::KSM, Ratio::from_percent(50)),
                (CurrencyId::USDT, Ratio::from_percent(50)),
                (CurrencyId::xDOT, Ratio::from_percent(50)),
            ],
            liquidation_incentive: vec![
                (CurrencyId::DOT, Ratio::from_percent(90)),
                (CurrencyId::KSM, Ratio::from_percent(90)),
                (CurrencyId::USDT, Ratio::from_percent(90)),
                (CurrencyId::xDOT, Ratio::from_percent(90)),
            ],
            liquidation_threshold: vec![
                (CurrencyId::DOT, Ratio::from_percent(80)),
                (CurrencyId::KSM, Ratio::from_percent(80)),
                (CurrencyId::USDT, Ratio::from_percent(90)),
                (CurrencyId::xDOT, Ratio::from_percent(80)),
            ],
            close_factor: vec![
                (CurrencyId::DOT, Ratio::from_percent(50)),
                (CurrencyId::KSM, Ratio::from_percent(50)),
                (CurrencyId::USDT, Ratio::from_percent(50)),
                (CurrencyId::xDOT, Ratio::from_percent(50)),
            ],
            reserve_factor: vec![
                (CurrencyId::DOT, Ratio::from_percent(15)),
                (CurrencyId::KSM, Ratio::from_percent(15)),
                (CurrencyId::USDT, Ratio::from_percent(15)),
                (CurrencyId::xDOT, Ratio::from_percent(15)),
            ],
        }
        .assimilate_storage::<Runtime>(&mut t)
        .unwrap();

        MOCK_PRICE_FEEDER::reset();

        // t.into()
        let mut ext = sp_io::TestExternalities::new(t);
        ext.execute_with(|| System::set_block_number(1));
        ext
    }
}

/// Progress to the given block, and then finalize the block.
pub(crate) fn run_to_block(n: BlockNumber) {
    Loans::on_finalize(System::block_number());
    for b in (System::block_number() + 1)..=n {
        Loans::on_initialize(System::block_number());
        System::set_block_number(b);
        if b != n {
            Loans::on_finalize(System::block_number());
        }
    }
}

pub fn dollar(d: u128) -> u128 {
    d.saturating_mul(TOKEN_DECIMAL)
}
