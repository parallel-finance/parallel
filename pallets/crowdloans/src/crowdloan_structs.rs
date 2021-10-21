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
use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::{traits::StaticLookup, DispatchError, DispatchResult, RuntimeDebug};
use sp_std::marker::PhantomData;
use sp_std::{boxed::Box, vec::Vec};

pub type ParaId = u32;

#[derive(Clone, Copy, PartialEq, Decode, Encode, RuntimeDebug, TypeInfo)]
pub enum VaultPhase {
    /// Vault is open for contributions
    CollectingContributions,
    /// The vault is closed
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
    pub currency: CurrencyId,
    /// Which phase the vault is at
    pub phase: VaultPhase,
    /// How we contribute coins to the crowdloan
    pub contribution_strategy: ContributionStrategy<ParaId, CurrencyId, Balance>,
    /// Tracks how many coins were contributed on the relay chain
    pub contributed: Balance,
}

#[allow(clippy::upper_case_acronyms)] // for XCM
#[derive(Clone, Copy, PartialEq, Decode, Encode, RuntimeDebug, TypeInfo)]
pub enum ContributionStrategy<ParaId, CurrencyId, Balance> {
    XCM,
    XCMWithProxy,
    _Phantom(PhantomData<(ParaId, CurrencyId, Balance)>),
}

pub trait ContributionStrategyExecutor<ParaId, CurrencyId, Balance> {
    /// A test function
    // fn hello_world(self);
    fn hello_world(self, para_id: ParaId);

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
    fn hello_world(self, para_id: ParaId) {
        println!("Hello World! Your ParaId = {}", para_id);
    }

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
        todo!()
    }
    fn refund(self, _: ParaId, _: CurrencyId) -> Result<(), DispatchError> {
        todo!()
    }
}

/// Relaychain participate call arguments
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct CrowdloanParticipateCall<T: Config> {
    /// Unbond amount
    #[codec(compact)]
    pub value: BalanceOf<T>,
}

#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum CrowdloanCall<T: Config> {
    #[codec(index = 0)]
    Participate(CrowdloanParticipateCall<T>),
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
    Crowdloan(CrowdloanCall<T>),
    #[codec(index = 16)]
    Utility(Box<UtilityCall<Self>>),
}

#[derive(Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum KusamaCall<T: Config> {
    #[codec(index = 4)]
    Balances(BalancesCall<T>),
    #[codec(index = 6)]
    Crowdloan(CrowdloanCall<T>),
    #[codec(index = 24)]
    Utility(Box<UtilityCall<Self>>),
}

#[derive(Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum PolkadotCall<T: Config> {
    #[codec(index = 5)]
    Balances(BalancesCall<T>),
    #[codec(index = 7)]
    Crowdloan(CrowdloanCall<T>),
    #[codec(index = 26)]
    Utility(Box<UtilityCall<Self>>),
}
