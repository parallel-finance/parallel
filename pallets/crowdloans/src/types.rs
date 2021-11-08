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

use super::{BalanceOf, Config};
use codec::{Decode, Encode, MaxEncodedLen};
use cumulus_primitives_core::ParaId;
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{StaticLookup, Zero},
    DispatchError, DispatchResult, MultiSignature, RuntimeDebug,
};
use sp_std::marker::PhantomData;

/// A destination account for payment.
#[derive(PartialEq, Eq, Copy, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum RewardDestination<AccountId> {
    /// Pay into the stash account, increasing the amount at stake accordingly.
    Staked,
    /// Pay into the stash account, not increasing the amount at stake.
    Stash,
    /// Pay into the controller account.
    Controller,
    /// Pay into a specified account.
    Account(AccountId),
    /// Receive no reward.
    None,
}

/// Relaychain staking.bond call arguments
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct StakingBondCall<T: Config> {
    /// Controller account
    pub controller: <T::Lookup as StaticLookup>::Source,
    /// Bond amount
    #[codec(compact)]
    pub value: BalanceOf<T>,
    /// A destination account for payment.
    pub payee: RewardDestination<T::AccountId>,
}

/// Relaychain staking.bond_extra call arguments
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct StakingBondExtraCall<T: Config> {
    /// Rebond amount
    #[codec(compact)]
    pub value: BalanceOf<T>,
}

/// Relaychain staking.unbond call arguments
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct StakingUnbondCall<T: Config> {
    /// Unbond amount
    #[codec(compact)]
    pub value: BalanceOf<T>,
}

/// Relaychain staking.rebond call arguments
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct StakingRebondCall<T: Config> {
    /// Rebond amount
    #[codec(compact)]
    pub value: BalanceOf<T>,
}

/// Relaychain staking.withdraw_unbonded call arguments
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct StakingWithdrawUnbondedCall {
    /// Withdraw amount
    pub num_slashing_spans: u32,
}

/// Relaychain staking.nominate call arguments
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct StakingNominateCall<T: Config> {
    /// List of nominate `targets`
    pub targets: Vec<<T::Lookup as StaticLookup>::Source>,
}

/// Relaychain staking.payout_stakers call arguments
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct StakingPayoutStakersCall<T: Config> {
    /// Stash account of validator
    pub validator_stash: T::AccountId,
    /// EraIndex
    pub era: u32,
}

#[derive(Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum StakingCall<T: Config> {
    #[codec(index = 0)]
    Bond(StakingBondCall<T>),
    #[codec(index = 1)]
    BondExtra(StakingBondExtraCall<T>),
    #[codec(index = 2)]
    Unbond(StakingUnbondCall<T>),
    #[codec(index = 3)]
    WithdrawUnbonded(StakingWithdrawUnbondedCall),
    #[codec(index = 5)]
    Nominate(StakingNominateCall<T>),
    #[codec(index = 18)]
    PayoutStakers(StakingPayoutStakersCall<T>),
    #[codec(index = 19)]
    Rebond(StakingRebondCall<T>),
}

/// Relaychain balances.transfer_keep_alive call arguments
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct BalancesTransferKeepAliveCall<T: Config> {
    /// dest account
    pub dest: <T::Lookup as StaticLookup>::Source,
    /// transfer amount
    #[codec(compact)]
    pub value: BalanceOf<T>,
}

/// Relaychain balances.transfer_all call arguments
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct BalancesTransferAllCall<T: Config> {
    /// dest account
    pub dest: <T::Lookup as StaticLookup>::Source,
    pub keep_alive: bool,
}

#[derive(Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum BalancesCall<T: Config> {
    #[codec(index = 3)]
    TransferKeepAlive(BalancesTransferKeepAliveCall<T>),
    #[codec(index = 4)]
    TransferAll(BalancesTransferAllCall<T>),
}

#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct CrowdloansContributeCall<T: Config> {
    /// - `crowdloan`: The crowdloan who you are contributing to
    #[codec(compact)]
    pub index: ParaId,
    /// - `value`: The amount of tokens you want to contribute to a parachain.
    #[codec(compact)]
    pub value: BalanceOf<T>,
    // `signature`: The signature if the fund has a verifier
    pub signature: Option<MultiSignature>,
}

