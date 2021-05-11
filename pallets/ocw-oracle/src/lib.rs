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

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use core::fmt;
use frame_support::{log, pallet_prelude::*};
use frame_system::pallet_prelude::*;
use frame_system::{
    ensure_none,
    offchain::{
        AppCrypto, CreateSignedTransaction, SendUnsignedTransaction, SignedPayload, Signer,
        SigningTypes,
    },
};
pub use module::*;
#[allow(unused_imports)]
use num_traits::float::FloatCore;
use num_traits::CheckedDiv;
use primitives::*;
use serde::{Deserialize, Deserializer};
use sp_core::crypto::KeyTypeId;
use sp_runtime::{
    offchain as rt_offchain,
    offchain::storage_lock::{BlockAndTime, StorageLock},
    transaction_validity::{
        InvalidTransaction, TransactionSource, TransactionValidity, ValidTransaction,
    },
    FixedPointNumber, RuntimeDebug,
};
use sp_std::{prelude::*, str};

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"demo");
pub const NUM_VEC_LEN: usize = 10;
/// The type to sign and send transactions.
pub const UNSIGNED_TXS_PRIORITY: u64 = 100;

pub const HTTP_HEADER_USER_AGENT: &str = "Parallel";
pub const FETCH_TIMEOUT_PERIOD: u64 = 3000; // in milli-seconds
pub const LOCK_TIMEOUT_EXPIRATION: u64 = FETCH_TIMEOUT_PERIOD * 3 + 1000; // in milli-seconds
pub const LOCK_BLOCK_EXPIRATION: u32 = 5; // in block number

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct Payload<Public> {
    list: Vec<PayloadDetail>,
    public: Public,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct PayloadDetail {
    price: Price,
    symbol: CurrencyId,
    timestamp: Timestamp,
}

impl<T: SigningTypes> SignedPayload<T> for Payload<T::Public> {
    fn public(&self) -> T::Public {
        self.public.clone()
    }
}

#[derive(Deserialize, Encode, Decode, Default, Clone)]
pub struct PriceJson {
    data: DataDetail,
    timestamp: Timestamp,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Encode, Decode, Default, Clone)]
pub struct DataDetail {
    #[serde(deserialize_with = "de_string_to_bytes")]
    id: Vec<u8>,
    #[serde(deserialize_with = "de_string_to_bytes")]
    symbol: Vec<u8>,
    #[serde(deserialize_with = "de_string_to_bytes")]
    priceUsd: Vec<u8>,
}

pub fn de_string_to_bytes<'de, D>(de: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(de)?;
    Ok(s.as_bytes().to_vec())
}

impl fmt::Debug for PriceJson {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{ data: {:?}, timestamp: {} }}",
            &self.data, &self.timestamp
        )
    }
}

impl fmt::Debug for DataDetail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{ id: {}, symbol: {}, priceUsd: {} }}",
            str::from_utf8(&self.id).map_err(|_| fmt::Error)?,
            str::from_utf8(&self.symbol).map_err(|_| fmt::Error)?,
            str::from_utf8(&self.priceUsd).map_err(|_| fmt::Error)?
        )
    }
}

