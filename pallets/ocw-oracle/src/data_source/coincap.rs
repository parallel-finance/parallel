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
pub struct Coincap;

impl<T: Config> DataSourceApi<T> for Coincap {
    fn get_ticker(
        symbol: CurrencyId,
        data_source_enum: DataSourceEnum,
        bytes: Vec<u8>,
    ) -> Result<TickerPayloadDetail, Error<T>> {
        let resp_str = str::from_utf8(&bytes).map_err(|_| <Error<T>>::HttpFetchingCoincapError)?;
        let json: PriceJson =
            serde_json::from_str(&resp_str).map_err(|_| <Error<T>>::HttpFetchingCoincapError)?;
        let price = Pallet::<T>::to_price(json.data.priceUsd)?;
        let r = TickerPayloadDetail {
            symbol,
            data_source_enum,
            price,
            timestamp: json.timestamp,
        };

        Ok(r)
    }
}
/// {
///     "data": {
///         "id": "polkadot",
///         "rank": "6",
///         "symbol": "DOT",
///         "name": "Polkadot",
///         "supply": "978738003.6347900000000000",
///         "maxSupply": null,
///         "marketCapUsd": "31311241603.3960096242304270",
///         "volumeUsd24Hr": "521745794.1342459328591593",
///         "priceUsd": "31.9914435600884307",
///         "changePercent24Hr": "3.1730378666842420",
///         "vwap24Hr": null,
///         "explorer": "https://polkascan.io/polkadot"
///     },
///     "timestamp": 1616844616682
/// }
#[derive(Deserialize, Encode, Decode, Default, Clone)]
struct PriceJson {
    data: DataDetail,
    timestamp: Timestamp,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Encode, Decode, Default, Clone)]
struct DataDetail {
    #[serde(deserialize_with = "de_string_to_bytes")]
    id: Vec<u8>,
    #[serde(deserialize_with = "de_string_to_bytes")]
    symbol: Vec<u8>,
    #[serde(deserialize_with = "de_string_to_bytes")]
    priceUsd: Vec<u8>,
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
