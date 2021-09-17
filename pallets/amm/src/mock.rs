use crate as pallet_amm;

use codec::{Decode, Encode, MaxEncodedLen};
use frame_benchmarking::frame_support::traits::tokens::{DepositConsequence, WithdrawConsequence};
use frame_support::traits::fungible::{
    Inspect as FungibleInspect, Mutate as FungibleMutate, Transfer as FungibleTransfer,
};
use frame_support::traits::fungibles::{Inspect, Mutate, Transfer};
use frame_support::{parameter_types, PalletId};
use frame_system as system;
use frame_system::EnsureRoot;
use primitives::currency::CurrencyOrAsset;
use primitives::{Balance, TokenSymbol};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::H256;
use sp_runtime::DispatchError;
use sp_runtime::DispatchResult;
pub use sp_runtime::Perbill;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    RuntimeDebug,
};
use std::marker::PhantomData;
pub const DOT: CurrencyOrAsset = CurrencyOrAsset::NativeCurrency(TokenSymbol::DOT);
pub const XDOT: CurrencyOrAsset = CurrencyOrAsset::NativeCurrency(TokenSymbol::xDOT);
pub const HKO: CurrencyOrAsset = CurrencyOrAsset::NativeCurrency(TokenSymbol::HKO);
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
        AMM: pallet_amm::<Instance1>::{Pallet, Call, Storage, Event<T>},
        PermissionedAMM: pallet_amm::<Instance2>::{Pallet, Call, Storage, Event<T>},
        DefaultAMM: pallet_amm::{Pallet, Call, Storage, Event<T>},
        Assets: pallet_assets::{Pallet, Call, Storage, Event<T>},
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

parameter_types! {
    pub const AssetDeposit: u64 = 1;
    pub const ApprovalDeposit: u64 = 1;
    pub const StringLimit: u32 = 50;
    pub const MetadataDepositBase: u64 = 1;
    pub const MetadataDepositPerByte: u64 = 1;
}

impl pallet_assets::Config for Test {
    type Event = Event;
    type Balance = u128;
    type AssetId = u32;
    type Currency = Balances;
    type ForceOrigin = EnsureRoot<AccountId>;
    type AssetDeposit = AssetDeposit;
    type MetadataDepositBase = MetadataDepositBase;
    type MetadataDepositPerByte = MetadataDepositPerByte;
    type ApprovalDeposit = ApprovalDeposit;
    type StringLimit = StringLimit;
    type Freezer = ();
    type Extra = ();
    type WeightInfo = ();
}

parameter_types! {
    pub const AMMPalletId: PalletId = PalletId(*b"par/ammp");
    pub const AllowPermissionlessPoolCreation: bool = true;
    pub const DefaultLpFee: Perbill = Perbill::from_perthousand(3);         // 0.3%
    pub const DefaultProtocolFee: Perbill = Perbill::from_perthousand(2);   // 0.2%
    pub const DefaultProtocolFeeReceiver: AccountId = AccountId(4_u64);
}

pub struct Adapter<AccountId> {
    phantom: PhantomData<AccountId>,
}

impl Inspect<AccountId> for Adapter<AccountId> {
    type AssetId = CurrencyOrAsset;
    type Balance = Balance;

    fn total_issuance(asset: Self::AssetId) -> Self::Balance {
        match asset {
            CurrencyOrAsset::NativeCurrency(_token) => Balances::total_issuance(),
            CurrencyOrAsset::Asset(asset_id) => Assets::total_issuance(asset_id),
        }
    }

    fn balance(asset: Self::AssetId, who: &AccountId) -> Self::Balance {
        match asset {
            CurrencyOrAsset::NativeCurrency(_token) => Balances::balance(who),
            CurrencyOrAsset::Asset(asset_id) => Assets::balance(asset_id, who),
        }
    }

    fn minimum_balance(asset: Self::AssetId) -> Self::Balance {
        match asset {
            CurrencyOrAsset::NativeCurrency(_token) => Balances::minimum_balance(),
            CurrencyOrAsset::Asset(asset_id) => Assets::minimum_balance(asset_id),
        }
    }

    fn reducible_balance(asset: Self::AssetId, who: &AccountId, keep_alive: bool) -> Self::Balance {
        match asset {
            CurrencyOrAsset::NativeCurrency(_token) => Balances::reducible_balance(who, keep_alive),
            CurrencyOrAsset::Asset(asset_id) => {
                Assets::reducible_balance(asset_id, who, keep_alive)
            }
        }
    }

