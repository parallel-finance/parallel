use frame_support::traits::tokens::Balance as BalanceT;
use pallet_traits::DistributionStrategy;
use primitives::DerivativeIndex;
use sp_runtime::FixedPointOperand;
pub struct AverageDistribution;
impl<Balance: BalanceT + FixedPointOperand> DistributionStrategy<Balance> for AverageDistribution {
    fn get_bond_distributions(
        bonding_amounts: &mut Vec<(DerivativeIndex, Balance)>,
        input: Balance,
        cap: Balance,
        min_nominator_bond: Balance,
    ) -> Vec<(DerivativeIndex, Balance)> {
        let length = TryInto::<Balance>::try_into(bonding_amounts.len()).unwrap_or_default();
        if length.is_zero() {
            return Default::default();
        }
        let mut distributions: Vec<(DerivativeIndex, Balance)> = vec![];
        let amount = input.checked_div(&length).unwrap_or_default();
        for (index, bonded) in bonding_amounts.iter() {
            if amount.saturating_add(*bonded) < min_nominator_bond {
                continue;
            }
            let amount = cap.saturating_sub(*bonded).min(amount);
            distributions.push((*index, amount));
        }

        distributions
    }

    fn get_unbond_distributions(
        bonding_amounts: &mut Vec<(DerivativeIndex, Balance)>,
        input: Balance,
        _cap: Balance,
        min_nominator_bond: Balance,
    ) -> Vec<(DerivativeIndex, Balance)> {
        let length = TryInto::<Balance>::try_into(bonding_amounts.len()).unwrap_or_default();
        if length.is_zero() {
            return Default::default();
        }
        let mut distributions: Vec<(DerivativeIndex, Balance)> = vec![];
        let amount = input.checked_div(&length).unwrap_or_default();
        for (index, bonded) in bonding_amounts.iter() {
            if bonded.saturating_sub(amount) < min_nominator_bond {
                continue;
            }
            distributions.push((*index, amount));
        }

        distributions
    }

    fn get_rebond_distributions(
        unbonding_amounts: &mut Vec<(DerivativeIndex, Balance, Balance)>,
        input: Balance,
        _cap: Balance,
        _min_nominator_bond: Balance,
    ) -> Vec<(DerivativeIndex, Balance)> {
        let length = TryInto::<Balance>::try_into(unbonding_amounts.len()).unwrap_or_default();
        if length.is_zero() {
            return Default::default();
        }
        let mut distributions: Vec<(DerivativeIndex, Balance)> = vec![];
        let amount = input.checked_div(&length).unwrap_or_default();
        for (index, _, _) in unbonding_amounts.iter() {
            distributions.push((*index, amount));
        }

        distributions
    }
}

pub struct MaximizationDistribution;
impl<Balance: BalanceT + FixedPointOperand> DistributionStrategy<Balance>
    for MaximizationDistribution
{
    fn get_bond_distributions(
        bonding_amounts: &mut Vec<(DerivativeIndex, Balance)>,
        input: Balance,
        cap: Balance,
        min_nominator_bond: Balance,
    ) -> Vec<(DerivativeIndex, Balance)> {
        //ascending sequence
        bonding_amounts.sort_by(|a, b| a.1.cmp(&b.1));

        let mut distributions: Vec<(DerivativeIndex, Balance)> = vec![];
        let mut remain = input;

        for (index, bonded) in bonding_amounts.iter() {
            if remain.is_zero() {
                break;
            }
            let amount = cap.saturating_sub(*bonded).min(remain);
            if amount.is_zero() {
                // `bonding_amounts` is an ascending sequence
                // if occurs an item that exceed the cap, the items after this one must all be exceeded
                break;
            }

            if amount.saturating_add(*bonded) < min_nominator_bond {
                continue;
            }

            distributions.push((*index, amount));
            remain = remain.saturating_sub(amount);
        }

        distributions
    }

    fn get_unbond_distributions(
        bonding_amounts: &mut Vec<(DerivativeIndex, Balance)>,
        input: Balance,
        _cap: Balance,
        min_nominator_bond: Balance,
    ) -> Vec<(DerivativeIndex, Balance)> {
        // descending sequence
        bonding_amounts.sort_by(|a, b| b.1.cmp(&a.1));

        let mut distributions: Vec<(DerivativeIndex, Balance)> = vec![];
        let mut remain = input;

        for (index, bonded) in bonding_amounts.iter() {
            if remain.is_zero() {
                break;
            }
            let amount = remain.min(bonded.saturating_sub(min_nominator_bond));
            if amount.is_zero() {
                continue;
            }
            distributions.push((*index, amount));
            remain = remain.saturating_sub(amount);
        }

        distributions
    }
    fn get_rebond_distributions(
        unbonding_amounts: &mut Vec<(DerivativeIndex, Balance, Balance)>,
        input: Balance,
        cap: Balance,
        _min_nominator_bond: Balance,
    ) -> Vec<(DerivativeIndex, Balance)> {
        // descending sequence
        unbonding_amounts.sort_by(|a, b| b.1.cmp(&a.1));

        let mut distributions: Vec<(DerivativeIndex, Balance)> = vec![];
        let mut remain = input;

        for (index, unlocking, active) in unbonding_amounts.iter() {
            if remain.is_zero() {
                break;
            }
            let amount = remain.min(*unlocking).min(cap.saturating_sub(*active));
            if amount.is_zero() {
                continue;
            }
            distributions.push((*index, amount));
            remain = remain.saturating_sub(amount);
        }

        distributions
    }
}
