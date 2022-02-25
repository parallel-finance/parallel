use frame_support::{
    construct_runtime,
    dispatch::Weight,
    parameter_types, sp_io,
    traits::{EnsureOneOf, Everything, GenesisBuild, Nothing, SortedMembers},
    weights::constants::WEIGHT_PER_SECOND,
    PalletId,
};
use frame_system::{EnsureRoot, EnsureSignedBy};
use orml_xcm_support::IsNativeConcrete;
use pallet_xcm::XcmPassthrough;
use polkadot_parachain::primitives::Sibling;

use polkadot_runtime_parachains::configuration::HostConfiguration;
use primitives::{currency::MultiCurrencyAdapter, tokens::*, Balance, ParaId, Rate, Ratio};
use sp_core::H256;
use sp_runtime::{
    generic,
    traits::{
        AccountIdConversion, AccountIdLookup, BlakeTwo256, BlockNumberProvider, Convert, One, Zero,
    },
    AccountId32,
    MultiAddress::Id,
};
pub use xcm::latest::prelude::*;
pub use xcm_builder::{
    AccountId32Aliases, AllowTopLevelPaidExecutionFrom, AllowUnpaidExecutionFrom,
    ChildParachainAsNative, ChildParachainConvertsVia, ChildSystemParachainAsSuperuser,
    CurrencyAdapter as XcmCurrencyAdapter, EnsureXcmOrigin, FixedRateOfFungible, FixedWeightBounds,
    IsConcrete, LocationInverter, NativeAsset, ParentAsSuperuser, ParentIsDefault,
    RelayChainAsNative, SiblingParachainAsNative, SiblingParachainConvertsVia,
    SignedAccountId32AsNative, SignedToAccountId32, SovereignSignedViaLocation, TakeWeightCredit,
};
use xcm_executor::{Config, XcmExecutor};
use xcm_simulator::{decl_test_network, decl_test_parachain, decl_test_relay_chain};

pub type AccountId = AccountId32;
pub type CurrencyId = u32;
pub use kusama_runtime;

use super::UnbondIndex;

parameter_types! {
    pub const ReservedXcmpWeight: Weight = WEIGHT_PER_SECOND / 4;
    pub const ReservedDmpWeight: Weight = WEIGHT_PER_SECOND / 4;
}

impl cumulus_pallet_parachain_system::Config for Test {
    type Event = Event;
    type OnSystemEvent = ();
    type SelfParaId = ParachainInfo;
    type DmpMessageHandler = DmpQueue;
    type ReservedDmpWeight = ReservedDmpWeight;
    type OutboundXcmpMessageSource = XcmpQueue;
    type XcmpMessageHandler = XcmpQueue;
    type ReservedXcmpWeight = ReservedXcmpWeight;
}

impl parachain_info::Config for Test {}

parameter_types! {
    pub DotLocation: MultiLocation = MultiLocation::parent();
    pub RelayNetwork: NetworkId = NetworkId::Kusama;
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
    pub DotPerSecond: (AssetId, u128) = (AssetId::Concrete(MultiLocation::parent()), 1);
}

parameter_types! {
    pub const NativeCurrencyId: CurrencyId = HKO;
    pub GiftAccount: AccountId = PalletId(*b"par/gift").into_account();
}

pub struct GiftConvert;
impl Convert<Balance, Balance> for GiftConvert {
    fn convert(_amount: Balance) -> Balance {
        return Zero::zero();
    }
}

pub type LocalAssetTransactor = MultiCurrencyAdapter<
    Assets,
    IsNativeConcrete<CurrencyId, CurrencyIdConvert>,
    AccountId,
    Balance,
    LocationToAccountId,
    CurrencyIdConvert,
    NativeCurrencyId,
    GiftAccount,
    GiftConvert,
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
    type Weigher = FixedWeightBounds<UnitWeightCost, Call, MaxInstructions>;
    type Trader = FixedRateOfFungible<DotPerSecond, ()>;
    type ResponseHandler = ();
    type SubscriptionService = PolkadotXcm;
    type AssetTrap = PolkadotXcm;
    type AssetClaims = PolkadotXcm;
}

impl cumulus_pallet_xcmp_queue::Config for Test {
    type Event = Event;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
    type ChannelInfo = ParachainSystem;
    type VersionWrapper = ();
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
    const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;

