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
    mock::{Amm, ExtBuilder, Origin, Runtime, Tokens, ALICE, BOB},
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
fn does_not_incur_slippage_if_amount_is_less_or_equal_than_12_pct() {
    ExtBuilder::default().build().execute_with(|| {
        let pool = Pool {
            amount_base: 50,
            amount_quote: 50,
            asset_base: CurrencyId::DOT,
            asset_quote: CurrencyId::xDOT,
        };
        let total_amount = pool.amount_base + pool.amount_quote;
        let x = total_amount / 100 * 12;
        Amm::create_pool(Origin::signed(ALICE), pool.clone()).unwrap();
        Amm::sell_with_exact_amount(Origin::signed(BOB), x, 0).unwrap();
        assert_eq!(
            Tokens::free_balance(CurrencyId::DOT, &BOB),
            token_units(1000).unwrap() - x
        );
        assert_eq!(
            Tokens::free_balance(CurrencyId::xDOT, &BOB),
            token_units(1000).unwrap() + x
        );
    });
}

#[test]
fn incurs_slippage_if_amount_is_greater_than_12_pct() {
    ExtBuilder::default().build().execute_with(|| {
        let pool = Pool {
            amount_base: 50,
            amount_quote: 50,
            asset_base: CurrencyId::DOT,
            asset_quote: CurrencyId::xDOT,
        };
        let total_amount = pool.amount_base + pool.amount_quote;
        let x = total_amount / 100 * 13;
        Amm::create_pool(Origin::signed(ALICE), pool.clone()).unwrap();
        Amm::sell_with_exact_amount(Origin::signed(BOB), x, 0).unwrap();
        assert_eq!(
            Tokens::free_balance(CurrencyId::DOT, &BOB),
            token_units(1000).unwrap() - x
        );
        assert!(Tokens::free_balance(CurrencyId::xDOT, &BOB) > token_units(1000).unwrap() + x);
    });
}

#[test]
fn sell_with_exact_amount_correctly_evaluates_all_pool_assets() {
    ExtBuilder::default().build().execute_with(|| {
        dbg!(Tokens::free_balance(CurrencyId::DOT, &BOB));
        let pool = Pool {
            amount_base: 50,
            amount_quote: 50,
            asset_base: CurrencyId::DOT,
            asset_quote: CurrencyId::xDOT,
        };
        Amm::create_pool(Origin::signed(ALICE), pool.clone()).unwrap();
        Amm::sell_with_exact_amount(Origin::signed(BOB), 1, 0).unwrap();
        let pool_account = Amm::pool_account(&0);
        assert_eq!(
            Tokens::free_balance(CurrencyId::DOT, &pool_account),
            pool.amount_base + 1
        );
        assert_eq!(
            Tokens::free_balance(CurrencyId::xDOT, &pool_account),
            pool.amount_quote - 1
        );
        assert_eq!(
            Tokens::free_balance(CurrencyId::DOT, &BOB),
            token_units(1000).unwrap() - 1
        );
        assert_eq!(
            Tokens::free_balance(CurrencyId::xDOT, &BOB),
            token_units(1000).unwrap() + 1
        );
    });
}
