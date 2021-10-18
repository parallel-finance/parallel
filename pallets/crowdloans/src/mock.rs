use sp_core::H256;
use sp_runtime::traits::BlakeTwo256;
use sp_runtime::traits::IdentityLookup;

use sp_runtime::{testing::Header, AccountId32};

use primitives::tokens::{DOT, XDOT};
use primitives::Balance;

use frame_support::traits::GenesisBuild;
use frame_support::{construct_runtime, parameter_types, PalletId};

use frame_system::EnsureRoot;

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

pub type AccountId = AccountId32;
pub type CurrencyId = u32;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
type BlockNumber = u64;
pub const DOT_DECIMAL: u128 = 10u128.pow(10);

parameter_types! {
    pub const StakingPalletId: PalletId = PalletId(*b"par/lqsk");

    pub const AssetDeposit: u64 = 1;
    pub const ApprovalDeposit: u64 = 1;
    pub const StringLimit: u32 = 50;
    pub const MetadataDepositBase: u64 = 1;
    pub const MetadataDepositPerByte: u64 = 1;

    pub const ExistentialDeposit: Balance = 1;
    pub const MaxLocks: u32 = 50;

    pub const AssetsStringLimit: u32 = 50;
}

impl frame_system::Config for Test {
    type BaseCallFilter = ();
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = BlockNumber;
    type Hash = H256;
    type Hashing = BlakeTwo256;
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

impl pallet_assets::Config for Test {
    type Event = Event;
    type Balance = Balance;
    type AssetId = CurrencyId;
    type Currency = Balances;
    type ForceOrigin = EnsureRoot<AccountId>;
    type AssetDeposit = AssetDeposit;
    type MetadataDepositBase = MetadataDepositBase;
    type MetadataDepositPerByte = MetadataDepositPerByte;
    type ApprovalDeposit = ApprovalDeposit;
    type StringLimit = AssetsStringLimit;
    type Freezer = ();
    type WeightInfo = ();
    type Extra = ();
}

impl crate::Config for Test {
    type Event = Event;
    type PalletId = StakingPalletId;
    type Assets = Assets;
}

// Configure a mock runtime to test the pallet.
construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Event<T>},
        Crowdloan: crate::{Pallet, Call, Storage, Event<T>},
        Assets: pallet_assets::{Pallet, Call, Storage, Event<T>},
    }
);

pub const ALICE: AccountId32 = AccountId32::new([1u8; 32]);

pub(crate) fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();

    GenesisBuild::<Test>::assimilate_storage(
        &crate::GenesisConfig {
            exchange_rate: 1,
            reserve_factor: 1,
        },
        &mut t,
    )
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| {
        Assets::force_create(Origin::root(), DOT, ALICE, true, 1).unwrap();
        Assets::force_create(Origin::root(), XDOT, ALICE, true, 1).unwrap();

        Assets::mint(Origin::signed(ALICE), DOT, ALICE, 100 * DOT_DECIMAL).unwrap();
        Assets::mint(Origin::signed(ALICE), DOT, ALICE, 100 * DOT_DECIMAL).unwrap();
    });

    ext
}
