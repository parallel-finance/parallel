use codec::{Decode, Encode};
use frame_support::traits::tokens::Balance as TokenBalance;
use frame_support::RuntimeDebug;
use num_traits::cast::ToPrimitive;
use num_traits::{CheckedDiv, CheckedMul};
use pallet_traits::ConvertToBigUint;
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{Saturating, UniqueSaturatedInto},
    ArithmeticError, SaturatedConversion,
};

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct PoolInfo<BlockNumber, BalanceOf> {
    pub is_active: bool,
    /// total amount of staking asset user deposited
    pub total_deposited: BalanceOf,
    /// withdraw not allowed under unlock_height
    pub unlock_height: BlockNumber,
    /// cool down duration after withdraw.
    pub cool_down_duration: BlockNumber,
    /// current reward duration
    pub reward_duration: BlockNumber,
    /// block number of reward ends
    pub period_finish: BlockNumber,
    /// block number of last reward update
    pub last_update_block: BlockNumber,
    /// pool reward number for one block.
    pub reward_rate: BalanceOf,
    /// pool reward index for one share staking asset.
    pub reward_per_share_stored: BalanceOf,
}

impl<BlockNumber: Default, BalanceOf: Default> Default for PoolInfo<BlockNumber, BalanceOf> {
    fn default() -> Self {
        Self {
            is_active: false,
            total_deposited: BalanceOf::default(),
            unlock_height: BlockNumber::default(),
            cool_down_duration: BlockNumber::default(),
            reward_duration: BlockNumber::default(),
            period_finish: BlockNumber::default(),
            last_update_block: BlockNumber::default(),
            reward_rate: BalanceOf::default(),
            reward_per_share_stored: BalanceOf::default(),
        }
    }
}

impl<
        BlockNumber: Copy + PartialOrd + Saturating + UniqueSaturatedInto<u128>,
        BalanceOf: ConvertToBigUint + TokenBalance,
    > PoolInfo<BlockNumber, BalanceOf>
{
    /// Return valid reward block for current block number.
    pub fn last_reward_block_applicable(&self, current_block_number: BlockNumber) -> BlockNumber {
        if current_block_number > self.period_finish {
            self.period_finish
        } else {
            current_block_number
        }
    }

    /// Calculate reward amount for one share of staking asset.
    /// Return ArithmeticError if it encounter an arithmetic error.
    pub fn reward_per_share(
        &self,
        current_block_number: BlockNumber,
    ) -> Result<BalanceOf, ArithmeticError> {
        if self.total_deposited.is_zero() {
            Ok(self.reward_per_share_stored)
        } else {
            let last_reward_block = self.last_reward_block_applicable(current_block_number);
            let block_diff =
                self.block_to_balance(last_reward_block.saturating_sub(self.last_update_block));
            let reward_per_share_add = block_diff
                .get_big_uint()
                .checked_mul(&self.reward_rate.get_big_uint())
                .and_then(|r| r.checked_mul(&self.amount_per_share().get_big_uint()))
                .and_then(|r| r.checked_div(&self.total_deposited.get_big_uint()))
                .and_then(|r| r.to_u128())
                .ok_or(ArithmeticError::Overflow)?;

            let ret = self
                .reward_per_share_stored
                .checked_add(&BalanceOf::saturated_from(reward_per_share_add))
                .ok_or(ArithmeticError::Overflow)?;
            Ok(ret)
        }
    }

    /// Update reward amount for one share of staking asset and updating block.
    /// Return ArithmeticError if it encounter an arithmetic error.
    pub fn update_reward_per_share(
        &mut self,
        current_block_number: BlockNumber,
    ) -> Result<(), ArithmeticError> {
        self.reward_per_share_stored = self.reward_per_share(current_block_number)?;
        self.last_update_block = self.last_reward_block_applicable(current_block_number);

        Ok(())
    }

    pub fn block_to_balance(&self, duration: BlockNumber) -> BalanceOf {
        BalanceOf::saturated_from(duration.saturated_into())
    }

    pub fn amount_per_share(&self) -> BalanceOf {
        BalanceOf::saturated_from(10_u64.pow(12))
    }
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct UserPosition<BalanceOf, BoundedBalance> {
    /// User balance in reward pool
    pub deposit_balance: BalanceOf,
    /// User lock balance item.
    pub lock_balance_items: BoundedBalance,
    /// User pending reward amount
    pub reward_amount: BalanceOf,
    /// User reward index
    pub reward_per_share_paid: BalanceOf,
}

impl<BalanceOf: Default, BoundedBalance: Default> Default
    for UserPosition<BalanceOf, BoundedBalance>
{
    fn default() -> Self {
        Self {
            deposit_balance: BalanceOf::default(),
            lock_balance_items: BoundedBalance::default(),
            reward_amount: BalanceOf::default(),
            reward_per_share_paid: BalanceOf::default(),
        }
    }
}
