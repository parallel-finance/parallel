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
use hex_literal::hex;
use primitives::{network::NetworkType, *};
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{crypto::UncheckedInto, sr25519};
use sp_runtime::{
    traits::{One, Zero},
    FixedPointNumber,
};
use vanilla_runtime::{
    opaque::SessionKeys,
    pallet_loans::{InterestRateModel, JumpModel, Market, MarketState},
    BalancesConfig, CollatorSelectionConfig, DemocracyConfig, GeneralCouncilConfig,
    GeneralCouncilMembershipConfig, GenesisConfig, LiquidStakingAgentMembershipConfig,
    LiquidStakingConfig, LoansConfig, OracleMembershipConfig, ParachainInfoConfig, SessionConfig,
    SudoConfig, SystemConfig, TechnicalCommitteeMembershipConfig, TokensConfig,
    ValidatorFeedersMembershipConfig, VestingConfig, WASM_BINARY,
};

use crate::chain_spec::{
    accumulate, as_properties, get_account_id_from_seed, get_authority_keys_from_seed, Extensions,
    TELEMETRY_URL,
};

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

pub fn vanilla_dev_config(id: ParaId) -> ChainSpec {
    ChainSpec::from_genesis(
        // Name
        "Vanilla Dev",
        // ID
        "vanilla-dev",
        ChainType::Development,
        move || {
            let root_key = get_account_id_from_seed::<sr25519::Public>("Dave");
            let invulnerables = vec![
                get_authority_keys_from_seed("Alice"),
                get_authority_keys_from_seed("Bob"),
                get_authority_keys_from_seed("Charlie"),
            ];
            let oracle_accounts = vec![get_account_id_from_seed::<sr25519::Public>("Ferdie")];
            let validator_feeders = vec![get_account_id_from_seed::<sr25519::Public>("Eve")];
            let liquid_staking_agents = vec![get_account_id_from_seed::<sr25519::Public>("Dave")];
            let initial_allocation: Vec<(AccountId, Balance)> = accumulate(
                vec![
                    // Faucet accounts
                    "5HHMY7e8UAqR5ZaHGaQnRW5EDR8dP7QpAyjeBu6V7vdXxxbf"
                        .parse()
                        .unwrap(),
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
                ]
                .iter()
                .flat_map(|x| {
                    if x == &"5HHMY7e8UAqR5ZaHGaQnRW5EDR8dP7QpAyjeBu6V7vdXxxbf"
                        .parse()
                        .unwrap()
                    {
                        vec![(x.clone(), 10_u128.pow(20))]
                    } else {
                        vec![(x.clone(), 10_u128.pow(16))]
                    }
                }),
            );
            let council = vec![
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                get_account_id_from_seed::<sr25519::Public>("Bob"),
                get_account_id_from_seed::<sr25519::Public>("Charlie"),
            ];
            let technical_committee = vec![
                get_account_id_from_seed::<sr25519::Public>("Dave"),
                get_account_id_from_seed::<sr25519::Public>("Eve"),
                get_account_id_from_seed::<sr25519::Public>("Ferdie"),
            ];

            vanilla_genesis(
                root_key,
                invulnerables,
                oracle_accounts,
                initial_allocation,
                validator_feeders,
                liquid_staking_agents,
                council,
                technical_committee,
                id,
            )
        },
        vec![],
        TelemetryEndpoints::new(vec![(TELEMETRY_URL.into(), 0)]).ok(),
        Some("vanilla-dev"),
        Some(as_properties(NetworkType::Heiko)),
        Extensions {
            relay_chain: "westend-local".into(),
            para_id: id.into(),
        },
    )
}

