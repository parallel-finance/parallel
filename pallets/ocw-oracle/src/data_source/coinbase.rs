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
pub struct Coinbase;

impl<T: Config> DataSourceApi<T> for Coinbase {
    fn get_ticker(
        symbol: CurrencyId,
        data_source_enum: DataSourceEnum,
        bytes: Vec<u8>,
    ) -> Result<TickerPayloadDetail, Error<T>> {
        let resp_str = str::from_utf8(&bytes).map_err(|_| <Error<T>>::HttpFetchingCoinbaseError)?;
        let json: Ticker =
            serde_json::from_str(&resp_str).map_err(|_| <Error<T>>::HttpFetchingCoinbaseError)?;
        let price = Pallet::<T>::to_price(json.price)?;
        let now = T::Time::now();
        let timestamp: Timestamp = now.try_into().or(Err(Error::<T>::ParseTimestampError))?;
        let r = TickerPayloadDetail {
            symbol,
            data_source_enum,
            price,
            timestamp,
        };

        Ok(r)
    }
}

/// {
///     "trade_id": 150096411,
///     "price": "54816.95",
///     "size": "0.00447155",
///     "time": "2021-03-27T11:35:16.390654Z",
///     "bid": "54816.94",
///     "ask": "54816.95",
///     "volume": "15023.66134397"
/// }
#[derive(Deserialize, Encode, Decode, Default, Clone)]
struct Ticker {
    #[serde(deserialize_with = "de_string_to_bytes")]
    price: Vec<u8>,
    #[serde(deserialize_with = "de_string_to_bytes")]
    time: Vec<u8>,
}
