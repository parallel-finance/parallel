#![cfg(test)]

use super::*;
use frame_support::{
    parameter_types,
    traits::{fungibles::InspectMetadata, ChangeMembers, EnsureOneOf, Everything},
};
use frame_system::{self as system, EnsureRoot};
use primitives::tokens::{HKO, KSM};
use system::EnsureSignedBy;

use crate::{self as bridge, ChainId, Config};
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{IdentityLookup, Zero},
};

pub type BlockNumber = u64;
pub type AccountId = u128;

type EnsureRootOrigin = EnsureRoot<AccountId>;

pub type CapOrigin = EnsureOneOf<EnsureRoot<AccountId>, EnsureSignedBy<AliceOrigin, AccountId>>;

pub struct AliceOrigin;
impl SortedMembers<AccountId> for AliceOrigin {
    fn sorted_members() -> Vec<AccountId> {
        vec![ALICE]
    }
}

// Account Ids
pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const CHARLIE: AccountId = 3;
pub const DAVE: AccountId = 4;
pub const EVE: AccountId = 5;
// pub const FERDIE: AccountId = 6;

// Chain Ids
pub const ETH: ChainId = 1;
pub const BNB: ChainId = 2;

// Asset Ids
pub const USDT: CurrencyId = 106;

// Currency Ids
pub const EHKO: CurrencyId = 0;
pub const EUSDT: CurrencyId = 2;

// Currencies
pub const EHKO_CURRENCY: BridgeToken = BridgeToken {
    id: EHKO,
    external: false,
    fee: 0,
    enable: true,
    out_cap: 100000000000000,
    in_cap: 100000000000000,
    out_amount: 0,
    in_amount: 0,
};

pub const EUSDT_CURRENCY: BridgeToken = BridgeToken {
    id: EUSDT,
    external: true,
    fee: 0,
    enable: true,
    out_cap: 100000000,
    in_cap: 100000000,
    out_amount: 0,
    in_amount: 0,
};

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
    pub const ExistentialDeposit: Balance = 10_000_000_000; // 0.01 HKO
    pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for Test {
    type Balance = Balance;
    type DustRemoval = ();
    type Event = Event;
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxLocks = MaxLocks;
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
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
    pub const BridgeMaxMembers: u32 = 100;
}

pub struct ChangeBridgeMembers;
impl ChangeMembers<AccountId> for ChangeBridgeMembers {
    fn change_members_sorted(_incoming: &[AccountId], _outgoing: &[AccountId], _new: &[AccountId]) {
        if let Err(e) = Bridge::change_vote_threshold() {
            log::error!(
                target: "bridge::change_members_sorted",
                "Failed to set vote threshold: {:?}",
                e,
            );
        };
    }
}

type BridgeMembershipInstance = pallet_membership::Instance1;
impl pallet_membership::Config<BridgeMembershipInstance> for Test {
    type Event = Event;
    type AddOrigin = EnsureRootOrigin;
    type RemoveOrigin = EnsureRootOrigin;
    type SwapOrigin = EnsureRootOrigin;
    type ResetOrigin = EnsureRootOrigin;
    type PrimeOrigin = EnsureRootOrigin;
    type MembershipInitialized = ();
    type MembershipChanged = ChangeBridgeMembers;
    type MaxMembers = BridgeMaxMembers;
    type WeightInfo = ();
}

parameter_types! {
    pub const ParallelHeiko: ChainId = 0;
    pub const BridgePalletId: PalletId = PalletId(*b"par/brid");
    pub const ProposalLifetime: BlockNumber = 50;
    pub const RootOperatorAccountId: AccountId = 7;
    pub const ThresholdPercentage: u32 = 80;
    pub GiftAccount: AccountId = PalletId(*b"par/gift").into_account();
}

pub struct GiftConvert;
impl BalanceConversion<Balance, CurrencyId, Balance> for GiftConvert {
    type Error = DispatchError;
    fn to_asset_balance(balance: Balance, asset_id: CurrencyId) -> Result<Balance, Self::Error> {
        let decimal = <Assets as InspectMetadata<AccountId>>::decimals(&asset_id);
        if decimal.is_zero() {
            return Ok(Zero::zero());
        }

        let default_gift_amount: u128 = 1_000_000_000_000 / 40; // 0.025HKO
        Ok(match asset_id {
            USDT if balance >= 300 * 10_u128.pow(decimal.into()) => default_gift_amount,
            HKO if balance >= 10_u128.pow((decimal - 1).into()) => default_gift_amount,
            _ => Zero::zero(),
        })
    }
}

