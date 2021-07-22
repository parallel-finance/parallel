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

//! Mocks for the nominee-election module.

use crate as pallet_nominee_election;

use super::*;
use frame_support::{
    construct_runtime, ord_parameter_types, parameter_types, traits::SortedMembers,
};
use frame_system::EnsureSignedBy;
use sp_core::H256;
use sp_runtime::{testing::Header, traits::IdentityLookup};

pub type AccountId = u128;
pub type BlockNumber = u64;

mod nominee_election {
    pub use super::super::*;
}

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
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type DbWeight = ();
    type BaseCallFilter = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
}

ord_parameter_types! {
    pub const One: AccountId = 1;
    pub const Two: AccountId = 2;
    pub const MaxValidators: u32 = 1;
    pub const ValidatorFeedersMembershipMaxMembers: u32 = 3;
}

pub struct Six;
impl SortedMembers<AccountId> for Six {
    fn sorted_members() -> Vec<AccountId> {
        vec![AccountId::from(6_u64)]
    }
}

impl Config for Runtime {
    type Event = Event;
    type UpdateOrigin = EnsureSignedBy<One, AccountId>;
    type WhitelistUpdateOrigin = EnsureSignedBy<Two, AccountId>;
    type MaxValidators = MaxValidators;
    type Members = Six;
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
        NomineeElection: pallet_nominee_election::{Pallet, Call, Storage, Event<T>, Config},
    }
);

pub struct ExtBuilder;

impl Default for ExtBuilder {
    fn default() -> Self {
        ExtBuilder
    }
}

pub const MOCK_OLD_COEFFICIENTS: NomineeCoefficients = NomineeCoefficients {
    crf: 100,
    nf: 1000,
    epf: 10,
};

pub const MOCK_NEW_COEFFICIENTS: NomineeCoefficients = NomineeCoefficients {
    crf: 10,
    nf: 100,
    epf: 1000,
};

pub const MOCK_VALIDATOR_THREE: ValidatorInfo<AccountId> = ValidatorInfo {
    name: None,
    address: 3,
    stakes: 9,
    score: 100,
};

pub const MOCK_VALIDATOR_FOUR: ValidatorInfo<AccountId> = ValidatorInfo {
    name: None,
    address: 4,
    stakes: 10,
    score: 99,
};

impl ExtBuilder {
    pub fn build(self) -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::default()
            .build_storage::<Runtime>()
            .unwrap();

        pallet_nominee_election::GenesisConfig {
            coefficients: MOCK_OLD_COEFFICIENTS,
        }
        .assimilate_storage::<Runtime>(&mut t)
        .unwrap();

        t.into()
    }
}
