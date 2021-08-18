use codec::{Decode, Encode};
use primitives::Balance;
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug)]
pub enum StakeingSettlementKind {
    Reward,
    Slash,
}

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, Default, RuntimeDebug)]
pub struct StakeMisc {
    pub liquid_amount: Balance,
    pub staking_amount: Balance,
}

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, Default, RuntimeDebug)]
pub struct UnstakeMisc {
    pub pending_amount: Balance,
    pub free_amount: Balance,
}
