use frame_support::{
    construct_runtime,
    dispatch::DispatchResult,
    dispatch::Weight,
    parameter_types, sp_io,
    traits::{GenesisBuild, SortedMembers},
    PalletId,
};
use frame_system::{EnsureRoot, EnsureSignedBy};
use orml_traits::XcmTransfer;

use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup, One},
};

use xcm::latest::prelude::*;

use primitives::{tokens::*, Balance, Rate, Ratio};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
type BlockNumber = u64;
type AccountId = u64;
type CurrencyId = u32;

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
    pub const AssetDeposit: Balance = 0;
    pub const ApprovalDeposit: Balance = 0;
    pub const AssetsStringLimit: u32 = 50;
    pub const MetadataDepositBase: Balance = 0;
    pub const MetadataDepositPerByte: Balance = 0;
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

pub struct AliceOrigin;
impl SortedMembers<AccountId> for AliceOrigin {
    fn sorted_members() -> Vec<AccountId> {
        vec![ALICE.into()]
    }
}

pub struct BobOrigin;
impl SortedMembers<AccountId> for BobOrigin {
    fn sorted_members() -> Vec<AccountId> {
        vec![BOB.into()]
    }
}

pub type BridgeOrigin = EnsureSignedBy<AliceOrigin, AccountId>;
pub type UpdateOrigin = EnsureSignedBy<BobOrigin, AccountId>;

parameter_types! {
    pub const StakingPalletId: PalletId = PalletId(*b"par/lqsk");
    pub const BaseXcmWeight: Weight = 0;
    pub Agent: MultiLocation = MultiLocation::new(1, Junctions::X1(Junction::AccountId32 {
        network: xcm::v0::NetworkId::Any,
        id: [0; 32]
    }));
    pub const PeriodBasis: BlockNumber = 5u64;
}

impl crate::Config for Test {
    type Event = Event;
    type PalletId = StakingPalletId;
    type BridgeOrigin = BridgeOrigin;
    type BaseXcmWeight = BaseXcmWeight;
    type XcmTransfer = MockXcmTransfer;
    type RelayAgent = Agent;
    type PeriodBasis = PeriodBasis;
    type WeightInfo = ();
    type Assets = Assets;
    type UpdateOrigin = UpdateOrigin;
}

construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Assets: pallet_assets::{Pallet, Call, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Event<T>},
        LiquidStaking: crate::{Pallet, Storage, Call, Event<T>},
    }
);

pub const ALICE: AccountId = 1u64;
pub const BOB: AccountId = 2u64;

pub struct MockXcmTransfer;
impl XcmTransfer<AccountId, Balance, CurrencyId> for MockXcmTransfer {
    fn transfer(
        _who: AccountId,
        _currency_id: CurrencyId,
        _amount: Balance,
        _to: MultiLocation,
        _dest_weight: Weight,
    ) -> DispatchResult {
        Ok(().into())
    }

    fn transfer_multi_asset(
        _who: AccountId,
        _asset: MultiAsset,
        _dest: MultiLocation,
        _dest_weight: Weight,
    ) -> DispatchResult {
        Ok(().into())
    }
}

pub(crate) fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();

    GenesisBuild::<Test>::assimilate_storage(
        &crate::GenesisConfig {
            exchange_rate: Rate::one(),
            reserve_factor: Ratio::from_perthousand(5),
        },
        &mut t,
    )
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| {
        Assets::force_create(Origin::root(), DOT, ALICE, true, 1).unwrap();
        Assets::force_create(Origin::root(), XDOT, ALICE, true, 1).unwrap();
        Assets::mint(Origin::signed(ALICE), DOT, ALICE, 100).unwrap();
        Assets::mint(Origin::signed(ALICE), XDOT, ALICE, 100).unwrap();

        LiquidStaking::set_liquid_currency(Origin::signed(BOB), XDOT).unwrap();
        LiquidStaking::set_staking_currency(Origin::signed(BOB), DOT).unwrap();
    });

    ext
}
