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

//! Mock file for loans benchmarking.
#![cfg(test)]

use super::*;

use frame_support::{
    construct_runtime, ord_parameter_types, parameter_types,
    traits::{SortedMembers, Time},
    PalletId,
};
use frame_system::EnsureRoot;
use orml_oracle::DefaultCombineData;
use orml_traits::parameter_type_with_key;
use orml_traits::DataProvider;
use orml_traits::DataProviderExtended;
use primitives::{
    Amount, Balance, CurrencyId, ExchangeRateProvider, Moment, Price, PriceWithDecimal,
};
use sp_core::H256;
use sp_runtime::{testing::Header, traits::IdentityLookup, FixedPointNumber};
use sp_std::vec::Vec;
use std::cell::RefCell;

pub type Block = sp_runtime::generic::Block<Header, UncheckedExtrinsic>;
pub type UncheckedExtrinsic = sp_runtime::generic::UncheckedExtrinsic<u32, Call, u64, ()>;

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
        Loans: pallet_loans::{Pallet, Storage, Call, Config, Event<T>},
        Oracle: orml_oracle::<Instance1>::{Pallet, Storage, Call, Event<T>},
        Prices: pallet_prices::{Pallet, Storage, Call, Event<T>},
        TimestampPallet: pallet_timestamp::{Pallet, Call, Storage, Inherent},
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

pub const DOT: CurrencyId = CurrencyId::DOT;
pub const KSM: CurrencyId = CurrencyId::KSM;
pub const NATIVE: CurrencyId = CurrencyId::Native;

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

impl orml_tokens::Config for Test {
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
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
}

thread_local! {
    static TIME: RefCell<u32> = RefCell::new(0);
}

pub struct Timestamp;
impl Time for Timestamp {
    type Moment = u32;

    fn now() -> Self::Moment {
        TIME.with(|v| *v.borrow())
    }
}

impl Timestamp {
    pub fn set_timestamp(val: u32) {
        TIME.with(|v| *v.borrow_mut() = val);
    }
}

parameter_types! {
    pub const MinimumCount: u32 = 3;
    pub const ExpiresIn: u32 = 600;
    pub const RootOperatorAccountId: AccountId = 4;
    pub static OracleMembers: Vec<AccountId> = vec![1, 2, 3];
}

pub struct Members;

impl SortedMembers<AccountId> for Members {
    fn sorted_members() -> Vec<AccountId> {
        OracleMembers::get()
    }
}

parameter_types! {
    pub const MaxHasDispatchedSize: u32 = 100;
}

impl orml_oracle::Config<Instance1> for Test {
    type Event = Event;
    type OnNewData = ();
    type CombineData = DefaultCombineData<Self, MinimumCount, ExpiresIn, Instance1>;
    type Time = Timestamp;
    type OracleKey = CurrencyId;
    type OracleValue = PriceWithDecimal;
    type RootOperatorAccountId = RootOperatorAccountId;
    type WeightInfo = ();
    type Members = Members;
    type MaxHasDispatchedSize = MaxHasDispatchedSize;
}

pub type TimeStampedPrice = orml_oracle::TimestampedValue<PriceWithDecimal, Moment>;
pub struct MockDataProvider;
impl DataProvider<CurrencyId, TimeStampedPrice> for MockDataProvider {
    fn get(currency_id: &CurrencyId) -> Option<TimeStampedPrice> {
        match *currency_id {
            DOT => Some(TimeStampedPrice {
                value: PriceWithDecimal {
                    price: Price::saturating_from_integer(100),
                    decimal: 10,
                },
                timestamp: 0,
            }),
            KSM => Some(TimeStampedPrice {
                value: PriceWithDecimal {
                    price: Price::saturating_from_integer(500),
                    decimal: 12,
                },
                timestamp: 0,
            }),
            _ => None,
        }
    }
}

impl DataProviderExtended<CurrencyId, TimeStampedPrice> for MockDataProvider {
    fn get_no_op(_key: &CurrencyId) -> Option<TimeStampedPrice> {
        None
    }

    fn get_all_values() -> Vec<(CurrencyId, Option<TimeStampedPrice>)> {
        vec![]
    }
}

pub struct LiquidStakingExchangeRateProvider;
impl ExchangeRateProvider for LiquidStakingExchangeRateProvider {
    fn get_exchange_rate() -> Rate {
        Rate::saturating_from_rational(150, 100)
    }
}

ord_parameter_types! {
    pub const StakingCurrency: CurrencyId = CurrencyId::KSM;
    pub const LiquidCurrency: CurrencyId = CurrencyId::xKSM;
}

impl pallet_prices::Config for Test {
    type Event = Event;
    type Source = MockDataProvider;
    type FeederOrigin = EnsureRoot<AccountId>;
    type StakingCurrency = StakingCurrency;
    type LiquidCurrency = LiquidCurrency;
    type LiquidStakingExchangeRateProvider = LiquidStakingExchangeRateProvider;
}

impl pallet_loans::Config for Test {
    type Event = Event;
    type Currency = Currencies;
    type PalletId = LoansPalletId;
    type PriceFeeder = Prices;
    type ReserveOrigin = EnsureRoot<AccountId>;
    type UpdateOrigin = EnsureRoot<AccountId>;
    type WeightInfo = ();
    type UnixTime = TimestampPallet;
}

impl crate::Config for Test {}

parameter_types! {
    pub const LoansPalletId: PalletId = PalletId(*b"par/loan");
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();

    pallet_loans::GenesisConfig {
        borrow_index: Rate::from(1),                           // 1
        exchange_rate: Rate::saturating_from_rational(2, 100), // 0.02
        markets: vec![],
        last_block_timestamp: 1,
    }
    .assimilate_storage::<Test>(&mut t)
    .unwrap();

    sp_io::TestExternalities::new(t)
}
