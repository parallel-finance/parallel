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
pub struct Binance;

impl<T: Config> DataSourceApi<T> for Binance {
    fn get_ticker(
        symbol: CurrencyId,
        data_source_enum: DataSourceEnum,
        bytes: Vec<u8>,
    ) -> Result<TickerPayloadDetail, Error<T>> {
        let resp_str = str::from_utf8(&bytes).map_err(|_| <Error<T>>::HttpFetchingBinanceError)?;
        let json: Ticker =
            serde_json::from_str(&resp_str).map_err(|_| <Error<T>>::HttpFetchingBinanceError)?;
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
///     "symbol": "DOTUSDT",
///     "price": "32.02420000"
/// }
#[derive(Deserialize, Encode, Decode, Default, Clone)]
struct Ticker {
    #[serde(deserialize_with = "de_string_to_bytes")]
    symbol: Vec<u8>,
    #[serde(deserialize_with = "de_string_to_bytes")]
    price: Vec<u8>,
}
