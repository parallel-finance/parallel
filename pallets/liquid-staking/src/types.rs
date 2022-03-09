use codec::{Decode, Encode, HasCompact};

use super::{BalanceOf, Config};
use frame_support::{dispatch::DispatchResult, traits::tokens::Balance as BalanceT};
use primitives::{ArithmeticKind, DerivativeIndex, EraIndex};
use scale_info::TypeInfo;
use sp_runtime::{traits::Zero, ArithmeticError, DispatchError, FixedPointOperand, RuntimeDebug};
use sp_std::{cmp::Ordering, result::Result, vec, vec::Vec};

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
    pub fn matching(
        &self,
        unbonding_amount: Balance,
    ) -> Result<(Balance, Balance, Balance), DispatchError> {
        use Ordering::*;

        let (bond_amount, rebond_amount, unbond_amount) = if matches!(
            self.total_stake_amount.cmp(&self.total_unstake_amount),
            Less | Equal
        ) {
            (
                Zero::zero(),
                Zero::zero(),
                self.total_unstake_amount - self.total_stake_amount,
            )
        } else {
            let amount = self.total_stake_amount - self.total_unstake_amount;
            if amount < unbonding_amount {
                (Zero::zero(), amount, Zero::zero())
            } else {
                (amount - unbonding_amount, unbonding_amount, Zero::zero())
            }
        };

        Ok((bond_amount, rebond_amount, unbond_amount))
    }

    pub fn update_total_stake_amount(
        &mut self,
        amount: Balance,
        kind: ArithmeticKind,
    ) -> DispatchResult {
        use ArithmeticKind::*;
        match kind {
            Addition => {
                self.total_stake_amount = self
                    .total_stake_amount
                    .checked_add(&amount)
                    .ok_or(ArithmeticError::Overflow)?;
            }
            Subtraction => {
                self.total_stake_amount = self
                    .total_stake_amount
                    .checked_sub(&amount)
                    .ok_or(ArithmeticError::Underflow)?;
            }
        }
        if self.total_stake_amount == self.total_unstake_amount {
            self.reset();
        }
        Ok(())
    }

    pub fn update_total_unstake_amount(
        &mut self,
        amount: Balance,
        kind: ArithmeticKind,
    ) -> DispatchResult {
        use ArithmeticKind::*;
        match kind {
            Addition => {
                self.total_unstake_amount = self
                    .total_unstake_amount
                    .checked_add(&amount)
                    .ok_or(ArithmeticError::Overflow)?;
            }
            Subtraction => {
                self.total_unstake_amount = self
                    .total_unstake_amount
                    .checked_sub(&amount)
                    .ok_or(ArithmeticError::Underflow)?;
            }
        }
        if self.total_stake_amount == self.total_unstake_amount {
            self.reset();
        }
        Ok(())
    }

    fn reset(&mut self) {
        self.total_unstake_amount = Zero::zero();
        self.total_stake_amount = Zero::zero();
    }
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub enum XcmRequest<T: Config> {
    Bond {
        index: DerivativeIndex,
        amount: BalanceOf<T>,
    },
    BondExtra {
        index: DerivativeIndex,
        amount: BalanceOf<T>,
    },
    Unbond {
        index: DerivativeIndex,
        amount: BalanceOf<T>,
    },
    Rebond {
        index: DerivativeIndex,
        amount: BalanceOf<T>,
    },
    WithdrawUnbonded {
        index: DerivativeIndex,
        num_slashing_spans: u32,
    },
    Nominate {
        index: DerivativeIndex,
        targets: Vec<T::AccountId>,
    },
}

/// Just a Balance/BlockNumber tuple to encode when a chunk of funds will be unlocked.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct UnlockChunk<Balance: HasCompact> {
    /// Amount of funds to be unlocked.
    #[codec(compact)]
    pub value: Balance,
    /// Era number at which point it'll be unlocked.
    #[codec(compact)]
    pub era: EraIndex,
}

/// The ledger of a (bonded) stash.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct StakingLedger<AccountId, Balance: HasCompact> {
    /// The stash account whose balance is actually locked and at stake.
    pub stash: AccountId,
    /// The total amount of the stash's balance that we are currently accounting for.
    /// It's just `active` plus all the `unlocking` balances.
    #[codec(compact)]
    pub total: Balance,
    /// The total amount of the stash's balance that will be at stake in any forthcoming
    /// rounds.
    #[codec(compact)]
    pub active: Balance,
    /// Any balance that is becoming free, which may eventually be transferred out
    /// of the stash (assuming it doesn't get slashed first).
    pub unlocking: Vec<UnlockChunk<Balance>>,
    /// List of eras for which the stakers behind a validator have claimed rewards. Only updated
    /// for validators.
    pub claimed_rewards: Vec<EraIndex>,
}

impl<AccountId, Balance: BalanceT + FixedPointOperand> StakingLedger<AccountId, Balance> {
    /// Initializes the default ledger using the given `Stash` account.
    pub fn new(stash: AccountId, value: Balance) -> Self {
        Self {
            stash,
            total: value,
            active: value,
            unlocking: vec![],
            claimed_rewards: vec![],
        }
    }

    /// Remove entries from `unlocking` that are sufficiently old and reduce the
    /// total by the sum of their balances.
    pub fn consolidate_unlocked(&mut self, current_era: EraIndex) {
        let mut total = self.total;
        self.unlocking.retain(|chunk| {
            if chunk.era > current_era {
                true
            } else {
                total = total.saturating_sub(chunk.value);
                false
            }
        });
        self.total = total;
    }

    /// Rebond funds that were scheduled for unlocking.
    pub fn rebond(&mut self, value: Balance) {
        let mut unlocking_balance: Balance = Zero::zero();

        while let Some(last) = self.unlocking.last_mut() {
            if unlocking_balance + last.value <= value {
                unlocking_balance += last.value;
                self.active += last.value;
                self.unlocking.pop();
            } else {
                let diff = value - unlocking_balance;

                unlocking_balance += diff;
                self.active += diff;
                last.value -= diff;
            }

            if unlocking_balance >= value {
                break;
            }
        }
    }

    /// Add some extra amount that have appeared in the stash `free_balance` into the balance up
    /// for staking.
    pub fn bond_extra(&mut self, value: Balance) {
        self.total += value;
        self.active += value;
    }

    /// Schedule a portion of the stash to be unlocked ready for transfer out after the bond
    /// period ends. If this leaves an amount actively bonded less than
    pub fn unbond(&mut self, value: Balance, target_era: EraIndex) {
        if let Some(mut chunk) = self
            .unlocking
            .last_mut()
            .filter(|chunk| chunk.era == target_era)
        {
            // To keep the chunk count down, we only keep one chunk per era. Since
            // `unlocking` is a FIFO queue, if a chunk exists for `era` we know that it will
            // be the last one.
            chunk.value = chunk.value.saturating_add(value);
        } else {
            self.unlocking.push(UnlockChunk {
                value,
                era: target_era,
            });
        };

        // Skipped the minimum balance check because the platform will
        // bond `MinNominatorBond` to make sure:
        // 1. No chill call is needed
        // 2. No minimum balance check
        self.active -= value;
    }
}
