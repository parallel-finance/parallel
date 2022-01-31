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
//! Given a supported `route`, executes the indicated trades on all the available AMM(s) pool(s).

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

mod benchmarking;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub mod weights;

#[frame_support::pallet]
pub mod pallet {
    use crate::weights::WeightInfo;
    use frame_support::{
        ensure,
        pallet_prelude::DispatchResultWithPostInfo,
        traits::{
            fungibles::{Inspect, Mutate, Transfer},
            Get, IsType,
        },
        transactional, BoundedVec, PalletId,
    };
    use frame_system::{ensure_signed, pallet_prelude::OriginFor};
    use primitives::{Balance, CurrencyId, AMM};
    use sp_runtime::traits::{One, Zero};

    pub type Route<T, I> = BoundedVec<
        (
            // Base asset
            AssetIdOf<T, I>,
            // Quote asset
            AssetIdOf<T, I>,
        ),
        <T as Config<I>>::MaxLengthRoute,
    >;

    pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
    pub(crate) type AssetIdOf<T, I = ()> =
        <<T as Config<I>>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;
    pub(crate) type BalanceOf<T, I = ()> =
        <<T as Config<I>>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::config]
    pub trait Config<I: 'static = ()>: frame_system::Config {
        type Event: From<Event<Self, I>> + IsType<<Self as frame_system::Config>::Event>;

        /// Router pallet id
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// Specify all the AMMs we are routing between
        type AMM: AMM<AccountIdOf<Self>, AssetIdOf<Self, I>, BalanceOf<Self, I>>;

        /// Weight information for extrinsics in this pallet.
        type AMMRouterWeightInfo: WeightInfo;

        /// How many routes we support at most
        #[pallet::constant]
        type MaxLengthRoute: Get<u32>;

        /// Currency type for deposit/withdraw assets to/from amm route
        /// module
        type Assets: Transfer<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
            + Inspect<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
            + Mutate<Self::AccountId, AssetId = CurrencyId, Balance = Balance>;
    }

    #[pallet::pallet]
    pub struct Pallet<T, I = ()>(_);

    #[pallet::error]
    pub enum Error<T, I = ()> {
        /// Input balance must not be zero
        ZeroBalance,
        /// Must input one route at least
        EmptyRoute,
        /// User hasn't enough tokens for transaction
        InsufficientBalance,
        /// Exceed the max length of routes we allow
        ExceedMaxLengthRoute,
        /// Input duplicated route
        DuplicatedRoute,

        // /// We received less coins than the minimum amount specified
        // UnexpectedSlippage,
        ///
        ///
        MaximumAmountInViolated,
        ///
        MinimumAmountOutViolated,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub (crate) fn deposit_event)]
    pub enum Event<T: Config<I>, I: 'static = ()> {
        /// Event emitted when swap is successful
        /// [sender, amount_in, route, amount_out]
        Traded(
            T::AccountId,
            BalanceOf<T, I>,
            Vec<AssetIdOf<T, I>>,
            BalanceOf<T, I>,
        ),
    }

    #[pallet::call]
    impl<T: Config<I>, I: 'static> Pallet<T, I> {
        // fn sanity_checks(
        //     origin: OriginFor<T>,
        //     route: Vec<AssetIdOf<T, I>>,
        // ) -> Result<(), DispatchError> {
        //     // Ensure the length of routes should be >= 1 at least.
        //     ensure!(!route.is_empty(), Error::<T, I>::EmptyRoute);

        //     // Ensure user do not input too many routes.
        //     ensure!(
        //         route.len() <= T::MaxLengthRoute::get() as usize,
        //         Error::<T, I>::ExceedMaxLengthRoute
        //     );

        //     // check for duplicates with O(n^2) complexity
        //     // only good for short routes and we have a cap checked above
        //     let contains_duplicate = (1..route.len()).any(|i| route[i..].contains(&route[i - 1]));

        //     // Ensure user doesn't input duplicated routes (a cycle in the graph)
        //     ensure!(!contains_duplicate, Error::<T, I>::DuplicatedRoute);

        //     Ok(())
        // }

        /// According specified route order to execute which pool or AMM instance.
        ///
        /// - `origin`: the trader.
        /// - `route`: the route user inputs
        /// - `amount_in`: the amount of trading assets
        /// - `min_amount_out`:

        // given amount is fixed, the output token amount is not known in advance
        #[pallet::weight(T::AMMRouterWeightInfo::swap_exact_tokens_for_tokens())]
        #[transactional]
        pub fn swap_exact_tokens_for_tokens(
            origin: OriginFor<T>,
            route: Vec<AssetIdOf<T, I>>,
            #[pallet::compact] amount_in: BalanceOf<T, I>,
            #[pallet::compact] min_amount_out: BalanceOf<T, I>,
        ) -> DispatchResultWithPostInfo {
            let trader = ensure_signed(origin)?;

            // Self::sanity_checks(origin, route)?;

            // Ensure the length of routes should be >= 1 at least.
            ensure!(!route.is_empty(), Error::<T, I>::EmptyRoute);

            // Ensure user do not input too many routes.
            ensure!(
                route.len() <= T::MaxLengthRoute::get() as usize,
                Error::<T, I>::ExceedMaxLengthRoute
            );

            // check for duplicates with O(n^2) complexity
            // only good for short routes and we have a cap checked above
            let contains_duplicate = (1..route.len()).any(|i| route[i..].contains(&route[i - 1]));

            // Ensure user doesn't input duplicated routes (a cycle in the graph)
            ensure!(!contains_duplicate, Error::<T, I>::DuplicatedRoute);

            // Ensure balances user input is bigger than zero.
            ensure!(
                amount_in > Zero::zero() && min_amount_out >= Zero::zero(),
                Error::<T, I>::ZeroBalance
            );

            // Ensure the trader has enough tokens for transaction.
            let from_currency_id = route[0];
            ensure!(
                <T as Config<I>>::Assets::balance(from_currency_id, &trader) > amount_in,
                Error::<T, I>::InsufficientBalance
            );

            let amounts = T::AMM::get_amounts_out(amount_in, route.clone())?;

            // make sure the required amount in does not violate our input
            ensure!(
                amounts[amounts.len() - 1] > min_amount_out,
                Error::<T, I>::MinimumAmountOutViolated
            );

            for i in 0..(route.len() - 1) {
                let next_index = i + 1;

                println!("{} -> {}", route[i], route[next_index]);

                T::AMM::swap(
                    &trader,
                    (route[i], route[next_index]),
                    amounts[i],
                    min_amount_out, // not used right now...
                )?;
            }

            Self::deposit_event(Event::Traded(trader, amount_in, route, min_amount_out));

            Ok(().into())
        }

        /// According specified route order to execute which pool or AMM instance.
        ///
        /// - `origin`: the trader.
        /// - `route`: the route user inputs
        /// - `amount_out`: the amount of trading assets
        /// - `max_amount_in`:

        // the output token amount is fixed, but the input token amount is not known
        #[pallet::weight(T::AMMRouterWeightInfo::swap_tokens_for_exact_tokens())]
        #[transactional]
        pub fn swap_tokens_for_exact_tokens(
            origin: OriginFor<T>,
            route: Vec<AssetIdOf<T, I>>,
            #[pallet::compact] minimum_out: BalanceOf<T, I>,
            #[pallet::compact] maximum_amount_in: BalanceOf<T, I>,
        ) -> DispatchResultWithPostInfo {
            let trader = ensure_signed(origin)?;

            // Ensure the length of routes should be >= 1 at least.
            ensure!(!route.is_empty(), Error::<T, I>::EmptyRoute);

            // Ensure user do not input too many routes.
            ensure!(
                route.len() <= T::MaxLengthRoute::get() as usize,
                Error::<T, I>::ExceedMaxLengthRoute
            );

            // check for duplicates with O(n^2) complexity
            // only good for short routes and we have a cap checked above
            let contains_duplicate = (1..route.len()).any(|i| route[i..].contains(&route[i - 1]));

            // Ensure user doesn't input duplicated routes (a cycle in the graph)
            ensure!(!contains_duplicate, Error::<T, I>::DuplicatedRoute);

            // Ensure balances user input is bigger than zero.
            ensure!(
                maximum_amount_in > Zero::zero(), // && min_amount_out >= Zero::zero(),
                Error::<T, I>::ZeroBalance
            );

            // calculate trading amounts
            let amounts = T::AMM::get_amounts_in(minimum_out, route.clone())?;

            // we need to check after calc so we know how much is expected to be input
            // Ensure the trader has enough tokens for transaction.
            let from_currency_id = route[0];
            ensure!(
                <T as Config<I>>::Assets::balance(from_currency_id, &trader) > amounts[0],
                Error::<T, I>::InsufficientBalance
            );

            // make sure the required amount in does not violate our input
            ensure!(
                maximum_amount_in > amounts[0],
                Error::<T, I>::MaximumAmountInViolated
            );

            for i in 0..(route.len() - 1) {
                let next_index = i + 1;
                println!("{} -> {}", route[i], route[next_index]);

                T::AMM::swap(
                    &trader,
                    (route[i], route[next_index]),
                    amounts[i],
                    maximum_amount_in, // not used right now...
                )?;
            }

            Self::deposit_event(Event::Traded(trader, minimum_out, route, maximum_amount_in));

            Ok(().into())
        }
    }
}