#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct CrowdloansWithdrawCall<T: Config> {
    /// - `who`: The account whose contribution should be withdrawn.
    pub who: T::AccountId,
    /// - `index`: The parachain to whose crowdloan the contribution was made.
    #[codec(compact)]
    pub index: ParaId,
}

#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct CrowdloansRefundCall<T: Config> {
    /// - `index`: The parachain to whose crowdloan the contribution was made.
    #[codec(compact)]
    pub index: ParaId,
    pub _ghost: PhantomData<T>,
}

#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum CrowdloansCall<T: Config> {
    #[codec(index = 1)]
    Contribute(CrowdloansContributeCall<T>),
    #[codec(index = 2)]
    Withdraw(CrowdloansWithdrawCall<T>),
    #[codec(index = 3)]
    Refund(CrowdloansRefundCall<T>),
}

#[derive(Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct ProxyProxyCall<T: Config> {
    pub real: T::AccountId,
    pub force_proxy_type: Option<ProxyType>,
    pub call: Box<<T as frame_system::Config>::Call>,
}

// TODO: fix westend, polkadot's proxy type
/// The type used to represent the kinds of proxying allowed.
#[derive(
    Copy,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Encode,
    Decode,
    RuntimeDebug,
    MaxEncodedLen,
    TypeInfo,
)]
pub enum ProxyType {
    Any,
    NonTransfer,
    Governance,
    Staking,
    IdentityJudgement,
    CancelProxy,
    Auction,
}

impl Default for ProxyType {
    fn default() -> Self {
        Self::Any
    }
}

#[derive(Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum ProxyCall<T: Config> {
    #[codec(index = 0)]
    Proxy(ProxyProxyCall<T>),
}

/// Relaychain utility.as_derivative call arguments
#[derive(Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct UtilityAsDerivativeCall<RelaychainCall> {
    /// derivative index
    pub index: u16,
    /// call
    pub call: RelaychainCall,
}

/// Relaychain utility.batch_all call arguments
#[derive(Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct UtilityBatchAllCall<RelaychainCall> {
    /// calls
    pub calls: Vec<RelaychainCall>,
}

#[derive(Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum UtilityCall<RelaychainCall> {
    #[codec(index = 1)]
    AsDerivative(UtilityAsDerivativeCall<RelaychainCall>),
    #[codec(index = 2)]
    BatchAll(UtilityBatchAllCall<RelaychainCall>),
}

#[derive(Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum WestendCall<T: Config> {
    #[codec(index = 4)]
    Balances(BalancesCall<T>),
    #[codec(index = 6)]
    Staking(StakingCall<T>),
    #[codec(index = 16)]
    Utility(Box<UtilityCall<Self>>),
    #[codec(index = 64)]
    Crowdloans(CrowdloansCall<T>),
}

#[derive(Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum KusamaCall<T: Config> {
    #[codec(index = 4)]
    Balances(BalancesCall<T>),
    #[codec(index = 6)]
    Staking(StakingCall<T>),
    #[codec(index = 24)]
    Utility(Box<UtilityCall<Self>>),
    #[codec(index = 73)]
    Crowdloans(CrowdloansCall<T>),
}

#[derive(Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum PolkadotCall<T: Config> {
    #[codec(index = 5)]
    Balances(BalancesCall<T>),
    #[codec(index = 7)]
    Staking(StakingCall<T>),
    #[codec(index = 26)]
    Utility(Box<UtilityCall<Self>>),
    #[codec(index = 73)]
    Crowdloans(CrowdloansCall<T>),
}

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
        (ctoken, relay_currency, contribution_strategy): (
            CurrencyId,
            CurrencyId,
            ContributionStrategy<ParaId, CurrencyId, Balance>,
        ),
    ) -> Self {
        Self {
            ctoken,
            relay_currency,
            contribution_strategy,
            ..Self::default()
        }
    }
}

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

impl<ParaId, CurrencyId, Balance> ContributionStrategyExecutor<ParaId, CurrencyId, Balance>
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