#[frame_support::pallet]
pub mod module {
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
        /// the precision of the price
        #[pallet::constant]
        type PricePrecision: Get<u8>;
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn offchain_worker(_block_number: T::BlockNumber) {
            let urls = [
                (CurrencyId::DOT, "https://api.coincap.io/v2/assets/polkadot"),
                (
                    CurrencyId::xDOT,
                    "https://api.coincap.io/v2/assets/polkadot",
                ),
                (CurrencyId::KSM, "https://api.coincap.io/v2/assets/kusama"),
                (CurrencyId::USDT, "https://api.coincap.io/v2/assets/tether"),
            ]
            .to_vec();

            match Self::fetch_price(urls) {
                Ok(res) => {
                    let _ = Self::offchain_price_unsigned_with_signed_payload(res);
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
        OffchainInvoke(Option<T::AccountId>),
    }

    #[pallet::error]
    pub enum Error<T> {
        // Error returned when not sure which ocw function to executed
        UnknownOffchainMux,

        // Error returned when making signed transactions in off-chain worker
        NoLocalAcctForSigning,
        OffchainSignedTxError,

        // Error returned when making unsigned transactions in off-chain worker
        OffchainUnsignedTxError,

        // Error returned when making unsigned transactions with signed payloads in off-chain worker
        OffchainUnsignedTxSignedPayloadError,

        // Error returned when fetching info
        HttpFetchingError,

        // Error when previous http is waiting
        AcquireStorageLockError,

        // Error when convert price
        ConvertToStringError,

        //Error when convert price
        ParsingToF64Error,
    }

    #[pallet::storage]
    #[pallet::getter(fn get_price)]
    pub type Prices<T: Config> =
        StorageMap<_, Twox64Concat, CurrencyId, Option<PriceDetail>, ValueQuery>;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        // const PricePrecision: u8 = T::PricePrecision::get();

        #[pallet::weight(10_000)]
        fn submit_price_unsigned_with_signed_payload(
            origin: OriginFor<T>,
            payload: Payload<T::Public>,
            _signature: T::Signature,
        ) -> DispatchResultWithPostInfo {
            // we don't need to verify the signature here because it has been verified in
            //   `validate_unsigned` function when sending out the unsigned tx.
            let _ = ensure_none(origin)?;
            log::info!("dot: {:?}", Prices::<T>::get(CurrencyId::DOT));
            log::info!("ksm: {:?}", Prices::<T>::get(CurrencyId::KSM));
            log::info!("usdt: {:?}", Prices::<T>::get(CurrencyId::USDT));
            Self::append_price(payload);
            Self::deposit_event(Event::<T>::OffchainInvoke(None));
            Ok(().into())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Append a new number to the tail of the list, removing an element from the head if reaching
        ///   the bounded length.
        fn append_price(payload: Payload<T::Public>) {
            let list = payload.list;
            for item in list {
                Prices::<T>::insert(&item.symbol, Some((item.price, item.timestamp)));
            }
        }

        pub fn fetch_price(
            urls: Vec<(CurrencyId, &str)>,
        ) -> Result<Vec<(CurrencyId, PriceJson)>, Error<T>> {
            let mut lock = StorageLock::<BlockAndTime<Self>>::with_block_and_time_deadline(
                b"offchain-demo::lock",
                LOCK_BLOCK_EXPIRATION,
                rt_offchain::Duration::from_millis(LOCK_TIMEOUT_EXPIRATION),
            );
            if let Ok(_guard) = lock.try_lock() {
                //TODO async http
                let mut res = Vec::new();
                for (currency_id, url) in urls.into_iter() {
                    if let Ok(json) = Self::fetch_n_parse(url) {
                        res.push((currency_id, json));
                    } else {
                        log::info!("error response: {}", url);
                    }
                }
                if !res.is_empty() {
                    return Ok(res);
                } else {
                    return Err(<Error<T>>::HttpFetchingError);
                }
            }
            Err(<Error<T>>::AcquireStorageLockError)
        }

        /// Fetch from remote and deserialize the JSON to a struct
        pub fn fetch_n_parse(url: &str) -> Result<PriceJson, Error<T>> {
            let resp_bytes = Self::fetch_from_remote(url).map_err(|e| {
                log::error!("fetch_from_remote error: {:?}", e);
                <Error<T>>::HttpFetchingError
            })?;
            let resp_str =
                str::from_utf8(&resp_bytes).map_err(|_| <Error<T>>::HttpFetchingError)?;
            // Print out our fetched JSON string
            // log::info!("{}", resp_str);
            let gh_info: PriceJson =
                serde_json::from_str(&resp_str).map_err(|_| <Error<T>>::HttpFetchingError)?;
            Ok(gh_info)
        }

        /// This function uses the `offchain::http` API to query the remote github information,
        ///   and returns the JSON response as vector of bytes.
        pub fn fetch_from_remote(url: &str) -> Result<Vec<u8>, Error<T>> {
            // log::info!("sending request to: {}", url);

            // Initiate an external HTTP GET request. This is using high-level wrappers from `sp_runtime`.
            let request = rt_offchain::http::Request::get(url);

            // Keeping the offchain worker execution time reasonable, so limiting the call to be within 3s.
            let timeout = sp_io::offchain::timestamp()
                .add(rt_offchain::Duration::from_millis(FETCH_TIMEOUT_PERIOD));

            // For github API request, we also need to specify `user-agent` in http request header.
            //   See: https://developer.github.com/v3/#user-agent-required
            let pending = request
                .add_header("User-Agent", HTTP_HEADER_USER_AGENT)
                .deadline(timeout) // Setting the timeout time
                .send() // Sending the request out by the host
                .map_err(|_| <Error<T>>::HttpFetchingError)?;

            // By default, the http request is async from the runtime perspective. So we are asking the
            //   runtime to wait here.
            // The returning value here is a `Result` of `Result`, so we are unwrapping it twice by two `?`
            //   ref: https://substrate.dev/rustdocs/v2.0.0/sp_runtime/offchain/http/struct.PendingRequest.html#method.try_wait
            let response = pending
                .try_wait(timeout)
                .map_err(|_| <Error<T>>::HttpFetchingError)?
                .map_err(|_| <Error<T>>::HttpFetchingError)?;

            if response.code != 200 {
                log::error!("Unexpected http request status code: {}", response.code);
                return Err(<Error<T>>::HttpFetchingError);
            }

            // Next we fully read the response body and collect it to a vector of bytes.
            Ok(response.body().collect::<Vec<u8>>())
        }

        fn offchain_price_unsigned_with_signed_payload(
            json_list: Vec<(CurrencyId, PriceJson)>,
        ) -> Result<(), Error<T>> {
            let signer = Signer::<T, T::AuthorityId>::any_account();

            let payload_list = {
                let mut v: Vec<PayloadDetail> = Vec::new();
                for item in json_list {
                    let (currency_id, json) = item;
                    let price = Self::to_price(json.data.priceUsd)?;
                    let symbol = currency_id;
                    let timestamp = json.timestamp;
                    v.push(PayloadDetail {
                        price,
                        symbol,
                        timestamp,
                    });
                }
                v
            };

            if let Some((_, res)) = signer.send_unsigned_transaction(
                |acct| Payload {
                    list: payload_list.clone(),
                    public: acct.public.clone(),
                },
                Call::submit_price_unsigned_with_signed_payload,
            ) {
                return res.map_err(|_| {
                    log::error!("Failed in offchain_unsigned_tx_signed_payload");
                    <Error<T>>::OffchainUnsignedTxSignedPayloadError
                });
            }
            // The case of `None`: no account is available for sending
            log::error!("No local account available");
            Err(<Error<T>>::NoLocalAcctForSigning)
        }

        fn to_price(val_u8: Vec<u8>) -> Result<Price, Error<T>> {
            // let val_u8: Vec<u8> = json.data.priceUsd;
            let val_f64: f64 = core::str::from_utf8(&val_u8)
                .map_err(|_| {
                    log::error!("val_u8 convert to string error");
                    <Error<T>>::ConvertToStringError
                })?
                .parse::<f64>()
                .map_err(|_| {
                    log::error!("string convert to f64 error");
                    <Error<T>>::ParsingToF64Error
                })?;

            let price: Price = Price::from_inner(
                (val_f64 * 10f64.powi(T::PricePrecision::get() as i32)).round() as u128,
            );
            Ok(price)
        }
    }

    #[allow(deprecated)] // ValidateUnsigned
    impl<T: Config> frame_support::unsigned::ValidateUnsigned for Pallet<T> {
        type Call = Call<T>;

        fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
            let valid_tx = |provide| {
                ValidTransaction::with_tag_prefix("ocw-demo")
                    .priority(UNSIGNED_TXS_PRIORITY)
                    .and_provides([&provide])
                    .longevity(3)
                    .propagate(true)
                    .build()
            };

            match call {
                Call::submit_price_unsigned_with_signed_payload(ref payload, ref signature) => {
                    if !SignedPayload::<T>::verify::<T::AuthorityId>(payload, signature.clone()) {
                        return InvalidTransaction::BadProof.into();
                    }
                    valid_tx(b"submit_price_unsigned_with_signed_payload".to_vec())
                }
                _ => InvalidTransaction::Call.into(),
            }
        }
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
        Self::get_price(currency_id).and_then(|(price, timestamp)| {
            price
                .checked_div(&Price::saturating_from_integer(CURRENCY_DECIMAL))
                .and_then(|price| Some((price, timestamp)))
        })
    }
}
