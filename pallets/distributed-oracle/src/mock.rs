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

use super::*;

use frame_support::{
    construct_runtime, ord_parameter_types, parameter_types, traits::Everything, PalletId,
};
use frame_system::EnsureRoot;
use frame_system::EnsureSignedBy;

use sp_runtime::{testing::Header, traits::IdentityLookup, FixedPointNumber};
use sp_std::vec::Vec;

use sp_core::H256;

pub use primitives::tokens::{DOT, HKO, KSM, PARA, PDOT, PHKO, PKSM, PUSDT, SDOT, SKSM, USDT};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Storage, Config, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Event<T>},
        Doracle: crate::{Pallet, Storage, Call, Event<T>},
        TimestampPallet: pallet_timestamp::{Pallet, Call, Storage, Inherent},
        Assets: pallet_assets::{Pallet, Call, Storage, Event<T>},
        CurrencyAdapter: pallet_currency_adapter::{Pallet, Call},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Test {
    type BaseCallFilter = Everything;
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
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

pub type AccountId = u128;
//pub type BlockNumber = u64;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const CHARLIE: AccountId = 3;
pub const DAVE: AccountId = 4;
pub const EVE: AccountId = 5;
pub const FRANK: AccountId = 6;

parameter_types! {
    pub const MinimumPeriod: u64 = 5;
}

impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

parameter_types! {
    pub const ExistentialDeposit: Balance = 1;
    pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for Test {
    type Balance = Balance;
    type DustRemoval = ();
    type Event = Event;
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Test>;
    type MaxLocks = MaxLocks;
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
}

parameter_types! {
    pub const AssetDeposit: u64 = 1;
    pub const ApprovalDeposit: u64 = 1;
    pub const AssetAccountDeposit: u64 = 1;
    pub const StringLimit: u32 = 50;
    pub const MetadataDepositBase: u64 = 1;
    pub const MetadataDepositPerByte: u64 = 1;
}

impl pallet_assets::Config for Test {
    type Event = Event;
    type Balance = Balance;
    type AssetId = CurrencyId;
    type Currency = Balances;
    type ForceOrigin = EnsureRoot<AccountId>;
    type AssetDeposit = AssetDeposit;
    type MetadataDepositBase = MetadataDepositBase;
    type MetadataDepositPerByte = MetadataDepositPerByte;
    type AssetAccountDeposit = AssetAccountDeposit;
    type ApprovalDeposit = ApprovalDeposit;
    type StringLimit = StringLimit;
    type Freezer = ();
    type Extra = ();
    type WeightInfo = ();
}

parameter_types! {
    pub const NativeCurrencyId: CurrencyId = HKO;
}

impl pallet_currency_adapter::Config for Test {
    type Assets = Assets;
    type Balances = Balances;
    type GetNativeCurrencyId = NativeCurrencyId;
}

parameter_types! {
    pub const StreamPalletId: PalletId = PalletId(*b"par/strm");
    pub const MinHoldTime: u128 = 10000_u128;
    pub const MinStake: u128 = 100_u128;
    pub const MinUnStake: u128 = 10_u128;
    pub const MinSlashedTime: u64 = 1800_u64;
    pub const TreasuryAmount: Balance = 100_000_000_000;
}

pub struct MockDataProvider;
impl DataProvider<CurrencyId, TimeStampedPrice> for MockDataProvider {
    fn get(asset_id: &CurrencyId) -> Option<TimeStampedPrice> {
        match *asset_id {
            DOT => Some(TimeStampedPrice {
                value: Price::saturating_from_integer(100),
                timestamp: 0,
            }),
            KSM => Some(TimeStampedPrice {
                value: Price::saturating_from_integer(500),
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

impl DataFeeder<CurrencyId, TimeStampedPrice, AccountId> for MockDataProvider {
    fn feed_value(_: AccountId, _: CurrencyId, _: TimeStampedPrice) -> sp_runtime::DispatchResult {
        Ok(())
    }
}

pub struct LiquidStakingExchangeRateProvider;
impl ExchangeRateProvider for LiquidStakingExchangeRateProvider {
    fn get_exchange_rate() -> Rate {
        Rate::saturating_from_rational(150, 100)
    }
}

ord_parameter_types! {
    pub const One: AccountId = 1;
}

pub struct Decimal;
#[allow(non_upper_case_globals)]
impl DecimalProvider<CurrencyId> for Decimal {
    fn get_decimal(asset_id: &CurrencyId) -> Option<u8> {
        match *asset_id {
            DOT | SDOT => Some(10),
            KSM | SKSM => Some(12),
            _ => None,
        }
    }
}

pub struct LiquidStaking;
impl LiquidStakingCurrenciesProvider<CurrencyId> for LiquidStaking {
    fn get_staking_currency() -> Option<CurrencyId> {
        Some(KSM)
    }
    fn get_liquid_currency() -> Option<CurrencyId> {
        Some(SKSM)
    }
}

// Config implementation for distributed oracle pallet
impl Config for Test {
    type Event = Event;
    type Source = MockDataProvider;
    type FeederOrigin = EnsureSignedBy<One, AccountId>;
    type Decimal = Decimal;
    type Assets = CurrencyAdapter;
    type PalletId = StreamPalletId;
    type UnixTime = TimestampPallet;
    type WeightInfo = ();
    type MinStake = MinStake;
    type MinUnstake = MinUnStake;
    type MinHoldTime = MinHoldTime;
    type StakingCurrency = NativeCurrencyId;
    type MinSlashedTime = MinSlashedTime;
    type Treasury = TreasuryAmount;
}

pub fn dollar(d: u128) -> u128 {
    d.saturating_mul(10_u128.pow(12))
}

// Initial settings for test
pub(crate) fn new_test_ext() -> sp_io::TestExternalities {
    let t = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| {
        // Init network tokens to execute extrinsics
        Balances::set_balance(Origin::root(), BOB, dollar(1000), dollar(0)).unwrap();
        Balances::set_balance(Origin::root(), ALICE, dollar(1000), dollar(0)).unwrap();
        Balances::set_balance(Origin::root(), DAVE, dollar(1000), dollar(0)).unwrap();
        // Init DOT to alice with full access
        Assets::force_create(Origin::root(), DOT, ALICE, true, 1).unwrap();
        // Alice mints DOT
        Assets::mint(Origin::signed(ALICE), DOT, ALICE, dollar(10000)).unwrap();
        Assets::mint(Origin::signed(ALICE), DOT, BOB, dollar(10000)).unwrap();
        Assets::mint(Origin::signed(ALICE), DOT, DAVE, dollar(10000)).unwrap();

        // Set block number and time
        System::set_block_number(0);
        TimestampPallet::set_timestamp(6000);
    });
    ext
}
