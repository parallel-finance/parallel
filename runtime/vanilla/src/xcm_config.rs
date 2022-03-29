// Copyright 2021 Parallel Finance Developer.
// This file is part of Parallel Finance.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
// http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::{
    constants::{fee::ksm_per_second, paras},
    AccountId, Assets, Balance, BlockNumber, Call, CurrencyAdapter, CurrencyId, DmpQueue,
    EnsureRootOrMoreThanHalfGeneralCouncil, Event, ExistentialDeposit, GiftAccount, GiftConvert,
    NativeCurrencyId, Origin, ParachainInfo, ParachainSystem, PolkadotXcm, RefundLocation, Runtime,
    TreasuryAccount, XcmpQueue, MAXIMUM_BLOCK_WEIGHT,
};

pub use cumulus_primitives_core::ParaId;
use frame_support::match_type;
use frame_support::traits::fungibles::Mutate;
use frame_support::PalletId;
pub use frame_support::{
    parameter_types,
    traits::{Everything, Get, Nothing},
    weights::Weight,
};
use orml_traits::parameter_type_with_key;
use orml_xcm_support::{IsNativeConcrete, MultiNativeAsset};
use pallet_xcm::XcmPassthrough;
use polkadot_parachain::primitives::Sibling;
use primitives::{currency::MultiCurrencyAdapter, tokens::*, xcm_gadget::AccountIdToMultiLocation};
use sp_runtime::traits::Convert;
use xcm::latest::prelude::*;
use xcm_builder::{
    AccountId32Aliases, AllowKnownQueryResponses, AllowSubscriptionsFrom,
    AllowTopLevelPaidExecutionFrom, EnsureXcmOrigin, FixedRateOfFungible, FixedWeightBounds,
    LocationInverter, ParentAsSuperuser, ParentIsPreset, RelayChainAsNative,
    SiblingParachainAsNative, SiblingParachainConvertsVia, SignedAccountId32AsNative,
    SignedToAccountId32, SovereignSignedViaLocation, TakeRevenue, TakeWeightCredit,
};
use xcm_executor::{Config, XcmExecutor};

impl orml_xcm::Config for Runtime {
    type Event = Event;
    type SovereignOrigin = EnsureRootOrMoreThanHalfGeneralCouncil;
}

