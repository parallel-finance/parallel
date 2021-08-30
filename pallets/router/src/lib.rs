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

//! # Router for Automatic Market Maker (AMM)
//!
//! Given a supported `router` like this `(0, USDT, KSM)`, we can get corresponding AMM or pool.

#![cfg_attr(not(feature = "std"), no_std)]

pub use crate::pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        ensure,
        pallet_prelude::DispatchResultWithPostInfo,
        traits::{Get, Hooks, IsType},
        transactional, PalletId,
    };
    use frame_system::{
        ensure_signed,
        pallet_prelude::{BlockNumberFor, OriginFor},
    };
    use orml_traits::{MultiCurrency, MultiCurrencyExtended};
    use primitives::{CurrencyId, AMM};
    use sp_runtime::traits::Zero;

    pub(crate) type BalanceOf<T> =
        <<T as Config>::Currency as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;

    pub(crate) type CurrencyIdOf<T> = <<T as Config>::Currency as MultiCurrency<
        <T as frame_system::Config>::AccountId,
    >>::CurrencyId;

    pub type Route<T> = Vec<(
        // ID of the AMM to use, as specified in the `Config` trait. Setting this
        // to 0 would take the first AMM instance specified in `type AMMs`.
        u8,
        // Base asset
        CurrencyIdOf<T>,
        // Quote asset
        CurrencyIdOf<T>,
    )>;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type PalletId: Get<PalletId>;

        /// Specify all the AMMs we are routing between
        type AMMs: Get<Route<Self>>;

        /// Trade interface
        type AMM: AMM<Self::AccountId, CurrencyId, BalanceOf<Self>>;

        type Currency: MultiCurrencyExtended<Self::AccountId>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::error]
    pub enum Error<T> {
        BalanceLow,
        EnptyRouters,
        InsufficientBalance,
        NotSupportRouter,
        TooSmallExpiry,
    }

    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId", BalanceOf<T> = "Balance")]
    #[pallet::generate_deposit(pub (crate) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Event emitted when swap is successful
        /// [sender, amount_in, route, amount_out]
        TradedSuccessfully(T::AccountId, BalanceOf<T>, Route<T>, BalanceOf<T>),
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        #[transactional]
        pub fn trade(
            origin: OriginFor<T>,
            route: Route<T>,
            #[pallet::compact] amount_in: BalanceOf<T>,
            #[pallet::compact] min_amount_out: BalanceOf<T>,
            #[pallet::compact] expiry: BlockNumberFor<T>,
        ) -> DispatchResultWithPostInfo {
            let trader = ensure_signed(origin)?;

            // Ensure the length of routers should be >= 1 at least.
            ensure!(!route.is_empty(), Error::<T>::EnptyRouters);

            // Ensure balances user input is bigger than zero.
            ensure!(
                amount_in > Zero::zero() && min_amount_out > Zero::zero(),
                Error::<T>::BalanceLow
            );

            // Ensure user iput a valid block number.
            let current_block_num = <frame_system::Pallet<T>>::block_number();
            ensure!(expiry > current_block_num, Error::<T>::TooSmallExpiry);

            // Ensure the trader has enough tokens for transaction.
            let (_, from_currency_id, _) = route[0];
            ensure!(
                T::Currency::free_balance(from_currency_id, &trader) > amount_in,
                Error::<T>::InsufficientBalance
            );

            // Get all AMM routers we're supporting now.
            let all_routers = T::AMMs::get();

            // Ensure the routers user input is valid.
            ensure!(
                route
                    .iter()
                    .all(|router| all_routers.iter().any(|r| r == router)),
                Error::<T>::NotSupportRouter
            );

            // router implementation
            // let amount_out = T::AMM::trade(&trader, (from_currency_id, to_currency_id), amount_in, min_amount_out))?;

            // Self::deposit_event(Event::TradedSuccessfully(
            //     trader,
            //     amount_in,
            //     route,
            //     amount_out,
            // ));

            Ok(().into())
        }
    }
}
