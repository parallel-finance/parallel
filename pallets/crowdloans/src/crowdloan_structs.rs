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

use crate::calls::*;
use crate::{BalanceOf, Config, Error};
use codec::{Decode, Encode};
use cumulus_primitives_core::ParaId;
use frame_support::traits::Get;
use scale_info::TypeInfo;
use sp_runtime::traits::AccountIdConversion;
use sp_runtime::{
    traits::{Convert, Zero},
    DispatchError, DispatchResult, RuntimeDebug, SaturatedConversion,
};
use sp_std::marker::PhantomData;
use xcm::{prelude::*};

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
#[scale_info(skip_type_params(T))]
pub struct Vault<T: Config, CurrencyId> {
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
    pub contribution_strategy: ContributionStrategy<CurrencyId>,
    /// Tracks how many coins were contributed on the relay chain
    pub contributed: BalanceOf<T>,
}

/// a default initalization for a vault
impl<T: Config, CurrencyId: Zero> Default for Vault<T, CurrencyId> {
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
impl<T: Config, CurrencyId: Zero> From<(CurrencyId, CurrencyId, ContributionStrategy<CurrencyId>)>
    for Vault<T, CurrencyId>
{
    fn from(currency_override: (CurrencyId, CurrencyId, ContributionStrategy<CurrencyId>)) -> Self {
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
pub enum ContributionStrategy<CurrencyId> {
    XCM,
    XCMWithProxy,
    _Phantom(PhantomData<CurrencyId>),
}

pub trait ContributionStrategyExecutor<CurrencyId> {
    /// Execute the strategy to contribute `amount` of coins to the crowdloan
    /// of the given parachain id
    fn execute<T: Config>(
        self,
        para_id: ParaId,
        currency: CurrencyId,
        amount: BalanceOf<T>,
    ) -> DispatchResult;

    /// Withdraw coins from the relay chain's crowdloans and send it back
    /// to our parachain
    fn withdraw(self, para_id: ParaId, currency: CurrencyId) -> DispatchResult;

    /// Ask for a refund of the coins on the relay chain
    fn refund(self, para_id: ParaId, currency: CurrencyId) -> DispatchResult;
}

impl<CurrencyId> ContributionStrategyExecutor<CurrencyId> for ContributionStrategy<CurrencyId> {
    // add code here
    fn execute<T: Config>(
        self,
        para_id: ParaId,
        _currency_id: CurrencyId,
        amount: BalanceOf<T>,
    ) -> Result<(), DispatchError> {
        let amount = amount.saturated_into::<u128>();
        let multi_asset = MultiAsset {
            id: AssetId::Concrete(MultiLocation::here()),
            fun: Fungibility::Fungible(2_000_000_000),
        };

        let _call_params = CrowdloanContributeCall::<T> {
            index: para_id,
            value: amount,
            signature: None,
        };

        // let contribute_call: DoubleEncoded<()> = KusamaCall::Crowdloan(CrowdloanCall::Contribute(call_params)).encode().into();
        let xcm = Xcm(vec![
            WithdrawAsset(MultiAssets::from(multi_asset.clone())),
            BuyExecution {
                fees: multi_asset.clone(),
                weight_limit: Unlimited,
            },
            Transact {
                origin_type: OriginKind::SovereignAccount,
                require_weight_at_most: u64::MAX,
                // call: contribute_call,
                call: vec![].into(),
            },
            RefundSurplus,
            DepositAsset {
                assets: multi_asset.into(),
                max_assets: 1,
                beneficiary: T::AccountIdToMultiLocation::convert(
                    T::SelfParaId::get().into_account(),
                ),
            },
        ]);

        // send xcm call
        let _response = T::XcmSender::send_xcm(MultiLocation::parent(), xcm)
            .map_err(|_| Error::<T>::SendXcmError)?;

        Ok(())
    }
    fn withdraw(self, _: ParaId, _: CurrencyId) -> Result<(), DispatchError> {
        Ok(())
    }
    fn refund(self, _: ParaId, _: CurrencyId) -> Result<(), DispatchError> {
        Ok(())
    }
}
