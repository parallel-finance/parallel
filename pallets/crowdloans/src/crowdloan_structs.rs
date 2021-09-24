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

use primitives::{currency::CurrencyId, AssetId, Balance, BlockNumber};

#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, PartialEq, codec::Decode, codec::Encode, sp_runtime::RuntimeDebug)]
pub enum VaultPhase {
    /// Vault is open for contributions until the included block number
    CollectingContributionsUntil(BlockNumber),
    /// The vault's assets have been contributed to its associated crowdloan
    Participated,
    /// The vault's crowdloan failed, we have to distribute its assets back
    /// to the contributors
    Failed,
    /// The vault's crowdloan succeeded, project tokens will be identified
    /// by the provided asset id
    Succeeded(AssetId),
    /// The vault's crowdloan succeeded and returned the vault's assets
    SucceededAndRefunded(AssetId),
}

#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, PartialEq, codec::Decode, codec::Encode, sp_runtime::RuntimeDebug)]
pub struct Vault {
    /// Asset used to represent the shares of project tokens for the contributors
    /// to this vault
    pub project_shares: AssetId,
    /// Asset used to represent the shares of currency (typically DOT or KSM)
    /// to be claimed back later on
    pub currency_shares: AssetId,
    /// Indicates in which currency contributions are received, in most
    /// cases this will be the asset representing the relay chain's native
    /// token
    pub currency: CurrencyId,
    /// Which phase the vault is at
    pub phase: VaultPhase,
    /// Amount of shares used to claim project tokens but that weren't used
    /// to claim the DOT / KSM refund yet
    pub claimed: Balance,
}
