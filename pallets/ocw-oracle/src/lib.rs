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

//! # Oracle Module
//!
//! A multi-source, multi-node decentralized oracle solution.
//!
//! ## Overview
//!
//! The Oracle module provides functionality for fetching HTTP data from multi-datasource, like exchange.
//! it's totally decentralized and implement by Substrate off-chain worker(AKA `OCW`).
//!
//! ### Terminology
//!
//! * **Off-Chain Worker**: Data fetching task will be trigger by `offchain_worker` function, every approved node
//!   will fetch price from different data source, then send it on-chain.
//! * **HTTP**: Execute the URL request off-chain, parse remote data into general data structure and wrapped by OCW
//! * **On-Chain Storage**: Storage off-chain data.
//! * **Data Source**: Include exchanges, DEX and any datasource, could by improved under governance.
//! * **Aggregation Strategy**: Combined on-chain data into one single price.
//!
//! ### Goals
//!
//! The Oracle in Parallel is designed to fetch price and provide reliable price to system:
//!
//! ## Interface
//!
//! ### Public Functions
//!
//! * `get_price`: Get designated currency price that already combined according to aggregation strategy.
//!
//! ### Permissioned Functions
//!
//! * `emergency_feed`: Creates a new asset class without taking any deposit.
//! * `change_url`: Destroys an asset class.
//! * `change_members`: Destroys an asset class.
//! * `change_aggregation_strategy`: Destroys an asset class.
//! * `change_url`: Destroys an asset class.
//!
//! ### Privileged Functions
//! * `on_finalize`: Execute aggregation strategy, combined on-chain data into single price.
//! * `offchain_worker`: Start fetching price.
//! * `submit_price`: Wrapped and submit price to on-chain.
//! * `append_price`: Storage price data on-chain.
//! * `update_round`: Update price data each round.
//! * `update_price`: Updata price data storage.
//! * `offchain_signed_tx`: Send transaction, the payload is price data.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use core::fmt;
use frame_support::{log, pallet_prelude::*, traits::Time};
use frame_system::offchain::{AppCrypto, CreateSignedTransaction, SendSignedTransaction, Signer};
use frame_system::pallet_prelude::*;
#[allow(unused_imports)]
use num_traits::float::FloatCore;
pub use pallet::*;
use primitives::*;
use serde::{Deserialize, Deserializer};
use sp_core::crypto::KeyTypeId;
use sp_runtime::{
    offchain as rt_offchain,
    offchain::storage_lock::{BlockAndTime, StorageLock},
    DispatchResult, RuntimeDebug,
};
use sp_std::{
    collections::{btree_set::BTreeSet, vec_deque::VecDeque},
    convert::TryInto,
    prelude::*,
    str,
};

mod aggregation_strategy;
mod data_source;
mod http;
#[cfg(test)]
mod tests;
mod util;
use self::aggregation_strategy::*;
use self::data_source::*;
use self::util::*;

/// The type to sign and send transactions.
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"vocw");
/// The price storage's length. 
pub const NUM_VEC_LEN: usize = 10;
/// HTTP header
pub const HTTP_HEADER_USER_AGENT: &str = "Parallel";
/// HTTP timeout, in milli-seconds
pub const FETCH_TIMEOUT_PERIOD: u64 = 2000;
/// HTTP request interval every batch, avoid refusing by remote server, in milli-seconds
pub const HTTP_INTERVAL: u64 = 500;
/// The OCW lock timeout expiration, in milli-seconds
pub const LOCK_TIMEOUT_EXPIRATION: u64 = FETCH_TIMEOUT_PERIOD * 5 + 1000;
/// The OCW lock block expiration, in block number
pub const LOCK_BLOCK_EXPIRATION: u32 = 5;
/// The limit when update round, also as the upper limit combine price, in block number
pub const UPDATE_ROUND_INDEX_LIMIT: u32 = 3;
/// The limit feed price emergency, in block number
pub const EMERGENCY_PRICE_FEED_LIMIT: u32 = 7;
/// The proposers threshold at every round
pub const MINIMUM_PROPOSERS: u32 = 1;

pub type RoundIndex<BlockNumber> = BlockNumber;

