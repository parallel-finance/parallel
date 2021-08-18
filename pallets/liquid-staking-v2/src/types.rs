use codec::{Decode, Encode, FullCodec};
use sp_runtime::{
    traits::{AtLeast32BitUnsigned, MaybeSerializeDeserialize},
    FixedPointNumber, FixedPointOperand, RuntimeDebug,
};
use sp_std::fmt::Debug;

use primitives::Rate;

#[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub enum StakingOperationType {
    Bond,
    Unbond,
    Rebond,
    Matching,
    TransferToRelaychain,
    RecordRewards,
    RecordSlashes,
}

#[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub enum ResponseStatus {
    Pending,
    Processing,
    Succeeded,
    Failed,
}

#[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub struct Operation<BlockNumber, Balance> {
    pub amount: Balance,
    pub block_number: BlockNumber,
    pub status: ResponseStatus,
}

#[derive(Copy, Clone, Eq, PartialEq, Default, Encode, Decode, RuntimeDebug)]
pub struct PoolLedger<Balance> {
    pub total_unstake_amount: Balance,
    pub total_stake_amount: Balance,
    pub operation_type: Option<StakingOperationType>,
}

impl<Balance> PoolLedger<Balance>
where
    Balance: AtLeast32BitUnsigned + FullCodec + Copy + MaybeSerializeDeserialize + Debug + Default,
{
    pub fn todo_after_new_era(&self) -> (StakingOperationType, Balance) {
        if self.total_stake_amount > self.total_unstake_amount {
            (
                StakingOperationType::Bond,
                self.total_stake_amount - self.total_unstake_amount,
            )
        } else if self.total_stake_amount < self.total_unstake_amount {
            (
                StakingOperationType::Unbond,
                self.total_unstake_amount - self.total_stake_amount,
            )
        } else {
            (StakingOperationType::Matching, 0u32.into())
        }
    }
}

/// The single user's stake/unsatke amount in each era
#[derive(Copy, Clone, Eq, PartialEq, Default, Encode, Decode, RuntimeDebug)]
pub struct UserLedger<Balance> {
    /// The token amount that user unstake during this era, will be calculated
    /// by exchangerate and xToken amount
    pub total_unstake_amount: Balance,
    /// The token amount that user stake during this era, this amount is equal
    /// to what the user input.
    pub total_stake_amount: Balance,
    /// The token amount that user have alreay claimed before the lock period,
    /// this will happen because, in matching pool total_unstake_amount and
    /// total_stake_amount can match each other
    pub claimed_unstake_amount: Balance,
    /// The token amount that user have alreay claimed before the lock period,
    pub claimed_stake_amount: Balance,
    /// To confirm that before lock period, user can only claim once because of
    /// the matching.
    pub claimed_matching: bool,
}

// (claim_unstake_amount_each_era,claim_stake_amount_each_era)
pub type WithdrawalAmount<Balance> = (Balance, Balance);

impl<Balance> UserLedger<Balance>
where
    Balance: AtLeast32BitUnsigned
        + FullCodec
        + Copy
        + MaybeSerializeDeserialize
        + Debug
        + Default
        + FixedPointOperand,
{
    pub fn remaining_withdrawal_limit(&self) -> WithdrawalAmount<Balance> {
        (
            self.total_unstake_amount
                .saturating_sub(self.claimed_unstake_amount),
            self.total_stake_amount
                .saturating_sub(self.claimed_stake_amount),
        )
    }

    // after matching mechanism，for bond operation, user who unstake can get all amount directly
    // and user who stake only get the matching part
    pub fn instant_withdrawal_by_bond(
        &self,
        pool: &PoolLedger<Balance>,
    ) -> WithdrawalAmount<Balance> {
        (
            self.total_unstake_amount,
            Rate::saturating_from_rational(self.total_stake_amount, pool.total_stake_amount)
                .saturating_mul_int(pool.total_unstake_amount),
        )
    }

    // after matching mechanism，for unbond operation, user who stake can get all amount directly
    // and user who unstake only get the matching part
    pub fn instant_withdrawal_by_unbond(
        &self,
        pool: &PoolLedger<Balance>,
    ) -> WithdrawalAmount<Balance> {
        (
            Rate::saturating_from_rational(self.total_unstake_amount, pool.total_unstake_amount)
                .saturating_mul_int(pool.total_stake_amount),
            self.total_stake_amount,
        )
    }
}
