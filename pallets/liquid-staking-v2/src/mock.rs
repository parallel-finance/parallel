use frame_support::{
    construct_runtime,
    dispatch::DispatchResult,
    dispatch::Weight,
    parameter_types, sp_io,
    traits::{Contains, GenesisBuild, SortedMembers},
    PalletId,
};
use frame_system::EnsureSignedBy;
use orml_traits::{parameter_type_with_key, XcmTransfer};

use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{AccountIdConversion, BlakeTwo256, IdentityLookup, One},
};

use xcm::v0::{Junction, MultiAsset, MultiLocation};

use primitives::{Amount, Balance, CurrencyId, Rate, Ratio, TokenSymbol};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
type BlockNumber = u64;
type AccountId = u64;

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
    pub const GetNativeCurrencyId: CurrencyId = CurrencyId::Token(TokenSymbol::HKO);
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

pub type BridgeOrigin = EnsureSignedBy<AliceOrigin, AccountId>;

parameter_types! {
    pub const StakingPalletId: PalletId = PalletId(*b"par/lqsk");
    pub const StakingCurrency: CurrencyId = CurrencyId::Token(TokenSymbol::DOT);
    pub const LiquidCurrency: CurrencyId = CurrencyId::Token(TokenSymbol::xDOT);
    pub const BaseXcmWeight: Weight = 0;
    pub const Agent: MultiLocation = MultiLocation::X2(
        Junction::Parent,
        Junction::AccountId32 {
           network: xcm::v0::NetworkId::Any,
           id: [0; 32]
    });
    pub const PeriodBasis: BlockNumber = 5u64;
}

impl crate::Config for Test {
    type Event = Event;
    type Currency = Currencies;
    type PalletId = StakingPalletId;
    type BridgeOrigin = BridgeOrigin;
    type BaseXcmWeight = BaseXcmWeight;
    type XcmTransfer = MockXcmTransfer;
    type RelayAgent = Agent;
    type PeriodBasis = PeriodBasis;
    type WeightInfo = ();
    type Assets = Assets;
    type UpdateOrigin = BridgeOrigin;
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

pub const ALICE: AccountId = 1u64;

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
    let mut storage = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();

    orml_tokens::GenesisConfig::<Test> {
        balances: vec![
            (ALICE, CurrencyId::Token(TokenSymbol::DOT), 100),
            (ALICE, CurrencyId::Token(TokenSymbol::xDOT), 100),
        ],
    }
    .assimilate_storage(&mut storage)
    .unwrap();

    GenesisBuild::<Test>::assimilate_storage(
        &crate::GenesisConfig {
            exchange_rate: Rate::one(),
            reserve_factor: Ratio::from_perthousand(5),
        },
        &mut storage,
    )
    .unwrap();

    storage.into()
}