pub fn vanilla_local_testnet_config(id: ParaId) -> ChainSpec {
    ChainSpec::from_genesis(
        // Name
        "Vanilla Local Testnet",
        // ID
        "vanilla-local",
        ChainType::Local,
        move || {
            let root_key = "5E5BxCjexvzgH9LsYUzMjD6gJaWiKkmadvjsHFPZmrXrK7Rf"
                .parse()
                .unwrap();
            let invulnerables: Vec<(AccountId, AuraId)> = vec![(
                // 5E5BxCjexvzgH9LsYUzMjD6gJaWiKkmadvjsHFPZmrXrK7Rf//collator
                hex!["0af2d7eedd51f8ef80ad82140631d58582034a6108f350d7ab04ded94c9d253f"].into(),
                hex!["0af2d7eedd51f8ef80ad82140631d58582034a6108f350d7ab04ded94c9d253f"]
                    .unchecked_into(),
            )];
            // 5E5BxCjexvzgH9LsYUzMjD6gJaWiKkmadvjsHFPZmrXrK7Rf//oracle
            let oracle_accounts = vec!["5DCp1LrGsmtPj5R1jLnxC5kPvSYJTgY1zxbF3TzrHHMoqsvt"
                .parse()
                .unwrap()];
            // 5E5BxCjexvzgH9LsYUzMjD6gJaWiKkmadvjsHFPZmrXrK7Rf//validator_feeder
            let validator_feeders = vec!["5CQ5wx8MYTWGzHY3DUspGBfMnyuMg2L84bWyxJJNq7AugUj6"
                .parse()
                .unwrap()];
            // 5E5BxCjexvzgH9LsYUzMjD6gJaWiKkmadvjsHFPZmrXrK7Rf//agent
            let liquid_staking_agents = vec!["5FFCMw8pyzK7QN9jiMeAg8fD45o5dVU1PGcJDAeCC94GVVWC"
                .parse()
                .unwrap()];
            let initial_allocation = accumulate(
                vec![
                    // Faucet accounts
                    "5HHMY7e8UAqR5ZaHGaQnRW5EDR8dP7QpAyjeBu6V7vdXxxbf"
                        .parse()
                        .unwrap(),
                    // Team members accounts
                    "5G4fc9GN6DeFQm4h2HKq3d9hBTsBJWSLWkyuk35cKHh2sqEz"
                        .parse()
                        .unwrap(),
                    "5DhZeTQqotvntGtrg69T2VK9pzUPXHiVyGUTmp5XFTDTT7ME"
                        .parse()
                        .unwrap(),
                    "5GBykvvrUz3vwTttgHzUEPdm7G1FND1reBfddQLdiaCbhoMd"
                        .parse()
                        .unwrap(),
                    "5G3f6iLDU6mbyEiJH8icoLhFy4RZ6TvWUZSkDwtg1nXTV3QK"
                        .parse()
                        .unwrap(),
                    "5G97JLuuT1opraWvfS6Smt4jaAZuyDquP9GjamKVcPC366qU"
                        .parse()
                        .unwrap(),
                    "5G9eFoXB95fdwFJK9utBf1AgiLvhPUvzArYR2knzXKrKtZPZ"
                        .parse()
                        .unwrap(),
                    "1Gu7GSgLSPrhc1Wci9wAGP6nvzQfaUCYqbfXxjYjMG9bob6"
                        .parse()
                        .unwrap(),
                ]
                .iter()
                .flat_map(|x: &AccountId| {
                    if x == &"5HHMY7e8UAqR5ZaHGaQnRW5EDR8dP7QpAyjeBu6V7vdXxxbf"
                        .parse()
                        .unwrap()
                    {
                        vec![(x.clone(), 10_u128.pow(20))]
                    } else {
                        vec![(x.clone(), 10_u128.pow(16))]
                    }
                }),
            );
            let council = vec![
                "5G3f6iLDU6mbyEiJH8icoLhFy4RZ6TvWUZSkDwtg1nXTV3QK"
                    .parse()
                    .unwrap(),
                "5GBykvvrUz3vwTttgHzUEPdm7G1FND1reBfddQLdiaCbhoMd"
                    .parse()
                    .unwrap(),
                "5DhZeTQqotvntGtrg69T2VK9pzUPXHiVyGUTmp5XFTDTT7ME"
                    .parse()
                    .unwrap(),
                "1Gu7GSgLSPrhc1Wci9wAGP6nvzQfaUCYqbfXxjYjMG9bob6"
                    .parse()
                    .unwrap(),
            ];
            let technical_committee = vec![];

            vanilla_genesis(
                root_key,
                invulnerables,
                oracle_accounts,
                initial_allocation,
                validator_feeders,
                liquid_staking_agents,
                council,
                technical_committee,
                id,
            )
        },
        vec![],
        TelemetryEndpoints::new(vec![(TELEMETRY_URL.into(), 0)]).ok(),
        Some("vanilla-local"),
        Some(as_properties(NetworkType::Heiko)),
        Extensions {
            relay_chain: "westend".into(),
            para_id: id.into(),
        },
    )
}

