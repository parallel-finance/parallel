use codec::{Decode, Encode};
use primitives::Balance;
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug)]
pub enum StakeingSettlementKind {
    Reward,
    Slash,
}

/// The user's unstake state in one era
#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, Default, RuntimeDebug)]
pub struct UnstakeMisc {
    /// The pending asset need to match or unbond_withdraw
    pub pending_amount: Balance,
    /// The free asset can be claimed
    pub free_amount: Balance,
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
