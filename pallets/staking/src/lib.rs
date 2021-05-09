// Copyright 2021 Parallel Finance Developer.
// This file is part of Parallel Finance.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
// http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # Liquid staking pallet
//!
//! This pallet manages the NPoS operations for relay chain asset.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    pallet_prelude::*, PalletId, transactional,
};
use frame_system::pallet_prelude::*;
use sp_runtime::{
    traits::AccountIdConversion, RuntimeDebug, FixedPointNumber,
};

use orml_traits::{MultiCurrency, MultiCurrencyExtended};

use primitives::{Amount, Balance, CurrencyId, Rate};
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Container for pending balance information
#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, Default)]
pub struct UnstakeInfo<BlockNumber> {
    pub amount: Balance,
    pub block_number: BlockNumber,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// Currency type used for staking and liquid assets
        type Currency: MultiCurrencyExtended<
            Self::AccountId,
            CurrencyId = CurrencyId,
            Balance = Balance,
            Amount = Amount,
        >;

        /// Currency used for staking
        #[pallet::constant]
        type StakingCurrency: Get<CurrencyId>;

        /// Currency used for liquid voucher
        #[pallet::constant]
        type LiquidCurrency: Get<CurrencyId>;

        /// The pallet id of liquid staking, keeps all the staking assets.
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// The origin which can withdraw staking assets.
        type WithdrawOrigin: EnsureOrigin<Self::Origin>;

