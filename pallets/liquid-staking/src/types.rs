use codec::{Decode, Encode, HasCompact};

use super::{BalanceOf, Config, DerivativeIndex, EraIndex};
use frame_support::{dispatch::DispatchResult, traits::tokens::Balance as BalanceT};
use primitives::ArithmeticKind;
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{AtLeast32BitUnsigned, Saturating, Zero},
    ArithmeticError, DispatchError, FixedPointOperand, RuntimeDebug,
};
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
        amount: BalanceOf<T>,
    },
    Rebond {
        amount: BalanceOf<T>,
    },
    WithdrawUnbonded {
        num_slashing_spans: u32,
        derivative_index: DerivativeIndex,
    },
    Nominate {
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

impl<AccountId, Balance: HasCompact + Copy + Saturating + AtLeast32BitUnsigned + Zero>
    StakingLedger<AccountId, Balance>
{
    /// Initializes the default object using the given `validator`.
    pub fn default_from(stash: AccountId) -> Self {
        Self {
            stash,
            total: Zero::zero(),
            active: Zero::zero(),
            unlocking: vec![],
            claimed_rewards: vec![],
        }
    }

    /// Remove entries from `unlocking` that are sufficiently old and reduce the
    /// total by the sum of their balances.
    pub fn consolidate_unlocked(self, current_era: EraIndex) -> Self {
        let mut total = self.total;
        let unlocking = self
            .unlocking
            .into_iter()
            .filter(|chunk| {
                if chunk.era > current_era {
                    true
                } else {
                    total = total.saturating_sub(chunk.value);
                    false
                }
            })
            .collect();

        Self {
            stash: self.stash,
            total,
            active: self.active,
            unlocking,
            claimed_rewards: self.claimed_rewards,
        }
    }

    /// Re-bond funds that were scheduled for unlocking.
    ///
    /// Returns the updated ledger, and the amount actually rebonded.
    pub fn rebond(mut self, value: Balance) -> (Self, Balance) {
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

        (self, unlocking_balance)
    }

    pub fn bond(mut self, value: Balance) -> Self {
        self.total = value;
        self.active = value;
        self
    }

    pub fn bond_extra(mut self, value: Balance) -> Self {
        self.total += value;
        self.active += value;
        self
    }

    pub fn unbond(mut self, value: Balance, target_era: EraIndex) -> Self {
        if let Some(mut chunk) = self
            .unlocking
            .last_mut()
            .filter(|chunk| chunk.era == target_era)
        {
            // To keep the chunk count down, we only keep one chunk per era. Since
            // `unlocking` is a FiFo queue, if a chunk exists for `era` we know that it will
            // be the last one.
            chunk.value = chunk.value.saturating_add(value);
        } else {
            self.unlocking.push(UnlockChunk {
                value,
                era: target_era,
            });
        };
        self
    }
}

// impl<T: Config> StakingLedger<T> {
//     /// New ledger
//     pub fn new() -> Self {
//         Self {
//             withdrawable: Zero::zero(),
//             unlocking: vec![],
//         }
//     }

//     /// Remove expired unlocking and calculate its amount to withdrawable
//     pub fn consolidate_unlocked(self, current_era: EraIndex) -> Self {
//         let mut withdrawable_amount: BalanceOf<T> = Zero::zero();
//         let unlocking = self
//             .unlocking
//             .into_iter()
//             .filter(|chunk| {
//                 if chunk.era > current_era {
//                     true
//                 } else {
//                     withdrawable_amount = withdrawable_amount.saturating_add(chunk.value);
//                     false
//                 }
//             })
//             .collect::<Vec<UnlockChunk<T>>>();

//         Self {
//             withdrawable: self.withdrawable.saturating_add(withdrawable_amount),
//             unlocking,
//         }
//     }
// }
