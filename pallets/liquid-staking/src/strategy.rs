use frame_support::{dispatch::DispatchResult, traits::tokens::Balance as BalanceT};
use primitives::{DerivativeIndex, StrategyLike};
use sp_runtime::{
    traits::{CheckedDiv, Zero},
    ArithmeticError, DispatchError, FixedPointOperand, RuntimeDebug,
};
pub struct AverageStrategy;
impl<Balance: BalanceT + FixedPointOperand> StrategyLike<Balance> for AverageStrategy {
    fn bond(
        active_bonded_amount: &mut Vec<(DerivativeIndex, Balance)>,
        input: Balance,
        capacity: Balance,
        min_bond_amount: Balance,
    ) -> Vec<(DerivativeIndex, Balance)> {
        //TODO: use capacity as limit
        let length = TryInto::<Balance>::try_into(active_bonded_amount.len()).unwrap_or_default();
        if length.is_zero() {
            return Default::default();
        }
        active_bonded_amount
            .iter()
            .map(|(index, _)| (*index, input.checked_div(&length).unwrap_or_default()))
            .collect()
    }

    fn unbond(
        active_bonded_amount: &mut Vec<(DerivativeIndex, Balance)>,
        input: Balance,
        capacity: Balance,
        min_bond_amount: Balance,
    ) -> Vec<(DerivativeIndex, Balance)> {
        Self::bond(active_bonded_amount, input, capacity, min_bond_amount)
    }

    fn rebond(
        unlocking_amount: &mut Vec<(DerivativeIndex, Balance)>,
        input: Balance,
        capacity: Balance,
        min_bond_amount: Balance,
    ) -> Vec<(DerivativeIndex, Balance)> {
        Self::bond(unlocking_amount, input, capacity, min_bond_amount)
    }
}

pub struct QueueStrategy;
impl<Balance: BalanceT + FixedPointOperand> StrategyLike<Balance> for QueueStrategy {
    fn bond(
        active_bonded_amount: &mut Vec<(DerivativeIndex, Balance)>,
        input: Balance,
        capacity: Balance,
        min_bond_amount: Balance,
    ) -> Vec<(DerivativeIndex, Balance)> {
        //ascending sequence
        active_bonded_amount.sort_by(|a, b| a.1.cmp(&b.1));

        let mut distributions: Vec<(DerivativeIndex, Balance)> = vec![];
        let mut remain = input;

        for (index, bonded) in active_bonded_amount.iter() {
            if remain.is_zero() {
                break;
            }
            let amount = capacity.saturating_sub(*bonded).min(remain);
            if amount.is_zero() {
                // `active_bonded_amount` is an ascending sequence
                // if occurs an item that exceed the capacity, the items after this one must all be exceeded
                break;
            }
            distributions.push((*index, amount));
            remain = remain.saturating_sub(amount);
        }

        distributions
    }

    fn unbond(
        active_bonded_amount: &mut Vec<(DerivativeIndex, Balance)>,
        input: Balance,
        capacity: Balance,
        min_bond_amount: Balance,
    ) -> Vec<(DerivativeIndex, Balance)> {
        // descending sequence
        active_bonded_amount.sort_by(|a, b| b.1.cmp(&a.1));

        let mut distributions: Vec<(DerivativeIndex, Balance)> = vec![];
        let mut remain = input;

        for (index, bonded) in active_bonded_amount.iter() {
            if remain.is_zero() {
                break;
            }
            let amount = remain.min(*bonded);
            distributions.push((*index, amount));
            remain = remain.saturating_sub(amount);
        }

        distributions
    }
    fn rebond(
        unlocking_amount: &mut Vec<(DerivativeIndex, Balance)>,
        input: Balance,
        capacity: Balance,
        min_bond_amount: Balance,
    ) -> Vec<(DerivativeIndex, Balance)> {
        Self::unbond(unlocking_amount, input, capacity, min_bond_amount)
    }
}