    type Origin = Origin;
    type Call = Call;
    type Event = Event;
    type SendXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
    type XcmRouter = XcmRouter;
    type ExecuteXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
    type XcmExecuteFilter = Everything;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type XcmTeleportFilter = Nothing;
    type XcmReserveTransferFilter = Everything;
    type Weigher = FixedWeightBounds<UnitWeightCost, Call, MaxInstructions>;
    type LocationInverter = LocationInverter<Ancestry>;
    type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
}

pub struct CurrencyIdConvert;
impl Convert<CurrencyId, Option<MultiLocation>> for CurrencyIdConvert {
    fn convert(id: CurrencyId) -> Option<MultiLocation> {
        match id {
            KSM => Some(MultiLocation::parent()),
            XKSM => Some(MultiLocation::new(
                1,
                X2(
                    Parachain(ParachainInfo::parachain_id().into()),
                    GeneralKey(b"xKSM".to_vec()),
                ),
            )),
            _ => None,
        }
    }
}

impl Convert<MultiLocation, Option<CurrencyId>> for CurrencyIdConvert {
    fn convert(location: MultiLocation) -> Option<CurrencyId> {
        match location {
            MultiLocation {
                parents: 1,
                interior: Here,
            } => Some(KSM),
            MultiLocation {
                parents: 1,
                interior: X2(Parachain(id), GeneralKey(key)),
            } if ParaId::from(id) == ParachainInfo::parachain_id() && key == b"xKSM".to_vec() => {
                Some(XKSM)
            }
            _ => None,
        }
    }
}

impl Convert<MultiAsset, Option<CurrencyId>> for CurrencyIdConvert {
    fn convert(a: MultiAsset) -> Option<CurrencyId> {
        if let MultiAsset {
            id: AssetId::Concrete(id),
            fun: _,
        } = a
        {
            Self::convert(id)
        } else {
            None
        }
    }
}

pub struct AccountIdToMultiLocation;
impl Convert<AccountId, MultiLocation> for AccountIdToMultiLocation {
    fn convert(account_id: AccountId) -> MultiLocation {
        X1(Junction::AccountId32 {
            network: NetworkId::Any,
            id: account_id.into(),
        })
        .into()
    }
}

parameter_types! {
    pub SelfLocation: MultiLocation = MultiLocation::new(1, X1(Parachain(ParachainInfo::parachain_id().into())));
    pub const BaseXcmWeight: Weight = 100_000_000;
    pub const MaxInstructions: u32 = 100;
    pub const MaxAssetsForTransfer: usize = 2;
}