    fn can_deposit(
        asset: Self::AssetId,
        who: &AccountId,
        amount: Self::Balance,
    ) -> DepositConsequence {
        match asset {
            CurrencyOrAsset::NativeCurrency(_token) => Balances::can_deposit(who, amount),
            CurrencyOrAsset::Asset(asset_id) => Assets::can_deposit(asset_id, who, amount),
        }
    }

    fn can_withdraw(
        asset: Self::AssetId,
        who: &AccountId,
        amount: Self::Balance,
    ) -> WithdrawConsequence<Self::Balance> {
        match asset {
            CurrencyOrAsset::NativeCurrency(_token) => Balances::can_withdraw(who, amount),
            CurrencyOrAsset::Asset(asset_id) => Assets::can_withdraw(asset_id, who, amount),
        }
    }
}

impl Mutate<AccountId> for Adapter<AccountId> {
    fn mint_into(asset: Self::AssetId, who: &AccountId, amount: Self::Balance) -> DispatchResult {
        match asset {
            CurrencyOrAsset::NativeCurrency(_token) => Balances::mint_into(who, amount),
            CurrencyOrAsset::Asset(asset_id) => Assets::mint_into(asset_id, who, amount),
        }
    }

    fn burn_from(
        asset: Self::AssetId,
        who: &AccountId,
        amount: Balance,
    ) -> Result<Balance, DispatchError> {
        match asset {
            CurrencyOrAsset::NativeCurrency(_token) => Balances::burn_from(who, amount),
            CurrencyOrAsset::Asset(asset_id) => Assets::burn_from(asset_id, who, amount),
        }
    }
}

impl Transfer<AccountId> for Adapter<AccountId>
where
    Assets: Transfer<AccountId>,
{
    fn transfer(
        asset: Self::AssetId,
        source: &AccountId,
        dest: &AccountId,
        amount: Self::Balance,
        keep_alive: bool,
    ) -> Result<Balance, DispatchError> {
        match asset {
            CurrencyOrAsset::NativeCurrency(_token) => {
                <Balances as FungibleTransfer<AccountId>>::transfer(
                    source, dest, amount, keep_alive,
                )
            }
            CurrencyOrAsset::Asset(asset_id) => <Assets as Transfer<AccountId>>::transfer(
                asset_id, source, dest, amount, keep_alive,
            ),
        }
    }
}

impl pallet_amm::Config<pallet_amm::Instance1> for Test {
    type Event = Event;
    type AMMCurrency = Adapter<AccountId>;
    type PalletId = AMMPalletId;
    type AMMWeightInfo = ();
    type AllowPermissionlessPoolCreation = AllowPermissionlessPoolCreation;
    type LpFee = DefaultLpFee;
    type ProtocolFee = DefaultProtocolFee;
    type ProtocolFeeReceiver = DefaultProtocolFeeReceiver;
}

parameter_types! {
    pub const PermissionedAMMPalletId: PalletId = PalletId(*b"par/ampe");
    pub const ForbidPermissionlessPoolCreation: bool = false;
}

impl pallet_amm::Config<pallet_amm::Instance2> for Test {
    type Event = Event;
    type AMMCurrency = Adapter<AccountId>;
    type PalletId = PermissionedAMMPalletId;
    type AMMWeightInfo = ();
    type AllowPermissionlessPoolCreation = ForbidPermissionlessPoolCreation;
    type LpFee = DefaultLpFee;
    type ProtocolFee = DefaultProtocolFee;
    type ProtocolFeeReceiver = DefaultProtocolFeeReceiver;
}

impl pallet_amm::Config for Test {
    type Event = Event;
    type AMMCurrency = Adapter<AccountId>;
    type PalletId = AMMPalletId;
    type AMMWeightInfo = ();
    type AllowPermissionlessPoolCreation = AllowPermissionlessPoolCreation;
    type LpFee = DefaultLpFee;
    type ProtocolFee = DefaultProtocolFee;
    type ProtocolFeeReceiver = DefaultProtocolFeeReceiver;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (1.into(), 100_000_000),
            (2.into(), 100_000_000),
            (3.into(), 100_000_000_0),
            (4.into(), 100_000_000_0),
        ],
    }
    .assimilate_storage(&mut t)
    .unwrap();
    t.into()
}
