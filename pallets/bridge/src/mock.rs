#![cfg(test)]

use super::*;
use frame_support::{parameter_types, traits::Everything};
use frame_system::{self as system, EnsureRoot};

use crate::{self as bridge, ChainId, Config};
use sp_core::H256;
use sp_runtime::{testing::Header, traits::IdentityLookup};

pub type BlockNumber = u64;
pub type AccountId = u128;

type EnsureRootOrigin = EnsureRoot<AccountId>;

pub const ALICE: AccountId = 1;

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
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
}

parameter_types! {
    pub const BridgeMaxMembers: u32 = 100;
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
    type MembershipChanged = ();
    type MaxMembers = BridgeMaxMembers;
    type WeightInfo = ();
}

parameter_types! {
    pub const DefaultRealyerThreshold: ChainId = 1;
    pub const ZeroAccountId: AccountId = 0u128;
    pub const BridgePalletId: PalletId = PalletId(*b"par/brid");
}

impl Config for Test {
    type Event = Event;
    type AdminMembers = BridgeMembership;

    type RootOperatorAccountId = ZeroAccountId;

    type ChainId = DefaultRealyerThreshold;
    type PalletId = BridgePalletId;
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
        Bridge::set_relayer_threshold(2).unwrap();
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
