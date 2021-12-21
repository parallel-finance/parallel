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

//! # Chainlink adapter pallet
//!
//! ## Overview
//!
//! This pallet works with chainlink pallet

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    pallet_prelude::*,
    dispatch::DispatchResult,
    traits::{
        tokens::{
            fungible::{Inspect, Mutate, Transfer},
            fungibles::{Inspect as Inspects, Mutate as Mutates, Transfer as Transfers},
            DepositConsequence, WithdrawConsequence,
        },
        Get, Time,
    },
};
use orml_oracle::DataProviderExtended;
use orml_traits::DataProvider;
use pallet_chainlink_feed::{traits::OnAnswerHandler, FeedInterface, FeedOracle,RoundData};
use primitives::*;
use sp_runtime::DispatchError;
use sp_runtime::traits::Convert;
use frame_system::pallet_prelude::*;
pub type FeedIdFor<T> = <T as pallet_chainlink_feed::Config>::FeedId;
pub type FeedValueFor<T> = <T as pallet_chainlink_feed::Config>::Value;
pub type MomentOf<T> = <<T as Config>::Time as Time>::Moment;
use sp_std::vec::Vec;
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_chainlink_feed::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// Convert feed_value type of chainlink to price type
        type Convert: Convert<FeedValueFor<Self>, Option<Price>>;
        /// Type to keep track of timestamped values
        type Time: Time;
        /// The origin which can map a `FeedId` of chainlink oracle to `CurrencyId`.
        type FeedMapOrigin: EnsureOrigin<Self::Origin>;
    }

    #[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Map feed_id to currency_id. \[feed_id, currency_id\]
		MapFeedId(FeedIdFor<T>, CurrencyId),
	}

    /// Stores the timestamp of the latest answer of each feed
    /// (feed) -> Timestamp
    #[pallet::storage]
    #[pallet::getter(fn latest_answer_timestamp)]
    pub type LatestAnswerTimestamp<T: Config> =
        StorageMap<_, Twox64Concat, FeedIdFor<T>, MomentOf<T>, ValueQuery>;

    /// Mapping from currency_id to feed_id
    ///
    /// FeedIdMapping: CurrencyId => FeedId
    #[pallet::storage]
    #[pallet::getter(fn feed_id_mapping)]
    pub type FeedIdMapping<T: Config> =
        StorageMap<_, Twox64Concat, CurrencyId, FeedIdFor<T>, OptionQuery>;

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Maps the given currency id to an existing chainlink feed.
        ///
        /// The dispatch origin of this call must be `FeedMapOrigin`.
        ///
        /// - `currency_id`: currency_id.
        /// - `feed_id`: feed_id in chainlink oracle.
        #[pallet::weight(10_000)]
        pub fn map_feed_id(
            origin: OriginFor<T>,
            currency_id: CurrencyId,
            feed_id: FeedIdFor<T>,
        ) -> DispatchResult {
            T::FeedMapOrigin::ensure_origin(origin)?;
            // if already mapped, update
            let old_feed_id = FeedIdMapping::<T>::mutate(&currency_id, |maybe_feed_id| {
                maybe_feed_id.replace(feed_id)
            });
            Self::deposit_event(Event::MapFeedId(feed_id, currency_id));
            Ok(())
        }
    }
}

impl<T: Config> Pallet<T> {
    fn get_price_from_chainlink_feed(currency_id: &CurrencyId) -> Option<Price> {
        Self::feed_id_mapping(currency_id)
            .and_then(<pallet_chainlink_feed::Pallet<T>>::feed)
            .map(|feed| feed.latest_data().answer)
            .and_then(T::Convert::convert)
    }
}

// Implement the `OnAnswerHandler` that gets called by the `chainlink pallet` on every new answer
impl<T: Config> OnAnswerHandler<T> for Pallet<T> {
    fn on_answer(feed_id: FeedIdFor<T>, _: RoundData<T::BlockNumber, FeedValueFor<T>>) {
        // keep track of the timestamp
        LatestAnswerTimestamp::<T>::insert(feed_id, T::Time::now());
    }
}

impl<T: Config> DataProvider<CurrencyId, Price> for Pallet<T> {
    fn get(key: &CurrencyId) -> Option<Price> {
        Self::get_price_from_chainlink_feed(key)
    }
}

impl<T: Config> DataProviderExtended<CurrencyId, TimeStampedPrice> for Pallet<T> 
where u64: From<<<T as pallet::Config>::Time as frame_support::traits::Time>::Moment>
{
    fn get_no_op(key: &CurrencyId) -> Option<TimeStampedPrice> {
        Self::get_price_from_chainlink_feed(key).map(|price| TimeStampedPrice {
            value: price,
            timestamp: Self::feed_id_mapping(key)
                .map(Self::latest_answer_timestamp)
                .map(sp_std::convert::TryFrom::try_from)
                .and_then(sp_std::result::Result::ok)
                .unwrap_or_default(),
        })
    }

    fn get_all_values() -> Vec<(CurrencyId, Option<TimeStampedPrice>)> {
        FeedIdMapping::<T>::iter()
            .map(|(currency_id, _)| {
                let maybe_price = Self::get_no_op(&currency_id);
                (currency_id, maybe_price)
            })
            .collect()
    }
}
