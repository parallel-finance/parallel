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

use codec::{Decode, Encode};
use frame_support::traits::Contains;
use frame_system::pallet_prelude::OriginFor;
use pallet_traits::EmergencyCallFilter;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{
        dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo},
        pallet_prelude::*,
    };

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
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub (crate) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Toggled Pallet
        /// [flag]
        ToggledPallet(bool),
        /// Toggled Call
        /// [flag]
        ToggledCall(bool),
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn disabled_pallets)]
    pub type DisabledPallets<T: Config> = StorageMap<_, Blake2_128Concat, u8, bool, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn disabled_calls)]
    pub type DisabledCalls<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, u8, Blake2_128Concat, u8, bool, ValueQuery>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Toggle the shutdown flag
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn toggle_pallet(origin: OriginFor<T>, pallet_idx: u8) -> DispatchResult {
            T::ShutdownOrigin::ensure_origin(origin)?;

            let updated_flag = !<DisabledPallets<T>>::get(pallet_idx);
            <DisabledPallets<T>>::insert(pallet_idx, updated_flag);

            // Emit an event.
            Self::deposit_event(Event::ToggledPallet(updated_flag));
            Ok(())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn toggle_call(origin: OriginFor<T>, pallet_idx: u8, call_idx: u8) -> DispatchResult {
            T::ShutdownOrigin::ensure_origin(origin)?;

            let updated_flag = !<DisabledCalls<T>>::get(pallet_idx, call_idx);
            <DisabledCalls<T>>::insert(pallet_idx, call_idx, updated_flag);

            // Emit an event.
            Self::deposit_event(Event::ToggledCall(updated_flag));
            Ok(())
        }
    }
}

impl<T: Config> EmergencyCallFilter<<T as Config>::Call> for Pallet<T> {
    fn contains(call: &<T as Config>::Call) -> bool {
        let (pallet_idx, call_idx): (u8, u8) = call
            .using_encoded(|mut bytes| Decode::decode(&mut bytes))
            .expect(
                "decode input is output of Call encode; Call guaranteed to have two enums; qed",
            );

        T::Whitelist::contains(call)
            || !Self::disabled_pallets(pallet_idx) && !Self::disabled_calls(pallet_idx, call_idx)
    }
}
