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

use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::{traits::Zero, DispatchError, DispatchResult, RuntimeDebug};
use sp_std::marker::PhantomData;

#[derive(Clone, Copy, PartialEq, Decode, Encode, RuntimeDebug, TypeInfo)]
pub enum VaultPhase {
    /// Vault is open for contributions
    CollectingContributions,
    /// The vault is closed and we should avoid future contributions. This happens when
    /// - there are no contribution
    /// - user cancelled
    /// - crowdloan reached its cap
    /// - parachain won the slot
    Closed,
    /// The vault's crowdloan failed, we have to distribute its assets back
    /// to the contributors
    Failed,
    /// The vault's crowdloan and its associated parachain slot expired, it is
    /// now possible to get back the money we put in
    Expired,
}

#[derive(Clone, Copy, PartialEq, Decode, Encode, RuntimeDebug, TypeInfo)]
// pub struct Vault<ParaId, CurrencyId, Balance> {
pub struct Vault<ParaId, CurrencyId, Balance> {
    /// Asset used to represent the shares of currency
    /// to be claimed back later on
    pub ctoken: CurrencyId,
    /// Indicates in which currency contributions are received, in most
    /// cases this will be the asset representing the relay chain's native
    /// token
    pub relay_currency: CurrencyId,
    /// Which phase the vault is at
    pub phase: VaultPhase,
    /// How we contribute coins to the crowdloan
    pub contribution_strategy: ContributionStrategy<ParaId, CurrencyId, Balance>,
    /// Tracks how many coins were contributed on the relay chain
    pub contributed: Balance,
}

/// a default initalization for a vault
impl<ParaId, CurrencyId: Zero, Balance: Zero> Default for Vault<ParaId, CurrencyId, Balance> {
    fn default() -> Self {
        Vault {
            ctoken: Zero::zero(),
            relay_currency: Zero::zero(),
            phase: VaultPhase::CollectingContributions,
            contribution_strategy: ContributionStrategy::XCM,
            contributed: Zero::zero(),
        }
    }
}

/// init default vault with ctoken and currency override
impl<ParaId, CurrencyId: Zero, Balance: Zero>
    From<(
        CurrencyId,
        CurrencyId,
        ContributionStrategy<ParaId, CurrencyId, Balance>,
    )> for Vault<ParaId, CurrencyId, Balance>
{
    fn from(
        currency_override: (
            CurrencyId,
            CurrencyId,
            ContributionStrategy<ParaId, CurrencyId, Balance>,
        ),
    ) -> Self {
        Self {
            ctoken: currency_override.0,
            relay_currency: currency_override.1,
            contribution_strategy: currency_override.2,
            ..Self::default()
        }
    }
}

#[allow(clippy::upper_case_acronyms)] // for XCM
#[derive(Clone, Copy, PartialEq, Decode, Encode, RuntimeDebug, TypeInfo)]
pub enum ContributionStrategy<ParaId, CurrencyId, Balance> {
    XCM,
    XCMWithProxy,
    _Phantom(PhantomData<(ParaId, CurrencyId, Balance)>),
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

impl<ParaId: std::fmt::Display, CurrencyId, Balance>
    ContributionStrategyExecutor<ParaId, CurrencyId, Balance>
    for ContributionStrategy<ParaId, CurrencyId, Balance>
{
    // add code here
    fn execute(
        self,
        _para_id: ParaId,
        _currency_id: CurrencyId,
        _amount: Balance,
    ) -> Result<(), DispatchError> {
        Ok(())
    }
    fn withdraw(self, _: ParaId, _: CurrencyId) -> Result<(), DispatchError> {
        Ok(())
    }
    fn refund(self, _: ParaId, _: CurrencyId) -> Result<(), DispatchError> {
        Ok(())
    }
}
