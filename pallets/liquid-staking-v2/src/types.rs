use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;
use primitives::Balance;

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

