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

//! # Route for Automatic Market Maker (AMM)
//!
//! Given a supported `route` like this `(0, USDT, KSM)`, we can get corresponding AMM or pool.

#![cfg_attr(not(feature = "std"), no_std)]

pub use crate::pallet::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

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
    use primitives::AMMAdaptor;
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

        /// Route pallet id
        #[pallet::constant]
        type RoutePalletId: Get<PalletId>;

        /// Specify all the AMMs we are routing between
        type AMMAdaptor: AMMAdaptor<Self::AccountId, CurrencyIdOf<Self>, BalanceOf<Self>>;

        /// Specify all the AMMs we are routing between
        type Routes: Get<Route<Self>>;

        /// How many routes we support at most
        #[pallet::constant]
        type MaxLengthRoute: Get<u8>;

        type Currency: MultiCurrencyExtended<Self::AccountId>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::error]
    pub enum Error<T> {
        /// Zero balance is not resonable
        ZeroBalance,
        /// Must input one route at least
        EmptyRoute,
        /// User hasn't enough tokens for transaction
        InsufficientBalance,
        /// Input wrong route that we don't support now
        NotSupportedRoute,
        /// The expiry is smaller than current block number
        TooSmallExpiry,
        /// Exceed the max length of routes we allow
        ExceedMaxLengthRoute,
        /// Input duplicated route
        DuplicatedRoute,
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
        /// According specified route order to execute which pool or AMM instance.
        ///
        /// - `origin`: the trader.
        /// - `route`: the route user inputs
        /// - `amount_in`: the amount of trading assets
        /// - `min_amount_out`:
        /// - `expiry`:
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

            // Ensure the length of routes should be >= 1 at least.
            ensure!(!route.is_empty(), Error::<T>::EmptyRoute);
            // Ensure user do not input too many routes.
            ensure!(
                route.len() <= T::MaxLengthRoute::get() as usize,
                Error::<T>::ExceedMaxLengthRoute
            );

            // Ensure balances user input is bigger than zero.
            ensure!(
                amount_in >= Zero::zero() && min_amount_out >= Zero::zero(),
                Error::<T>::ZeroBalance
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

            // Get all AMM routes we're supporting now.
            let all_routes = T::Routes::get();

            // Ensure the routes user input are valid.
            ensure!(
                route.iter().all(|r| all_routes.iter().any(|_r| r == _r)),
                Error::<T>::NotSupportedRoute
            );

            let mut amount_out: BalanceOf<T> = Zero::zero();
            for sub_route in route.iter() {
                let (id, from_currency_id, to_currency_id) = sub_route;
                let amm_instance = T::AMMAdaptor::get_amm_instance(*id);

                amount_out = amount_in;
                amount_out = amm_instance.trade(
                    &trader,
                    (*from_currency_id, *to_currency_id),
                    amount_out,
                    min_amount_out,
                )?;
            }

            Self::deposit_event(Event::TradedSuccessfully(
                trader, amount_in, route, amount_out,
            ));

            Ok(().into())
        }
    }
}
