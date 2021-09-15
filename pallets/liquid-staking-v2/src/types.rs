use codec::{Decode, Encode};
use sp_runtime::{
    traits::{AtLeast32BitUnsigned, Zero},
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
