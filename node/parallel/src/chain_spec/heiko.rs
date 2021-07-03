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
use heiko_runtime::{
    currency::EXISTENTIAL_DEPOSIT,
    opaque::SessionKeys,
    pallet_loans::{InterestRateModel, JumpModel, Market, MarketState},
    BalancesConfig, CollatorSelectionConfig, CouncilConfig, DemocracyConfig, ElectionsConfig,
    GenesisConfig, LiquidStakingConfig, LoansConfig, OracleMembershipConfig, ParachainInfoConfig,
    SessionConfig, SudoConfig, SystemConfig, TechnicalCommitteeConfig, TokensConfig, WASM_BINARY,
};
use primitives::*;
use sc_service::ChainType;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::sr25519;
use sp_runtime::{traits::One, FixedPointNumber};

use crate::chain_spec::{get_account_id_from_seed, get_authority_keys_from_seed, Extensions};

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

pub fn development_config(id: ParaId) -> ChainSpec {
    ChainSpec::from_genesis(
        // Name
        "Heiko Development",
        // ID
        "heiko-dev",
        ChainType::Development,
        move || {
            testnet_genesis(
                "5HHMY7e8UAqR5ZaHGaQnRW5EDR8dP7QpAyjeBu6V7vdXxxbf"
                    .parse()
                    .unwrap(),
                vec![
                    get_authority_keys_from_seed("Alice"),
                    get_authority_keys_from_seed("Bob"),
                    get_authority_keys_from_seed("Charlie"),
                ],
                vec!["5GTb3uLbk9VsyGD6taPyk69p2Hfa21GuzmMF52oJnqTQh2AA"
                    .parse()
                    .unwrap()],
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
            relay_chain: "relay-dev".into(),
            para_id: id.into(),
        },
    )
}

pub fn local_testnet_config(id: ParaId) -> ChainSpec {
    ChainSpec::from_genesis(
        // Name
        "Heiko Testnet",
        // ID
        "heiko-local",
        ChainType::Local,
        move || {
            testnet_genesis(
                "5HHMY7e8UAqR5ZaHGaQnRW5EDR8dP7QpAyjeBu6V7vdXxxbf"
                    .parse()
                    .unwrap(),
                vec![
                    get_authority_keys_from_seed("Alice"),
                    get_authority_keys_from_seed("Bob"),
                    get_authority_keys_from_seed("Charlie"),
                ],
                vec!["5GTb3uLbk9VsyGD6taPyk69p2Hfa21GuzmMF52oJnqTQh2AA"
                    .parse()
                    .unwrap()],
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
            relay_chain: "relay-local".into(),
            para_id: id.into(),
        },
    )
}

fn testnet_genesis(
    root_key: AccountId,
    invulnerables: Vec<(AccountId, AuraId)>,
    oracle_accounts: Vec<AccountId>,
    endowed_accounts: Vec<AccountId>,
    id: ParaId,
) -> GenesisConfig {
    GenesisConfig {
        system: SystemConfig {
            code: WASM_BINARY
                .expect("WASM binary was not build, please build it!")
                .to_vec(),
            changes_trie_config: Default::default(),
        },
        balances: BalancesConfig {
            balances: {
                let mut endowed_accounts = endowed_accounts.clone();
                endowed_accounts.extend_from_slice(&oracle_accounts);

                endowed_accounts
                    .into_iter()
                    .map(|k| (k, 10_u128.pow(21)))
                    .collect()
            },
        },
        collator_selection: CollatorSelectionConfig {
            invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
            candidacy_bond: EXISTENTIAL_DEPOSIT * 16,
            desired_candidates: 16,
        },
        session: SessionConfig {
            keys: invulnerables
                .iter()
                .cloned()
                .map(|(acc, aura)| {
                    (
                        acc.clone(),          // account id
                        acc.clone(),          // validator id
                        SessionKeys { aura }, // session keys
                    )
                })
                .collect(),
        },
        aura: Default::default(),
        aura_ext: Default::default(),
        sudo: SudoConfig { key: root_key },
        parachain_info: ParachainInfoConfig { parachain_id: id },
        tokens: TokensConfig {
            balances: endowed_accounts
                .iter()
                .flat_map(|x| {
                    vec![
                        (x.clone(), CurrencyId::KSM, 10_u128.pow(15)),
                        (x.clone(), CurrencyId::USDT, 10_u128.pow(9)),
                    ]
                })
                .collect(),
        },
        loans: LoansConfig {
            borrow_index: Rate::one(),                             // 1
            exchange_rate: Rate::saturating_from_rational(2, 100), // 0.02
            markets: vec![
                (
                    CurrencyId::KSM,
                    Market {
                        close_factor: Ratio::from_percent(50),
                        collateral_factor: Ratio::from_percent(50),
                        liquidate_incentive: Rate::saturating_from_rational(110, 100),
                        state: MarketState::Active,
                        rate_model: InterestRateModel::Jump(JumpModel::new_model(
                            Rate::saturating_from_rational(2, 100),
                            Rate::saturating_from_rational(10, 100),
                            Rate::saturating_from_rational(32, 100),
                            Ratio::from_percent(80),
                        )),
                        reserve_factor: Ratio::from_percent(15),
                    },
                ),
                (
                    CurrencyId::USDT,
                    Market {
                        close_factor: Ratio::from_percent(50),
                        collateral_factor: Ratio::from_percent(50),
                        liquidate_incentive: Rate::saturating_from_rational(110, 100),
                        state: MarketState::Active,
                        rate_model: InterestRateModel::Jump(JumpModel::new_model(
                            Rate::saturating_from_rational(2, 100),
                            Rate::saturating_from_rational(10, 100),
                            Rate::saturating_from_rational(32, 100),
                            Ratio::from_percent(80),
                        )),
                        reserve_factor: Ratio::from_percent(15),
                    },
                ),
                (
                    CurrencyId::xKSM,
                    Market {
                        close_factor: Ratio::from_percent(50),
                        collateral_factor: Ratio::from_percent(50),
                        liquidate_incentive: Rate::saturating_from_rational(110, 100),
                        state: MarketState::Active,
                        rate_model: InterestRateModel::Jump(JumpModel::new_model(
                            Rate::saturating_from_rational(2, 100),
                            Rate::saturating_from_rational(10, 100),
                            Rate::saturating_from_rational(32, 100),
                            Ratio::from_percent(80),
                        )),
                        reserve_factor: Ratio::from_percent(15),
                    },
                ),
            ],
            last_block_timestamp: 0,
        },
        liquid_staking: LiquidStakingConfig {
            exchange_rate: Rate::saturating_from_rational(100, 100), // 1
            reserve_factor: Ratio::from_perthousand(5),
        },
        democracy: DemocracyConfig::default(),
        elections: ElectionsConfig {
            members: endowed_accounts
                .iter()
                .take((endowed_accounts.len() + 1) / 2)
                .cloned()
                .map(|member| (member, 0))
                .collect(),
        },
        council: CouncilConfig::default(),
        technical_committee: TechnicalCommitteeConfig {
            members: endowed_accounts
                .iter()
                .take((endowed_accounts.len() + 1) / 2)
                .cloned()
                .collect(),
            phantom: Default::default(),
        },
        technical_membership: Default::default(),
        treasury: Default::default(),
        oracle_membership: OracleMembershipConfig {
            members: oracle_accounts,
            phantom: Default::default(),
        },
    }
}
