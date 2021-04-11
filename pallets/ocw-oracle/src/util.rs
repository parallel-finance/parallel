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
    pub(crate) fn to_price(val_u8: Vec<u8>) -> Result<Price, Error<T>> {
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

        let price = (val_f64 * 10f64.powi(T::PricePrecision::get() as i32)).round() as Price;
        Ok(price)
    }
}

pub fn de_string_to_bytes<'de, D>(de: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(de)?;
    Ok(s.as_bytes().to_vec())
}
