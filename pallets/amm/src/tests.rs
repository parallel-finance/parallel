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

#![cfg(test)]

use crate::{
    mock::{Amm, ExtBuilder, Origin, Runtime, Tokens, ALICE},
    Pool, Pools,
};
use orml_traits::MultiCurrency;
use parallel_primitives::{token_units, CurrencyId};

#[test]
fn create_pool_stores_a_pool() {
    ExtBuilder::default().build().execute_with(|| {
        let pool = Pool {
            amount_base: 50,
            amount_quote: 50,
            asset_base: CurrencyId::DOT,
            asset_quote: CurrencyId::xDOT,
        };
        Amm::create_pool(Origin::signed(ALICE), pool.clone()).unwrap();
        assert_eq!(Pools::<Runtime>::get(0).unwrap(), pool)
    });
}

#[test]
fn sell_with_exact_amount_correctly_evaluates_all_pool_assets() {
    ExtBuilder::default().build().execute_with(|| {
        let pool = Pool {
            amount_base: 50,
            amount_quote: 50,
            asset_base: CurrencyId::DOT,
            asset_quote: CurrencyId::xDOT,
        };
        Amm::create_pool(Origin::signed(ALICE), pool.clone()).unwrap();
        Amm::sell_with_exact_amount(Origin::signed(ALICE), 1, 0).unwrap();
        let pool_account = Amm::pool_account(&0);
        assert_eq!(Tokens::free_balance(CurrencyId::DOT, &pool_account), 51);
        assert_eq!(Tokens::free_balance(CurrencyId::xDOT, &pool_account), 49);
        assert_eq!(
            Tokens::free_balance(CurrencyId::DOT, &ALICE),
            token_units(1000).unwrap() - 51
        );
        assert_eq!(
            Tokens::free_balance(CurrencyId::xDOT, &ALICE),
            token_units(1000).unwrap() - 49
        );
    });
}
