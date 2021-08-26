use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::dispatch::DispatchResult;
use frame_support::{
    construct_runtime,
    dispatch::Weight,
    parameter_types, sp_io,
    traits::{Contains, GenesisBuild, SortedMembers},
    PalletId,
};
use frame_system::EnsureSignedBy;
use orml_traits::{parameter_type_with_key, MultiCurrency, XcmTransfer};
use serde::{Deserialize, Serialize};
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{AccountIdConversion, BlakeTwo256, IdentityLookup, One},
};
use sp_std::convert::TryInto;
use xcm::v0::{Junction, MultiAsset, MultiLocation};

use primitives::{Amount, Balance, CurrencyId, Rate};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
type BlockNumber = u64;

const DOT_DECIMAL: u128 = 10u128.pow(10);

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

parameter_type_with_key! {
    pub ExistentialDeposits: |_currency_id: CurrencyId| -> Balance {
        Default::default()
    };
}

pub struct DustRemovalWhitelist;
impl Contains<AccountId> for DustRemovalWhitelist {
    fn contains(a: &AccountId) -> bool {
        vec![StakingPalletId::get().into_account()].contains(a)
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
    pub const GetNativeCurrencyId: CurrencyId = CurrencyId::HKO;
}

impl orml_currencies::Config for Test {
    type Event = Event;
    type MultiCurrency = Tokens;
    type NativeCurrency =
        orml_currencies::BasicCurrencyAdapter<Test, Balances, Amount, BlockNumber>;
    type GetNativeCurrencyId = GetNativeCurrencyId;
    type WeightInfo = ();
}

pub struct AliceOrigin;
impl SortedMembers<AccountId> for AliceOrigin {
    fn sorted_members() -> Vec<AccountId> {
        vec![1u64.into()]
    }
}

#[derive(
    Encode, Decode, Default, Eq, PartialEq, Copy, Clone, Debug, PartialOrd, Ord, MaxEncodedLen,
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

pub type BridgeOrigin = EnsureSignedBy<AliceOrigin, AccountId>;

parameter_types! {
    pub const StakingPalletId: PalletId = PalletId(*b"par/lqsk");
    pub const StakingCurrency: CurrencyId = CurrencyId::DOT;
    pub const LiquidCurrency: CurrencyId = CurrencyId::xDOT;
    pub const BaseXcmWeight: Weight = 0;
    pub const Agent: MultiLocation = MultiLocation::X2(
        Junction::Parent,
        Junction::AccountId32 {
           network: xcm::v0::NetworkId::Any,
           id: [0; 32]
    }
    );
}

impl crate::Config for Test {
    type Event = Event;
    type Currency = Currencies;
    type StakingCurrency = StakingCurrency;
    type LiquidCurrency = LiquidCurrency;
    type PalletId = StakingPalletId;
    type BridgeOrigin = BridgeOrigin;
    type BaseXcmWeight = BaseXcmWeight;
    type XcmTransfer = Currencies;
    type Agent = Agent;
    type WeightInfo = ();
}

construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Event<T>},
        Tokens: orml_tokens::{Pallet, Storage, Config<T>, Event<T>},
        Currencies: orml_currencies::{Pallet, Call, Event<T>},
        LiquidStaking: crate::{Pallet, Storage, Call, Event<T>},
    }
);

#[allow(non_upper_case_globals)]
pub(crate) const Alice: AccountId = AccountId(1u64);
#[allow(non_upper_case_globals)]
pub(crate) const Bob: AccountId = AccountId(2u64);

impl XcmTransfer<AccountId, Balance, CurrencyId> for Currencies {
    fn transfer(
        who: AccountId,
        currency_id: CurrencyId,
        amount: Balance,
        mut to: MultiLocation,
        _dest_weight: Weight,
    ) -> DispatchResult {
        <Test as orml_currencies::Config>::MultiCurrency::withdraw(currency_id, &who, amount)?;
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
    let mut storage = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();

    orml_tokens::GenesisConfig::<Test> {
        balances: vec![
            (Alice, CurrencyId::DOT, 100),
            (Alice, CurrencyId::xDOT, 100),
            (Bob, CurrencyId::DOT, 100 * DOT_DECIMAL),
        ],
    }
    .assimilate_storage(&mut storage)
    .unwrap();

    crate::GenesisConfig {
        exchange_rate: Rate::one(),
    }
    .assimilate_storage::<Test>(&mut storage)
    .unwrap();

    storage.into()
}
