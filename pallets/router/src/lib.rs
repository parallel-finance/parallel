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

    pub(crate) type BalanceOf<T> =
        <<T as Config>::Currency as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;

    pub type Route = Vec<(
        // ID of the AMM to use, as specified in the `Config` trait. Setting this
        // to 0 would take the first AMM instance specified in `type AMMs`.
        u8,
        // Base asset
        CurrencyId,
        // Quote asset
        CurrencyId,
    )>;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type PalletId: Get<PalletId>;

        /// Specify all the AMMs we are routing between
        type AMMs: Get<Vec<Route>>;

        /// Trade interface
        type AMM: AMM<Self::AccountId, CurrencyId, BalanceOf<Self>>;

        type Currency: MultiCurrencyExtended<Self::AccountId>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::error]
    pub enum Error<T> {}

    #[pallet::event]
    pub enum Event<T: Config> {}

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[allow(unused_variables)]
        #[pallet::weight(10_000)]
        #[transactional]
        pub fn trade(
            origin: OriginFor<T>,
            route: Route,
            #[pallet::compact] amount_in: BalanceOf<T>,
            #[pallet::compact] min_amount_out: BalanceOf<T>,
            #[pallet::compact] expiry: BlockNumberFor<T>,
        ) -> DispatchResultWithPostInfo {
            let trader = ensure_signed(origin)?;

            let all_routers = T::AMMs::get();

            // router implementation
            // T::AMM::trade(...);

            Ok(().into())
        }
    }
}
