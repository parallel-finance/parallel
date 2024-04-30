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

// Groups common pool related structures

use super::{AccountIdOf, AssetIdOf, BalanceOf, Config};

use codec::{Decode, Encode};

use frame_system::pallet_prelude::BlockNumberFor;
use primitives::{LeasePeriod, ParaId, Timestamp, TrieIndex, VaultId};
use scale_info::TypeInfo;
use sp_runtime::{traits::Zero, RuntimeDebug};
use sp_std::vec::Vec;

#[derive(PartialEq, Eq, Copy, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum VaultPhase {
    /// Vault is open for contributions but wont execute contribute call on relaychain
    Pending = 0,
    /// Vault is open for contributions
    Contributing = 1,
    /// The vault is closed and we should avoid future contributions. This happens when
    /// - there are no contribution
    /// - user cancelled
    /// - crowdloan reached its cap
    /// - parachain won the slot
    Closed = 2,
    /// The vault's crowdloan failed, we have to distribute its assets back
    /// to the contributors
    Failed = 3,
    /// Phase between Closed and Expired so we know this parachain won the auction
    Succeeded = 4,
    /// The vault's crowdloan and its associated parachain slot expired, it is
    /// now possible to get back the money we put in
    Expired = 5,
}

#[derive(PartialEq, Eq, Copy, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct Vault<T: Config> {
    /// Asset used to represent the shares of currency
    /// to be claimed back later on
    pub ctoken: AssetIdOf<T>,
    /// Which phase the vault is at
    pub phase: VaultPhase,
    /// Tracks how many coins were contributed on the relay chain
    pub contributed: BalanceOf<T>,
    /// Tracks how many coins were gathered but not contributed on the relay chain
    pub pending: BalanceOf<T>,
    /// Tracks how many coins were contributing on relaychain but didn't receive confirmation yet
    pub flying: BalanceOf<T>,
    /// How we contribute coins to the crowdloan
    pub contribution_strategy: ContributionStrategy,
    /// parallel enforced limit
    pub cap: BalanceOf<T>,
    /// block that vault ends
    pub end_block: BlockNumberFor<T>,
    /// child storage trie index where we store all contributions
    pub trie_index: TrieIndex,
    /// lease start period index
    pub lease_start: LeasePeriod,
    /// lease end period index
    pub lease_end: LeasePeriod,
}

/// init default vault with ctoken and currency override
impl<T: Config> Vault<T> {
    pub fn new(
        lease_start: LeasePeriod,
        lease_end: LeasePeriod,
        ctoken: AssetIdOf<T>,
        contribution_strategy: ContributionStrategy,
        cap: BalanceOf<T>,
        end_block: BlockNumberFor<T>,
        trie_index: TrieIndex,
    ) -> Self {
        Self {
            lease_start,
            lease_end,
            ctoken,
            phase: VaultPhase::Pending,
            contributed: Zero::zero(),
            pending: Zero::zero(),
            flying: Zero::zero(),
            contribution_strategy,
            cap,
            end_block,
            trie_index,
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum ContributionStrategy {
    XCM = 0,
    XCMPROXY = 1,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub enum XcmRequest<T: Config> {
    Contribute {
        crowdloan: ParaId,
        vault_id: VaultId,
        who: AccountIdOf<T>,
        amount: BalanceOf<T>,
        referral_code: Vec<u8>,
    },
    Withdraw {
        crowdloan: ParaId,
        vault_id: VaultId,
        amount: BalanceOf<T>,
        target_phase: VaultPhase,
    },
}

#[derive(PartialEq, Eq, Copy, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum ChildStorageKind {
    Pending,
    Flying,
    Contributed,
}

#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum Releases {
    V0_0_0,
    V1_0_0,
    V2_0_0,
    V3_0_0,
}

impl Default for Releases {
    fn default() -> Self {
        Releases::V0_0_0
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct BonusConfig<Balance> {
    // The bonus per-value of the contribute
    pub bonus_per_token: Balance,
    // The start time of the stream
    pub start_time: Timestamp,
    // The end time of the stream
    pub end_time: Timestamp,
}

impl<Balance: Default> Default for BonusConfig<Balance> {
    fn default() -> Self {
        BonusConfig {
            bonus_per_token: Default::default(),
            start_time: Default::default(),
            end_time: Default::default(),
        }
    }
}

impl<Balance> BonusConfig<Balance> {
    pub fn check(&self) -> bool {
        self.end_time > self.start_time
    }
}
