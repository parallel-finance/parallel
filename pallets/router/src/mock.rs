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

//! Mocks for the router module.

use super::*;
use crate as pallet_route;
use frame_support::{
    construct_runtime, parameter_types,
    traits::{
        fungible::{
            Inspect as FungibleInspect, Mutate as FungibleMutate, Transfer as FungibleTransfer,
        },
        fungibles::{Inspect, Mutate, Transfer},
    },
    PalletId,
};
use frame_system::EnsureRoot;
pub use primitives::{currency::CurrencyId, tokens, Amount, Balance, AMM};
use sp_core::H256;
use sp_runtime::{testing::Header, traits::IdentityLookup, DispatchError, DispatchResult, Perbill};

pub type AccountId = u128;
pub type BlockNumber = u64;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const CHARLIE: AccountId = 3;
pub const DAVE: AccountId = 4;

pub const DOT: CurrencyId = CurrencyId::Asset(tokens::DOT);
pub const XDOT: CurrencyId = CurrencyId::Asset(tokens::XDOT);
pub const USDT: CurrencyId = CurrencyId::Asset(tokens::USDT);
pub const KSM: CurrencyId = CurrencyId::Asset(tokens::KSM);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for Runtime {
    type Origin = Origin;
    type Index = u64;
    type BlockNumber = BlockNumber;
    type Call = Call;
    type Hash = H256;
    type Hashing = ::sp_runtime::traits::BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type BlockWeights = ();
    type BlockLength = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type DbWeight = ();
    type BaseCallFilter = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
}

