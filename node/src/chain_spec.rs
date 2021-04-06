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

use cumulus_primitives_core::ParaId;
use primitives::*;
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{sr25519, Pair, Public};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{IdentifyAccount, Verify};
use vanilla_runtime::{AuraConfig, GrandpaConfig};

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type VanillaChainSpec =
    sc_service::GenericChainSpec<parallel_runtime::GenesisConfig, Extensions>;
pub type DevChainSpec = sc_service::GenericChainSpec<vanilla_runtime::GenesisConfig>;

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

/// The extensions for the [`ChainSpec`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
#[serde(deny_unknown_fields)]
pub struct Extensions {
    /// The relay chain of the Parachain.
    pub relay_chain: String,
    /// The id of the Parachain.
    pub para_id: u32,
}

impl Extensions {
    /// Try to get the extension from the given `ChainSpec`.
    pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
        sc_chain_spec::get_extension(chain_spec.extensions())
    }
}

type AccountPublic = <Signature as Verify>::Signer;

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

pub fn development_config(id: ParaId) -> DevChainSpec {
    DevChainSpec::from_genesis(
        // Name
        "Development",
        // ID
        "dev",
        ChainType::Development,
        move || {
            development_genesis(
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
                ],
                id,
            )
        },
        vec![],
        None,
        None,
        None,
        None,
    )
}