/// Price detail structure
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct PriceDetailOf<BlockNumber> {
    /// The round index of each batch, define by OCW
    index: RoundIndex<BlockNumber>,
    /// The blocknumber when submit price succeed.
    blocknumber: BlockNumber,
    /// The price
    price: Price,
    /// The timestamp
    timestamp: Timestamp,
}
/// Storage feed accounts array at each round
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct Round<T: Config> {
    /// The round index of each batch, define by OCW
    index: RoundIndex<T::BlockNumber>,
    /// The collection of account that submit price
    provider: Vec<T::AccountId>,
    /// Reveal if this round's prices have been cimbined
    combined: bool,
    /// Reveal ths previous blocknumber that price has been combined
    last_combined: T::BlockNumber,
}

/// The data that fetch from remote data source
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct TickerPayloadDetail {
    /// Currency id
    symbol: CurrencyId,
    /// Data source type
    data_source_enum: DataSourceEnum,
    /// Price data
    price: Price,
    /// Timestamp
    timestamp: Timestamp,
}

/// Payload that will be sent from OCW to on-chain method
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct Payload<BlockNumber> {
    /// The round index of each batch, define by OCW
    index: RoundIndex<BlockNumber>,
    /// The data fetch by node
    list: Vec<TickerPayloadDetail>,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    pub mod crypto {
        use super::KEY_TYPE;
        use sp_core::sr25519::Signature as Sr25519Signature;
        use sp_runtime::{
            app_crypto::{app_crypto, sr25519},
            traits::Verify,
            MultiSignature, MultiSigner,
        };
        app_crypto!(sr25519, KEY_TYPE);

        pub struct TestAuthId;
        // implemented for ocw-runtime
        impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for TestAuthId {
            type RuntimeAppPublic = Public;
            type GenericSignature = sp_core::sr25519::Signature;
            type GenericPublic = sp_core::sr25519::Public;
        }

        // implemented for mock runtime in test
        impl
            frame_system::offchain::AppCrypto<
                <Sr25519Signature as Verify>::Signer,
                Sr25519Signature,
            > for TestAuthId
        {
            type RuntimeAppPublic = Public;
            type GenericSignature = sp_core::sr25519::Signature;
            type GenericPublic = sp_core::sr25519::Public;
        }
    }

    /// This pallet's configuration trait
    #[pallet::config]
    pub trait Config: frame_system::Config + CreateSignedTransaction<Call<Self>> {
        /// The identifier type for an offchain worker.
        type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
        /// The overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// The overarching dispatch call type.
        type Call: From<Call<Self>>;
        /// Time provider
        type Time: Time;
        /// the precision of the price
        #[pallet::constant]
        type PricePrecision: Get<u8>;
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        //TODO we may need to calculate the gas cost in "on_initialize" method.
        fn on_finalize(blocknumber: T::BlockNumber) {
            for currency_id in Self::ocw_oracle_currencies().iter() {
                if let Some(round) = OcwOracleRound::<T>::get(currency_id) {
                    let round_index: RoundIndex<T::BlockNumber> = round.index;
                    let provider: Vec<T::AccountId> = round.provider;
                    let mut combined: bool = round.combined;
                    let mut last_combined: T::BlockNumber = round.last_combined;

                    let block_interval = blocknumber - round_index;
                    let update =
                        <T as frame_system::Config>::BlockNumber::from(UPDATE_ROUND_INDEX_LIMIT);
                    let expired =
                        <T as frame_system::Config>::BlockNumber::from(EMERGENCY_PRICE_FEED_LIMIT);

                    if block_interval <= update && !combined {
                        //in case every time only one node submit price.
                        if blocknumber - last_combined >= expired {
                            OcwOracleAggregationStrategy::<T>::put(Some(
                                AggregationStrategyEnum::EMERGENCY,
                            ));
                            log::error!(
                                "{:?} is expired, last combined is {:?}, need emergence feed!",
                                currency_id,
                                last_combined
                            );
                        } else if let Some(aggregate_strategy) =
                            Self::ocw_oracle_aggregation_strategy()
                        {
                            if provider.len() < MINIMUM_PROPOSERS as usize {
                                log::warn!(
                                    "minimum proposers is {:?} but now have {:?}",
                                    MINIMUM_PROPOSERS,
                                    provider.len()
                                );
                                continue;
                            }
                            match aggregate_price::<T>(
                                aggregate_strategy,
                                &round_index,
                                &provider,
                                &currency_id.clone(),
                            ) {
                                Ok(price_detail) => {
                                    Prices::<T>::insert(currency_id, Some(price_detail));
                                    combined = true;
                                    last_combined = blocknumber;
                                    OcwOracleRound::<T>::insert(
                                        currency_id,
                                        Some(Round {
                                            index: round_index,
                                            provider,
                                            combined,
                                            last_combined,
                                        }),
                                    );
                                    log::info!(
                                        "{:?} is combined,price is {:?}",
                                        currency_id,
                                        price_detail
                                    );
                                }
                                Err(e) => log::error!(
                                    "error {:?} occurs when combined {:?} price!",
                                    e,
                                    currency_id
                                ),
                            }
                        } else {
                            OcwOracleAggregationStrategy::<T>::put(Some(
                                AggregationStrategyEnum::EMERGENCY,
                            ));
                            log::error!(
                                "{:?} aggregate strategy is empty, need emergence feed!",
                                currency_id
                            );
                        }
                    } else if block_interval >= expired {
                        OcwOracleAggregationStrategy::<T>::put(Some(
                            AggregationStrategyEnum::EMERGENCY,
                        ));
                        log::error!("{:?} is expired, need emergence feed!", currency_id);
                    } else {
                        log::warn!(
                            "current blocknumber is {:?}, last combined round is {:?}",
                            blocknumber,
                            last_combined
                        );
                    }
                } else {
                    if let Some(_) = Self::get_price(currency_id) {
                        OcwOracleAggregationStrategy::<T>::put(Some(
                            AggregationStrategyEnum::EMERGENCY,
                        ));
                        log::error!("{:?} is absence, need emergence feed!", currency_id);
                    } else {
                        log::info!("Initial ocw oracle, fetch {:?}!", currency_id);
                    }
                }
            }
        }

        fn offchain_worker(blocknumber: T::BlockNumber) {
            // completely, we should iter from local key container, check if any local key approved by the on-chain "Members",
            // but actually, any node that not approved but insist submiting price on-chain, will be refused.
            // so let's make a simple judgment here.
            let members = Self::members();
            let can_sign = Signer::<T, T::AuthorityId>::any_account().can_sign();
            if members.len() == 0 || !can_sign {
                log::error!("approved members {:?}, signer {:?}", members, can_sign);
                return;
            }
            match Self::fetch_ticker_price() {
                Ok(res) => {
                    let _ = Self::offchain_signed_tx(res, blocknumber);
                }
                Err(e) => {
                    log::error!("offchain_worker error: {:?}", e);
                }
            }
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub (crate) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Submit price by OCW 
        OffchainInvoke(Option<T::AccountId>, RoundIndex<T::BlockNumber>),
        /// Price was succeed append
        AppendPrice(Option<T::AccountId>, Option<PriceDetailOf<T::BlockNumber>>),
        /// Start emergency feed
        EmergencyFeed(Option<T::AccountId>, CurrencyId, Option<PriceDetail>),
        /// Change URL
        ChangeUrl(CurrencyId, DataSourceEnum, Vec<u8>),
        /// Change approved members
        ChangeMembers(Vec<T::AccountId>),
        /// Change price aggregation strategy
        ChangeAggregationStrategy(AggregationStrategyEnum),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Error returned when not sure which ocw function to executed
        UnknownOffchainMux,
        /// Error returned when making signed transactions in off-chain worker
        NoLocalAcctForSigning,
        /// Error when sign tx in off chain worker
        OffchainSignedTxError,
        /// Error returned when making unsigned transactions in off-chain worker
        OffchainUnsignedTxError,
        /// Error returned when making unsigned transactions with signed payloads in off-chain worker
        OffchainUnsignedTxSignedPayloadError,
        /// Error returned when fetching info
        HttpFetchingError,
        /// Error when fetching currency empty
        FetchingCurrencyEmptyError,
        /// Error when fetching binance price
        HttpFetchingBinanceError,
        /// Error when fetching coinbase price
        HttpFetchingCoinbaseError,
        /// Error when fetching coincap price
        HttpFetchingCoincapError,
        /// Error when previous http is waiting
        AcquireStorageLockError,
        /// Error when convert price
        ConvertToStringError,
        /// Error when convert price
        ParsingToF64Error,
        /// Error when prase url from bytes
        ParseUrlError,
        /// Method not implement
        NotImplement,
        /// Not allowed to feed price
        NoPermission,
        /// Error when parse timestamp
        ParseTimestampError,
        /// Error when emergency feed
        EmergencyFeedFail,
    }

    /// Detail of combined price, get by currency id.
    #[pallet::storage]
    #[pallet::getter(fn get_price)]
    pub type Prices<T: Config> =
        StorageMap<_, Twox64Concat, CurrencyId, Option<PriceDetail>, ValueQuery>;

    /// Detail of request url
    /// CurrencyId -> DataSourceEnum -> Url
    #[pallet::storage]
    #[pallet::getter(fn ocw_oracle_request_url)]
    pub type OcwOracleRequestUrl<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        CurrencyId,
        Twox64Concat,
        DataSourceEnum,
        Option<Url>,
        ValueQuery,
    >;

    /// Detail of price feed round
    /// CurrencyId -> Round
    #[pallet::storage]
    #[pallet::getter(fn ocw_oracle_round)]
    pub type OcwOracleRound<T: Config> =
        StorageMap<_, Twox64Concat, CurrencyId, Option<Round<T>>, ValueQuery>;

    /// Detail of submit prices
    #[pallet::storage]
    #[pallet::getter(fn ocw_oracle_price)]
    pub type OcwOraclePrice<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        T::AccountId,
        Twox64Concat,
        (DataSourceEnum, CurrencyId),
        Option<VecDeque<PriceDetailOf<T::BlockNumber>>>,
        ValueQuery,
    >;

    /// Detail of currencies that will fetch price by OCW
    #[pallet::storage]
    #[pallet::getter(fn ocw_oracle_currencies)]
    pub type OcwOracleCurrencies<T: Config> = StorageValue<_, Vec<CurrencyId>, ValueQuery>;

    /// Detail of data source that will fetch prices from.
    #[pallet::storage]
    #[pallet::getter(fn ocw_oracle_data_source)]
    pub type OcwOracleDataSource<T: Config> = StorageValue<_, Vec<DataSourceEnum>, ValueQuery>;

    /// The aggregation strategy
    #[pallet::storage]
    #[pallet::getter(fn ocw_oracle_aggregation_strategy)]
    pub type OcwOracleAggregationStrategy<T: Config> =
        StorageValue<_, Option<AggregationStrategyEnum>, ValueQuery>;

    /// Collection of members that approved submit price
    #[pallet::storage]
    #[pallet::getter(fn members)]
    pub type Members<T: Config> = StorageValue<_, BTreeSet<T::AccountId>, ValueQuery>;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        //TODO calculate the weight here
        #[pallet::weight(1_000_000)]
        pub(crate) fn submit_price(
            origin: OriginFor<T>,
            payload: Payload<T::BlockNumber>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            if !Self::members().contains(&who) {
                log::error!("submit_price error: {:?}", Error::<T>::NoPermission);
                return Err(Error::<T>::NoPermission.into());
            }
            Self::append_price(who, payload);
            Ok(().into())
        }

        /// Energency feed method
        #[pallet::weight(10_000)]
        pub fn emergency_feed(
            origin: OriginFor<T>,
            currency_id: CurrencyId,
            price: Price,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(Self::members().contains(&who), Error::<T>::NoPermission);
            if Self::ocw_oracle_aggregation_strategy() == Some(AggregationStrategyEnum::EMERGENCY) {
                let now = T::Time::now();
                let timestamp: Timestamp =
                    now.try_into().or(Err(Error::<T>::ParseTimestampError))?;
                Prices::<T>::insert(currency_id, Some((price, timestamp)));
                Self::deposit_event(Event::<T>::EmergencyFeed(Some(who), currency_id, Some((price, timestamp))));
            } else {
                return Err(Error::<T>::EmergencyFeedFail.into());
            }
            Ok(().into())
        }

        /// Change URL method
        #[pallet::weight(10_000)]
        pub fn change_url(
            origin: OriginFor<T>,
            currency_id: CurrencyId,
            data_source_enum: DataSourceEnum,
            url: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let _ = ensure_root(origin)?;
            OcwOracleRequestUrl::<T>::try_mutate(
                currency_id,
                data_source_enum,
                |old_url| -> DispatchResult {
                    *old_url = Some(url.clone());
                    Ok(())
                },
            )?;
            Self::deposit_event(Event::<T>::ChangeUrl(currency_id, data_source_enum, url));
            Ok(().into())
        }

        /// Change members method
        #[pallet::weight(10_000)]
        pub fn change_members(
            origin: OriginFor<T>,
            members: Vec<T::AccountId>,
        ) -> DispatchResultWithPostInfo {
            let _ = ensure_root(origin)?;
            let mut set = BTreeSet::new();
            members.iter().for_each(|account| {
                set.insert(account.clone());
            });
            Members::<T>::put(set);
            Self::deposit_event(Event::<T>::ChangeMembers(members));
            Ok(().into())
        }

        /// Change aggregation strategy method
        #[pallet::weight(10_000)]
        pub fn change_aggregation_strategy(
            origin: OriginFor<T>,
            strategy: AggregationStrategyEnum,
        ) -> DispatchResultWithPostInfo {
            let _ = ensure_root(origin)?;
            let current_is_emergency =
                Self::ocw_oracle_aggregation_strategy() == Some(AggregationStrategyEnum::EMERGENCY);
            let input_is_not_emergency = strategy != AggregationStrategyEnum::EMERGENCY;
            if current_is_emergency && input_is_not_emergency {
                let last_combined = Self::block_number();
                for currency_id in Self::ocw_oracle_currencies().iter() {
                    if let Some(round) = OcwOracleRound::<T>::get(currency_id) {
                        OcwOracleRound::<T>::insert(
                            currency_id,
                            Some(Round {
                                last_combined,
                                ..round
                            }),
                        );
                    }
                }
            }
            OcwOracleAggregationStrategy::<T>::put(Some(strategy));
            Self::deposit_event(Event::<T>::ChangeAggregationStrategy(strategy));
            Ok(().into())
        }

        //FIXME : Just easily for test, remove when production.
        // move this function to test.rs
        #[pallet::weight(10_000)]
        pub fn insert_initial_data(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let _ = ensure_root(origin)?;
            let aggregation_strategy = AggregationStrategyEnum::MEDIAN;
            let currencies = vec![
                CurrencyId::DOT,
                CurrencyId::KSM,
                CurrencyId::BTC,
                CurrencyId::USDT,
                CurrencyId::xDOT,
            ];
            let data_source_type = vec![
                DataSourceEnum::BINANCE,
                DataSourceEnum::COINBASE,
                DataSourceEnum::COINCAP,
            ];
            let ocw_oracle_request_url = vec![
                (
                    CurrencyId::DOT,
                    DataSourceEnum::BINANCE,
                    "https://api.binance.com/api/v3/ticker/price?symbol=DOTUSDT"
                        .as_bytes()
                        .to_vec(),
                ),
                (
                    CurrencyId::KSM,
                    DataSourceEnum::BINANCE,
                    "https://api.binance.com/api/v3/ticker/price?symbol=KSMUSDT"
                        .as_bytes()
                        .to_vec(),
                ),
                (
                    CurrencyId::BTC,
                    DataSourceEnum::BINANCE,
                    "https://api.binance.com/api/v3/ticker/price?symbol=BTCUSDT"
                        .as_bytes()
                        .to_vec(),
                ),
                (
                    CurrencyId::xDOT,
                    DataSourceEnum::BINANCE,
                    "https://api.binance.com/api/v3/ticker/price?symbol=DOTUSDT"
                        .as_bytes()
                        .to_vec(),
                ),
                (
                    CurrencyId::BTC,
                    DataSourceEnum::COINBASE,
                    "https://api.pro.coinbase.com/products/btc-usd/ticker"
                        .as_bytes()
                        .to_vec(),
                ),
                (
                    CurrencyId::DOT,
                    DataSourceEnum::COINCAP,
                    "https://api.coincap.io/v2/assets/polkadot"
                        .as_bytes()
                        .to_vec(),
                ),
                (
                    CurrencyId::KSM,
                    DataSourceEnum::COINCAP,
                    "https://api.coincap.io/v2/assets/kusama"
                        .as_bytes()
                        .to_vec(),
                ),
                (
                    CurrencyId::BTC,
                    DataSourceEnum::COINCAP,
                    "https://api.coincap.io/v2/assets/bitcoin"
                        .as_bytes()
                        .to_vec(),
                ),
                (
                    CurrencyId::USDT,
                    DataSourceEnum::COINCAP,
                    "https://api.coincap.io/v2/assets/tether"
                        .as_bytes()
                        .to_vec(),
                ),
                (
                    CurrencyId::xDOT,
                    DataSourceEnum::COINCAP,
                    "https://api.coincap.io/v2/assets/polkadot"
                        .as_bytes()
                        .to_vec(),
                ),
            ];
            OcwOracleAggregationStrategy::<T>::put(Some(aggregation_strategy.clone()));
            OcwOracleCurrencies::<T>::put(currencies.clone());
            OcwOracleDataSource::<T>::put(data_source_type.clone());
            ocw_oracle_request_url
                .iter()
                .for_each(|(currency_id, data_source_type, url)| {
                    OcwOracleRequestUrl::<T>::insert(currency_id, data_source_type, Some(url));
                });
            Ok(().into())
        }
    }
}

