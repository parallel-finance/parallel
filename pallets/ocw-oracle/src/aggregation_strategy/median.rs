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
pub struct Median;

impl<T: Config> AggregationStrategyApi<T> for Median {
    fn aggregate_price(
        round_index: &RoundIndex<T::BlockNumber>,
        provider: &Vec<T::AccountId>,
        currency_id: &CurrencyId,
    ) -> Result<PriceDetail, Error<T>> {
        let mut prices = vec![];
        provider.iter().for_each(|account_id| {
            Pallet::<T>::ocw_oracle_data_source().iter().for_each(
                |data_source_enum: &DataSourceEnum| {
                    let ovp: Option<VecDeque<PriceDetailOf<T::BlockNumber>>> =
                        Pallet::<T>::ocw_oracle_price(account_id, (data_source_enum, currency_id));
                    if let Some(vp) = ovp {
                        if let Some(p) = vp.back() {
                            if round_index == &p.index {
                                prices.push(p.price);
                            } else {
                                log::warn!(
                                    "price round index is {:?}, while this round is {:?}",
                                    p.index,
                                    round_index
                                );
                            }
                        }
                    }
                },
            );
        });
        let count = prices.len() as u32;
        prices.sort_by(|a, b| a.cmp(b));
        let median_index = count / 2;
        let now = T::Time::now();
        let timestamp: Timestamp = now.try_into().or(Err(Error::<T>::ParseTimestampError))?;
        Ok((prices[median_index as usize].clone(), timestamp))
    }
}
