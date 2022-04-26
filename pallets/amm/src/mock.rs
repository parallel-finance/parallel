use crate as pallet_amm;

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{parameter_types, traits::Everything, traits::SortedMembers, PalletId};
use frame_system::{self as system, EnsureRoot};
use primitives::{tokens, Balance, CurrencyId, Ratio};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    RuntimeDebug,
};
use system::EnsureSignedBy;

pub const DOT: CurrencyId = tokens::DOT;
pub const SDOT: CurrencyId = tokens::SDOT;
pub const KSM: CurrencyId = tokens::KSM;
pub const SAMPLE_LP_TOKEN: CurrencyId = 42;
pub const SAMPLE_LP_TOKEN_2: CurrencyId = 43;

pub const ALICE: AccountId = AccountId(1);
pub const BOB: AccountId = AccountId(2);
pub const CHARLIE: AccountId = AccountId(3);
pub const EVE: AccountId = AccountId(4);
pub const FRANK: AccountId = AccountId(5);
pub const PROTOCOL_FEE_RECEIVER: AccountId = AccountId(99);

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
type BlockNumber = u64;

#[derive(
    Encode,
    Decode,
    Default,
    Eq,
    PartialEq,
    Copy,
    Clone,
    RuntimeDebug,
    PartialOrd,
    Ord,
    MaxEncodedLen,
    TypeInfo,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Hash))]
pub struct AccountId(pub u64);

impl sp_std::fmt::Display for AccountId {
    fn fmt(&self, f: &mut sp_std::fmt::Formatter<'_>) -> sp_std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u64> for AccountId {
    fn from(account_id: u64) -> Self {
        Self(account_id)
    }
}

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Event<T>},
        AMM: pallet_amm::{Pallet, Call, Storage, Event<T>},
        Assets: pallet_assets::{Pallet, Call, Storage, Event<T>},
        CurrencyAdapter: pallet_currency_adapter::{Pallet, Call},
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
    pub const AMMPalletId: PalletId = PalletId(*b"par/ammp");
    pub DefaultLpFee: Ratio = Ratio::from_rational(25u32, 10000u32);        // 0.25%
    pub DefaultProtocolFee: Ratio = Ratio::from_rational(5u32, 10000u32);   // 0.05%
    pub const DefaultProtocolFeeReceiver: AccountId = PROTOCOL_FEE_RECEIVER;
    pub const MinimumLiquidity: u128 = 1_000u128;
    pub const LockAccountId: AccountId = AccountId(1_u64);
    pub const MaxLengthRoute: u8 = 10;
}

pub struct AliceCreatePoolOrigin;
impl SortedMembers<AccountId> for AliceCreatePoolOrigin {
    fn sorted_members() -> Vec<AccountId> {
        vec![ALICE]
    }
}

impl pallet_amm::Config for Test {
    type Event = Event;
    type Assets = CurrencyAdapter;
    type PalletId = AMMPalletId;
    type LockAccountId = LockAccountId;
    type AMMWeightInfo = ();
    type CreatePoolOrigin = EnsureSignedBy<AliceCreatePoolOrigin, AccountId>;
    type LpFee = DefaultLpFee;
    type ProtocolFee = DefaultProtocolFee;
    type MinimumLiquidity = MinimumLiquidity;
    type ProtocolFeeReceiver = DefaultProtocolFeeReceiver;
    type MaxLengthRoute = MaxLengthRoute;
    type GetNativeCurrencyId = NativeCurrencyId;
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
        balances: vec![
            (ALICE, 100_000_000),
            (BOB, 100_000_000),
            (CHARLIE, 1000_000_000),
            (EVE, 1000_000_000),
            (FRANK, 1000_000_000),
        ],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| {
        Assets::force_create(Origin::root(), tokens::DOT, ALICE, true, 1).unwrap();
        Assets::force_create(Origin::root(), tokens::SDOT, ALICE, true, 1).unwrap();
        Assets::force_create(Origin::root(), tokens::KSM, ALICE, true, 1).unwrap();
        Assets::force_create(Origin::root(), SAMPLE_LP_TOKEN, ALICE, true, 1).unwrap();
        Assets::force_create(Origin::root(), SAMPLE_LP_TOKEN_2, ALICE, true, 1).unwrap();

        Assets::mint(Origin::signed(ALICE), tokens::DOT, ALICE, 100_000_000).unwrap();

        Assets::mint(
            Origin::signed(ALICE),
            tokens::DOT,
            BOB,
            100_000_000_000_000_000_000,
        )
        .unwrap();
        Assets::mint(Origin::signed(ALICE), tokens::DOT, CHARLIE, 1000_000_000).unwrap();
        Assets::mint(Origin::signed(ALICE), tokens::DOT, EVE, 1000_000_000).unwrap();
        Assets::mint(
            Origin::signed(ALICE),
            tokens::DOT,
            FRANK,
            100_000_000_000_000_000_000,
        )
        .unwrap();

        Assets::mint(Origin::signed(ALICE), tokens::SDOT, ALICE, 100_000_000).unwrap();
        Assets::mint(
            Origin::signed(ALICE),
            tokens::SDOT,
            BOB,
            100_000_000_000_000_000_000,
        )
        .unwrap();
        Assets::mint(Origin::signed(ALICE), tokens::SDOT, CHARLIE, 1000_000_000).unwrap();
        Assets::mint(Origin::signed(ALICE), tokens::SDOT, EVE, 1000_000_000).unwrap();

        Assets::mint(Origin::signed(ALICE), tokens::KSM, ALICE, 100_000_000).unwrap();
        Assets::mint(Origin::signed(ALICE), tokens::KSM, BOB, 100_000_000).unwrap();
        Assets::mint(
            Origin::signed(ALICE),
            tokens::KSM,
            FRANK,
            100_000_000_000_000_000_000,
        )
        .unwrap();
    });

    ext
}

/// Progress to the given block, and then finalize the block.
pub(crate) fn run_to_block(n: BlockNumber) {
    for b in (System::block_number() + 1)..=n {
        System::set_block_number(b);
    }
}