impl<T: Config> Pallet<T> {
    /// Update round and price.
    /// Append a new number to the tail of the price list, removing an element from the head if reaching the bounded length.  
    fn append_price(who: T::AccountId, payload: Payload<T::BlockNumber>) {
        let round_blocknumber = payload.index;
        if round_blocknumber >= Self::block_number() {
            log::error!("guile node: {:?}", who);
            return;
        }
        let list = payload.list;
        // 1 get submit currencies
        let mut submit_currencies: Vec<CurrencyId> = vec![];
        list.iter().for_each(|ticker| {
            if !submit_currencies.contains(&ticker.symbol) {
                submit_currencies.push(ticker.symbol);
            }
        });
        // 2 update round info
        let update_currency_set = Self::update_round(&who, round_blocknumber, submit_currencies);
        if update_currency_set.len() == 0 {
            log::error!("update_currency_set empty!");
            return;
        }
        // 3 update price info
        Self::update_price(&who, round_blocknumber, update_currency_set, list);

        Self::deposit_event(Event::<T>::OffchainInvoke(Some(who), round_blocknumber));
    }

    /// update the round
    fn update_round(
        who: &T::AccountId,
        round_blocknumber: T::BlockNumber,
        submit_currencies: Vec<CurrencyId>,
    ) -> BTreeSet<CurrencyId> {
        let mut rst: BTreeSet<CurrencyId> = BTreeSet::new();
        for currency_id in submit_currencies.iter() {
            if let Some(round) = OcwOracleRound::<T>::get(currency_id) {
                let mut round_index: RoundIndex<T::BlockNumber> = round.index;
                if round_blocknumber < round_index {
                    log::warn!(
                        "submit round {:?} is behind current round {:?}",
                        round_blocknumber,
                        round_index
                    );
                    continue;
                }
                let mut provider: Vec<T::AccountId> = round.provider;
                if round_blocknumber == round_index {
                    if provider.contains(who) {
                        log::warn!(
                            "account {:?} already submit at current round {:?}",
                            who,
                            round_index
                        );
                        continue;
                    }
                    provider.push(who.clone());
                } else if round_blocknumber > round_index {
                    let expired = Self::block_number() - round_index
                        > <T as frame_system::Config>::BlockNumber::from(UPDATE_ROUND_INDEX_LIMIT);
                    if round.combined || expired {
                        provider.clear();
                        round_index = round_blocknumber;
                        provider.push(who.clone());
                    } else {
                        log::warn!(
                            "submit round {:?} is beyond current round {:?}",
                            round_blocknumber,
                            round_index
                        );
                        continue;
                    }
                }
                OcwOracleRound::<T>::insert(
                    currency_id,
                    Some(Round {
                        index: round_index,
                        provider,
                        combined: false,
                        last_combined: round.last_combined,
                    }),
                );
            } else {
                OcwOracleRound::<T>::insert(
                    currency_id,
                    Some(Round {
                        index: round_blocknumber,
                        provider: vec![who.clone()],
                        combined: false,
                        last_combined: round_blocknumber,
                    }),
                );
            }
            rst.insert(currency_id.clone());
        }
        rst
    }

