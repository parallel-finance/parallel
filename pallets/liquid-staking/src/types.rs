use codec::{Decode, Encode};

use frame_support::weights::Weight;
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{AtLeast32BitUnsigned, Zero},
    RuntimeDebug,
};
use sp_std::cmp::Ordering;
/// Category of staking settlement at the end of era.
#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, TypeInfo)]
pub enum StakingSettlementKind {
    Reward,
    Slash,
}

/// The matching pool's total stake & unstake amount in one era
#[derive(Copy, Clone, Eq, PartialEq, Default, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct MatchingLedger<Balance> {
    /// The total stake amount in one era
    pub total_stake_amount: Balance,
    /// The total unstake amount in one era
    /// **NOTE** will be calculated by: exchangeRate * xToken amount
    pub total_unstake_amount: Balance,
}

impl<Balance: AtLeast32BitUnsigned + Copy + Clone> MatchingLedger<Balance> {
    /// Matching requests in current period.
    ///
    /// `unbonding_amount` is the total amount of the unbonding asset in relaychain.
    ///
    /// the returned tri-tuple is formed as `(bond_amount, rebond_amount, unbond_amount)`.
    pub fn matching(&self, unbonding_amount: Balance) -> (Balance, Balance, Balance) {
        use Ordering::*;

        if matches!(
            self.total_stake_amount.cmp(&self.total_unstake_amount),
            Less | Equal
        ) {
            return (
                Zero::zero(),
                Zero::zero(),
                self.total_unstake_amount - self.total_stake_amount,
            );
        }

        let amount = self.total_stake_amount - self.total_unstake_amount;
        if amount < unbonding_amount {
            (Zero::zero(), amount, Zero::zero())
        } else {
            (amount - unbonding_amount, unbonding_amount, Zero::zero())
        }
    }

    #[inline]
    pub fn is_matched(&self) -> bool {
        self.total_stake_amount == self.total_unstake_amount
    }
}

/// The xcm weight when execute staking call wrapped in xcm message
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
        }
    }
}
