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

use crate::AccountIdOf;

use super::{AssetIdOf, BalanceOf, Config, Error, Pallet as Crowdloans};

use codec::{Decode, Encode};
use frame_support::{
    require_transactional,
    traits::{fungibles::Mutate, Get},
};
use scale_info::{prelude::*, TypeInfo};
use sp_runtime::{
    traits::{BlockNumberProvider, Zero},
    DispatchError, DispatchResult, RuntimeDebug,
};
use sp_std::boxed::Box;
use xcm::latest::prelude::*;

use primitives::{ump::*, ParaId};

#[derive(PartialEq, Eq, Copy, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum VaultPhase {
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
    /// Asset used to represent the shares of currency
    /// to be claimed back later on
    pub ctoken: AssetIdOf<T>,
    /// Which phase the vault is at
    pub phase: VaultPhase,
    /// How we contribute coins to the crowdloan
    pub contribution_strategy: ContributionStrategy,
    /// XCM Transaction payment strategy
    pub transaction_payment_strategy: TransactionPaymentStrategy,
    /// Tracks how many coins were contributed on the relay chain
    pub contributed: BalanceOf<T>,
}

/// init default vault with ctoken and currency override
impl<T: Config>
    From<(
        AssetIdOf<T>,
        ContributionStrategy,
        TransactionPaymentStrategy,
    )> for Vault<T>
{
    fn from(
        (ctoken, contribution_strategy, transaction_payment_strategy): (
            AssetIdOf<T>,
            ContributionStrategy,
            TransactionPaymentStrategy,
        ),
    ) -> Self {
        Self {
            ctoken,
            contribution_strategy,
            transaction_payment_strategy,
            phase: VaultPhase::Contributing,
            contributed: Zero::zero(),
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum ContributionStrategy {
    XCM,
}

#[derive(PartialEq, Eq, Copy, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum TransactionPaymentStrategy {
    Fees,
    Reserves,
}

pub trait ContributionStrategyExecutor {
    /// Execute the strategy to contribute `amount` of coins to the crowdloan
    /// of the given parachain id
    fn contribute<T: Config>(
        self,
        who: &AccountIdOf<T>,
        para_id: ParaId,
        amount: BalanceOf<T>,
    ) -> DispatchResult;

    /// Withdraw coins from the relay chain's crowdloans and send it back
    /// to our parachain
    fn withdraw<T: Config>(self, para_id: ParaId, amount: BalanceOf<T>) -> DispatchResult;
}

impl ContributionStrategyExecutor for ContributionStrategy {
    #[require_transactional]
    fn contribute<T: Config>(
        self,
        who: &AccountIdOf<T>,
        para_id: ParaId,
        amount: BalanceOf<T>,
    ) -> Result<(), DispatchError> {
        T::Assets::burn_from(
            T::RelayCurrency::get(),
            &Crowdloans::<T>::account_id(),
            amount,
        )?;

        switch_relay!({
            let call =
                RelaychainCall::Utility(Box::new(UtilityCall::BatchAll(UtilityBatchAllCall {
                    calls: vec![
                        RelaychainCall::<T>::System(SystemCall::Remark(SystemRemarkCall {
                            remark: format!(
                                "{:?}#{:?}",
                                T::BlockNumberProvider::current_block_number(),
                                who
                            )
                            .into_bytes(),
                        })),
                        RelaychainCall::<T>::Crowdloans(CrowdloansCall::Contribute(
                            CrowdloansContributeCall {
                                index: para_id,
                                value: amount,
                                signature: None,
                            },
                        )),
                    ],
                })));

            let msg = Crowdloans::<T>::ump_transact(
                call.encode().into(),
                Crowdloans::<T>::xcm_weight().contribute_weight,
            )?;

            if let Err(_e) = T::XcmSender::send_xcm(MultiLocation::parent(), msg) {
                return Err(Error::<T>::SendXcmError.into());
            }
        });

        Ok(())
    }

    #[require_transactional]
    fn withdraw<T: Config>(
        self,
        para_id: ParaId,
        amount: BalanceOf<T>,
    ) -> Result<(), DispatchError> {
        switch_relay!({
            let call =
                RelaychainCall::<T>::Crowdloans(CrowdloansCall::Withdraw(CrowdloansWithdrawCall {
                    who: Crowdloans::<T>::para_account_id(),
                    index: para_id,
                }));

            let msg = Crowdloans::<T>::ump_transact(
                call.encode().into(),
                Crowdloans::<T>::xcm_weight().withdraw_weight,
            )?;

            if let Err(_e) = T::XcmSender::send_xcm(MultiLocation::parent(), msg) {
                return Err(Error::<T>::SendXcmError.into());
            }
        });

        T::Assets::mint_into(
            T::RelayCurrency::get(),
            &Crowdloans::<T>::account_id(),
            amount,
        )?;

        Ok(())
    }
}
