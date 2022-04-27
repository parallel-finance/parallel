use crate as pallet_farming;

// use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{parameter_types, traits::Everything, PalletId};
use frame_system::{self as system, EnsureRoot};
use pallet_traits::DecimalProvider;
use primitives::{Balance, CurrencyId};
#[cfg(feature = "std")]
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};

pub const EHKO: CurrencyId = 0;
pub const STAKE_TOKEN: CurrencyId = 1;
pub const REWARD_TOKEN: CurrencyId = 2;
pub const BIG_DECIMAL_STAKE_TOKEN: CurrencyId = 3;
pub const BIG_DECIMAL_REWARD_TOKEN: CurrencyId = 4;
pub const LOCK_DURATION: u64 = 20;

pub type AccountId = u128;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const REWARD_TOKEN_PAYER: AccountId = 3;
pub const CHARLIE: AccountId = 4;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
type BlockNumber = u64;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Event<T>},
        Assets: pallet_assets::{Pallet, Call, Storage, Event<T>},
        CurrencyAdapter: pallet_currency_adapter::{Pallet, Call},
        Farming: pallet_farming::{Pallet, Call, Storage, Event<T>},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
    type BaseCallFilter = Everything;
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
    type MaxConsumers = frame_support::traits::ConstU32<16>;
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
    pub const FarmingPalletId: PalletId = PalletId(*b"par/farm");
    pub const MaxUserLockItemsCount: u32 = 3;
    pub const LockPoolMaxDuration: u32 = 2628000;
    pub const CoolDownMaxDuration: u32 = 50400;
}

pub struct Decimal;
impl DecimalProvider<CurrencyId> for Decimal {
    fn get_decimal(asset_id: &CurrencyId) -> Option<u8> {
        match *asset_id {
            BIG_DECIMAL_STAKE_TOKEN | BIG_DECIMAL_REWARD_TOKEN => Some(24),
            EHKO => Some(12),
            STAKE_TOKEN => Some(12),
            _ => Some(10),
        }
    }
}

impl pallet_farming::Config for Test {
    type UpdateOrigin = EnsureRoot<AccountId>;
    type WeightInfo = ();
    type Event = Event;
    type Assets = CurrencyAdapter;
    type PalletId = FarmingPalletId;
    type MaxUserLockItemsCount = MaxUserLockItemsCount;
    type LockPoolMaxDuration = LockPoolMaxDuration;
    type CoolDownMaxDuration = CoolDownMaxDuration;
    type Decimal = Decimal;
}

parameter_types! {
    pub const NativeCurrencyId: CurrencyId = 0;
}

impl pallet_currency_adapter::Config for Test {
    type Assets = Assets;
    type Balances = Balances;
    type GetNativeCurrencyId = NativeCurrencyId;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![(ALICE, 100_000_000), (BOB, 100_000_000)],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| {
        Assets::force_create(Origin::root(), STAKE_TOKEN, ALICE, true, 1).unwrap();
        Assets::force_create(Origin::root(), REWARD_TOKEN, REWARD_TOKEN_PAYER, true, 1).unwrap();
        Assets::force_create(Origin::root(), BIG_DECIMAL_STAKE_TOKEN, ALICE, true, 1).unwrap();
        Assets::force_create(
            Origin::root(),
            BIG_DECIMAL_REWARD_TOKEN,
            REWARD_TOKEN_PAYER,
            true,
            1,
        )
        .unwrap();

        Assets::mint(Origin::signed(ALICE), STAKE_TOKEN, ALICE, 500_000_000).unwrap();
        Assets::mint(Origin::signed(ALICE), STAKE_TOKEN, BOB, 500_000_000).unwrap();
        Assets::mint(
            Origin::signed(ALICE),
            STAKE_TOKEN,
            CHARLIE,
            1_100_000_000_000_000,
        )
        .unwrap();
        Assets::mint(
            Origin::signed(REWARD_TOKEN_PAYER),
            REWARD_TOKEN,
            REWARD_TOKEN_PAYER,
            3_000_000_000_000_000,
        )
        .unwrap();
        Assets::mint(
            Origin::signed(ALICE),
            BIG_DECIMAL_STAKE_TOKEN,
            ALICE,
            100_000_000_000_000_000_000_000_000,
        )
        .unwrap();
        Assets::mint(
            Origin::signed(REWARD_TOKEN_PAYER),
            BIG_DECIMAL_REWARD_TOKEN,
            REWARD_TOKEN_PAYER,
            11_000_000_000_000_000_000_000_000_000_000,
        )
        .unwrap();

        Farming::create(
            Origin::root(),
            STAKE_TOKEN,
            REWARD_TOKEN,
            LOCK_DURATION,
            100,
        )
        .unwrap();
        let pool_info = Farming::pools((STAKE_TOKEN, REWARD_TOKEN, LOCK_DURATION)).unwrap();
        assert_eq!(pool_info.is_active, false);
        Farming::set_pool_status(
            Origin::root(),
            STAKE_TOKEN,
            REWARD_TOKEN,
            LOCK_DURATION,
            true,
        )
        .unwrap();

        System::set_block_number(0);
        run_to_block(1);
    });

    ext
}

/// Progress to the given block, and then finalize the block.
pub(crate) fn run_to_block(n: BlockNumber) {
    for b in (System::block_number() + 1)..=n {
        System::set_block_number(b);
    }
}