impl orml_xtokens::Config for Test {
    type Event = Event;
    type Balance = Balance;
    type CurrencyId = CurrencyId;
    type CurrencyIdConvert = CurrencyIdConvert;
    type AccountIdToMultiLocation = AccountIdToMultiLocation;
    type SelfLocation = SelfLocation;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type Weigher = FixedWeightBounds<UnitWeightCost, Call, MaxInstructions>;
    type BaseXcmWeight = BaseXcmWeight;
    type LocationInverter = LocationInverter<Ancestry>;
    type MaxAssetsForTransfer = MaxAssetsForTransfer;
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
type BlockNumber = u32;

pub const KSM_DECIMAL: u128 = 10u128.pow(12);

parameter_types! {
    pub const BlockHashCount: BlockNumber = 250;
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
    type BlockNumber = BlockNumber;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = AccountIdLookup<AccountId, ()>;
    type Header = generic::Header<BlockNumber, BlakeTwo256>;
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

pub struct AliceOrigin;
impl SortedMembers<AccountId> for AliceOrigin {
    fn sorted_members() -> Vec<AccountId> {
        vec![ALICE]
    }
}

pub struct BobOrigin;
impl SortedMembers<AccountId> for BobOrigin {
    fn sorted_members() -> Vec<AccountId> {
        vec![BOB]
    }
}

pub type RelayOrigin = EnsureOneOf<EnsureRoot<AccountId>, EnsureSignedBy<AliceOrigin, AccountId>>;
pub type UpdateOrigin = EnsureOneOf<EnsureRoot<AccountId>, EnsureSignedBy<BobOrigin, AccountId>>;

impl pallet_utility::Config for Test {
    type Event = Event;
    type Call = Call;
    type PalletsOrigin = OriginCaller;
    type WeightInfo = pallet_utility::weights::SubstrateWeight<Test>;
}

parameter_types! {
    pub const XcmHelperPalletId: PalletId = PalletId(*b"par/fees");
    pub const NotifyTimeout: BlockNumber = 100;
    pub RefundLocation: AccountId = para_a_id().into_account();
}

impl pallet_xcm_helper::Config for Test {
    type Event = Event;
    type UpdateOrigin = UpdateOrigin;
    type Assets = Assets;
    type XcmSender = XcmRouter;
    type PalletId = XcmHelperPalletId;
    type RelayNetwork = RelayNetwork;
    type NotifyTimeout = NotifyTimeout;
    type AccountIdToMultiLocation = AccountIdToMultiLocation;
    type RefundLocation = RefundLocation;
    type BlockNumberProvider = frame_system::Pallet<Test>;
    type WeightInfo = ();
}

pub struct RelayChainBlockNumberProvider<T>(sp_std::marker::PhantomData<T>);

impl<T: cumulus_pallet_parachain_system::Config> BlockNumberProvider
    for RelayChainBlockNumberProvider<T>
{
    type BlockNumber = primitives::BlockNumber;

    fn current_block_number() -> Self::BlockNumber {
        cumulus_pallet_parachain_system::Pallet::<T>::validation_data()
            .map(|d| d.relay_parent_number)
            .unwrap_or_default()
    }
}

parameter_types! {
    pub const StakingPalletId: PalletId = PalletId(*b"par/lqsk");
    pub const DerivativeIndex: u16 = 0;
    pub const EraLength: BlockNumber = 0;
    pub SelfParaId: ParaId = para_a_id();
    pub const MinStake: Balance = 0;
    pub const MinUnstake: Balance = 0;
    pub const StakingCurrency: CurrencyId = KSM;
    pub const LiquidCurrency: CurrencyId = XKSM;
    pub const XcmFees: Balance = 0;
    pub const BondingDuration: UnbondIndex = 3;
}

impl crate::Config for Test {
    type Event = Event;
    type Origin = Origin;
    type Call = Call;
    type UpdateOrigin = UpdateOrigin;
    type PalletId = StakingPalletId;
    type SelfParaId = SelfParaId;
    type WeightInfo = ();
    type StakingCurrency = StakingCurrency;
    type LiquidCurrency = LiquidCurrency;
    type DerivativeIndex = DerivativeIndex;
    type XcmFees = XcmFees;
    type Assets = Assets;
    type RelayOrigin = RelayOrigin;
    type EraLength = EraLength;
    type MinStake = MinStake;
    type MinUnstake = MinUnstake;
    type XCM = XcmHelper;
    type BondingDuration = BondingDuration;
    type RelayChainBlockNumberProvider = RelayChainBlockNumberProvider<Test>;
    type Members = BobOrigin;
}

parameter_types! {
    pub const AssetDeposit: Balance = KSM_DECIMAL;
    pub const ApprovalDeposit: Balance = 0;
    pub const AssetAccountDeposit: Balance = 0;
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
    type AssetAccountDeposit = AssetAccountDeposit;
    type ApprovalDeposit = ApprovalDeposit;
    type StringLimit = AssetsStringLimit;
    type Freezer = ();
    type WeightInfo = ();
    type Extra = ();
}

construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Event<T>},
        Utility: pallet_utility::{Pallet, Call, Event},
        Assets: pallet_assets::{Pallet, Call, Storage, Event<T>},
        LiquidStaking: crate::{Pallet, Storage, Call, Event<T>},
        ParachainSystem: cumulus_pallet_parachain_system::{Pallet, Call, Config, Storage, Inherent, Event<T>},
        ParachainInfo: parachain_info::{Pallet, Storage, Config},
        XcmpQueue: cumulus_pallet_xcmp_queue::{Pallet, Call, Storage, Event<T>},
        DmpQueue: cumulus_pallet_dmp_queue::{Pallet, Call, Storage, Event<T>},
        CumulusXcm: cumulus_pallet_xcm::{Pallet, Event<T>, Origin},
        PolkadotXcm: pallet_xcm::{Pallet, Call, Event<T>, Origin},
        XcmHelper: pallet_xcm_helper::{Pallet, Storage, Call, Event<T>},
        XTokens: orml_xtokens::{Pallet, Storage, Call, Event<T>},
    }
);

