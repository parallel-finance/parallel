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

//! Mocks for the prices module.

use super::*;
use frame_support::{construct_runtime, ord_parameter_types, parameter_types};
use frame_system::{EnsureRoot, EnsureSignedBy};
use sp_core::H256;
use sp_runtime::{testing::Header, traits::IdentityLookup, FixedPointNumber};

pub type AccountId = u128;
pub type BlockNumber = u64;

mod prices {
    pub use super::super::*;
}

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;

pub const DOT: AssetId = 10;
#[allow(non_upper_case_globals)]
pub const xDOT: AssetId = 11;
pub const KSM: AssetId = 20;
#[allow(non_upper_case_globals)]
pub const xKSM: AssetId = 21;

parameter_types! {
    pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for Runtime {
    type Origin = Origin;
    type Index = u64;
    type BlockNumber = BlockNumber;
    type Call = Call;
    type Hash = H256;
    type Hashing = ::sp_runtime::traits::BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type BlockWeights = ();
    type BlockLength = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type DbWeight = ();
    type BaseCallFilter = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
}

pub type TimeStampedPrice = orml_oracle::TimestampedValue<Price, Moment>;
pub struct MockDataProvider;
impl DataProvider<AssetId, TimeStampedPrice> for MockDataProvider {
    fn get(asset_id: &AssetId) -> Option<TimeStampedPrice> {
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

impl DataProviderExtended<AssetId, TimeStampedPrice> for MockDataProvider {
    fn get_no_op(_key: &AssetId) -> Option<TimeStampedPrice> {
        None
    }

    fn get_all_values() -> Vec<(AssetId, Option<TimeStampedPrice>)> {
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
    pub const One: AccountId = 1;
    pub const StakingCurrency: AssetId = KSM;
    pub const LiquidCurrency: AssetId = xKSM;
}
pub struct Decimal;
#[allow(non_upper_case_globals)]
impl DecimalProvider for Decimal {
    fn get_decimal(asset_id: &AssetId) -> u8 {
        match *asset_id {
            DOT | xDOT => 10,
            KSM | xKSM => 12,
            _ => 0,
        }
    }
}

pub struct LiquidStaking;
impl LiquidStakingCurrenciesProvider<AssetId> for LiquidStaking {
    fn get_staking_currency() -> Option<AssetId> {
        Some(KSM)
    }
    fn get_liquid_currency() -> Option<AssetId> {
        Some(xKSM)
    }
}

pub mod currency {
    use primitives::Balance;

    pub const MILLICENTS: Balance = 10_000_000;
    pub const CENTS: Balance = 1_000 * MILLICENTS; // assume this is worth about a cent.
    pub const DOLLARS: Balance = 100 * CENTS;

    pub const EXISTENTIAL_DEPOSIT: u128 = 10 * CENTS; // 0.1 Native Token Balance

    pub const fn deposit(items: u32, bytes: u32) -> Balance {
        items as Balance * 15 * CENTS + (bytes as Balance) * 6 * CENTS
    }
}
use currency::*;
parameter_types! {
    pub const AssetDeposit: Balance = DOLLARS; // 1 UNIT deposit to create asset
    pub const ApprovalDeposit: Balance = EXISTENTIAL_DEPOSIT;
    pub const AssetsStringLimit: u32 = 50;
    /// Key = 32 bytes, Value = 36 bytes (32+1+1+1+1)
    // https://github.com/paritytech/substrate/blob/069917b/frame/assets/src/lib.rs#L257L271
    pub const MetadataDepositBase: Balance = deposit(1, 68);
    pub const MetadataDepositPerByte: Balance = deposit(0, 1);
}

impl pallet_assets::Config for Runtime {
    type Event = Event;
    type Balance = Balance;
    type AssetId = AssetId;
    type Currency = Balances;
    type ForceOrigin = EnsureRoot<Self::AccountId>;
    type AssetDeposit = AssetDeposit;
    type MetadataDepositBase = MetadataDepositBase;
    type MetadataDepositPerByte = MetadataDepositPerByte;
    type ApprovalDeposit = ApprovalDeposit;
    type StringLimit = AssetsStringLimit;
    type Freezer = ();
    type WeightInfo = ();
    type Extra = ();
}

parameter_types! {
    pub const ExistentialDeposit: u128 = currency::EXISTENTIAL_DEPOSIT;
    pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for Runtime {
    type MaxLocks = MaxLocks;
    /// The type for recording an account's balance.
    type Balance = Balance;
    /// The ubiquitous event type.
    type Event = Event;
    type DustRemoval = ();
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
}

impl Config for Runtime {
    type Event = Event;
    type Source = MockDataProvider;
    type FeederOrigin = EnsureSignedBy<One, AccountId>;
    type LiquidStakingCurrenciesProvider = LiquidStaking;
    type LiquidStakingExchangeRateProvider = LiquidStakingExchangeRateProvider;
    type Decimal = Decimal;
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Prices: prices::{Pallet, Storage, Call, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        Assets: pallet_assets::{Pallet, Call, Storage, Event<T>},
    }
);

pub struct ExtBuilder;

impl Default for ExtBuilder {
    fn default() -> Self {
        ExtBuilder
    }
}

pub fn dollar(d: u128) -> u128 {
    d.saturating_mul(10_u128.pow(12))
}

impl ExtBuilder {
    pub fn build(self) -> sp_io::TestExternalities {
        let t = frame_system::GenesisConfig::default()
            .build_storage::<Runtime>()
            .unwrap();

            let mut ext = sp_io::TestExternalities::new(t);
            ext.execute_with(|| {
                Assets::force_create(Origin::root(), DOT, ALICE, true, 1).unwrap();
                Assets::force_create(Origin::root(), KSM, ALICE, true, 1).unwrap();
                Assets::force_create(Origin::root(), xKSM, ALICE, true, 1).unwrap();
                Assets::force_create(Origin::root(), xDOT, ALICE, true, 1).unwrap();
                Assets::mint(Origin::signed(ALICE), KSM, ALICE, dollar(1000)).unwrap();
                Assets::mint(Origin::signed(ALICE), DOT, ALICE, dollar(1000)).unwrap();
                Assets::mint(Origin::signed(ALICE), xKSM, ALICE, dollar(1000)).unwrap();
                Assets::mint(Origin::signed(ALICE), xDOT, ALICE, dollar(1000)).unwrap();
                Assets::mint(Origin::signed(ALICE), KSM, BOB, dollar(1000)).unwrap();
                Assets::mint(Origin::signed(ALICE), DOT, BOB, dollar(1000)).unwrap();
        
                System::set_block_number(0);
            });
            ext
    }
}
