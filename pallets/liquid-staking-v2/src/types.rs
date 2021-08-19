use codec::{Decode, Encode};
use primitives::Balance;
use sp_runtime::{traits::Zero, RuntimeDebug};

/// Category of staking settlement at the end of era.
#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug)]
pub enum StakingSettlementKind {
    Reward,
    Slash,
}

/// The user's unstake state in one era
#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, Default, RuntimeDebug)]
pub struct UnstakeMisc<Balance> {
    /// The total asset that want to withdraw unbond
    pub total_amount: Balance,
    /// The claimed asset
    pub claimed_amount: Balance,
}

/// The matching pool's total stake & unstake amount in one era
#[derive(Copy, Clone, Eq, PartialEq, Default, Encode, Decode, RuntimeDebug)]
pub struct MatchingLedger {
    /// The total stake amount in one era
    pub total_stake_amount: Balance,
    /// The total unstake amount in one era
    /// **NOTE** will be calculated by: exchangeRate * xToken amount
    pub total_unstake_amount: Balance,
}

/// The operation to be done on relaychain before setting new era
#[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub enum StakingOperationType {
    Bond,
    Unbond,
    WithdrawUnbonded,
    NoOp,
}

/// The in-flight/succeeded/failed operation
#[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub struct Operation<BlockNumber, Balance> {
    pub status: OperationSatus,
    pub block_number: BlockNumber,
    pub amount: Balance,
}

/// The operation status on relaychain
#[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub enum OperationSatus {
    Pending,
    Succeeded,
    Failed,
}

impl MatchingLedger {
    /// Before setting new era, ledger needs to check if bond/unbond should be done
    /// on relaychain
    pub fn summary(&self) -> (StakingOperationType, OperationSatus, Balance) {
        use OperationSatus::*;
        use StakingOperationType::*;

        if self.total_stake_amount > self.total_unstake_amount {
            (
                Bond,
                Pending,
                self.total_stake_amount - self.total_unstake_amount,
            )
        } else if self.total_unstake_amount > self.total_stake_amount {
            (
                Unbond,
                Pending,
                self.total_unstake_amount - self.total_stake_amount,
            )
        } else {
            (NoOp, Succeeded, Zero::zero())
        }
    }
}
