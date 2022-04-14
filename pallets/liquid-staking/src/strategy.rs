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
        let length = TryInto::<Balance>::try_into(active_bonded_amount.len()).unwrap_or_default();
        if length.is_zero() {
            return Default::default();
        }
        let mut distributions: Vec<(DerivativeIndex, Balance)> = vec![];
        let amount = input.checked_div(&length).unwrap_or_default();
        for (index, bonded) in active_bonded_amount.iter() {
            if amount.saturating_add(*bonded) < min_bond_amount {
                continue;
            }
            let amount = capacity.saturating_sub(*bonded).min(amount);
            distributions.push((*index, amount));
        }

        distributions
    }

    fn unbond(
        active_bonded_amount: &mut Vec<(DerivativeIndex, Balance)>,
        input: Balance,
        _capacity: Balance,
        min_bond_amount: Balance,
    ) -> Vec<(DerivativeIndex, Balance)> {
        let length = TryInto::<Balance>::try_into(active_bonded_amount.len()).unwrap_or_default();
        if length.is_zero() {
            return Default::default();
        }
        let mut distributions: Vec<(DerivativeIndex, Balance)> = vec![];
        let amount = input.checked_div(&length).unwrap_or_default();
        for (index, bonded) in active_bonded_amount.iter() {
            if bonded.saturating_sub(amount) < min_bond_amount {
                continue;
            }
            distributions.push((*index, amount));
        }

        distributions
    }

    fn rebond(
        unlocking_amount: &mut Vec<(DerivativeIndex, Balance)>,
        input: Balance,
        _capacity: Balance,
        _min_bond_amount: Balance,
    ) -> Vec<(DerivativeIndex, Balance)> {
        let length = TryInto::<Balance>::try_into(unlocking_amount.len()).unwrap_or_default();
        if length.is_zero() {
            return Default::default();
        }
        let mut distributions: Vec<(DerivativeIndex, Balance)> = vec![];
        let amount = input.checked_div(&length).unwrap_or_default();
        for (index, _) in unlocking_amount.iter() {
            distributions.push((*index, amount));
        }

        distributions
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

            if amount.saturating_add(*bonded) < min_bond_amount {
                continue;
            }

            distributions.push((*index, amount));
            remain = remain.saturating_sub(amount);
        }

        distributions
    }

    fn unbond(
        active_bonded_amount: &mut Vec<(DerivativeIndex, Balance)>,
        input: Balance,
        _capacity: Balance,
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
            let amount = remain.min(bonded.saturating_sub(min_bond_amount));
            if amount.is_zero() {
                continue;
            }
            distributions.push((*index, amount));
            remain = remain.saturating_sub(amount);
        }

        distributions
    }
    fn rebond(
        unlocking_amount: &mut Vec<(DerivativeIndex, Balance)>,
        input: Balance,
        _capacity: Balance,
        _min_bond_amount: Balance,
    ) -> Vec<(DerivativeIndex, Balance)> {
        // descending sequence
        unlocking_amount.sort_by(|a, b| b.1.cmp(&a.1));

        let mut distributions: Vec<(DerivativeIndex, Balance)> = vec![];
        let mut remain = input;

        for (index, unlocking) in unlocking_amount.iter() {
            if remain.is_zero() {
                break;
            }
            let amount = remain.min(*unlocking);
            if amount.is_zero() {
                continue;
            }
            distributions.push((*index, amount));
            remain = remain.saturating_sub(amount);
        }

        distributions
    }
}
