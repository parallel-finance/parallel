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

use primitives::*;
use sc_service::ChainType;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
#[allow(unused_imports)]
use sp_core::{sr25519, Pair, Public};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::{
    traits::{IdentifyAccount, One, Verify},
    FixedPointNumber,
};
use vanilla_runtime::{AuraConfig, GrandpaConfig, VanillaOracleConfig, WASM_BINARY};

pub type VanillaChainSpec = sc_service::GenericChainSpec<vanilla_runtime::GenesisConfig>;

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

#[allow(dead_code)]
type AccountPublic = <Signature as Verify>::Signer;

#[allow(dead_code)]
/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
    (get_from_seed::<AuraId>(s), get_from_seed::<GrandpaId>(s))
}

pub fn development_config() -> Result<VanillaChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    Ok(VanillaChainSpec::from_genesis(
        // Name
        "Development",
        // ID
        "dev",
        ChainType::Development,
        move || {
            testnet_genesis(
                wasm_binary,
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                vec![authority_keys_from_seed("Alice")],
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie"),
                    get_account_id_from_seed::<sr25519::Public>("Dave"),
                    get_account_id_from_seed::<sr25519::Public>("Eve"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
                    // Parallel team accounts
                    "5HHMY7e8UAqR5ZaHGaQnRW5EDR8dP7QpAyjeBu6V7vdXxxbf"
                        .parse()
                        .unwrap(),
                ],
            )
        },
        vec![],
        None,
        None,
        None,
        None,
    ))
}

pub fn local_testnet_config() -> Result<VanillaChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Testnet wasm not available".to_string())?;

    Ok(VanillaChainSpec::from_genesis(
        // Name
        "Local Testnet",
        // ID
        "local_testnet",
        ChainType::Local,
        move || {
            testnet_genesis(
                wasm_binary,
                "5HHMY7e8UAqR5ZaHGaQnRW5EDR8dP7QpAyjeBu6V7vdXxxbf"
                    .parse()
                    .unwrap(),
                vec![authority_keys_from_seed("Alice")],
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie"),
                    get_account_id_from_seed::<sr25519::Public>("Dave"),
                    get_account_id_from_seed::<sr25519::Public>("Eve"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
                    // Parallel team accounts
                    "5HHMY7e8UAqR5ZaHGaQnRW5EDR8dP7QpAyjeBu6V7vdXxxbf"
                        .parse()
                        .unwrap(),
                ],
            )
        },
        vec![],
        None,
        None,
        None,
        None,
    ))
}

fn testnet_genesis(
    wasm_binary: &[u8],
    root_key: AccountId,
    initial_authorities: Vec<(AuraId, GrandpaId)>,
    endowed_accounts: Vec<AccountId>,
) -> vanilla_runtime::GenesisConfig {
    vanilla_runtime::GenesisConfig {
        frame_system: vanilla_runtime::SystemConfig {
            code: wasm_binary.to_vec(),
            changes_trie_config: Default::default(),
        },
        pallet_aura: AuraConfig {
            authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
        },
        pallet_grandpa: GrandpaConfig {
            authorities: initial_authorities
                .iter()
                .map(|x| (x.1.clone(), 1))
                .collect(),
        },
        pallet_balances: vanilla_runtime::BalancesConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, 1 << 60))
                .collect(),
        },
        pallet_sudo: vanilla_runtime::SudoConfig { key: root_key },
        orml_oracle_Instance1: VanillaOracleConfig {
            members: endowed_accounts.clone().into(),
            phantom: Default::default(),
        },
        orml_tokens: vanilla_runtime::TokensConfig {
            endowed_accounts: endowed_accounts
                .iter()
                .flat_map(|x| {
                    vec![
                        (x.clone(), CurrencyId::DOT, 1_000 * TOKEN_DECIMAL),
                        (x.clone(), CurrencyId::KSM, 1_000 * TOKEN_DECIMAL),
                        (x.clone(), CurrencyId::USDT, 1_000 * TOKEN_DECIMAL),
                        (x.clone(), CurrencyId::xDOT, 1_000 * TOKEN_DECIMAL),
                    ]
                })
                .collect(),
        },
        pallet_loans: vanilla_runtime::LoansConfig {
            currencies: vec![
                CurrencyId::DOT,
                CurrencyId::KSM,
                CurrencyId::USDT,
                CurrencyId::xDOT,
            ],
            borrow_index: Rate::one(),                                  // 1
            exchange_rate: Rate::saturating_from_rational(2, 100),      // 0.02
            base_rate_per_year: Rate::saturating_from_rational(2, 100), // 0.02
            multiplier_per_year: Multiplier::saturating_from_rational(1, 10), // 0.1
            jump_multiplier_per_year: Multiplier::saturating_from_rational(11, 10), // 1.1
            kink: Ratio::from_percent(80),                              // 0.8
            collateral_factor: vec![
                (CurrencyId::DOT, Ratio::from_percent(50)),
                (CurrencyId::KSM, Ratio::from_percent(50)),
                (CurrencyId::USDT, Ratio::from_percent(50)),
                (CurrencyId::xDOT, Ratio::from_percent(50)),
            ],
            liquidation_incentive: vec![
                (CurrencyId::DOT, Ratio::from_percent(90)),
                (CurrencyId::KSM, Ratio::from_percent(90)),
                (CurrencyId::USDT, Ratio::from_percent(90)),
                (CurrencyId::xDOT, Ratio::from_percent(90)),
            ],
            //TODO : please refer to https://github.com/parallel-finance/parallel/issues/46
            liquidation_threshold: vec![
                (CurrencyId::DOT, Ratio::from_percent(90)),
                (CurrencyId::KSM, Ratio::from_percent(90)),
                (CurrencyId::USDT, Ratio::from_percent(90)),
                (CurrencyId::xDOT, Ratio::from_percent(90)),
            ],
            close_factor: vec![
                (CurrencyId::DOT, Ratio::from_percent(50)),
                (CurrencyId::KSM, Ratio::from_percent(50)),
                (CurrencyId::USDT, Ratio::from_percent(50)),
                (CurrencyId::xDOT, Ratio::from_percent(50)),
            ],
            reserve_factor: vec![
                (CurrencyId::DOT, Ratio::from_percent(15)),
                (CurrencyId::KSM, Ratio::from_percent(15)),
                (CurrencyId::USDT, Ratio::from_percent(15)),
                (CurrencyId::xDOT, Ratio::from_percent(15)),
            ],
        },
        pallet_staking: vanilla_runtime::StakingConfig {},
    }
}
