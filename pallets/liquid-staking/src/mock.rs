use frame_support::{
    construct_runtime,
    dispatch::Weight,
    parameter_types, sp_io,
    traits::{Contains, Everything, GenesisBuild, SortedMembers},
    weights::constants::WEIGHT_PER_SECOND,
    PalletId,
};
use frame_system::{EnsureRoot, EnsureSignedBy};
use orml_traits::parameter_type_with_key;
use primitives::TokenSymbol;

use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{AccountIdConversion, BlakeTwo256, Convert, IdentityLookup, One},
    AccountId32,
};

use primitives::{Amount, Balance, CurrencyId, Rate, Ratio};

use cumulus_primitives_core::ParaId;
use orml_xcm_support::{IsNativeConcrete, MultiCurrencyAdapter};
use pallet_xcm::XcmPassthrough;
use polkadot_parachain::primitives::Sibling;
pub use xcm::v0::{
    Error as XcmError,
    Junction::{self, GeneralKey, Parachain, Parent},
    MultiAsset,
    MultiLocation::{self, X1, X2, X3},
    NetworkId, Xcm,
};
pub use xcm_builder::{
    AccountId32Aliases, AllowTopLevelPaidExecutionFrom, AllowUnpaidExecutionFrom,
    ChildParachainAsNative, ChildParachainConvertsVia, ChildSystemParachainAsSuperuser,
    CurrencyAdapter as XcmCurrencyAdapter, EnsureXcmOrigin, FixedRateOfConcreteFungible,
    FixedWeightBounds, IsConcrete, LocationInverter, NativeAsset, ParentAsSuperuser,
    ParentIsDefault, RelayChainAsNative, SiblingParachainAsNative, SiblingParachainConvertsVia,
    SignedAccountId32AsNative, SignedToAccountId32, SovereignSignedViaLocation, TakeWeightCredit,
};
use xcm_executor::{Config, XcmExecutor};
use xcm_simulator::{decl_test_network, decl_test_parachain, decl_test_relay_chain};
pub type AccountId = AccountId32;
pub use westend_runtime;

parameter_types! {
    pub const ReservedXcmpWeight: Weight = WEIGHT_PER_SECOND / 4;
    pub const ReservedDmpWeight: Weight = WEIGHT_PER_SECOND / 4;
}

impl cumulus_pallet_parachain_system::Config for Test {
    type Event = Event;
    type OnValidationData = ();
    type SelfParaId = ParachainInfo;
    type DmpMessageHandler = DmpQueue;
    type ReservedDmpWeight = ReservedDmpWeight;
    type OutboundXcmpMessageSource = XcmpQueue;
    type XcmpMessageHandler = XcmpQueue;
    type ReservedXcmpWeight = ReservedXcmpWeight;
}

impl parachain_info::Config for Test {}

parameter_types! {
    pub const DotLocation: MultiLocation = MultiLocation::X1(Parent);
    pub const RelayNetwork: NetworkId = NetworkId::Kusama;
    pub RelayChainOrigin: Origin = cumulus_pallet_xcm::Origin::Relay.into();
    pub Ancestry: MultiLocation = Parachain(ParachainInfo::parachain_id().into()).into();
}

pub type LocationToAccountId = (
    ParentIsDefault<AccountId>,
    SiblingParachainConvertsVia<Sibling, AccountId>,
    AccountId32Aliases<RelayNetwork, AccountId>,
);

pub type XcmOriginToCallOrigin = (
    SovereignSignedViaLocation<LocationToAccountId, Origin>,
    RelayChainAsNative<RelayChainOrigin, Origin>,
    SiblingParachainAsNative<cumulus_pallet_xcm::Origin, Origin>,
    SignedAccountId32AsNative<RelayNetwork, Origin>,
    XcmPassthrough<Origin>,
);

parameter_types! {
    pub const UnitWeightCost: Weight = 1;
    pub DotPerSecond: (MultiLocation, u128) = (X1(Parent), 1);
}

