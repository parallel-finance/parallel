use codec::{Decode, Encode};
use primitives::Balance;
use sp_runtime::RuntimeDebug;

/// Category of staking settlement at the end of era.
#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug)]
pub enum StakingSettlementKind {
    Reward,
    Slash,
}

/// Unbonding operation
#[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub struct Operation<BlockNumber, Balance> {
    pub amount: Balance,
    pub block_number: BlockNumber,
    pub status: ResponseStatus,
}

/// Unbonding operation status
#[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub enum ResponseStatus {
    Pending,
    Succeeded,
}

/// The user's unstake state in one era
#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, Default, RuntimeDebug)]
pub struct UnstakeMisc {
    /// The total asset that want to withdraw unbond
    pub total_amount: Balance,
    /// The claimed asset
    pub claimed_amount: Balance,
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