pub const ALICE: AccountId32 = AccountId32::new([1u8; 32]);
pub const BOB: AccountId32 = AccountId32::new([2u8; 32]);

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
        Assets::force_create(Origin::root(), KSM, Id(ALICE), true, 1).unwrap();
        Assets::force_set_metadata(
            Origin::root(),
            KSM,
            b"Kusama".to_vec(),
            b"KSM".to_vec(),
            12,
            false,
        )
        .unwrap();
        Assets::force_create(Origin::root(), XKSM, Id(ALICE), true, 1).unwrap();
        Assets::force_set_metadata(
            Origin::root(),
            XKSM,
            b"Parallel Kusama".to_vec(),
            b"XKSM".to_vec(),
            12,
            false,
        )
        .unwrap();
        Assets::mint(Origin::signed(ALICE), KSM, Id(ALICE), ksm(100f64)).unwrap();
        Assets::mint(Origin::signed(ALICE), XKSM, Id(ALICE), ksm(100f64)).unwrap();
        Assets::mint(Origin::signed(ALICE), KSM, Id(BOB), ksm(20000f64)).unwrap();

        LiquidStaking::update_market_cap(Origin::signed(BOB), ksm(10000f64)).unwrap();
        Assets::mint(
            Origin::signed(ALICE),
            KSM,
            Id(XcmHelper::account_id()),
            ksm(30f64),
        )
        .unwrap();

        XcmHelper::update_xcm_fees(Origin::signed(BOB), ksm(10f64)).unwrap();
    });

    ext
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
        Runtime = kusama_runtime::Runtime,
        XcmConfig = kusama_runtime::xcm_config::XcmConfig,
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

pub type KusamaRuntime = kusama_runtime::Runtime;
pub type RelayBalances = pallet_balances::Pallet<KusamaRuntime>;
pub type RelayStaking = pallet_staking::Pallet<KusamaRuntime>;
pub type RelayStakingEvent = pallet_staking::Event<KusamaRuntime>;
pub type RelaySystem = frame_system::Pallet<KusamaRuntime>;
pub type RelayEvent = kusama_runtime::Event;
pub type ParaSystem = frame_system::Pallet<Test>;

pub fn para_a_id() -> ParaId {
    ParaId::from(1)
}

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
        System::set_block_number(1);
        Assets::force_create(Origin::root(), KSM, Id(ALICE), true, 1).unwrap();
        Assets::force_set_metadata(
            Origin::root(),
            KSM,
            b"Kusama".to_vec(),
            b"KSM".to_vec(),
            12,
            false,
        )
        .unwrap();
        Assets::force_create(Origin::root(), XKSM, Id(ALICE), true, 1).unwrap();
        Assets::force_set_metadata(
            Origin::root(),
            XKSM,
            b"Parallel Kusama".to_vec(),
            b"XKSM".to_vec(),
            12,
            false,
        )
        .unwrap();
        Assets::mint(Origin::signed(ALICE), KSM, Id(ALICE), ksm(10000f64)).unwrap();
        Assets::mint(
            Origin::signed(ALICE),
            KSM,
            Id(XcmHelper::account_id()),
            ksm(30f64),
        )
        .unwrap();

        LiquidStaking::update_market_cap(Origin::signed(BOB), ksm(10000f64)).unwrap();
        XcmHelper::update_xcm_fees(Origin::signed(BOB), ksm(10f64)).unwrap();
    });

    ext
}

pub fn relay_ext() -> sp_io::TestExternalities {
    use kusama_runtime::{Runtime, System};
    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Runtime>()
        .unwrap();

    pallet_balances::GenesisConfig::<Runtime> {
        balances: vec![
            (ALICE, ksm(100f64)),
            (para_a_id().into_account(), ksm(1_000_000f64)),
        ],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    polkadot_runtime_parachains::configuration::GenesisConfig::<Runtime> {
        config: HostConfiguration {
            max_code_size: 1024u32,
            ..Default::default()
        },
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}

pub fn ksm(n: f64) -> Balance {
    ((n * 1000000f64) as u128) * KSM_DECIMAL / 1000000u128
}