pub type LocalAssetTransactor = MultiCurrencyAdapter<
    Tokens,
    (),
    IsNativeConcrete<CurrencyId, CurrencyIdConvert>,
    AccountId,
    LocationToAccountId,
    CurrencyId,
    CurrencyIdConvert,
>;

pub type XcmRouter = ParachainXcmRouter<ParachainInfo>;
pub type Barrier = AllowUnpaidExecutionFrom<Everything>;

pub struct XcmConfig;
impl Config for XcmConfig {
    type Call = Call;
    type XcmSender = XcmRouter;
    type AssetTransactor = LocalAssetTransactor;
    type OriginConverter = XcmOriginToCallOrigin;
    type IsReserve = NativeAsset;
    type IsTeleporter = ();
    type LocationInverter = LocationInverter<Ancestry>;
    type Barrier = Barrier;
    type Weigher = FixedWeightBounds<UnitWeightCost, Call>;
    type Trader = FixedRateOfConcreteFungible<DotPerSecond, ()>;
    type ResponseHandler = ();
}

impl cumulus_pallet_xcmp_queue::Config for Test {
    type Event = Event;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type ChannelInfo = ParachainSystem;
}

impl cumulus_pallet_dmp_queue::Config for Test {
    type Event = Event;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
}

impl cumulus_pallet_xcm::Config for Test {
    type Event = Event;
    type XcmExecutor = XcmExecutor<XcmConfig>;
}

pub type LocalOriginToLocation = SignedToAccountId32<Origin, AccountId, RelayNetwork>;

impl pallet_xcm::Config for Test {
    type Event = Event;
    type SendXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
    type XcmRouter = XcmRouter;
    type ExecuteXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
    type XcmExecuteFilter = Everything;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type XcmTeleportFilter = ();
    type XcmReserveTransferFilter = Everything;
    type Weigher = FixedWeightBounds<UnitWeightCost, Call>;
    type LocationInverter = LocationInverter<Ancestry>;
}

pub struct CurrencyIdConvert;
impl Convert<CurrencyId, Option<MultiLocation>> for CurrencyIdConvert {
    fn convert(id: CurrencyId) -> Option<MultiLocation> {
        match id {
            CurrencyId::Token(TokenSymbol::DOT) => Some(X1(Parent)),
            CurrencyId::Token(TokenSymbol::xDOT) => Some(X3(
                Parent,
                Parachain(ParachainInfo::parachain_id().into()),
                GeneralKey(b"xDOT".to_vec()),
            )),
            _ => None,
        }
    }
}

impl Convert<MultiLocation, Option<CurrencyId>> for CurrencyIdConvert {
    fn convert(location: MultiLocation) -> Option<CurrencyId> {
        match location {
            X1(Parent) => Some(CurrencyId::Token(TokenSymbol::DOT)),
            X3(Parent, Parachain(id), GeneralKey(key))
                if ParaId::from(id) == ParachainInfo::parachain_id() && key == b"xDOT".to_vec() =>
            {
                Some(CurrencyId::Token(TokenSymbol::xDOT))
            }
            _ => None,
        }
    }
}

impl Convert<MultiAsset, Option<CurrencyId>> for CurrencyIdConvert {
    fn convert(a: MultiAsset) -> Option<CurrencyId> {
        if let MultiAsset::ConcreteFungible { id, amount: _ } = a {
            Self::convert(id)
        } else {
            None
        }
    }
}

pub struct AccountIdToMultiLocation;
impl Convert<AccountId, MultiLocation> for AccountIdToMultiLocation {
    fn convert(account_id: AccountId) -> MultiLocation {
        MultiLocation::from(Junction::AccountId32 {
            network: NetworkId::Any,
            id: account_id.into(),
        })
    }
}

parameter_types! {
    pub SelfLocation: MultiLocation = X2(Parent, Parachain(ParachainInfo::parachain_id().into()));
    pub const BaseXcmWeight: Weight = 100_000_000;
}

