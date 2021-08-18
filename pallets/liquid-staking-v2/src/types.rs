use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug)]
pub enum StakingSettlementKind {
    Reward,
    Slash,
}

#[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub enum StakingRelayOperation<Balance> {
    Bond(Balance),
    Unbond(Balance),
    NoOp,
}

#[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub struct Operation<BlockNumber, Balance> {
    pub amount: Balance,
    pub block_number: BlockNumber,
    pub status: ResponseStatus,
}

#[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub enum ResponseStatus {
    Waiting,
    Processing,
    Succeeded,
    Failed,
}

/// The matching pool's total stake & unstake amount in one era
#[derive(Copy, Clone, Eq, PartialEq, Default, Encode, Decode, RuntimeDebug)]
pub struct PoolLedger<Balance> {
    /// The matching pool's total unstake amount
    /// **NOTE** will be calculated by: exchangeRate * xToken amount
    pub total_unstake_amount: Balance,
}

/// The single user's stake & unstake amount in one era
#[derive(Copy, Clone, Eq, PartialEq, Default, Encode, Decode, RuntimeDebug)]
pub struct UserLedger<Balance> {
    /// The token amount that user unstaked
    /// **NOTE** will be calculated by: exchangeRate * xToken amount
    pub total_unstake_amount: Balance,
}
