use frame_support::{require_transactional, traits::Get};
use orml_traits::MultiCurrency;
use sp_runtime::{traits::Zero, ArithmeticError, DispatchError, DispatchResult, FixedPointNumber};

use primitives::EraIndex;

use crate::{Config, Error, ExchangeRate, MatchingPoolByEra, MatchingQueueByUser, Pallet};

pub(crate) type BalanceOf<T> =
    <<T as Config>::Currency as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;

//todo change the return type
pub trait LiquidStakingProtocol<AccountId, Balance> {
    fn stake(who: &AccountId, amount: Balance) -> DispatchResult;
    fn unstake(who: &AccountId, amount: Balance) -> Result<Balance, DispatchError>;
    fn claim(who: &AccountId) -> DispatchResult;
}

impl<T: Config> LiquidStakingProtocol<T::AccountId, BalanceOf<T>> for Pallet<T> {
    // After confirmed bond on relaychain,
    // after update exchangerate (record_reward),
    // and then mint/deposit xKSM.
    #[require_transactional]
    fn stake(who: &T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
        //todo reserve, insurance pool
        T::Currency::transfer(T::StakingCurrency::get(), who, &Self::account_id(), amount)?;

        MatchingQueueByUser::<T>::try_mutate(
            who,
            &Self::current_era(),
            |user_ledger_per_era| -> DispatchResult {
                user_ledger_per_era.total_stake_amount = user_ledger_per_era
                    .total_stake_amount
                    .checked_add(amount)
                    .ok_or(ArithmeticError::Overflow)?;

                Ok(())
            },
        )?;

        MatchingPoolByEra::<T>::try_mutate(
            &Self::current_era(),
            |pool_ledger_per_era| -> DispatchResult {
                pool_ledger_per_era.total_stake_amount = pool_ledger_per_era
                    .total_stake_amount
                    .checked_add(amount)
                    .ok_or(ArithmeticError::Overflow)?;

                Ok(())
            },
        )?;
        Ok(().into())
    }

    // After confirmed unbond on relaychain,
    // and then burn/withdraw xKSM.
    // before update exchangerate (record_reward)
    #[require_transactional]
    fn unstake(who: &T::AccountId, amount: BalanceOf<T>) -> Result<BalanceOf<T>, DispatchError> {
        // can not burn directly because we have match mechanism
        T::Currency::transfer(T::LiquidCurrency::get(), who, &Self::account_id(), amount)?;

        let exchange_rate = ExchangeRate::<T>::get();
        let asset_amount = exchange_rate
            .checked_mul_int(amount)
            .ok_or(Error::<T>::InvalidExchangeRate)?;

        MatchingQueueByUser::<T>::try_mutate(
            who,
            &Self::current_era(),
            |user_ledger_per_era| -> DispatchResult {
                user_ledger_per_era.total_unstake_amount = user_ledger_per_era
                    .total_unstake_amount
                    .checked_add(asset_amount)
                    .ok_or(ArithmeticError::Overflow)?;

                Ok(())
            },
        )?;

        MatchingPoolByEra::<T>::try_mutate(
            &Self::current_era(),
            |pool_ledger_per_era| -> DispatchResult {
                pool_ledger_per_era.total_unstake_amount = pool_ledger_per_era
                    .total_unstake_amount
                    .checked_add(asset_amount)
                    .ok_or(ArithmeticError::Overflow)?;

                Ok(())
            },
        )?;
        Ok(asset_amount)
    }

    #[require_transactional]
    fn claim(who: &T::AccountId) -> DispatchResult {
        let mut withdrawable_stake_amount = 0u128;
        let mut withdrawable_unstake_amount = 0u128;
        let mut remove_record_from_user_queue = Vec::<EraIndex>::new();
        MatchingQueueByUser::<T>::iter_prefix(who).for_each(|(era_index, user_ledger_per_era)| {
            Self::accumulate_claim_by_era(
                who,
                era_index,
                user_ledger_per_era,
                &mut withdrawable_stake_amount,
                &mut withdrawable_unstake_amount,
                &mut remove_record_from_user_queue,
            );
        });

        // remove finished records from MatchingQueue
        if !remove_record_from_user_queue.is_empty() {
            remove_record_from_user_queue.iter().for_each(|era_index| {
                MatchingQueueByUser::<T>::remove(who, era_index);
            });
        }

        // transfer xKSM from palletId to who
        if !withdrawable_stake_amount.is_zero() {
            let xtoken_amount = ExchangeRate::<T>::get()
                .reciprocal()
                .and_then(|r| r.checked_mul_int(withdrawable_stake_amount))
                .ok_or(Error::<T>::InvalidExchangeRate)?;
            T::Currency::transfer(
                T::LiquidCurrency::get(),
                &Self::account_id(),
                who,
                xtoken_amount,
            )?;
        }

        // transfer KSM from palletId to who
        if !withdrawable_unstake_amount.is_zero() {
            T::Currency::transfer(
                T::StakingCurrency::get(),
                &Self::account_id(),
                who,
                withdrawable_unstake_amount,
            )?;
        }
        Ok(().into())
    }
}
