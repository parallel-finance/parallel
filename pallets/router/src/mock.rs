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

//! Mocks for the router module.

use super::*;
use crate as pallet_route;

use frame_support::{construct_runtime, parameter_types, traits::Everything, PalletId};
use frame_system::EnsureRoot;
use sp_core::H256;
use sp_runtime::{testing::Header, traits::IdentityLookup, Perbill};

pub use primitives::{tokens, Amount, Balance, CurrencyId, AMM};

pub type AccountId = u128;
pub type BlockNumber = u64;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const CHARLIE: AccountId = 3;
pub const DAVE: AccountId = 4;

pub const DOT: CurrencyId = tokens::DOT;
pub const XDOT: CurrencyId = tokens::XDOT;
pub const USDT: CurrencyId = tokens::USDT;
pub const KSM: CurrencyId = tokens::KSM;

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
    type BaseCallFilter = Everything;
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
}

// pallet-balances configuration
parameter_types! {
    pub const ExistentialDeposit: Balance = 1;
    pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for Runtime {
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

// pallet-assets configuration
parameter_types! {
    pub const AssetDeposit: u64 = 1;
    pub const ApprovalDeposit: u64 = 1;
    pub const StringLimit: u32 = 50;
    pub const MetadataDepositBase: u64 = 1;
    pub const MetadataDepositPerByte: u64 = 1;
}

impl pallet_assets::Config for Runtime {
    type Event = Event;
    type Balance = Balance;
    type AssetId = CurrencyId;
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

// AMM instance initialization
parameter_types! {
    pub const AMMPalletId: PalletId = PalletId(*b"par/ammp");
    pub const AllowPermissionlessPoolCreation: bool = true;
    pub const DefaultLpFee: Perbill = Perbill::from_perthousand(3);         // 0.3%
    pub const DefaultProtocolFee: Perbill = Perbill::from_perthousand(2);   // 0.2%
    pub const DefaultProtocolFeeReceiver: AccountId = CHARLIE;
}

impl pallet_amm::Config for Runtime {
    type Event = Event;
    type Assets = CurrencyAdapter;
    type PalletId = AMMPalletId;
    type AMMWeightInfo = ();
    type AllowPermissionlessPoolCreation = AllowPermissionlessPoolCreation;
    type LpFee = DefaultLpFee;
    type ProtocolFee = DefaultProtocolFee;
    type ProtocolFeeReceiver = DefaultProtocolFeeReceiver;
}

parameter_types! {
    pub const NativeCurrencyId: CurrencyId = 0;
}

impl pallet_currency_adapter::Config for Runtime {
    type Assets = Assets;
    type Balances = Balances;
    type GetNativeCurrencyId = NativeCurrencyId;
}

parameter_types! {
    pub const MaxLengthRoute: u8 = 10;
    pub const RouterPalletId: PalletId = PalletId(*b"ammroute");
}

impl Config for Runtime {
    type Event = Event;
    type RouterPalletId = RouterPalletId;
    type AMM = DefaultAMM;
    type AMMRouterWeightInfo = ();
    type MaxLengthRoute = MaxLengthRoute;
    type Assets = CurrencyAdapter;
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
        Assets: pallet_assets::{Pallet, Call, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Event<T>},
        // AMM instances
        DefaultAMM: pallet_amm::{Pallet, Call, Storage, Event<T>},
        // AMM Route
        AMMRoute: pallet_route::{Pallet, Call, Event<T>},
        CurrencyAdapter: pallet_currency_adapter::{Pallet, Call},
    }
);

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Runtime>()
        .unwrap();
    pallet_balances::GenesisConfig::<Runtime> {
        balances: vec![(ALICE, 100_000_000), (BOB, 100_000_000)],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| {
        Assets::force_create(Origin::root(), tokens::DOT, ALICE, true, 1).unwrap();
        Assets::force_create(Origin::root(), tokens::XDOT, ALICE, true, 1).unwrap();
        Assets::force_create(Origin::root(), tokens::KSM, ALICE, true, 1).unwrap();
        Assets::force_create(Origin::root(), tokens::USDT, ALICE, true, 1).unwrap();

        Assets::mint(Origin::signed(ALICE), tokens::DOT, ALICE, 10_000).unwrap();
        Assets::mint(Origin::signed(ALICE), tokens::XDOT, ALICE, 10_000).unwrap();
        Assets::mint(Origin::signed(ALICE), tokens::KSM, ALICE, 10_000).unwrap();

        Assets::mint(Origin::signed(ALICE), tokens::DOT, DAVE, 1000_000_000).unwrap();
        Assets::mint(Origin::signed(ALICE), tokens::KSM, DAVE, 1000_000_000).unwrap();
        Assets::mint(Origin::signed(ALICE), tokens::XDOT, DAVE, 1000_000_000).unwrap();
        Assets::mint(Origin::signed(ALICE), tokens::USDT, DAVE, 1000_000_000).unwrap();
    });

    ext
}

pub(crate) fn run_to_block(n: u64) {
    System::set_block_number(n);
}