        /// The maximum assets can be withdrawed to a multisig account.
        #[pallet::constant]
        type MaxWithdrawAmount: Get<Balance>;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// ExchangeRate is invalid
        InvalidExchangeRate,
        /// Calculation overflow
        Overflow,
        /// Calculation underflow
        Underflow,
        /// The withdraw assets exceed the threshold
        ExcessWithdraw,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// The assets get staked successfully
        Staked(T::AccountId, Balance),
        /// The voucher get unstaked successfully
        Unstaked(T::AccountId, Balance),
        /// The withdraw request is successful
        WithdrawSuccess(T::AccountId, Balance),
        /// The rewards are recorded successfully
        RewardsRecorded(T::AccountId, Balance),
    }

    /// The exchange rate converts staking native token to voucher.
    #[pallet::storage]
    #[pallet::getter(fn exchange_rate)]
    pub type ExchangeRate<T: Config> = StorageValue<_, Rate, ValueQuery>;

    /// The total amount of a staking asset.
    #[pallet::storage]
    #[pallet::getter(fn total_staking)]
    pub type TotalStakingAsset<T: Config> = StorageValue<_, Balance, ValueQuery>;

    /// The total amount of staking voucher.
    #[pallet::storage]
    #[pallet::getter(fn total_voucher)]
    pub type TotalVoucher<T: Config> = StorageValue<_, Balance, ValueQuery>;

    /// The queue stores all the pending unstaking requests.
    #[pallet::storage]
    #[pallet::getter(fn account_pending_unstake)]
    pub type AccountPendingUnstake<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        UnstakeInfo<T::BlockNumber>,
    >;

    // /// The queue stores all the unstaking requests in processing.
    // #[pallet::storage]
    // #[pallet::getter(fn unstaking_processing_queue)]
    // pub type UnstakingProcessingQueue<T: Config> = StorageValue<_, Vec<UnstakingInfo>, ValueQuery>;

    // /// The queue stroes all the processed unstaking requests and wait for redeeming.
    // #[pallet::storage]
    // #[pallet::getter(fn unstaking_redeeming_queue)]
    // pub type UnstakingRedeemingQueue<T: Config> = StorageValue<_, Vec<UnstakingInfo>, ValueQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig {
        pub exchange_rate: Rate,
    }

    #[cfg(feature = "std")]
    impl Default for GenesisConfig {
        fn default() -> Self {
            Self { exchange_rate: Rate::default() }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig {
        fn build(&self) {
            ExchangeRate::<T>::put(self.exchange_rate);
        }
    }

    #[cfg(feature = "std")]
    impl GenesisConfig {
        /// Direct implementation of `GenesisBuild::build_storage`.
        ///
        /// Kept in order not to break dependency.
        pub fn build_storage<T: Config>(&self) -> Result<sp_runtime::Storage, String> {
            <Self as GenesisBuild<T>>::build_storage(self)
        }

        /// Direct implementation of `GenesisBuild::assimilate_storage`.
        ///
        /// Kept in order not to break dependency.
        pub fn assimilate_storage<T: Config>(
            &self,
            storage: &mut sp_runtime::Storage
        ) -> Result<(), String> {
            <Self as GenesisBuild<T>>::assimilate_storage(self, storage)
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Put assets under staking, the native assets will be transferred to the account
        /// owned by the pallet, user receive voucher in return, such vocher can be further
        /// used as collateral for lending. 
        ///
        /// - `amount`: the amount of staking assets
        #[pallet::weight(10_000)]
        #[transactional]
        pub fn stake(
            origin: OriginFor<T>,
            amount: Balance,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            let exchange_rate = ExchangeRate::<T>::get();
            let voucher_amount = exchange_rate
                .reciprocal()
                .and_then(|r| r.checked_mul_int(amount))
                .ok_or(Error::<T>::InvalidExchangeRate)?;

            T::Currency::transfer(
                T::StakingCurrency::get(),
                &sender,
                &Self::account_id(),
                amount
            )?;
            T::Currency::deposit(T::LiquidCurrency::get(), &sender, voucher_amount)?;
            TotalVoucher::<T>::try_mutate(|b| -> DispatchResult {
                *b = b.checked_add(voucher_amount).ok_or(Error::<T>::Overflow)?;
                Ok(())
            })?;
            TotalStakingAsset::<T>::try_mutate(|b| -> DispatchResult {
                *b = b.checked_add(amount).ok_or(Error::<T>::Overflow)?;
                Ok(())
            })?;

            Self::deposit_event(Event::Staked(sender, amount));
            Ok(().into())
        }

        /// Withdraw assets from liquid staking pool for offchain relay chain nomination.
        ///
        /// May only be called from `T::WithdrawOrigin`.
        ///
        /// - `agent`: the multisig account of relay chain.
        /// - `amount`: the requested assets.
        #[pallet::weight(10_000)]
        #[transactional]
        pub fn withdraw(
            origin: OriginFor<T>,
            agent: T::AccountId,
            amount: Balance,
        ) -> DispatchResultWithPostInfo {
            T::WithdrawOrigin::ensure_origin(origin)?;
            ensure!(amount <= T::MaxWithdrawAmount::get(), Error::<T>::ExcessWithdraw);
            
            T::Currency::transfer(
                T::StakingCurrency::get(),
                &Self::account_id(),
                &agent,
                amount,
            )?;

            Self::deposit_event(Event::WithdrawSuccess(agent, amount));
            Ok(().into())
        }

        /// Record the staking rewards, no real transfer.
        /// TODO restrict the times an account can report in one day and max rewards.
        ///
        /// May only be called from `T::WithdrawOrigin`.
        ///
        /// - `amount`: the rewarded assets.
        #[pallet::weight(10_000)]
        #[transactional]
        pub fn record_rewards(
            origin: OriginFor<T>,
            agent: T::AccountId,
            amount: Balance,
        ) -> DispatchResultWithPostInfo {
            T::WithdrawOrigin::ensure_origin(origin)?;
            
            TotalStakingAsset::<T>::try_mutate(|b| -> DispatchResult {
                *b = b.checked_add(amount).ok_or(Error::<T>::Overflow)?;
                Ok(())
            })?;
            let exchange_rate = Rate::checked_from_rational(
                TotalStakingAsset::<T>::get(),
                TotalVoucher::<T>::get(),
            ).ok_or(Error::<T>::InvalidExchangeRate)?;
            ExchangeRate::<T>::put(exchange_rate);

            Self::deposit_event(Event::RewardsRecorded(agent, amount));
            Ok(().into())
        }

        /// Unstake by exchange voucher for assets, the assets will not be avaliable immediately.
        /// Instead, the request is recorded and pending for the nomination accounts in relay
        /// chain to do the `unbond` operation.
        ///
        /// - `amount`: the amount of unstaking voucher
        #[pallet::weight(10_000)]
        #[transactional]
        pub fn unstake(
            origin: OriginFor<T>,
            amount: Balance,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            let exchange_rate = ExchangeRate::<T>::get();
            let asset_amount = exchange_rate
                .checked_mul_int(amount)
                .ok_or(Error::<T>::InvalidExchangeRate)?;

            AccountPendingUnstake::<T>::try_mutate(&sender, |info| -> DispatchResult {
                let block_number = frame_system::Pallet::<T>::block_number();
                let new_info = info.map_or::<Result<_, Error<T>>, _>(
                    Ok(UnstakeInfo { amount: asset_amount, block_number }),
                    |mut v| {
                        v.amount = v.amount.checked_add(asset_amount).ok_or(Error::<T>::Overflow)?;
                        v.block_number = block_number;
                        Ok(v)
                    }
                )?;
                *info = Some(new_info);
                Ok(())
            })?;
            T::Currency::withdraw(T::LiquidCurrency::get(), &sender, amount)?;
            TotalVoucher::<T>::try_mutate(|b| -> DispatchResult {
                *b = b.checked_sub(amount).ok_or(Error::<T>::Underflow)?;
                Ok(())
            })?;
            // TODO should it update after applied onbond operation?
            TotalStakingAsset::<T>::try_mutate(|b| -> DispatchResult {
                *b = b.checked_sub(asset_amount).ok_or(Error::<T>::Underflow)?;
                Ok(())
            })?;

            Self::deposit_event(Event::Unstaked(sender, amount));
            Ok(().into())
        }

    }
}

impl<T: Config> Pallet<T> {
    pub fn account_id() -> T::AccountId {
        T::PalletId::get().into_account()
    }
}
