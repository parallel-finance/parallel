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

use crate::*;

impl<T: Config> Pallet<T> {
    pub fn fetch_ticker_price() -> Result<Vec<TickerPayloadDetail>, Error<T>> {
        let mut lock = StorageLock::<BlockAndTime<Self>>::with_block_and_time_deadline(
            b"offchain-ocw-oracle::lock",
            LOCK_BLOCK_EXPIRATION,
            rt_offchain::Duration::from_millis(LOCK_TIMEOUT_EXPIRATION),
        );
        if let Ok(_guard) = lock.try_lock() {
            let currency_vec = OcwOracleCurrencies::<T>::get();
            let data_source_vec = OcwOracleDataSource::<T>::get();

            let mut res = Vec::new();
            for currency_id in currency_vec.iter() {
                //TODO async http in same currency
                for data_source in data_source_vec.iter() {
                    if let Some(url_bytes) = OcwOracleRequestUrl::<T>::get(currency_id, data_source)
                    {
                        let url =
                            str::from_utf8(&url_bytes).map_err(|_| <Error<T>>::ParseUrlError)?;
                        let resp_bytes = Self::fetch_from_remote(url).map_err(|e| {
                            log::error!("fetch_from_remote error: {:?}", e);
                            <Error<T>>::HttpFetchingError
                        })?;
                        let payload_detail: TickerPayloadDetail =
                            get_ticker::<T>(currency_id.clone(), data_source.clone(), resp_bytes)?;
                        res.push(payload_detail);
                    }
                }
                sp_io::offchain::sleep_until(
                    sp_io::offchain::timestamp()
                        .add(sp_core::offchain::Duration::from_millis(HTTP_INTERVAL)),
                );
            }

            if res.len() > 0 {
                return Ok(res);
            } else {
                return Err(<Error<T>>::FetchingCurrencyEmptyError.into());
            }
        }
        Err(<Error<T>>::AcquireStorageLockError.into())
    }

    /// This function uses the `offchain::http` API to query the remote github information,
    ///   and returns the JSON response as vector of bytes.
    pub fn fetch_from_remote(url: &str) -> Result<Vec<u8>, Error<T>> {
        // log::info!("sending request to: {}", url);

        // Initiate an external HTTP GET request. This is using high-level wrappers from `sp_runtime`.
        let request = rt_offchain::http::Request::get(url);

        // Keeping the offchain worker execution time reasonable, so limiting the call to be within 2s.
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
}
