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
use frame_support::traits::{Contains, GenesisBuild, Hooks};
use frame_support::{construct_runtime, parameter_types, PalletId};
pub use primitives::{Amount, Balance, CurrencyId, TokenSymbol, AMM};
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{AccountIdConversion, IdentityLookup},
    Perbill,
};

pub type AccountId = u128;
pub type BlockNumber = u64;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const CHARILE: AccountId = 3;
pub const DAVE: AccountId = 4;

pub const DOT: CurrencyId = CurrencyId::Token(TokenSymbol::DOT);
pub const XDOT: CurrencyId = CurrencyId::Token(TokenSymbol::xDOT);

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

// orml-tokens configuration
parameter_types! {
    pub const RoutePalletId: PalletId = PalletId(*b"ammroute");
}

orml_traits::parameter_type_with_key! {
    pub ExistentialDeposits: |_currency_id: CurrencyId| -> Balance {
        Default::default()
    };
}

pub struct DustRemovalWhitelist;
impl Contains<AccountId> for DustRemovalWhitelist {
    fn contains(a: &AccountId) -> bool {
        vec![RoutePalletId::get().into_account()].contains(a)
    }
}

impl orml_tokens::Config for Runtime {
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

// orml-currencies configuration
parameter_types! {
    pub const GetNativeCurrencyId: CurrencyId = CurrencyId::Token(TokenSymbol::HKO);
}

impl orml_currencies::Config for Runtime {
    type Event = Event;
    type MultiCurrency = Tokens;
    type NativeCurrency =
        orml_currencies::BasicCurrencyAdapter<Runtime, Balances, Amount, BlockNumber>;
    type GetNativeCurrencyId = GetNativeCurrencyId;
    type WeightInfo = ();
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

// AMM instance initialization
parameter_types! {
    pub const AMMPalletId: PalletId = PalletId(*b"par/ammp");
    pub const AllowPermissionlessPoolCreation: bool = true;
    pub const DefaultLpFee: Perbill = Perbill::from_perthousand(3);         // 0.3%
    pub const DefaultProtocolFee: Perbill = Perbill::from_perthousand(2);   // 0.2%
    pub const DefaultProtocolFeeReceiver: AccountId = CHARILE;
}

impl pallet_amm::Config for Runtime {
    type Event = Event;
    type Currency = Currencies;
    type PalletId = AMMPalletId;
    type WeightInfo = ();
    type AllowPermissionlessPoolCreation = AllowPermissionlessPoolCreation;
    type LpFee = DefaultLpFee;
    type ProtocolFee = DefaultProtocolFee;
    type ProtocolFeeReceiver = DefaultProtocolFeeReceiver;
}

parameter_types! {
    pub const MaxLengthRoute: u8 = 10;
    pub Routes: Route = vec![
        (0, DOT, XDOT),
        (1, XDOT, DOT),
        (2, XDOT, CurrencyId::Token(TokenSymbol::KSM)),
        (3, CurrencyId::Token(TokenSymbol::KSM), XDOT),
        (4, CurrencyId::Token(TokenSymbol::xDOT), XDOT),
        (5, XDOT, CurrencyId::Token(TokenSymbol::xDOT)),
    ];
}

impl Config for Runtime {
    type Event = Event;
    type RoutePalletId = RoutePalletId;
    type AMMAdaptor = DOT2XDOT;
    type Routes = Routes;
    type MaxLengthRoute = MaxLengthRoute;
    type Currency = Currencies;
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
        Balances: pallet_balances::{Pallet, Call, Storage, Event<T>},
        Tokens: orml_tokens::{Pallet, Storage, Config<T>, Event<T>},
        Currencies: orml_currencies::{Pallet, Call, Event<T>},
        // AMM instances
        DOT2XDOT: pallet_amm::{Pallet, Call, Storage, Event<T>},
        // AMM Route
        AMMRoute: pallet_route::{Pallet, Call, Event<T>},
    }
);

pub struct ExtBuilder {
    pub balances: Vec<(AccountId, CurrencyId, Balance)>,
}

impl Default for ExtBuilder {
    fn default() -> Self {
        Self {
            balances: vec![
                (ALICE.into(), DOT, 10_000),
                (BOB.into(), DOT, 10_000),
                (DAVE.into(), DOT, 1_000_000_000),
                (DAVE.into(), XDOT, 1_000_000_000),
                (BOB.into(), XDOT, 5_000),
            ],
        }
    }
}

impl ExtBuilder {
    pub fn build(self) -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::default()
            .build_storage::<Runtime>()
            .unwrap();

        orml_tokens::GenesisConfig::<Runtime> {
            balances: self.balances.clone(),
        }
        .assimilate_storage(&mut t)
        .unwrap();
        t.into()
    }
}

pub(crate) fn run_to_block(n: u64) {
    while System::block_number() < n {
        AMMRoute::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        AMMRoute::on_initialize(System::block_number());
    }
}