fn vanilla_genesis(
    root_key: AccountId,
    invulnerables: Vec<(AccountId, AuraId)>,
    oracle_accounts: Vec<AccountId>,
    initial_allocation: Vec<(AccountId, Balance)>,
    validator_feeders: Vec<AccountId>,
    liquid_staking_agents: Vec<AccountId>,
    council: Vec<AccountId>,
    technical_committee: Vec<AccountId>,
    id: ParaId,
) -> GenesisConfig {
    let vesting_list: Vec<(AccountId, BlockNumber, BlockNumber, u32, Balance)> =
        serde_json::from_str(include_str!(
            "../../../../resources/vanilla-vesting-HKO.json"
        ))
        .unwrap();
    GenesisConfig {
        system: SystemConfig {
            code: WASM_BINARY
                .expect("WASM binary was not build, please build it!")
                .to_vec(),
            changes_trie_config: Default::default(),
        },
        balances: BalancesConfig {
            balances: initial_allocation.clone(),
        },
        collator_selection: CollatorSelectionConfig {
            invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
            candidacy_bond: Zero::zero(),
            desired_candidates: 16,
            ..Default::default()
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
        parachain_system: Default::default(),
        sudo: SudoConfig { key: root_key },
        parachain_info: ParachainInfoConfig { parachain_id: id },
        tokens: TokensConfig {
            balances: initial_allocation
                .iter()
                .flat_map(|(x, _)| {
                    if x == &"5HHMY7e8UAqR5ZaHGaQnRW5EDR8dP7QpAyjeBu6V7vdXxxbf"
                        .parse()
                        .unwrap()
                    {
                        vec![(x.clone(), CurrencyId::USDT, 10_u128.pow(20))]
                    } else {
                        vec![(x.clone(), CurrencyId::USDT, 10_u128.pow(9))]
                    }
                })
                .collect(),
        },
        loans: LoansConfig {
            borrow_index: Rate::one(),                             // 1
            exchange_rate: Rate::saturating_from_rational(2, 100), // 0.02
            last_block_timestamp: 0,
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
        },
        liquid_staking: LiquidStakingConfig {
            exchange_rate: Rate::saturating_from_rational(100, 100), // 1
            reserve_factor: Ratio::from_perthousand(5),
        },
        democracy: DemocracyConfig::default(),
        general_council: GeneralCouncilConfig::default(),
        general_council_membership: GeneralCouncilMembershipConfig {
            members: council,
            phantom: Default::default(),
        },
        technical_committee: Default::default(),
        technical_committee_membership: TechnicalCommitteeMembershipConfig {
            members: technical_committee,
            phantom: Default::default(),
        },
        treasury: Default::default(),
        oracle_membership: OracleMembershipConfig {
            members: oracle_accounts,
            phantom: Default::default(),
        },
        liquid_staking_agent_membership: LiquidStakingAgentMembershipConfig {
            members: liquid_staking_agents,
            phantom: Default::default(),
        },
        validator_feeders_membership: ValidatorFeedersMembershipConfig {
            members: validator_feeders,
            phantom: Default::default(),
        },
        vesting: VestingConfig {
            vesting: vesting_list,
        },
    }
}