impl orml_xtokens::Config for Test {
    type Event = Event;
    type Balance = Balance;
    type CurrencyId = CurrencyId;
    type CurrencyIdConvert = CurrencyIdConvert;
    type AccountIdToMultiLocation = AccountIdToMultiLocation;
    type SelfLocation = SelfLocation;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type Weigher = FixedWeightBounds<UnitWeightCost, Call>;
    type BaseXcmWeight = BaseXcmWeight;
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
type BlockNumber = u64;
pub const DOT_DECIMAL: u128 = 10u128.pow(10);

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
    type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
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
        vec![ALICE]
    }
}

pub type BridgeOrigin = EnsureSignedBy<AliceOrigin, AccountId>;

parameter_types! {
    pub const StakingPalletId: PalletId = PalletId(*b"par/lqsk");
    pub const StakingCurrency: CurrencyId = CurrencyId::Token(TokenSymbol::DOT);
    pub const LiquidCurrency: CurrencyId = CurrencyId::Token(TokenSymbol::xDOT);
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
    type StakingCurrency = StakingCurrency;
    type LiquidCurrency = LiquidCurrency;
    type PalletId = StakingPalletId;
    type BridgeOrigin = BridgeOrigin;
    type BaseXcmWeight = BaseXcmWeight;
    type XcmTransfer = XTokens;
    type RelayAgent = Agent;
    type PeriodBasis = PeriodBasis;
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
        ParachainSystem: cumulus_pallet_parachain_system::{Pallet, Call, Config, Storage, Inherent, Event<T>},
        ParachainInfo: parachain_info::{Pallet, Storage, Config},
        XcmpQueue: cumulus_pallet_xcmp_queue::{Pallet, Call, Storage, Event<T>},
        DmpQueue: cumulus_pallet_dmp_queue::{Pallet, Call, Storage, Event<T>},
        CumulusXcm: cumulus_pallet_xcm::{Pallet, Event<T>, Origin},
        PolkadotXcm: pallet_xcm::{Pallet, Call, Event<T>, Origin},

        XTokens: orml_xtokens::{Pallet, Storage, Call, Event<T>},
    }
);

pub const ALICE: AccountId32 = AccountId32::new([1u8; 32]);

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

//initial parchain and relaychain for testing
decl_test_parachain! {
    pub struct ParaA {
        Runtime = Test,
        XcmpMessageHandler = XcmpQueue,
        DmpMessageHandler = DmpQueue,
        new_ext = para_ext(1),
    }
}

decl_test_relay_chain! {
    pub struct Relay {
        Runtime = westend_runtime::Runtime,
        XcmConfig = westend_runtime::XcmConfig,
        new_ext = relay_ext(),
    }
}

decl_test_network! {
    pub struct TestNet {
        relay_chain = Relay,
        parachains = vec![
            (1, ParaA),
        ],
    }
}

pub type RelayBalances = pallet_balances::Pallet<westend_runtime::Runtime>;

pub fn para_ext(para_id: u32) -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();

    let parachain_info_config = parachain_info::GenesisConfig {
        parachain_id: para_id.into(),
    };
    <parachain_info::GenesisConfig as GenesisBuild<Test, _>>::assimilate_storage(
        &parachain_info_config,
        &mut t,
    )
    .unwrap();

    orml_tokens::GenesisConfig::<Test> {
        balances: vec![(
            ALICE,
            CurrencyId::Token(TokenSymbol::DOT),
            1_000 * DOT_DECIMAL,
        )],
    }
    .assimilate_storage(&mut t)
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
    ext.execute_with(|| System::set_block_number(1));
    ext
}

pub fn relay_ext() -> sp_io::TestExternalities {
    use westend_runtime::{Runtime, System};
    let para_a = ParaId::from(1).into_account();
    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Runtime>()
        .unwrap();

    pallet_balances::GenesisConfig::<Runtime> {
        balances: vec![(ALICE, 100 * DOT_DECIMAL), (para_a, 100 * DOT_DECIMAL)],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}
