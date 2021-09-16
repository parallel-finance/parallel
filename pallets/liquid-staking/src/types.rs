use super::{BalanceOf, Config};
use codec::{Decode, Encode};
use sp_runtime::{
    traits::{AtLeast32BitUnsigned, StaticLookup, Zero},
    RuntimeDebug,
};
use sp_std::{cmp::Ordering, vec::Vec};

/// Category of staking settlement at the end of era.
#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug)]
pub enum StakingSettlementKind {
    Reward,
    Slash,
}

/// The matching pool's total stake & unstake amount in one era
#[derive(Copy, Clone, Eq, PartialEq, Default, Encode, Decode, RuntimeDebug)]
pub struct MatchingLedger<Balance> {
    /// The total stake amount in one era
    pub total_stake_amount: Balance,
    /// The total unstake amount in one era
    /// **NOTE** will be calculated by: exchangeRate * xToken amount
    pub total_unstake_amount: Balance,
}

impl<Balance> MatchingLedger<Balance>
where
    Balance: AtLeast32BitUnsigned + Copy + Clone,
{
    /// Matching requests in current period.
    ///
    /// `unbonding_amount` is the total amount of the unbonding asset in relaychain.
    ///
    /// the returned tri-tuple is formed as `(bond_amount, rebond_amount, unbond_amount)`.
    pub fn matching(&self, unbonding_amount: Balance) -> (Balance, Balance, Balance) {
        use Ordering::*;

        match self.total_stake_amount.cmp(&self.total_unstake_amount) {
            Greater => {
                let amount = self.total_stake_amount - self.total_unstake_amount;
                if amount < unbonding_amount {
                    (Zero::zero(), amount, Zero::zero())
                } else {
                    (amount - unbonding_amount, unbonding_amount, Zero::zero())
                }
            }
            Less | Equal => (
                Zero::zero(),
                Zero::zero(),
                self.total_unstake_amount - self.total_stake_amount,
            ),
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.total_stake_amount.is_zero() && self.total_unstake_amount.is_zero()
    }
}

/// A destination account for payment.
#[derive(PartialEq, Eq, Copy, Clone, Encode, Decode, RuntimeDebug)]
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
#[derive(Clone, Encode, Decode, RuntimeDebug)]
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
#[derive(Clone, Encode, Decode, RuntimeDebug)]
pub struct StakingBondExtraCall<T: Config> {
    /// Rebond amount
    #[codec(compact)]
    pub value: BalanceOf<T>,
}

/// Relaychain staking.unbond call arguments
#[derive(Clone, Encode, Decode, RuntimeDebug)]
pub struct StakingUnbondCall<T: Config> {
    /// Unbond amount
    #[codec(compact)]
    pub value: BalanceOf<T>,
}

/// Relaychain staking.rebond call arguments
#[derive(Clone, Encode, Decode, RuntimeDebug)]
pub struct StakingRebondCall<T: Config> {
    /// Rebond amount
    #[codec(compact)]
    pub value: BalanceOf<T>,
}

/// Relaychain staking.withdraw_unbonded call arguments
#[derive(Clone, Encode, Decode, RuntimeDebug)]
pub struct StakingWithdrawUnbondedCall {
    /// Withdraw amount
    pub num_slashing_spans: u32,
}

/// Relaychain staking.nominate call arguments
#[derive(Clone, Encode, Decode, RuntimeDebug)]
pub struct StakingNominateCall<T: Config> {
    /// List of nominate `targets`
    pub targets: Vec<<T::Lookup as StaticLookup>::Source>,
}

/// Relaychain staking.payout_stakers call arguments
#[derive(Clone, Encode, Decode, RuntimeDebug)]
pub struct StakingPayoutStakersCall<T: Config> {
    /// Stash account of validator
    pub validator_stash: T::AccountId,
    /// EraIndex
    pub era: u32,
}

#[derive(Encode, Decode, RuntimeDebug)]
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
#[derive(Clone, Encode, Decode, RuntimeDebug)]
pub struct BalancesTransferKeepAliveCall<T: Config> {
    /// dest account
    pub dest: <T::Lookup as StaticLookup>::Source,
    /// transfer amount
    #[codec(compact)]
    pub value: BalanceOf<T>,
}

/// Relaychain balances.transfer_all call arguments
#[derive(Clone, Encode, Decode, RuntimeDebug)]
pub struct BalancesTransferAllCall<T: Config> {
    /// dest account
    pub dest: <T::Lookup as StaticLookup>::Source,
    pub keep_alive: bool,
}

#[derive(Encode, Decode, RuntimeDebug)]
pub enum BalancesCall<T: Config> {
    #[codec(index = 3)]
    TransferKeepAlive(BalancesTransferKeepAliveCall<T>),
    #[codec(index = 4)]
    TransferAll(BalancesTransferAllCall<T>),
}

/// Relaychain utility.as_derivative call arguments
#[derive(Encode, Decode, RuntimeDebug)]
pub struct UtilityAsDerivativeCall<RelaychainCall> {
    /// derivative index
    pub index: u16,
    /// call
    pub call: RelaychainCall,
}

/// Relaychain utility.batch_all call arguments
#[derive(Encode, Decode, RuntimeDebug)]
pub struct UtilityBatchAllCall<RelaychainCall> {
    /// calls
    pub calls: Vec<RelaychainCall>,
}

#[derive(Encode, Decode, RuntimeDebug)]
pub enum UtilityCall<RelaychainCall> {
    #[codec(index = 1)]
    AsDerivative(UtilityAsDerivativeCall<RelaychainCall>),
    #[codec(index = 2)]
    BatchAll(UtilityBatchAllCall<RelaychainCall>),
}

pub mod westend {
    use super::*;

    #[derive(Encode, Decode, RuntimeDebug)]
    pub enum RelaychainCall<T: Config> {
        #[codec(index = 4)]
        Balances(BalancesCall<T>),
        #[codec(index = 6)]
        Staking(StakingCall<T>),
        #[codec(index = 16)]
        Utility(Box<UtilityCall<Self>>),
    }
}

pub mod kusama {
    use super::*;

    #[derive(Encode, Decode, RuntimeDebug)]
    pub enum RelaychainCall<T: Config> {
        #[codec(index = 4)]
        Balances(BalancesCall<T>),
        #[codec(index = 6)]
        Staking(StakingCall<T>),
        #[codec(index = 24)]
        Utility(Box<UtilityCall<Self>>),
    }
}

pub mod polkadot {
    use super::*;

    #[derive(Encode, Decode, RuntimeDebug)]
    pub enum RelaychainCall<T: Config> {
        #[codec(index = 5)]
        Balances(BalancesCall<T>),
        #[codec(index = 7)]
        Staking(StakingCall<T>),
        #[codec(index = 26)]
        Utility(Box<UtilityCall<Self>>),
    }
}

#[cfg(feature = "westend")]
pub use westend::RelaychainCall;

#[cfg(feature = "kusama")]
pub use kusama::RelaychainCall;

#[cfg(feature = "polkadot")]
pub use polkadot::RelaychainCall;
