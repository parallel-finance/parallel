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
use parallel_runtime::ParallelOracleConfig;
use primitives::*;
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
#[allow(unused_imports)]
use sp_core::{sr25519, Pair, Public};
use sp_runtime::{
    traits::{IdentifyAccount, One, Verify},
    FixedPointNumber,
};

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ParallelChainSpec =
    sc_service::GenericChainSpec<parallel_runtime::GenesisConfig, Extensions>;

#[allow(dead_code)]
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

pub fn development_config(id: ParaId) -> ParallelChainSpec {
    ParallelChainSpec::from_genesis(
        // Name
        "Development",
        // ID
        "dev",
        ChainType::Development,
        move || {
            testnet_genesis(
                "5HHMY7e8UAqR5ZaHGaQnRW5EDR8dP7QpAyjeBu6V7vdXxxbf"
                    .parse()
                    .unwrap(),
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
                id,
            )
        },
        vec![],
        None,
        None,
        None,
        Extensions {
            relay_chain: "rococo-dev".into(),
            para_id: id.into(),
        },
    )
}

pub fn local_testnet_config(id: ParaId) -> ParallelChainSpec {
    ParallelChainSpec::from_genesis(
        // Name
        "Local Testnet",
        // ID
        "local_testnet",
        ChainType::Local,
        move || {
            testnet_genesis(
                "5HHMY7e8UAqR5ZaHGaQnRW5EDR8dP7QpAyjeBu6V7vdXxxbf"
                    .parse()
                    .unwrap(),
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
        orml_oracle_Instance1: ParallelOracleConfig {
            members: endowed_accounts.clone().into(),
            phantom: Default::default(),
        },
        orml_tokens: parallel_runtime::TokensConfig {
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
        pallet_loans: parallel_runtime::LoansConfig {
            currencies: vec![
                CurrencyId::DOT,
                CurrencyId::KSM,
                CurrencyId::USDT,
                CurrencyId::xDOT,
            ],
            borrow_index: Rate::one(),                             // 1
            exchange_rate: Rate::saturating_from_rational(2, 100), // 0.02
            base_rate: Rate::saturating_from_rational(2, 100),     // 2%
            kink_rate: Rate::saturating_from_rational(10, 100),    // 10%
            full_rate: Rate::saturating_from_rational(32, 100),    // 32%
            kink_utilization: Ratio::from_percent(80),             // 80%
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
        pallet_staking: parallel_runtime::StakingConfig {
            exchange_rate: Rate::saturating_from_rational(2, 100), // 0.02
        },
    }
}
