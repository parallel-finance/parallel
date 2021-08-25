use crate as pallet_amm;

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::traits::Contains;
use frame_support::{parameter_types, traits::GenesisBuild, PalletId};
use frame_system as system;
use orml_traits::parameter_type_with_key;
use primitives::{Amount, Balance, CurrencyId};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::H256;
use sp_runtime::traits::AccountIdConversion;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    RuntimeDebug,
};

pub const DOT: CurrencyId = CurrencyId::DOT;
pub const XDOT: CurrencyId = CurrencyId::xDOT;
pub const NATIVE: CurrencyId = CurrencyId::HKO;
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
        Tokens: orml_tokens::{Pallet, Storage, Config<T>, Event<T>},
        Currencies: orml_currencies::{Pallet, Call, Event<T>},
        AMM: pallet_amm::{Pallet, Call, Storage, Event<T>},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
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

parameter_type_with_key! {
    pub ExistentialDeposits: |_currency_id: CurrencyId| -> Balance {
        Default::default()
    };
}

pub struct DustRemovalWhitelist;
impl Contains<AccountId> for DustRemovalWhitelist {
    fn contains(a: &AccountId) -> bool {
        vec![AMMPalletId::get().into_account()].contains(a)
    }
}

impl orml_tokens::Config for Test {
    type Event = Event;
    type Balance = Balance;
    type Amount = Amount;
    type CurrencyId = CurrencyId;
    type OnDust = ();
    type ExistentialDeposits = ExistentialDeposits;
    type WeightInfo = ();
    type MaxLocks = MaxLocks;
    type DustRemovalWhitelist = DustRemovalWhitelist;
}

parameter_types! {
    pub const GetNativeCurrencyId: CurrencyId = NATIVE;
}

impl orml_currencies::Config for Test {
    type Event = Event;
    type MultiCurrency = Tokens;
    type NativeCurrency =
        orml_currencies::BasicCurrencyAdapter<Test, Balances, Amount, BlockNumber>;
    type GetNativeCurrencyId = GetNativeCurrencyId;
    type WeightInfo = ();
}

parameter_types! {
    pub const AMMPalletId: PalletId = PalletId(*b"par/ammp");
}

impl pallet_amm::Config for Test {
    type Event = Event;
    type Currency = Currencies;
    type PalletId = AMMPalletId;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();
    orml_tokens::GenesisConfig::<Test> {
        balances: vec![
            (1.into(), CurrencyId::DOT, 100),
            (1.into(), CurrencyId::xDOT, 100),
        ],
    }
    .assimilate_storage(&mut t)
    .unwrap();
    t.into()
}
