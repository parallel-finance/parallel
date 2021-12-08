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

use primitives::ump::XcmFeesPaymentStrategy;
use scale_info::TypeInfo;
use sp_runtime::{traits::Zero, RuntimeDebug};

#[derive(PartialEq, Eq, Copy, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum VaultPhase {
    /// Vault is open for contributions but wont execute contribute call on relaychain
    Pending,
    /// Vault is open for contributions
    Contributing,
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
    /// Vault ID
    pub id: u32,
    /// Asset used to represent the shares of currency
    /// to be claimed back later on
    pub ctoken: AssetIdOf<T>,
    /// Which phase the vault is at
    pub phase: VaultPhase,
    /// Tracks how many coins were contributed on the relay chain
    pub contributed: BalanceOf<T>,
    /// Tracks how many coins were gathered but not contributed on the relay chain
    pub pending: BalanceOf<T>,
    /// How we contribute coins to the crowdloan
    pub contribution_strategy: ContributionStrategy,
    /// XCM Transaction payment strategy
    pub xcm_fees_payment_strategy: XcmFeesPaymentStrategy,
}

/// init default vault with ctoken and currency override
impl<T: Config> Vault<T> {
    pub fn new(
        id: u32,
        ctoken: AssetIdOf<T>,
        contribution_strategy: ContributionStrategy,
        xcm_fees_payment_strategy: XcmFeesPaymentStrategy,
    ) -> Self {
        Self {
            id,
            ctoken,
            phase: VaultPhase::Pending,
            contributed: Zero::zero(),
            pending: Zero::zero(),
            contribution_strategy,
            xcm_fees_payment_strategy,
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum ContributionStrategy {
    XCM,
}
