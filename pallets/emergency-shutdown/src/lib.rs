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
//!

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

use frame_support::traits::Contains;
use frame_system::pallet_prelude::OriginFor;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// This can be used by the runtime to define which calls should be allowed in an emergency shutdown state.
        type Whitelist: Contains<Self::Call>;

        /// The origin which can shutdown.
        type ShutdownOrigin: EnsureOrigin<Self::Origin>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub (crate) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Toggled Shutdown Flag
        ToggledShutdownFlag,
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    pub struct Default;
    impl frame_support::traits::Get<bool> for Default {
        fn get() -> bool {
            false
        }
    }

    /// Represent shutdown flag
    #[pallet::storage]
    #[pallet::getter(fn something)]
    pub type IsShutDownFlagOn<T> = StorageValue<_, bool, ValueQuery, Default>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Toggle the shutdown flag
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn toggle_shutdown_flag(origin: OriginFor<T>) -> DispatchResult {
            T::ShutdownOrigin::ensure_origin(origin)?;

            // Update storage.
            <IsShutDownFlagOn<T>>::put(true);

            // Emit an event.
            Self::deposit_event(Event::ToggledShutdownFlag);
            Ok(())
        }
    }
}
