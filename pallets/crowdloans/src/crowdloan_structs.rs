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

use frame_support::pallet_prelude::DispatchResult;
use primitives::{Balance, CurrencyId};

pub type ParaId = u32;

#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, PartialEq, codec::Decode, codec::Encode, sp_runtime::RuntimeDebug)]
pub enum ContributionStrategy<ParaId, CurrencyId, Balance> {
    Placeholder(ParaId, CurrencyId, Balance),
    // --- Examples
    XCM,
    XCMWithProxy,
}

#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, PartialEq, codec::Decode, codec::Encode, sp_runtime::RuntimeDebug)]
pub enum ClaimStrategy<ParaId> {
    Placeholder(ParaId),
    // --- Examples
    Teleport(ParaId),
    ReserveOnStatemine(ParaId),
}

#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, PartialEq, codec::Decode, codec::Encode, sp_runtime::RuntimeDebug)]
pub enum VaultPhase<CurrencyId, Balance> {
    /// Vault is open for contributions
    CollectingContributions,
    /// The vault is closed
    Closed,
    /// The vault's crowdloan failed, we have to distribute its assets back
    /// to the contributors
    Failed,
    /// The vault's crowdloan succeeded, project tokens will be identified
    /// by the provided asset id
    Succeeded(CurrencyId, Balance),
    /// The vault's crowdloan succeeded and returned the vault's assets
    SucceededAndRefunded(CurrencyId, Balance),
}

pub trait ContributionStrategyExecutor<ParaId, CurrencyId, Balance> {
    /// Execute the strategy to contribute `amount` of coins to the crowdloan
    /// of the given parachain id
    fn execute(self, para_id: ParaId, currency: CurrencyId, amount: Balance) -> DispatchResult;

    /// Withdraw coins from the relay chain's crowdloans and send it back
    /// to our parachain
    fn withdraw(self, para_id: ParaId, currency: CurrencyId) -> DispatchResult;

    /// Ask for a refund of the coins on the relay chain
    fn refund(self, para_id: ParaId, currency: CurrencyId) -> DispatchResult;
}

pub trait ClaimStrategyExecutor<ParaId> {
    /// Execute the strategy to claim some project tokens on the parachain
    /// with id `para_id`
    fn execute(self, para_id: ParaId) -> DispatchResult;
}

impl ContributionStrategyExecutor<ParaId, CurrencyId, Balance>
    for ContributionStrategy<ParaId, CurrencyId, Balance>
{
    // add code here
    fn execute(
        self,
        _: ParaId,
        _: CurrencyId,
        _: Balance,
    ) -> Result<(), sp_runtime::DispatchError> {
        todo!()
    }
    fn withdraw(self, _: ParaId, _: CurrencyId) -> Result<(), sp_runtime::DispatchError> {
        todo!()
    }
    fn refund(self, _: ParaId, _: CurrencyId) -> Result<(), sp_runtime::DispatchError> {
        todo!()
    }
}

impl ClaimStrategyExecutor<ParaId> for ClaimStrategy<ParaId> {
    // add code here
    fn execute(self, _: ParaId) -> Result<(), sp_runtime::DispatchError> {
        todo!()
    }
}

#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, PartialEq, codec::Decode, codec::Encode, sp_runtime::RuntimeDebug)]
pub struct Vault<ParaId, CurrencyId, Balance> {
    /// Asset used to represent the shares of project tokens for the contributors
    /// to this vault
    pub project_shares: CurrencyId,
    /// Asset used to represent the shares of currency (typically DOT or KSM)
    /// to be claimed back later on
    pub currency_shares: CurrencyId,
    /// Indicates in which currency contributions are received, in most
    /// cases this will be the asset representing the relay chain's native
    /// token
    pub currency: CurrencyId,
    /// Which phase the vault is at
    pub phase: VaultPhase<CurrencyId, Balance>,
    /// How we contribute coins to the crowdloan
    pub contribution_strategy: ContributionStrategy<ParaId, CurrencyId, Balance>,
    /// How we claim project tokens
    pub claim_strategy: ClaimStrategy<ParaId>,
    /// Tracks how many coins were contributed on the relay chain
    pub contributed: Balance,
}
