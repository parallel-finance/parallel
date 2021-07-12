use crate as pallet_liquid_staking;

use codec::{Decode, Encode};
use frame_support::{
    dispatch::{DispatchResult, Weight},
    parameter_types,
    traits::{GenesisBuild, MaxEncodedLen, SortedMembers},
    PalletId,
};
use frame_system::{
    self as system, ensure_signed, pallet_prelude::OriginFor, EnsureOneOf, EnsureRoot,
    EnsureSignedBy,
};
use orml_traits::{parameter_type_with_key, MultiCurrency};
use primitives::{Amount, Balance, CurrencyId, Rate, Ratio, XTransfer};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    FixedPointNumber, RuntimeDebug,
};
use sp_std::convert::TryInto;
use xcm::v0::{Junction, MultiLocation};

pub const DOT: CurrencyId = CurrencyId::DOT;
pub const XDOT: CurrencyId = CurrencyId::xDOT;
pub const NATIVE: CurrencyId = CurrencyId::Native;
pub const DOT_DECIMAL: u128 = 10u128.pow(10);
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
pub struct AccountId(u64);

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

impl From<AccountId> for [u8; 32] {
    fn from(account_id: AccountId) -> Self {
        let mut b: Vec<u8> = account_id.0.to_be_bytes().iter().cloned().collect();
        b.resize_with(32, Default::default);
        b.try_into().unwrap()
    }
}

impl From<[u8; 32]> for AccountId {
    fn from(account_id32: [u8; 32]) -> Self {
        AccountId::from(u64::from_be_bytes(account_id32[0..8].try_into().unwrap()))
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

impl orml_tokens::Config for Test {
    type Event = Event;
    type Balance = Balance;
    type Amount = Amount;
    type CurrencyId = CurrencyId;
    type OnDust = ();
    type ExistentialDeposits = ExistentialDeposits;
    type WeightInfo = ();
    type MaxLocks = MaxLocks;
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

pub struct Six;
impl SortedMembers<AccountId> for Six {
    fn sorted_members() -> Vec<AccountId> {
        vec![AccountId::from(6_u64)]
    }
}

type EnsureRootOrSix =
    EnsureOneOf<AccountId, EnsureRoot<AccountId>, EnsureSignedBy<Six, AccountId>>;

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
    type WithdrawOrigin = EnsureRootOrSix;
    type MaxWithdrawAmount = MaxWithdrawAmount;
    type MaxAccountProcessingUnstake = MaxAccountProcessingUnstake;
    type WeightInfo = ();
    type XTransfer = Currencies;
    type Members = Members;
}

pub struct Members;

impl SortedMembers<AccountId> for Members {
    fn sorted_members() -> Vec<AccountId> {
        vec![2.into(), 10000.into()]
    }
}

impl XTransfer<Test, CurrencyId, AccountId, Balance> for Currencies {
    fn xtransfer(
        from: OriginFor<Test>,
        currency_id: CurrencyId,
        mut to: MultiLocation,
        amount: Balance,
        _weight: Weight,
    ) -> DispatchResult {
        let from = ensure_signed(from)?;
        <Test as orml_currencies::Config>::MultiCurrency::withdraw(currency_id, &from, amount)?;
        if let Some(Junction::AccountId32 {
            id: account_id32, ..
        }) = to.take_last()
        {
            let account_id: AccountId = account_id32.into();
            <Test as orml_currencies::Config>::MultiCurrency::deposit(
                currency_id,
                &account_id,
                amount,
            )?;
        }
        Ok(())
    }
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();
    orml_tokens::GenesisConfig::<Test> {
        balances: vec![
            (1.into(), CurrencyId::DOT, 100),
            (11.into(), CurrencyId::DOT, 100 * DOT_DECIMAL),
        ],
    }
    .assimilate_storage(&mut t)
    .unwrap();
    pallet_liquid_staking::GenesisConfig {
        exchange_rate: Rate::saturating_from_rational(2, 100), // 0.02
        reserve_factor: Ratio::from_perthousand(5),
    }
    .assimilate_storage::<Test>(&mut t)
    .unwrap();
    t.into()
}