pub struct CurrencyIdConvert;
impl Convert<CurrencyId, Option<MultiLocation>> for CurrencyIdConvert {
    fn convert(id: CurrencyId) -> Option<MultiLocation> {
        match id {
            KSM => Some(MultiLocation::parent()),
            SKSM => Some(MultiLocation::new(
                1,
                X2(
                    Parachain(ParachainInfo::parachain_id().into()),
                    GeneralKey(b"sKSM".to_vec()),
                ),
            )),
            HKO => Some(MultiLocation::new(
                1,
                X2(
                    Parachain(ParachainInfo::parachain_id().into()),
                    GeneralKey(b"HKO".to_vec()),
                ),
            )),
            KAR => Some(MultiLocation::new(
                1,
                X2(
                    Parachain(paras::karura::ID),
                    GeneralKey(paras::karura::KAR_KEY.to_vec()),
                ),
            )),
            KUSD => Some(MultiLocation::new(
                1,
                X2(
                    Parachain(paras::karura::ID),
                    GeneralKey(paras::karura::KUSD_KEY.to_vec()),
                ),
            )),
            LKSM => Some(MultiLocation::new(
                1,
                X2(
                    Parachain(paras::karura::ID),
                    GeneralKey(paras::karura::LKSM_KEY.to_vec()),
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
            } if ParaId::from(id) == ParachainInfo::parachain_id() && key == b"sKSM".to_vec() => {
                Some(SKSM)
            }
            MultiLocation {
                parents: 0,
                interior: X1(GeneralKey(key)),
            } if key == b"sKSM".to_vec() => Some(SKSM),
            MultiLocation {
                parents: 1,
                interior: X2(Parachain(id), GeneralKey(key)),
            } if ParaId::from(id) == ParachainInfo::parachain_id() && key == b"HKO".to_vec() => {
                Some(HKO)
            }
            MultiLocation {
                parents: 0,
                interior: X1(GeneralKey(key)),
            } if key == b"HKO".to_vec() => Some(HKO),
            MultiLocation {
                parents: 1,
                interior: X2(Parachain(id), GeneralKey(key)),
            } if id == paras::karura::ID && key == paras::karura::KUSD_KEY.to_vec() => Some(KUSD),
            MultiLocation {
                parents: 1,
                interior: X2(Parachain(id), GeneralKey(key)),
            } if id == paras::karura::ID && key == paras::karura::KAR_KEY.to_vec() => Some(KAR),
            MultiLocation {
                parents: 1,
                interior: X2(Parachain(id), GeneralKey(key)),
            } if id == paras::karura::ID && key == paras::karura::LKSM_KEY.to_vec() => Some(LKSM),
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

parameter_types! {
    pub SelfLocation: MultiLocation = MultiLocation::new(1, X1(Parachain(ParachainInfo::parachain_id().into())));
    pub const BaseXcmWeight: Weight = 150_000_000;
    pub const MaxInstructions: u32 = 100;
    pub const MaxAssetsForTransfer: usize = 2;
}

parameter_type_with_key! {
    pub ParachainMinFee: |_location: MultiLocation| -> u128 {
        u128::MAX
    };
}

impl orml_xtokens::Config for Runtime {
    type Event = Event;
    type Balance = Balance;
    type CurrencyId = CurrencyId;
    type CurrencyIdConvert = CurrencyIdConvert;
    type AccountIdToMultiLocation = AccountIdToMultiLocation<AccountId>;
    type SelfLocation = SelfLocation;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type Weigher = FixedWeightBounds<BaseXcmWeight, Call, MaxInstructions>;
    type BaseXcmWeight = BaseXcmWeight;
    type LocationInverter = LocationInverter<Ancestry>;
    type MaxAssetsForTransfer = MaxAssetsForTransfer;
    type MinXcmFee = ParachainMinFee;
}

/// Local origins on this chain are allowed to dispatch XCM sends/executions. However, we later
/// block this via `ExecuteXcmOrigin`.
pub type LocalOriginToLocation = SignedToAccountId32<Origin, AccountId, RelayNetwork>;

/// The means for routing XCM messages which are not for local execution into the right message
/// queues.
pub type XcmRouter = (
    // Two routers - use UMP to communicate with the relay chain:
    cumulus_primitives_utility::ParentAsUmp<ParachainSystem, PolkadotXcm>,
    // ..and XCMP to communicate with the sibling chains.
    XcmpQueue,
);

impl pallet_xcm::Config for Runtime {
    const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;

    type Origin = Origin;
    type Call = Call;
    type Event = Event;
    type SendXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
    type XcmRouter = XcmRouter;
    type ExecuteXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
    type XcmExecuteFilter = Nothing;
    type XcmReserveTransferFilter = Everything;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    // Teleporting is disabled.
    type XcmTeleportFilter = Nothing;
    type Weigher = FixedWeightBounds<BaseXcmWeight, Call, MaxInstructions>;
    type LocationInverter = LocationInverter<Ancestry>;
    type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
}

impl cumulus_pallet_xcm::Config for Runtime {
    type Event = Event;
    type XcmExecutor = XcmExecutor<XcmConfig>;
}

impl cumulus_pallet_xcmp_queue::Config for Runtime {
    type Event = Event;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type ExecuteOverweightOrigin = EnsureRootOrMoreThanHalfGeneralCouncil;
    type ChannelInfo = ParachainSystem;
    type VersionWrapper = PolkadotXcm;
    type ControllerOrigin = EnsureRootOrMoreThanHalfGeneralCouncil;
    type ControllerOriginConverter = XcmOriginToTransactDispatchOrigin;
}

impl cumulus_pallet_dmp_queue::Config for Runtime {
    type Event = Event;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type ExecuteOverweightOrigin = EnsureRootOrMoreThanHalfGeneralCouncil;
}

parameter_types! {
    pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 4;
    pub const ReservedDmpWeight: Weight =  MAXIMUM_BLOCK_WEIGHT / 4;
}

impl cumulus_pallet_parachain_system::Config for Runtime {
    type Event = Event;
    type OnSystemEvent = ();
    type SelfParaId = ParachainInfo;
    type DmpMessageHandler = DmpQueue;
    type OutboundXcmpMessageSource = XcmpQueue;
    type XcmpMessageHandler = XcmpQueue;
    type ReservedXcmpWeight = ReservedXcmpWeight;
    type ReservedDmpWeight = ReservedDmpWeight;
}

impl parachain_info::Config for Runtime {}

parameter_types! {
    pub const RelayLocation: MultiLocation = MultiLocation::parent();
    pub RelayNetwork: NetworkId = NetworkId::Kusama;
    pub VanillaNetwork: NetworkId = NetworkId::Named("vanilla".into());
    pub RelayChainOrigin: Origin = cumulus_pallet_xcm::Origin::Relay.into();
    pub Ancestry: MultiLocation = MultiLocation::new(0, X1(Parachain(ParachainInfo::parachain_id().into())));
}

/// Type for specifying how a `MultiLocation` can be converted into an `AccountId`. This is used
/// when determining ownership of accounts for asset transacting and when attempting to use XCM
/// `Transact` in order to determine the dispatch Origin.
pub type LocationToAccountId = (
    // The parent (Relay-chain) origin converts to the default `AccountId`.
    ParentIsPreset<AccountId>,
    // Sibling parachain origins convert to AccountId via the `ParaId::into`.
    SiblingParachainConvertsVia<Sibling, AccountId>,
    // Straight up local `AccountId32` origins just alias directly to `AccountId`.
    AccountId32Aliases<RelayNetwork, AccountId>,
);

/// Means for transacting assets on this chain.
pub type LocalAssetTransactor = MultiCurrencyAdapter<
    // Use this currency:
    CurrencyAdapter,
    // Use this currency when it is a fungible asset matching the given location or name:
    IsNativeConcrete<CurrencyId, CurrencyIdConvert>,
    // Our chain's account ID type (we can't get away without mentioning it explicitly):
    AccountId,
    Balance,
    // Do a simple punn to convert an AccountId32 MultiLocation into a native chain account ID:
    LocationToAccountId,
    CurrencyIdConvert,
    NativeCurrencyId,
    ExistentialDeposit,
    GiftAccount,
    GiftConvert,
>;

/// This is the type we use to convert an (incoming) XCM origin into a local `Origin` instance,
/// ready for dispatching a transaction with Xcm's `Transact`. There is an `OriginKind` which can
/// biases the kind of local `Origin` it will become.
pub type XcmOriginToTransactDispatchOrigin = (
    // Sovereign account converter; this attempts to derive an `AccountId` from the origin location
    // using `LocationToAccountId` and then turn that into the usual `Signed` origin. Useful for
    // foreign chains who want to have a local sovereign account on this chain which they control.
    SovereignSignedViaLocation<LocationToAccountId, Origin>,
    // Native converter for Relay-chain (Parent) location; will converts to a `Relay` origin when
    // recognised.
    RelayChainAsNative<RelayChainOrigin, Origin>,
    // Native converter for sibling Parachains; will convert to a `SiblingPara` origin when
    // recognised.
    SiblingParachainAsNative<cumulus_pallet_xcm::Origin, Origin>,
    // Superuser converter for the Relay-chain (Parent) location. This will allow it to issue a
    // transaction from the Root origin.
    ParentAsSuperuser<Origin>,
    // Native signed account converter; this just converts an `AccountId32` origin into a normal
    // `Origin::Signed` origin of the same 32-byte value.
    SignedAccountId32AsNative<RelayNetwork, Origin>,
    // Xcm origins can be represented natively under the Xcm pallet's Xcm origin.
    XcmPassthrough<Origin>,
);

parameter_types! {
    pub KsmPerSecond: (AssetId, u128) = (AssetId::Concrete(MultiLocation::parent()), ksm_per_second());
    pub SKSMPerSecond: (AssetId, u128) = (
        MultiLocation::new(
            1,
            X2(Parachain(ParachainInfo::parachain_id().into()), GeneralKey(b"sKSM".to_vec())),
        ).into(),
        ksm_per_second()
    );
    pub SKSMPerSecondOfCanonicalLocation: (AssetId, u128) = (
        MultiLocation::new(
            0,
            X1(GeneralKey(b"sKSM".to_vec())),
        ).into(),
        ksm_per_second()
    );
    pub HkoPerSecond: (AssetId, u128) = (
        MultiLocation::new(
            1,
            X2(Parachain(ParachainInfo::parachain_id().into()), GeneralKey(b"HKO".to_vec())),
        ).into(),
        ksm_per_second() * 30
    );
    pub HkoPerSecondOfCanonicalLocation: (AssetId, u128) = (
        MultiLocation::new(
            0,
            X1(GeneralKey(b"HKO".to_vec())),
        ).into(),
        ksm_per_second() * 30
    );
    pub KusdPerSecond: (AssetId, u128) = (
        MultiLocation::new(
            1,
            X2(Parachain(paras::karura::ID), GeneralKey(paras::karura::KUSD_KEY.to_vec())),
        ).into(),
        ksm_per_second() * 400
    );
    pub KarPerSecond: (AssetId, u128) = (
        MultiLocation::new(
            1,
            X2(Parachain(paras::karura::ID), GeneralKey(paras::karura::KAR_KEY.to_vec())),
        ).into(),
        ksm_per_second() * 50
    );
    pub LKSMPerSecond: (AssetId, u128) = (
        MultiLocation::new(
            1,
            X2(Parachain(paras::karura::ID), GeneralKey(paras::karura::LKSM_KEY.to_vec())),
        ).into(),
        ksm_per_second()
    );
}

match_type! {
    pub type ParentOrSiblings: impl Contains<MultiLocation> = {
        MultiLocation { parents: 1, interior: Here } |
        MultiLocation { parents: 1, interior: X1(_) }
    };
}

pub type Barrier = (
    TakeWeightCredit,
    AllowKnownQueryResponses<PolkadotXcm>,
    AllowSubscriptionsFrom<ParentOrSiblings>,
    AllowTopLevelPaidExecutionFrom<Everything>,
);

pub struct ToTreasury;
impl TakeRevenue for ToTreasury {
    fn take_revenue(revenue: MultiAsset) {
        if let MultiAsset {
            id: AssetId::Concrete(id),
            fun: Fungibility::Fungible(amount),
        } = revenue
        {
            if let Some(currency_id) = CurrencyIdConvert::convert(id) {
                let _ = Assets::mint_into(currency_id, &TreasuryAccount::get(), amount);
            }
        }
    }
}

pub type Trader = (
    FixedRateOfFungible<KsmPerSecond, ToTreasury>,
    FixedRateOfFungible<SKSMPerSecond, ToTreasury>,
    FixedRateOfFungible<SKSMPerSecondOfCanonicalLocation, ToTreasury>,
    FixedRateOfFungible<HkoPerSecond, ToTreasury>,
    FixedRateOfFungible<HkoPerSecondOfCanonicalLocation, ToTreasury>,
    FixedRateOfFungible<KusdPerSecond, ToTreasury>,
    FixedRateOfFungible<KarPerSecond, ToTreasury>,
    FixedRateOfFungible<LKSMPerSecond, ToTreasury>,
);

pub struct XcmConfig;
impl Config for XcmConfig {
    type Call = Call;
    type XcmSender = XcmRouter;
    // How to withdraw and deposit an asset.
    type AssetTransactor = LocalAssetTransactor;
    type OriginConverter = XcmOriginToTransactDispatchOrigin;
    type IsReserve = MultiNativeAsset;
    // Teleporting is disabled.
    type IsTeleporter = ();
    type LocationInverter = LocationInverter<Ancestry>;
    type Barrier = Barrier;
    type Weigher = FixedWeightBounds<BaseXcmWeight, Call, MaxInstructions>;
    type Trader = Trader;
    type ResponseHandler = PolkadotXcm;
    type SubscriptionService = PolkadotXcm;
    type AssetTrap = PolkadotXcm;
    type AssetClaims = PolkadotXcm;
}

parameter_types! {
    pub const XcmHelperPalletId: PalletId = PalletId(*b"par/fees");
    pub const NotifyTimeout: BlockNumber = 100;
}

impl pallet_xcm_helper::Config for Runtime {
    type Event = Event;
    type UpdateOrigin = EnsureRootOrMoreThanHalfGeneralCouncil;
    type Assets = Assets;
    type XcmSender = XcmRouter;
    type RelayNetwork = RelayNetwork;
    type PalletId = XcmHelperPalletId;
    type NotifyTimeout = NotifyTimeout;
    type AccountIdToMultiLocation = AccountIdToMultiLocation<AccountId>;
    type RefundLocation = RefundLocation;
    type BlockNumberProvider = frame_system::Pallet<Runtime>;
    type WeightInfo = pallet_xcm_helper::weights::SubstrateWeight<Runtime>;
}