    /// update the price
    fn update_price(
        who: &T::AccountId,
        round_blocknumber: T::BlockNumber,
        update_currency_set: BTreeSet<CurrencyId>,
        list: Vec<TickerPayloadDetail>,
    ) {
        for ticker_payload in list.iter() {
            let currency_id = ticker_payload.symbol;
            if !update_currency_set.contains(&currency_id) {
                log::warn!("currency {:?} price is not allowed to update!", currency_id);
                continue;
            }
            let data_source_enum = ticker_payload.data_source_enum;
            let price = ticker_payload.price;
            let timestamp = ticker_payload.timestamp;
            match OcwOraclePrice::<T>::try_mutate(
                who,
                (data_source_enum, currency_id),
                |option_price_vec| -> DispatchResult {
                    let mut pv = VecDeque::new();
                    let price = PriceDetailOf {
                        index: round_blocknumber,
                        blocknumber: Self::block_number(),
                        price,
                        timestamp,
                    };
                    if let Some(price_vec) = option_price_vec {
                        let mut last_price = None;
                        if let Some(p) = price_vec.back() {
                            last_price = Some(p.clone())
                        }
                        if price_vec.len() == NUM_VEC_LEN {
                            let _ = price_vec.pop_front();
                        }
                        price_vec.push_back(price);
                        pv.append(price_vec);
                        Self::deposit_event(Event::<T>::AppendPrice(Some(who.clone()), last_price));
                    } else {
                        pv.push_back(price);
                        Self::deposit_event(Event::<T>::AppendPrice(Some(who.clone()), None));
                    }
                    *option_price_vec = Some(pv);
                    Ok(())
                },
            ) {
                Ok(_) => continue,
                Err(e) => log::error!(
                    "error occurs, account {:?} failed update price on {:?}, error msg: {:?}",
                    who,
                    (data_source_enum, currency_id),
                    e
                ),
            }
        }
    }

