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

use super::{AssetIdOf, BalanceOf, Config};

use codec::{Decode, Encode};
use cumulus_primitives_core::ParaId;
use scale_info::TypeInfo;
use sp_runtime::{traits::Zero, DispatchError, DispatchResult, RuntimeDebug};

#[derive(PartialEq, Eq, Copy, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
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

#[derive(PartialEq, Eq, Copy, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct Vault<T: Config> {
    /// Asset used to represent the shares of currency
    /// to be claimed back later on
    pub ctoken: AssetIdOf<T>,
    /// Which phase the vault is at
    pub phase: VaultPhase,
    /// How we contribute coins to the crowdloan
    pub contribution_strategy: ContributionStrategy,
    /// Tracks how many coins were contributed on the relay chain
    pub contributed: BalanceOf<T>,
}

/// init default vault with ctoken and currency override
impl<T: Config> From<(AssetIdOf<T>, ContributionStrategy)> for Vault<T> {
    fn from((ctoken, contribution_strategy): (AssetIdOf<T>, ContributionStrategy)) -> Self {
        Self {
            ctoken,
            contribution_strategy,
            phase: VaultPhase::CollectingContributions,
            contributed: Zero::zero(),
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum ContributionStrategy {
    XCM,
    XCMWithProxy,
}

pub trait ContributionStrategyExecutor {
    /// Execute the strategy to contribute `amount` of coins to the crowdloan
    /// of the given parachain id
    fn execute<T: Config>(
        self,
        para_id: ParaId,
        currency: AssetIdOf<T>,
        amount: BalanceOf<T>,
    ) -> DispatchResult;

    /// Withdraw coins from the relay chain's crowdloans and send it back
    /// to our parachain
    fn withdraw<T: Config>(self, para_id: ParaId, currency: AssetIdOf<T>) -> DispatchResult;

    /// Ask for a refund of the coins on the relay chain
    fn refund<T: Config>(self, para_id: ParaId, currency: AssetIdOf<T>) -> DispatchResult;
}

impl ContributionStrategyExecutor for ContributionStrategy {
    // add code here
    fn execute<T: Config>(
        self,
        _para_id: ParaId,
        _currency_id: AssetIdOf<T>,
        _amount: BalanceOf<T>,
    ) -> Result<(), DispatchError> {
        Ok(())
    }
    fn withdraw<T: Config>(self, _: ParaId, _: AssetIdOf<T>) -> Result<(), DispatchError> {
        Ok(())
    }
    fn refund<T: Config>(self, _: ParaId, _: AssetIdOf<T>) -> Result<(), DispatchError> {
        Ok(())
    }
}
