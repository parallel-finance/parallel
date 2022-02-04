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
        ensure, log,
        pallet_prelude::{DispatchResult, DispatchResultWithPostInfo},
        require_transactional,
        traits::{
            fungibles::{Inspect, Mutate, Transfer},
            Get, IsType,
        },
        transactional, BoundedVec, PalletId,
    };
    use frame_system::{ensure_signed, pallet_prelude::OriginFor};
    use primitives::{Balance, CurrencyId, AMM};
    use scale_info::prelude::collections::HashMap;
    use sp_runtime::{traits::Zero, DispatchError};
    use sp_std::{cmp::Reverse, vec::Vec};

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
        /// A more specific UnexpectedSlippage when trading exact amount out
        MaximumAmountInViolated,
        /// A more specific UnexpectedSlippage when trading exact amount in
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

    impl<T: Config<I>, I: 'static> Pallet<T, I> {
        /// Check that routes are unique and that the length > 0 and < MaxLengthRoute
        #[require_transactional]
        pub fn route_checks(route: &[AssetIdOf<T, I>]) -> DispatchResult {
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

            Ok(())
        }

        pub fn get_best_route(
            amount_in: BalanceOf<T, I>,
            token_in: AssetIdOf<T, I>,
            token_out: AssetIdOf<T, I>,
        ) -> Result<Vec<(Vec<AssetIdOf<T, I>>, BalanceOf<T, I>)>, DispatchError> {
            // get all the pool asset pairs from the AMM
            let pools = T::AMM::get_pools()?;

            let mut map: HashMap<u32, Vec<u32>> = HashMap::new();

            // build a non directed graph from pool asset pairs
            pools.into_iter().for_each(|(a, b)| {
                map.entry(a).or_insert_with(|| Vec::new()).push(b);
                map.entry(b).or_insert_with(|| Vec::new()).push(a);
            });

            // do dfs
            let mut path = Vec::new();
            let mut paths = Vec::new();

            let mut start = token_in;
            let mut end = token_out;

            let mut queue: Vec<(u32, u32, Vec<u32>)> = vec![(start, end, path)];

            // ref https://stackoverflow.com/a/5683519
            // impl rp https://gist.github.com/rust-play/3271efdcb15c632cf3fc91c04d46447a
            while !queue.is_empty() {
                let m = queue.swap_remove(0);
                start = m.0;
                end = m.1;
                path = m.2;

                path.push(start);

                if start == end {
                    paths.push(path.clone());
                }

                // cant error because we fetch pools above
                let adjacents = map.get(&start).unwrap();

                let mut difference = vec![];
                for i in adjacents {
                    if !path.contains(i) {
                        difference.push(i);
                    }
                }

                for node in difference {
                    queue.push((*node, end, path.clone()));
                }
            }

            let mut output_routes = Self::get_output_routes(amount_in, paths).unwrap();

            output_routes.sort_by_key(|k| Reverse(k.1));

            log::trace!(
                target: "router::get_best_route",
                "amount in :{:?}, token_in: {:?}, token_out: {:?}, output_routes: {:?}",
                amount_in,
                token_in,
                token_out,
                output_routes
            );

            Ok(output_routes)
        }

        ///  Returns output routes for given amount from all available routes
        pub fn get_output_routes(
            amount_in: BalanceOf<T, I>,
            routes: Vec<Vec<AssetIdOf<T, I>>>,
        ) -> Result<Vec<(Vec<AssetIdOf<T, I>>, BalanceOf<T, I>)>, DispatchError> {
            let mut output_routes = Vec::new();

            for route in routes {
                let amounts = T::AMM::get_amounts_out(amount_in, route.clone())?;
                output_routes.push((route, amounts[amounts.len() - 1]));
            }

            Ok(output_routes)
        }
    }

    #[pallet::call]
    impl<T: Config<I>, I: 'static> Pallet<T, I> {
        /// Given input amount is fixed, the output token amount is not known in advance.
        ///
        /// - `origin`: the trader.
        /// - `route`: the route user inputs
        /// - `amount_in`: the amount of trading assets
        /// - `min_amount_out`: the minimum a trader is willing to recieve
        #[pallet::weight(T::AMMRouterWeightInfo::swap_exact_tokens_for_tokens())]
        #[transactional]
        pub fn swap_exact_tokens_for_tokens(
            origin: OriginFor<T>,
            route: Vec<AssetIdOf<T, I>>,
            #[pallet::compact] amount_in: BalanceOf<T, I>,
            #[pallet::compact] min_amount_out: BalanceOf<T, I>,
        ) -> DispatchResultWithPostInfo {
            let trader = ensure_signed(origin)?;

            // do all checks on routes
            Self::route_checks(&route)?;

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
                T::AMM::swap(&trader, (route[i], route[next_index]), amounts[i])?;
            }

            Self::deposit_event(Event::Traded(
                trader,
                amounts[0],
                route,
                amounts[amounts.len() - 1],
            ));

            Ok(().into())
        }

        /// Given the output token amount is fixed, the input token amount is not known.
        ///
        /// - `origin`: the trader.
        /// - `route`: the route user inputs
        /// - `amount_out`: the amount of trading assets
        /// - `max_amount_in`: the maximum a trader is willing to input
        #[pallet::weight(T::AMMRouterWeightInfo::swap_tokens_for_exact_tokens())]
        #[transactional]
        pub fn swap_tokens_for_exact_tokens(
            origin: OriginFor<T>,
            route: Vec<AssetIdOf<T, I>>,
            #[pallet::compact] amount_out: BalanceOf<T, I>,
            #[pallet::compact] max_amount_in: BalanceOf<T, I>,
        ) -> DispatchResultWithPostInfo {
            let trader = ensure_signed(origin)?;

            // do all checks on routes
            Self::route_checks(&route)?;

            // Ensure balances user input is bigger than zero.
            ensure!(
                max_amount_in > Zero::zero() && max_amount_in >= Zero::zero(),
                Error::<T, I>::ZeroBalance
            );

            // calculate trading amounts
            let amounts = T::AMM::get_amounts_in(amount_out, route.clone())?;

            // we need to check after calc so we know how much is expected to be input
            // Ensure the trader has enough tokens for transaction.
            let from_currency_id = route[0];
            ensure!(
                <T as Config<I>>::Assets::balance(from_currency_id, &trader) > amounts[0],
                Error::<T, I>::InsufficientBalance
            );

            // make sure the required amount in does not violate our input
            ensure!(
                max_amount_in > amounts[0],
                Error::<T, I>::MaximumAmountInViolated
            );

            for i in 0..(route.len() - 1) {
                let next_index = i + 1;
                T::AMM::swap(&trader, (route[i], route[next_index]), amounts[i])?;
            }

            Self::deposit_event(Event::Traded(
                trader,
                amounts[0],
                route,
                amounts[amounts.len() - 1],
            ));

            Ok(().into())
        }
    }
}
