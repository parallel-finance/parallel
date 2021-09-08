use super::{BalanceOf, Config};
use codec::{Decode, Encode};
use sp_runtime::{
    traits::{AtLeast32BitUnsigned, StaticLookup},
    RuntimeDebug,
};
use sp_std::cmp::Ordering;

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
                    (0u32.into(), amount, 0u32.into())
                } else {
                    (amount - unbonding_amount, unbonding_amount, 0u32.into())
                }
            }
            Less => {
                let amount = self.total_unstake_amount - self.total_stake_amount;
                (0u32.into(), 0u32.into(), amount)
            }
            Equal => (0u32.into(), 0u32.into(), 0u32.into()),
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
    /// [pallet index, call index]
    pub call_index: [u8; 2],
    /// Controller account
    pub controller: <T::Lookup as StaticLookup>::Source,
    /// Bonded amount
    #[codec(compact)]
    pub value: BalanceOf<T>,
    /// A destination account for payment.
    pub payee: RewardDestination<T::AccountId>,
}

/// Relaychain staking.bond_extra call arguments
#[derive(Clone, Encode, Decode, RuntimeDebug)]
pub struct StakingBondExtraCall<T: Config> {
    /// [pallet index, call index]
    pub call_index: [u8; 2],
    /// Bonded amount
    #[codec(compact)]
    pub value: BalanceOf<T>,
}
