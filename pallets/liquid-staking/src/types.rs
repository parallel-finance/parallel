use codec::{Decode, Encode};

use super::{BalanceOf, Config};
use frame_support::{dispatch::DispatchResult, traits::tokens::Balance as BalanceT};
use primitives::{ArithmeticKind, LiquidStakingConvert};
use scale_info::TypeInfo;
use sp_runtime::{traits::Zero, ArithmeticError, DispatchError, FixedPointOperand, RuntimeDebug};
use sp_std::{cmp::Ordering, result::Result, vec::Vec};

/// The matching pool's total stake & unstake amount in one era
#[derive(Copy, Clone, Eq, PartialEq, Default, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct MatchingLedger<Balance: BalanceT> {
    /// The total stake amount in one era
    pub total_stake_amount: Balance,
    /// The total unstake amount in one era
    pub total_unstake_amount: Balance,
}

impl<Balance: BalanceT + FixedPointOperand> MatchingLedger<Balance> {
    /// Matching requests in current period.
    ///
    /// `unbonding_amount` is the total amount of the unbonding asset on the relaychain.
    ///
    /// the returned tri-tuple is formed as `(bond_amount, rebond_amount, unbond_amount)`.
    pub fn matching<T: LiquidStakingConvert<Balance>>(
        &mut self,
        unbonding_amount: Balance,
    ) -> Result<(Balance, Balance, Balance), DispatchError> {
        use Ordering::*;

        let unstake_asset_amout =
            T::liquid_to_staking(self.total_unstake_amount).ok_or(ArithmeticError::Overflow)?;

        let (bond_amount, rebond_amount, unbond_amount) = if matches!(
            self.total_stake_amount.cmp(&unstake_asset_amout),
            Less | Equal
        ) {
            (
                Zero::zero(),
                Zero::zero(),
                unstake_asset_amout - self.total_stake_amount,
            )
        } else {
            let amount = self.total_stake_amount - unstake_asset_amout;
            if amount < unbonding_amount {
                (Zero::zero(), amount, Zero::zero())
            } else {
                (amount - unbonding_amount, unbonding_amount, Zero::zero())
            }
        };

        self.total_stake_amount = bond_amount
            .checked_add(&rebond_amount)
            .ok_or(ArithmeticError::Overflow)?;
        if unbond_amount.is_zero() {
            self.total_unstake_amount = Zero::zero();
        } else {
            self.total_unstake_amount =
                T::staking_to_liquid(unbond_amount).ok_or(ArithmeticError::Overflow)?;
        }

        Ok((bond_amount, rebond_amount, unbond_amount))
    }

    pub fn update_total_stake_amount(
        &mut self,
        amount: Balance,
        kind: ArithmeticKind,
    ) -> DispatchResult {
        match kind {
            ArithmeticKind::Addition => {
                self.total_stake_amount = self
                    .total_stake_amount
                    .checked_add(&amount)
                    .ok_or(ArithmeticError::Overflow)?;
            }
            ArithmeticKind::Subtraction => {
                self.total_stake_amount = self
                    .total_stake_amount
                    .checked_sub(&amount)
                    .ok_or(ArithmeticError::Underflow)?;
            }
        }
        Ok(())
    }

    pub fn update_total_unstake_amount(
        &mut self,
        amount: Balance,
        kind: ArithmeticKind,
    ) -> DispatchResult {
        match kind {
            ArithmeticKind::Addition => {
                self.total_unstake_amount = self
                    .total_unstake_amount
                    .checked_add(&amount)
                    .ok_or(ArithmeticError::Overflow)?;
            }
            ArithmeticKind::Subtraction => {
                self.total_unstake_amount = self
                    .total_unstake_amount
                    .checked_sub(&amount)
                    .ok_or(ArithmeticError::Underflow)?;
            }
        }
        Ok(())
    }
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub enum XcmRequest<T: Config> {
    Bond {
        amount: BalanceOf<T>,
    },
    BondExtra {
        amount: BalanceOf<T>,
    },
    Unbond {
        liquid_amount: BalanceOf<T>,
    },
    Rebond {
        amount: BalanceOf<T>,
    },
    WithdrawUnbonded {
        num_slashing_spans: u32,
        amount: BalanceOf<T>,
    },
    Nominate {
        targets: Vec<T::AccountId>,
    },
}
