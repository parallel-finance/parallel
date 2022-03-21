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

//! # Emergency Shut-Down pallet
//!
//! ## Overview
//! Emergency shutdown calls not in whitelist

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub use pallet::*;

use frame_support::traits::Contains;
use frame_system::pallet_prelude::OriginFor;
use sp_std::prelude::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
    use frame_support::pallet_prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// This can be used by the runtime to define which calls should be allowed in an emergency shutdown state.
        type Whitelist: Contains<<Self as Config>::Call>;

        /// The origin which can shutdown.
        type ShutdownOrigin: EnsureOrigin<Self::Origin>;

        /// The overarching call type.
        type Call: Parameter
            + Dispatchable<Origin = Self::Origin, PostInfo = PostDispatchInfo>
            + GetDispatchInfo
            + From<frame_system::Call<Self>>;

        ///  A dynamic filter which happens during runtime
        type EmergencyCallFilter: EmergencyCallFilter<Self>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub (crate) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Toggled Pallet Flag
        /// [flag]
        ToggledPalletFlag(bool),
        /// Toggled Call Flag
        /// [flag]
        ToggledCallFlag(bool),
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn disable_calls)]
    pub type DisabledCalls<T: Config> =
        StorageMap<_, Blake2_128Concat, <T as Config>::Call, bool, ValueQuery>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Toggle the shutdown flag
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn toggle_call(origin: OriginFor<T>, call: Box<<T as Config>::Call>) -> DispatchResult {
            T::ShutdownOrigin::ensure_origin(origin)?;

            let updated_flag = !<DisabledCalls<T>>::get(*call.clone());
            <DisabledCalls<T>>::insert(*call, updated_flag);

            // Emit an event.
            Self::deposit_event(Event::ToggledPalletFlag(updated_flag));
            Ok(())
        }
    }
}

pub trait EmergencyCallFilter<T: Config> {
    fn is_call_filtered(call: &<T as Config>::Call) -> bool;
}

impl<T: Config> EmergencyCallFilter<T> for Pallet<T> {
    fn is_call_filtered(call: &<T as Config>::Call) -> bool {
        if T::Whitelist::contains(call) {
            true
        } else {
            !Self::disable_calls(call)
        }
    }
}
