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
//! This pallet manages the NPoS operations for relay chain assets.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::collapsible_if)]

use frame_support::transactional;
use frame_support::{pallet_prelude::*, PalletId};
use frame_system::pallet_prelude::*;
use orml_traits::{MultiCurrency, MultiCurrencyExtended};
use primitives::{Amount, Balance, CurrencyId};
use sp_runtime::{traits::AccountIdConversion, RuntimeDebug, FixedPointNumber};
use sp_std::convert::TryInto;
use sp_std::vec::Vec;
use primitives::Rate;

pub use module::*;

mod staking;

/// Container for pending balance information
#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, Default)]
pub struct PendingBalance<Moment> {
    pub balance: Balance,
    pub timestamp: Moment,
}

#[frame_support::pallet]
pub mod module {
    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_timestamp::Config {
        /// The overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// Currency type for deposit/withdraw collateral assets to/from loans
        /// module
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
        InvalidExchangeRate,
        Overflow,
    }

    #[pallet::event]
    pub enum Event<T: Config> {}

    /// The exchange rate converts staking native token to voucher.
    #[pallet::storage]
    #[pallet::getter(fn exchange_rate)]
    pub type ExchangeRate<T: Config> = StorageValue<_, Rate, ValueQuery>;

    /// The total amount of a staking asset.
    #[pallet::storage]
    #[pallet::getter(fn total_staking)]
    pub type TotalStaking<T: Config> = StorageValue<_, Balance, ValueQuery>;

    /// The total amount of staking voucher.
    #[pallet::storage]
    #[pallet::getter(fn total_voucher)]
    pub type TotalVoucher<T: Config> = StorageValue<_, Balance, ValueQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig {}

    #[cfg(feature = "std")]
    impl Default for GenesisConfig {
        fn default() -> Self {
            GenesisConfig {}
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig {
        fn build(&self) {
            T::Currency::update_balance(
                CurrencyId::xDOT,
                &Pallet::<T>::account_id(),
                1_000_000_000_000_000_000_000_000_000,
            )
            .unwrap();
        }
    }

    #[cfg(feature = "std")]
    impl GenesisConfig {
        /// Direct implementation of `GenesisBuild::build_storage`.
        ///
        /// Kept in order not to break dependency.
        pub fn build_storage<T: Config>(&self) -> Result<sp_runtime::Storage, String> {
            <Self as frame_support::traits::GenesisBuild<T>>::build_storage(self)
        }

        /// Direct implementation of `GenesisBuild::assimilate_storage`.
        ///
        /// Kept in order not to break dependency.
        pub fn assimilate_storage<T: Config>(
            &self,
            storage: &mut sp_runtime::Storage,
        ) -> Result<(), String> {
            <Self as frame_support::traits::GenesisBuild<T>>::assimilate_storage(self, storage)
        }
    }

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
        fn on_finalize(_now: T::BlockNumber) {}
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Put assets under staking, 
        /// * the native assets will be transferred to the account owned by the pallet,
        /// * user receive voucher in return, such vocher can be further used in loans pallet. 
        ///
        /// Ensured atomic.
        #[pallet::weight(10_000)]
        #[transactional]
        pub fn stake(
            origin: OriginFor<T>,
            amount: Balance
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
            TotalStaking::<T>::mutate(|b| -> DispatchResult {
                b.checked_add(amount).ok_or(Error::<T>::Overflow)?;
                Ok(())
            });

            Ok(().into())
        }

    }
}

impl<T: Config> Pallet<T> {
    pub fn account_id() -> T::AccountId {
        T::PalletId::get().into_account()
    }
}