impl Config for Test {
    type Event = Event;
    type RelayMembers = BridgeMembership;
    type RootOperatorAccountId = RootOperatorAccountId;
    type UpdateChainOrigin = EnsureRoot<AccountId>;
    type UpdateTokenOrigin = EnsureRoot<AccountId>;
    type CapOrigin = CapOrigin;
    type ChainId = ParallelHeiko;
    type PalletId = BridgePalletId;
    type Assets = CurrencyAdapter;
    type GiftAccount = GiftAccount;
    type GiftConvert = GiftConvert;
    type NativeCurrencyId = NativeCurrencyId;
    type ExistentialDeposit = ExistentialDeposit;
    type ProposalLifetime = ProposalLifetime;
    type ThresholdPercentage = ThresholdPercentage;

    type WeightInfo = ();
}

pub type Block = sp_runtime::generic::Block<Header, UncheckedExtrinsic>;
pub type UncheckedExtrinsic = sp_runtime::generic::UncheckedExtrinsic<u32, u64, Call, ()>;

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        System: system::{Pallet, Call, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Event<T>},
        Assets: pallet_assets::{Pallet, Call, Storage, Event<T>},
        CurrencyAdapter: pallet_currency_adapter::{Pallet, Call},
        Bridge: bridge::{Pallet, Call, Storage, Event<T>},
        // Membership
        BridgeMembership: pallet_membership::<Instance1>::{Pallet, Call, Storage, Event<T>, Config<T>},
    }
);

pub(crate) fn new_test_ext() -> sp_io::TestExternalities {
    let t = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| {
        Assets::force_create(Origin::root(), KSM, ALICE, true, 1).unwrap();
        Assets::force_set_metadata(
            Origin::root(),
            KSM,
            b"Kusama".to_vec(),
            b"KSM".to_vec(),
            12,
            false,
        )
        .unwrap();
        Assets::force_create(Origin::root(), USDT, ALICE, true, 1).unwrap();
        Assets::force_set_metadata(
            Origin::root(),
            USDT,
            b"USDT".to_vec(),
            b"USDT".to_vec(),
            6,
            false,
        )
        .unwrap();

        Balances::set_balance(Origin::root(), EVE, dollar(100), dollar(0)).unwrap();
        Balances::set_balance(
            Origin::root(),
            PalletId(*b"par/gift").into_account(),
            dollar(1000000),
            dollar(0),
        )
        .unwrap();

        BridgeMembership::add_member(Origin::root(), ALICE).unwrap();
        BridgeMembership::add_member(Origin::root(), BOB).unwrap();
        BridgeMembership::add_member(Origin::root(), CHARLIE).unwrap();

        Bridge::register_chain(Origin::root(), ETH).unwrap();
        Bridge::register_bridge_token(Origin::root(), HKO, EHKO_CURRENCY).unwrap();
        Bridge::register_bridge_token(Origin::root(), USDT, EUSDT_CURRENCY).unwrap();

        System::set_block_number(0);
        run_to_block(1);
    });
    ext
}

// Checks events against the latest.
pub fn assert_events(mut expected: Vec<Event>) {
    let mut actual: Vec<Event> = system::Pallet::<Test>::events()
        .iter()
        .map(|e| e.event.clone())
        .collect();

    expected.reverse();

    for evt in expected {
        let next = actual.pop().expect("event expected");
        assert_eq!(next, evt.into(), "Events don't match (actual,expected)");
    }
}

/// Progress to the given block, and then finalize the block.
pub(crate) fn run_to_block(n: BlockNumber) {
    for b in (System::block_number() + 1)..=n {
        System::set_block_number(b);
    }
}

pub fn dollar(d: u128) -> u128 {
    d.saturating_mul(10_u128.pow(12))
}

pub fn usdt(d: u128) -> u128 {
    d.saturating_mul(10_u128.pow(6))
}
