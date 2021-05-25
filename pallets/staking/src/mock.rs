use crate as pallet_liquid_staking;
use frame_support::{ord_parameter_types, parameter_types, traits::GenesisBuild, PalletId};
use frame_system::{self as system, EnsureSignedBy};
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    FixedPointNumber,
};

use orml_traits::parameter_type_with_key;

use primitives::{Amount, Balance, CurrencyId, Rate};

pub const DOT: CurrencyId = CurrencyId::DOT;
pub const XDOT: CurrencyId = CurrencyId::xDOT;
pub const NATIVE: CurrencyId = CurrencyId::Native;

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
        Tokens: orml_tokens::{Pallet, Storage, Config<T>, Event<T>},
        Currencies: orml_currencies::{Pallet, Call, Event<T>},
        LiquidStaking: pallet_liquid_staking::{Pallet, Storage, Call, Config, Event<T>},
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
    type AccountId = u64;
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
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
}

parameter_type_with_key! {
    pub ExistentialDeposits: |_currency_id: CurrencyId| -> Balance {
        Default::default()
    };
}

impl orml_tokens::Config for Test {
    type Event = Event;
    type Balance = Balance;
    type Amount = Amount;
    type CurrencyId = CurrencyId;
    type OnDust = ();
    type ExistentialDeposits = ExistentialDeposits;
    type WeightInfo = ();
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

ord_parameter_types! {
    pub const Six: u64 = 6;
}

parameter_types! {
    pub const LiquidStakingPalletId: PalletId = PalletId(*b"par/liqu");
    pub const StakingCurrency: CurrencyId = DOT;
    pub const LiquidCurrency: CurrencyId = XDOT;
    pub const MaxWithdrawAmount: Balance = 10;
    pub const MaxAccountProcessingUnstake: u32 = 5;
}

impl pallet_liquid_staking::Config for Test {
    type Event = Event;
    type Currency = Currencies;
    type PalletId = LiquidStakingPalletId;
    type StakingCurrency = StakingCurrency;
    type LiquidCurrency = LiquidCurrency;
    type WithdrawOrigin = EnsureSignedBy<Six, u64>;
    type MaxWithdrawAmount = MaxWithdrawAmount;
    type MaxAccountProcessingUnstake = MaxAccountProcessingUnstake;
}

// BUild genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();
    orml_tokens::GenesisConfig::<Test> {
        endowed_accounts: vec![(1, CurrencyId::DOT, 100)],
    }
    .assimilate_storage(&mut t)
    .unwrap();
    pallet_liquid_staking::GenesisConfig {
        exchange_rate: Rate::saturating_from_rational(2, 100), // 0.02
    }
    .assimilate_storage::<Test>(&mut t)
    .unwrap();
    t.into()
}
