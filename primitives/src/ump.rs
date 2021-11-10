use super::ParaId;

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::Weight;
use frame_system::Config;
use scale_info::TypeInfo;
use sp_runtime::{traits::StaticLookup, MultiSignature, RuntimeDebug};
use sp_std::{boxed::Box, marker::PhantomData, vec::Vec};

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
    pub value: u128,
    /// A destination account for payment.
    pub payee: RewardDestination<T::AccountId>,
}

/// Relaychain staking.bond_extra call arguments
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct StakingBondExtraCall {
    /// Rebond amount
    #[codec(compact)]
    pub value: u128,
}

/// Relaychain staking.unbond call arguments
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct StakingUnbondCall {
    /// Unbond amount
    #[codec(compact)]
    pub value: u128,
}

/// Relaychain staking.rebond call arguments
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct StakingRebondCall {
    /// Rebond amount
    #[codec(compact)]
    pub value: u128,
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
    BondExtra(StakingBondExtraCall),
    #[codec(index = 2)]
    Unbond(StakingUnbondCall),
    #[codec(index = 3)]
    WithdrawUnbonded(StakingWithdrawUnbondedCall),
    #[codec(index = 5)]
    Nominate(StakingNominateCall<T>),
    #[codec(index = 18)]
    PayoutStakers(StakingPayoutStakersCall<T>),
    #[codec(index = 19)]
    Rebond(StakingRebondCall),
}

/// Relaychain balances.transfer_keep_alive call arguments
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct BalancesTransferKeepAliveCall<T: Config> {
    /// dest account
    pub dest: <T::Lookup as StaticLookup>::Source,
    /// transfer amount
    #[codec(compact)]
    pub value: u128,
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
pub struct CrowdloansContributeCall {
    /// - `crowdloan`: The crowdloan who you are contributing to
    #[codec(compact)]
    pub index: ParaId,
    /// - `value`: The amount of tokens you want to contribute to a parachain.
    #[codec(compact)]
    pub value: u128,
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
    Contribute(CrowdloansContributeCall),
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
    Any = 0,
    NonTransfer = 1,
    Governance = 2,
    Staking = 3,
    CancelProxy = 6,
    Auction = 7,
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

/// The xcm weight when execute ump call wrapped in xcm message
#[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct XcmWeightMisc<Weight> {
    /// The weight when execute bond xcm message
    pub bond_weight: Weight,
    /// The weight when execute bond_extra xcm message
    pub bond_extra_weight: Weight,
    /// The weight when execute unbond xcm message
    pub unbond_weight: Weight,
    /// The weight when execute rebond xcm message
    pub rebond_weight: Weight,
    /// The weight when execute withdraw_unbonded xcm message
    pub withdraw_unbonded_weight: Weight,
    /// The weight when execute nominate xcm message
    pub nominate_weight: Weight,
    /// The weight when execute contribute xcm message
    pub contribute_weight: Weight,
    /// The weight when execute withdraw xcm message
    pub withdraw_weight: Weight,
    /// The weight when execute refund xcm message
    pub refund_weight: Weight,
}

impl Default for XcmWeightMisc<Weight> {
    fn default() -> Self {
        let default_weight = 3_000_000_000;
        XcmWeightMisc {
            bond_weight: default_weight,
            bond_extra_weight: default_weight,
            unbond_weight: default_weight,
            rebond_weight: default_weight,
            withdraw_unbonded_weight: default_weight,
            nominate_weight: default_weight,
            contribute_weight: default_weight,
            withdraw_weight: default_weight,
            refund_weight: default_weight,
        }
    }
}

#[macro_export]
macro_rules! switch_relay {
    ({ $( $code:tt )* }) => {
        if <T as Config>::RelayNetwork::get() == NetworkId::Polkadot {
            use primitives::ump::PolkadotCall as RelaychainCall;

            $( $code )*
        } else if <T as Config>::RelayNetwork::get() == NetworkId::Kusama {
            use primitives::ump::KusamaCall as RelaychainCall;

            $( $code )*
        } else {
            unreachable!()
        }
    }
}
