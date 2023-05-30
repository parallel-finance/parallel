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
    construct_runtime, parameter_types,
    traits::{AsEnsureOriginWithArg, Everything},
    PalletId,
};
use frame_system::{EnsureRoot, EnsureSigned};
use sp_core::H256;
use sp_runtime::{testing::Header, traits::IdentityLookup};
use sp_std::vec::Vec;

pub use primitives::tokens::{DOT, HKO, KSM, PDOT, PHKO, PKSM, PUSDT, USDT};

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
        Streaming: crate::{Pallet, Storage, Call, Event<T>},
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
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = ::sp_runtime::traits::BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type RuntimeEvent = RuntimeEvent;
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

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const DAVE: AccountId = 3;

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
    pub const ExistentialDeposit: Balance = 10_000;
    pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for Test {
    type Balance = Balance;
    type DustRemoval = ();
    type RuntimeEvent = RuntimeEvent;
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
    type RuntimeEvent = RuntimeEvent;
    type Balance = Balance;
    type AssetId = CurrencyId;
    type AssetIdParameter = codec::Compact<CurrencyId>;
    type Currency = Balances;
    type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<AccountId>>;
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
    type RemoveItemsLimit = frame_support::traits::ConstU32<1000>;
    type CallbackHandle = ();
    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper = ();
}

parameter_types! {
    pub const NativeCurrencyId: CurrencyId = HKO;
}

impl pallet_currency_adapter::Config for Test {
    type Assets = Assets;
    type Balances = Balances;
    type GetNativeCurrencyId = NativeCurrencyId;
    type LockOrigin = EnsureRoot<AccountId>;
}

parameter_types! {
    pub const StreamPalletId: PalletId = PalletId(*b"par/strm");
    pub const MaxStreamsCount: u32 = 128;
    pub const MaxFinishedStreamsCount: u32 = 2;
}

impl Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type PalletId = StreamPalletId;
    type MaxStreamsCount = MaxStreamsCount;
    type MaxFinishedStreamsCount = MaxFinishedStreamsCount;
    type UnixTime = TimestampPallet;
    type Assets = CurrencyAdapter;
    type UpdateOrigin = EnsureRoot<AccountId>;
    type WeightInfo = ();
    type NativeCurrencyId = NativeCurrencyId;
    type NativeExistentialDeposit = ExistentialDeposit;
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
        // Init network tokens to execute extrinsic
        Balances::set_balance(RuntimeOrigin::root(), ALICE, dollar(1000), dollar(0)).unwrap();
        Balances::set_balance(RuntimeOrigin::root(), BOB, dollar(1000), dollar(0)).unwrap();
        Balances::set_balance(RuntimeOrigin::root(), DAVE, dollar(1000), dollar(0)).unwrap();
        // Init DOT to alice with full access
        Assets::force_create(RuntimeOrigin::root(), DOT.into(), ALICE, true, 1).unwrap();
        // Alice mints DOT
        Assets::mint(
            RuntimeOrigin::signed(ALICE),
            DOT.into(),
            ALICE,
            dollar(10000),
        )
        .unwrap();
        Assets::mint(RuntimeOrigin::signed(ALICE), DOT.into(), BOB, dollar(10000)).unwrap();
        Assets::mint(
            RuntimeOrigin::signed(ALICE),
            DOT.into(),
            DAVE,
            dollar(10000),
        )
        .unwrap();
        // Set block number and time
        System::set_block_number(0);
        TimestampPallet::set_timestamp(6000);

        // Set minimum deposit for DOT
        Streaming::set_minimum_deposit(RuntimeOrigin::root(), DOT, dollar(0)).unwrap();
    });
    ext
}