    fn block_number() -> T::BlockNumber {
        <frame_system::Pallet<T>>::block_number()
    }

    //TODO should remove the default key in node/service.rs, please refer to xxxx
    fn offchain_signed_tx(
        payload_list: Vec<TickerPayloadDetail>,
        blocknumber: T::BlockNumber,
    ) -> Result<(), Error<T>> {
        let signer = Signer::<T, T::AuthorityId>::any_account();
        let payload = Payload {
            index: blocknumber,
            list: payload_list.clone(),
        };
        if let Some((_, res)) =
            signer.send_signed_transaction(|_acct| Call::submit_price(payload.clone()))
        {
            return res.map_err(|_| {
                log::error!("Failed in offchain_signed_tx");
                <Error<T>>::OffchainUnsignedTxSignedPayloadError
            });
        }
        // The case of `None`: no account is available for sending
        log::error!("No local account available");
        Err(<Error<T>>::NoLocalAcctForSigning)
    }
}

impl<T: Config> rt_offchain::storage_lock::BlockNumberProvider for Pallet<T> {
    type BlockNumber = T::BlockNumber;
    fn current_block_number() -> Self::BlockNumber {
        <frame_system::Pallet<T>>::block_number()
    }
}

impl<T: Config> PriceFeeder for Pallet<T> {
    fn get_price(currency_id: &CurrencyId) -> Option<PriceDetail> {
        Self::get_price(currency_id)
    }
}
