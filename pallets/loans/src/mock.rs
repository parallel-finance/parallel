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

use frame_support::{construct_runtime, parameter_types, PalletId};
use frame_system::EnsureRoot;

use primitives::{AssetId, Balance, Price, PriceDetail, PriceFeeder, Rate};
use sp_core::H256;

use sp_runtime::{testing::Header, traits::IdentityLookup};
use sp_std::vec::Vec;
use std::{cell::RefCell, collections::HashMap};

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
        Loans: crate::{Pallet, Storage, Call, Config, Event<T>},
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

pub const DOT: AssetId = 0;
pub const KSM: AssetId = 1;
pub const USDT: AssetId = 3;
pub const XDOT: AssetId = 4;

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

pub struct MockPriceFeeder;

impl MockPriceFeeder {
    thread_local! {
        pub static PRICES: RefCell<HashMap<AssetId, Option<PriceDetail>>> = {
            RefCell::new(
                vec![DOT, KSM, USDT, XDOT]
                    .iter()
                    .map(|&x| (x, Some((Price::saturating_from_integer(1), 1))))
                    .collect()
            )
        };
    }

    pub fn set_price(asset_id: AssetId, price: Price) {
        Self::PRICES.with(|prices| {
            prices.borrow_mut().insert(asset_id, Some((price, 1u64)));
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
    fn get_price(asset_id: &AssetId) -> Option<PriceDetail> {
        Self::PRICES.with(|prices| *prices.borrow().get(asset_id).unwrap())
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
    type Balance = u128;
    type AssetId = u32;
    type Currency = Balances;
    type ForceOrigin = EnsureRoot<AccountId>;
    type AssetDeposit = AssetDeposit;
    type MetadataDepositBase = MetadataDepositBase;
    type MetadataDepositPerByte = MetadataDepositPerByte;
    type ApprovalDeposit = ApprovalDeposit;
    type StringLimit = StringLimit;
    type Freezer = ();
    type Extra = ();
    type WeightInfo = ();
}

parameter_types! {
    pub const LoansPalletId: PalletId = PalletId(*b"par/loan");
}

impl Config for Test {
    type Event = Event;
    type PriceFeeder = MockPriceFeeder;
    type PalletId = LoansPalletId;
    type ReserveOrigin = EnsureRoot<AccountId>;
    type UpdateOrigin = EnsureRoot<AccountId>;
    type WeightInfo = ();
    type UnixTime = TimestampPallet;
    type Assets = Assets;
}

pub(crate) fn new_test_ext() -> sp_io::TestExternalities {
    let t = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| {
        // Init assets
        // Balances::make_free_balance_be(&ALICE, 10);
        // Balances::make_free_balance_be(&BOB, 10);
        Assets::force_create(Origin::root(), DOT, ALICE, true, 1).unwrap();
        Assets::force_create(Origin::root(), KSM, ALICE, true, 1).unwrap();
        Assets::force_create(Origin::root(), USDT, ALICE, true, 1).unwrap();
        Assets::force_create(Origin::root(), XDOT, ALICE, true, 1).unwrap();
        Assets::mint(Origin::signed(ALICE), KSM, ALICE, dollar(1000)).unwrap();
        Assets::mint(Origin::signed(ALICE), DOT, ALICE, dollar(1000)).unwrap();
        Assets::mint(Origin::signed(ALICE), USDT, ALICE, dollar(1000)).unwrap();
        Assets::mint(Origin::signed(ALICE), KSM, BOB, dollar(1000)).unwrap();
        Assets::mint(Origin::signed(ALICE), DOT, BOB, dollar(1000)).unwrap();

        // Init Markets
        Loans::add_market(Origin::root(), KSM, MARKET_MOCK).unwrap();
        Loans::active_market(Origin::root(), KSM).unwrap();
        Loans::add_market(Origin::root(), DOT, MARKET_MOCK).unwrap();
        Loans::active_market(Origin::root(), DOT).unwrap();
        Loans::add_market(Origin::root(), USDT, MARKET_MOCK).unwrap();
        Loans::active_market(Origin::root(), USDT).unwrap();

        System::set_block_number(0);
        TimestampPallet::set_timestamp(6000);
    });
    ext
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
    state: MarketState::Pending,
    rate_model: InterestRateModel::Jump(JumpModel {
        base_rate: Rate::from_inner(Rate::DIV / 100 * 2),
        jump_rate: Rate::from_inner(Rate::DIV / 100 * 10),
        full_rate: Rate::from_inner(Rate::DIV / 100 * 32),
        jump_utilization: Ratio::from_percent(80),
    }),
    reserve_factor: Ratio::from_percent(15),
};
