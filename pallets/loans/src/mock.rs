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

#![cfg(test)]

mod loans {
    pub use super::super::*;
}

use loans::*;

use frame_support::{construct_runtime, parameter_types, traits::Contains, PalletId};
use frame_system::EnsureRoot;
use orml_traits::parameter_type_with_key;
use primitives::{Amount, Balance, CurrencyId, Price, PriceDetail, PriceFeeder, Rate, TokenSymbol};
use sp_core::H256;
use sp_runtime::traits::One;
use sp_runtime::{testing::Header, traits::IdentityLookup};
use sp_std::vec::Vec;
use std::cell::RefCell;
use std::collections::HashMap;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Storage, Config, Event<T>},
        Tokens: orml_tokens::{Pallet, Storage, Event<T>, Config<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Event<T>},
        Currencies: orml_currencies::{Pallet, Call, Event<T>},
        Loans: loans::{Pallet, Storage, Call, Config, Event<T>},
        TimestampPallet: pallet_timestamp::{Pallet, Call, Storage, Inherent},
        Assets: pallet_assets::<Instance1>::{Pallet, Call, Storage, Event<T>},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Test {
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
pub const CHARLIE: AccountId = 3;

pub const DOT: CurrencyId = CurrencyId::Token(TokenSymbol::DOT);
pub const KSM: CurrencyId = CurrencyId::Token(TokenSymbol::KSM);
pub const USDT: CurrencyId = CurrencyId::Token(TokenSymbol::USDT);
pub const XDOT: CurrencyId = CurrencyId::Token(TokenSymbol::xDOT);
pub const XKSM: CurrencyId = CurrencyId::Token(TokenSymbol::xKSM);
pub const NATIVE: CurrencyId = CurrencyId::Token(TokenSymbol::HKO);

parameter_types! {
    pub const MinimumPeriod: u64 = 5;
}

impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

parameter_type_with_key! {
    pub ExistentialDeposits: |_currency_id: CurrencyId| -> Balance {
        Default::default()
    };
}

pub struct DustRemovalWhitelist;
impl Contains<AccountId> for DustRemovalWhitelist {
    fn contains(a: &AccountId) -> bool {
        vec![LoansPalletId::get().into_account()].contains(a)
    }
}

impl orml_tokens::Config for Test {
    type Event = Event;
    type Balance = Balance;
    type Amount = Amount;
    type CurrencyId = CurrencyId;
    type WeightInfo = ();
    type OnDust = ();
    type ExistentialDeposits = ExistentialDeposits;
    type MaxLocks = MaxLocks;
    type DustRemovalWhitelist = DustRemovalWhitelist;
}

parameter_types! {
    pub const GetNativeCurrencyId: CurrencyId = NATIVE;
}

impl orml_currencies::Config for Test {
    type Event = Event;
    type MultiCurrency = Tokens;
    type NativeCurrency =
        orml_currencies::BasicCurrencyAdapter<Test, Balances, Amount, BlockNumber>;
    type GetNativeCurrencyId = GetNativeCurrencyId;
    type WeightInfo = ();
}

parameter_types! {
    pub const ExistentialDeposit: Balance = 1;
    pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for Test {
    type MaxLocks = MaxLocks;
    type Balance = Balance;
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type AccountStore = System;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Test>;
}

pub struct MockPriceFeeder;

impl MockPriceFeeder {
    thread_local! {
        pub static PRICES: RefCell<HashMap<CurrencyId, Option<PriceDetail>>> = {
            RefCell::new(
                vec![DOT, KSM, USDT, XDOT]
                    .iter()
                    .map(|&x| (x, Some((Price::saturating_from_integer(1), 1))))
                    .collect()
            )
        };
    }

    pub fn set_price(currency_id: CurrencyId, price: Price) {
        Self::PRICES.with(|prices| {
            prices.borrow_mut().insert(currency_id, Some((price, 1u64)));
        });
    }

    pub fn reset() {
        Self::PRICES.with(|prices| {
            for (_, val) in prices.borrow_mut().iter_mut() {
                *val = Some((Price::saturating_from_integer(1), 1u64));
            }
        })
    }
}

impl PriceFeeder for MockPriceFeeder {
    fn get_price(currency_id: &CurrencyId) -> Option<PriceDetail> {
        Self::PRICES.with(|prices| *prices.borrow().get(currency_id).unwrap())
    }
}

parameter_types! {
    pub const AssetDeposit: u64 = 1;
    pub const ApprovalDeposit: u64 = 1;
    pub const StringLimit: u32 = 50;
    pub const MetadataDepositBase: u64 = 1;
    pub const MetadataDepositPerByte: u64 = 1;
}

type AssetsInstance = pallet_assets::Instance1;
impl pallet_assets::Config<AssetsInstance> for Test {
    type Event = Event;
    type Balance = u64;
    type AssetId = u32;
    type Currency = Balances;
    type ForceOrigin = EnsureRoot<AccountId>;
    type AssetDeposit = AssetDeposit;
    type MetadataDepositBase = MetadataDepositBase;
    type MetadataDepositPerByte = MetadataDepositPerByte;
    type ApprovalDeposit = ApprovalDeposit;
    type StringLimit = StringLimit;
    type Freezer = ();
    type WeightInfo = ();
    type Extra = ();
}

impl Config for Test {
    type Event = Event;
    type Currency = Currencies;
    type PalletId = LoansPalletId;
    type PriceFeeder = MockPriceFeeder;
    type ReserveOrigin = EnsureRoot<AccountId>;
    type UpdateOrigin = EnsureRoot<AccountId>;
    type WeightInfo = ();
    type UnixTime = TimestampPallet;
    type Assets = Assets;
}

parameter_types! {
    pub const LoansPalletId: PalletId = PalletId(*b"par/loan");
}

pub struct ExtBuilder {
    balances: Vec<(AccountId, CurrencyId, Balance)>,
}

impl Default for ExtBuilder {
    fn default() -> Self {
        Self {
            balances: vec![
                (ALICE, DOT, million_dollar(1000)),
                (ALICE, KSM, million_dollar(1000)),
                (ALICE, USDT, million_dollar(1000)),
                (BOB, DOT, million_dollar(1000)),
                (BOB, KSM, million_dollar(1000)),
                (BOB, USDT, million_dollar(1000)),
            ],
        }
    }
}

impl ExtBuilder {
    pub fn build(self) -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();

        orml_tokens::GenesisConfig::<Test> {
            balances: self.balances.clone(),
        }
        .assimilate_storage(&mut t)
        .unwrap();

        loans::GenesisConfig {
            borrow_index: Rate::one(),                             // 1
            exchange_rate: Rate::saturating_from_rational(2, 100), // 0.02
            markets: vec![
                (CurrencyId::Token(TokenSymbol::DOT), MARKET_MOCK),
                (CurrencyId::Token(TokenSymbol::KSM), MARKET_MOCK),
                (CurrencyId::Token(TokenSymbol::USDT), MARKET_MOCK),
                (CurrencyId::Token(TokenSymbol::xDOT), MARKET_MOCK),
            ],
            last_block_timestamp: 0,
        }
        .assimilate_storage::<Test>(&mut t)
        .unwrap();

        // t.into()
        let mut ext = sp_io::TestExternalities::new(t);
        ext.execute_with(|| {
            System::set_block_number(0);
            TimestampPallet::set_timestamp(6000);
        });
        ext
    }
}

/// Progress to the given block, and then finalize the block.
pub(crate) fn run_to_block(n: BlockNumber) {
    Loans::on_finalize(System::block_number());
    for b in (System::block_number() + 1)..=n {
        System::set_block_number(b);
        Loans::on_initialize(System::block_number());
        TimestampPallet::set_timestamp(6000 * b);
        if b != n {
            Loans::on_finalize(System::block_number());
        }
    }
}

pub(crate) fn process_block(n: BlockNumber) {
    System::set_block_number(n);
    Loans::on_initialize(n);
    TimestampPallet::set_timestamp(6000 * n);
    Loans::on_finalize(n);
}

// TODO make decimals more explicit
pub fn dollar(d: u128) -> u128 {
    d.saturating_mul(10_u128.pow(12))
}

pub fn million_dollar(d: u128) -> u128 {
    dollar(d) * 10_u128.pow(6)
}

pub const MARKET_MOCK: Market = Market {
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
