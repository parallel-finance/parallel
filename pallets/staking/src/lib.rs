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
use primitives::{Amount, Balance, CurrencyId, Rate};
use sp_runtime::{
    traits::AccountIdConversion, RuntimeDebug, FixedPointNumber,
};
use orml_traits::{MultiCurrency, MultiCurrencyExtended};
pub use pallet::*;

/// Container for pending balance information
#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, Default)]
pub struct PendingBalance<Moment> {
    pub balance: Balance,
    pub timestamp: Moment,
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
    }

    #[pallet::error]
    pub enum Error<T> {
        /// ExchangeRate is invalid
        InvalidExchangeRate,
        /// Calculation overflow
        Overflow,
        /// Calculation underflow
        Underflow,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// The assets get staked successfully
        Staked(T::AccountId, Balance),
        /// The voucher get unstaked successfully
        Unstaked(T::AccountId, Balance),
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

            T::Currency::transfer(T::StakingCurrency::get(), &sender, &Self::account_id(), amount)?;
            T::Currency::deposit(T::LiquidCurrency::get(), &sender, voucher_amount)?;
            TotalVoucher::<T>::try_mutate(|b| -> DispatchResult {
                b.checked_add(voucher_amount).ok_or(Error::<T>::Overflow)?;
                Ok(())
            })?;
            TotalStakingAsset::<T>::try_mutate(|b| -> DispatchResult {
                b.checked_add(amount).ok_or(Error::<T>::Overflow)?;
                Ok(())
            })?;

            Self::deposit_event(Event::Staked(sender, amount));
            Ok(().into())
        }

        /// Unstake by exchange voucher for assets
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

            T::Currency::transfer(T::StakingCurrency::get(), &Self::account_id(), &sender, asset_amount)?;
            T::Currency::withdraw(T::LiquidCurrency::get(), &sender, amount)?;
            TotalVoucher::<T>::try_mutate(|b| -> DispatchResult {
                b.checked_sub(amount).ok_or(Error::<T>::Underflow)?;
                Ok(())
            })?;
            TotalStakingAsset::<T>::try_mutate(|b| -> DispatchResult {
                b.checked_sub(asset_amount).ok_or(Error::<T>::Underflow)?;
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