pub fn local_testnet_config(id: ParaId) -> VanillaChainSpec {
    VanillaChainSpec::from_genesis(
        // Name
        "Local Testnet",
        // ID
        "local_testnet",
        ChainType::Local,
        move || {
            testnet_genesis(
                get_account_id_from_seed::<sr25519::Public>("Alice"),
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
                ],
                id,
            )
        },
        vec![],
        None,
        None,
        None,
        Extensions {
            relay_chain: "rococo-local".into(),
            para_id: id.into(),
        },
    )
}
fn development_genesis(
    root_key: AccountId,
    initial_authorities: Vec<(AuraId, GrandpaId)>,
    endowed_accounts: Vec<AccountId>,
    _id: ParaId,
) -> vanilla_runtime::GenesisConfig {
    vanilla_runtime::GenesisConfig {
        frame_system: vanilla_runtime::SystemConfig {
            code: vanilla_runtime::WASM_BINARY
                .expect("WASM binary was not build, please build it!")
                .to_vec(),
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
        orml_tokens: vanilla_runtime::TokensConfig {
            endowed_accounts: endowed_accounts
                .iter()
                .flat_map(|x| {
                    vec![
                        (x.clone(), CurrencyId::DOT, 1_000 * TOKEN_DECIMAL),
                        (x.clone(), CurrencyId::KSM, 1_000 * TOKEN_DECIMAL),
                        (x.clone(), CurrencyId::BTC, 1_000 * TOKEN_DECIMAL),
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
                CurrencyId::BTC,
                CurrencyId::USDT,
                CurrencyId::xDOT,
            ],
            // total_supply: 1000 * TOKEN_DECIMAL, // 1000
            // total_borrows: 600 * TOKEN_DECIMAL, // 600
            borrow_index: RATE_DECIMAL,               // 1
            exchange_rate: 2 * RATE_DECIMAL / 100,    // 0.02
            base_rate: 2 * RATE_DECIMAL / 100,        // 0.02
            multiplier_per_year: RATE_DECIMAL / 10,   // 0.1
            jump_muiltiplier: 11 * RATE_DECIMAL / 10, // 1.1
            kink: 8 * RATE_DECIMAL / 10,              // 0.8
            collateral_rate: vec![
                (CurrencyId::DOT, 5 * RATE_DECIMAL / 10),
                (CurrencyId::KSM, 5 * RATE_DECIMAL / 10),
                (CurrencyId::BTC, 5 * RATE_DECIMAL / 10),
                (CurrencyId::USDT, 5 * RATE_DECIMAL / 10),
                (CurrencyId::xDOT, 5 * RATE_DECIMAL / 10),
            ],
            liquidation_incentive: vec![
                (CurrencyId::DOT, 9 * RATE_DECIMAL / 10),
                (CurrencyId::KSM, 9 * RATE_DECIMAL / 10),
                (CurrencyId::BTC, 9 * RATE_DECIMAL / 10),
                (CurrencyId::USDT, 9 * RATE_DECIMAL / 10),
                (CurrencyId::xDOT, 9 * RATE_DECIMAL / 10),
            ],
            //FIXME :In fact,"liquidation_threshold" should be higher than "collateral_rate",
            //but for test, let's make it lower
            liquidation_threshold: vec![
                (CurrencyId::DOT, 40 * RATE_DECIMAL / 100),
                (CurrencyId::KSM, 40 * RATE_DECIMAL / 100),
                (CurrencyId::BTC, 40 * RATE_DECIMAL / 100),
                (CurrencyId::USDT, 40 * RATE_DECIMAL / 100),
                (CurrencyId::xDOT, 40 * RATE_DECIMAL / 100),
            ],
            close_factor: vec![
                (CurrencyId::DOT, 5 * RATE_DECIMAL / 10),
                (CurrencyId::KSM, 5 * RATE_DECIMAL / 10),
                (CurrencyId::BTC, 5 * RATE_DECIMAL / 10),
                (CurrencyId::USDT, 5 * RATE_DECIMAL / 10),
                (CurrencyId::xDOT, 5 * RATE_DECIMAL / 10),
            ],
        },
        pallet_staking: vanilla_runtime::StakingConfig {},
    }
}

fn testnet_genesis(
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    id: ParaId,
) -> parallel_runtime::GenesisConfig {
    parallel_runtime::GenesisConfig {
        frame_system: parallel_runtime::SystemConfig {
            code: parallel_runtime::WASM_BINARY
                .expect("WASM binary was not build, please build it!")
                .to_vec(),
            changes_trie_config: Default::default(),
        },
        pallet_balances: parallel_runtime::BalancesConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, 1 << 60))
                .collect(),
        },
        pallet_sudo: parallel_runtime::SudoConfig { key: root_key },
        parachain_info: parallel_runtime::ParachainInfoConfig { parachain_id: id },
        orml_tokens: parallel_runtime::TokensConfig {
            endowed_accounts: endowed_accounts
                .iter()
                .flat_map(|x| {
                    vec![
                        (x.clone(), CurrencyId::DOT, 1_000 * TOKEN_DECIMAL),
                        (x.clone(), CurrencyId::KSM, 1_000 * TOKEN_DECIMAL),
                        (x.clone(), CurrencyId::BTC, 1_000 * TOKEN_DECIMAL),
                        (x.clone(), CurrencyId::USDT, 1_000 * TOKEN_DECIMAL),
                        (x.clone(), CurrencyId::xDOT, 1_000 * TOKEN_DECIMAL),
                    ]
                })
                .collect(),
        },
        pallet_loans: parallel_runtime::LoansConfig {
            currencies: vec![
                CurrencyId::DOT,
                CurrencyId::KSM,
                CurrencyId::BTC,
                CurrencyId::USDT,
                CurrencyId::xDOT,
            ],
            // total_supply: 1000 * TOKEN_DECIMAL, // 1000
            // total_borrows: 600 * TOKEN_DECIMAL, // 600
            borrow_index: RATE_DECIMAL,               // 1
            exchange_rate: 2 * RATE_DECIMAL / 100,    // 0.02
            base_rate: 2 * RATE_DECIMAL / 100,        // 0.02
            multiplier_per_year: RATE_DECIMAL / 10,   // 0.1
            jump_muiltiplier: 11 * RATE_DECIMAL / 10, // 1.1
            kink: 8 * RATE_DECIMAL / 10,              // 0.8
            collateral_rate: vec![
                (CurrencyId::DOT, 5 * RATE_DECIMAL / 10),
                (CurrencyId::KSM, 5 * RATE_DECIMAL / 10),
                (CurrencyId::BTC, 5 * RATE_DECIMAL / 10),
                (CurrencyId::USDT, 5 * RATE_DECIMAL / 10),
                (CurrencyId::xDOT, 5 * RATE_DECIMAL / 10),
            ],
            liquidation_incentive: vec![
                (CurrencyId::DOT, 9 * RATE_DECIMAL / 10),
                (CurrencyId::KSM, 9 * RATE_DECIMAL / 10),
                (CurrencyId::BTC, 9 * RATE_DECIMAL / 10),
                (CurrencyId::USDT, 9 * RATE_DECIMAL / 10),
                (CurrencyId::xDOT, 9 * RATE_DECIMAL / 10),
            ],
            //FIXME :In fact,"liquidation_threshold" should be higher than "collateral_rate",
            //but for test, let's make it lower
            liquidation_threshold: vec![
                (CurrencyId::DOT, 40 * RATE_DECIMAL / 100),
                (CurrencyId::KSM, 40 * RATE_DECIMAL / 100),
                (CurrencyId::BTC, 40 * RATE_DECIMAL / 100),
                (CurrencyId::USDT, 40 * RATE_DECIMAL / 100),
                (CurrencyId::xDOT, 40 * RATE_DECIMAL / 100),
            ],
            close_factor: vec![
                (CurrencyId::DOT, 5 * RATE_DECIMAL / 10),
                (CurrencyId::KSM, 5 * RATE_DECIMAL / 10),
                (CurrencyId::BTC, 5 * RATE_DECIMAL / 10),
                (CurrencyId::USDT, 5 * RATE_DECIMAL / 10),
                (CurrencyId::xDOT, 5 * RATE_DECIMAL / 10),
            ],
        },
        pallet_staking: parallel_runtime::StakingConfig {},
    }
}
