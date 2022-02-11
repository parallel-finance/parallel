use codec::{Decode, Encode};

use super::{BalanceOf, Config};
use frame_support::traits::tokens::Balance as BalanceT;
use primitives::ExchangeRateProvider;
use scale_info::TypeInfo;
use sp_runtime::{
    traits::Zero, ArithmeticError, DispatchError, FixedPointNumber, FixedPointOperand, RuntimeDebug,
};
use sp_std::{cmp::Ordering, vec::Vec};

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
    /// `unbonding_amount` is the total amount of the unbonding asset in relaychain.
    ///
    /// the returned tri-tuple is formed as `(bond_amount, rebond_amount, unbond_amount)`.
    pub fn matching<T: ExchangeRateProvider>(
        &self,
        unbonding_amount: Balance,
    ) -> Result<(Balance, Balance, Balance), DispatchError> {
        use Ordering::*;

        let unstake_asset_amout = T::get_exchange_rate()
            .checked_mul_int(self.total_unstake_amount)
            .ok_or(ArithmeticError::Overflow)?;

        if matches!(
            self.total_stake_amount.cmp(&unstake_asset_amout),
            Less | Equal
        ) {
            return Ok((
                Zero::zero(),
                Zero::zero(),
                unstake_asset_amout - self.total_stake_amount,
            ));
        }

        let amount = self.total_stake_amount - unstake_asset_amout;
        if amount < unbonding_amount {
            Ok((Zero::zero(), amount, Zero::zero()))
        } else {
            Ok((amount - unbonding_amount, unbonding_amount, Zero::zero()))
        }
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
        amount: BalanceOf<T>,
    },
    Nominate {
        targets: Vec<T::AccountId>,
    },
}
