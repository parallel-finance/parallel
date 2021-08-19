use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;

/// Category of staking settlement at the end of era.
#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug)]
pub enum StakingSettlementKind {
    Reward,
    Slash,
}

#[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub struct Operation<BlockNumber, Balance> {
    pub amount: Balance,
    pub block_number: BlockNumber,
    pub status: ResponseStatus,
}

#[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub enum ResponseStatus {
    Pending,
    Succeeded,
}