// pallet-balances configuration
parameter_types! {
    pub const ExistentialDeposit: Balance = 1;
    pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for Runtime {
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

// pallet-assets configuration
parameter_types! {
    pub const AssetDeposit: u64 = 1;
    pub const ApprovalDeposit: u64 = 1;
    pub const StringLimit: u32 = 50;
    pub const MetadataDepositBase: u64 = 1;
    pub const MetadataDepositPerByte: u64 = 1;
}

impl pallet_assets::Config for Runtime {
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

// AMM instance initialization
parameter_types! {
    pub const AMMPalletId: PalletId = PalletId(*b"par/ammp");
    pub const AllowPermissionlessPoolCreation: bool = true;
    pub const DefaultLpFee: Perbill = Perbill::from_perthousand(3);         // 0.3%
    pub const DefaultProtocolFee: Perbill = Perbill::from_perthousand(2);   // 0.2%
    pub const DefaultProtocolFeeReceiver: AccountId = CHARLIE;
}

impl pallet_amm::Config for Runtime {
    type Event = Event;
    type AMMCurrency = assets_adapter::Adapter<AccountId>;
    type PalletId = AMMPalletId;
    type AMMWeightInfo = ();
    type AllowPermissionlessPoolCreation = AllowPermissionlessPoolCreation;
    type LpFee = DefaultLpFee;
    type ProtocolFee = DefaultProtocolFee;
    type ProtocolFeeReceiver = DefaultProtocolFeeReceiver;
}

parameter_types! {
    pub const MaxLengthRoute: u8 = 10;
    pub const RouterPalletId: PalletId = PalletId(*b"ammroute");
}

impl Config for Runtime {
    type Event = Event;
    type RouterPalletId = RouterPalletId;
    type AMM = DefaultAMM;
    type AMMRouterWeightInfo = ();
    type MaxLengthRoute = MaxLengthRoute;
    type AMMCurrency = assets_adapter::Adapter<AccountId>;
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Assets: pallet_assets::{Pallet, Call, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Event<T>},
        // AMM instances
        DefaultAMM: pallet_amm::{Pallet, Call, Storage, Event<T>},
        // AMM Route
        AMMRoute: pallet_route::{Pallet, Call, Event<T>},
    }
);

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Runtime>()
        .unwrap();
    pallet_balances::GenesisConfig::<Runtime> {
        balances: vec![(ALICE, 100_000_000), (BOB, 100_000_000)],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| {
        Assets::force_create(Origin::root(), tokens::DOT, ALICE, true, 1).unwrap();
        Assets::force_create(Origin::root(), tokens::XDOT, ALICE, true, 1).unwrap();
        Assets::force_create(Origin::root(), tokens::KSM, ALICE, true, 1).unwrap();
        Assets::force_create(Origin::root(), tokens::USDT, ALICE, true, 1).unwrap();

        Assets::mint(Origin::signed(ALICE), tokens::DOT, ALICE, 10_000).unwrap();
        Assets::mint(Origin::signed(ALICE), tokens::XDOT, ALICE, 10_000).unwrap();
        Assets::mint(Origin::signed(ALICE), tokens::KSM, ALICE, 10_000).unwrap();

        Assets::mint(Origin::signed(ALICE), tokens::DOT, DAVE, 1000_000_000).unwrap();
        Assets::mint(Origin::signed(ALICE), tokens::KSM, DAVE, 1000_000_000).unwrap();
        Assets::mint(Origin::signed(ALICE), tokens::XDOT, DAVE, 1000_000_000).unwrap();
        Assets::mint(Origin::signed(ALICE), tokens::USDT, DAVE, 1000_000_000).unwrap();
    });

    ext
}

pub(crate) fn run_to_block(n: u64) {
    System::set_block_number(n);
}

// Assets adapter helper
pub mod assets_adapter {
    use super::*;
    use core::marker::PhantomData;
    use frame_support::traits::tokens::{DepositConsequence, WithdrawConsequence};

    pub struct Adapter<AccountId> {
        phantom: PhantomData<AccountId>,
    }

    impl Inspect<AccountId> for Adapter<AccountId> {
        type AssetId = CurrencyId;
        type Balance = Balance;

        fn total_issuance(asset: Self::AssetId) -> Self::Balance {
            match asset {
                CurrencyId::Native => Balances::total_issuance(),
                CurrencyId::Asset(asset_id) => Assets::total_issuance(asset_id),
            }
        }

        fn balance(asset: Self::AssetId, who: &AccountId) -> Self::Balance {
            match asset {
                CurrencyId::Native => Balances::balance(who),
                CurrencyId::Asset(asset_id) => Assets::balance(asset_id, who),
            }
        }

        fn minimum_balance(asset: Self::AssetId) -> Self::Balance {
            match asset {
                CurrencyId::Native => Balances::minimum_balance(),
                CurrencyId::Asset(asset_id) => Assets::minimum_balance(asset_id),
            }
        }

        fn reducible_balance(
            asset: Self::AssetId,
            who: &AccountId,
            keep_alive: bool,
        ) -> Self::Balance {
            match asset {
                CurrencyId::Native => Balances::reducible_balance(who, keep_alive),
                CurrencyId::Asset(asset_id) => Assets::reducible_balance(asset_id, who, keep_alive),
            }
        }

        fn can_deposit(
            asset: Self::AssetId,
            who: &AccountId,
            amount: Self::Balance,
        ) -> DepositConsequence {
            match asset {
                CurrencyId::Native => Balances::can_deposit(who, amount),
                CurrencyId::Asset(asset_id) => Assets::can_deposit(asset_id, who, amount),
            }
        }

        fn can_withdraw(
            asset: Self::AssetId,
            who: &AccountId,
            amount: Self::Balance,
        ) -> WithdrawConsequence<Self::Balance> {
            match asset {
                CurrencyId::Native => Balances::can_withdraw(who, amount),
                CurrencyId::Asset(asset_id) => Assets::can_withdraw(asset_id, who, amount),
            }
        }
    }

    impl Mutate<AccountId> for Adapter<AccountId> {
        fn mint_into(
            asset: Self::AssetId,
            who: &AccountId,
            amount: Self::Balance,
        ) -> DispatchResult {
            match asset {
                CurrencyId::Native => Balances::mint_into(who, amount),
                CurrencyId::Asset(asset_id) => Assets::mint_into(asset_id, who, amount),
            }
        }

        fn burn_from(
            asset: Self::AssetId,
            who: &AccountId,
            amount: Balance,
        ) -> Result<Balance, DispatchError> {
            match asset {
                CurrencyId::Native => Balances::burn_from(who, amount),
                CurrencyId::Asset(asset_id) => Assets::burn_from(asset_id, who, amount),
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
                CurrencyId::Native => <Balances as FungibleTransfer<AccountId>>::transfer(
                    source, dest, amount, keep_alive,
                ),
                CurrencyId::Asset(asset_id) => <Assets as Transfer<AccountId>>::transfer(
                    asset_id, source, dest, amount, keep_alive,
                ),
            }
        }
    }
}
