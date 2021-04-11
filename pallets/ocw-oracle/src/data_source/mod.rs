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

mod binance;
mod coinbase;
mod coincap;
use self::binance::Binance;
use self::coinbase::Coinbase;
use self::coincap::Coincap;

pub trait DataSourceApi<T: Config> {
    fn get_ticker(
        symbol: CurrencyId,
        source: DataSourceEnum,
        bytes: Vec<u8>,
    ) -> Result<TickerPayloadDetail, Error<T>>;
}

pub fn get_ticker<T: Config>(
    symbol: CurrencyId,
    source: DataSourceEnum,
    bytes: Vec<u8>,
) -> Result<TickerPayloadDetail, Error<T>> {
    match source {
        DataSourceEnum::BINANCE => Binance::get_ticker(symbol, source, bytes),
        DataSourceEnum::COINBASE => Coinbase::get_ticker(symbol, source, bytes),
        DataSourceEnum::COINCAP => Coincap::get_ticker(symbol, source, bytes),
    }
}
