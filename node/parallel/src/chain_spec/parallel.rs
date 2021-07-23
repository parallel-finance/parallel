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
use heiko_runtime::pallet_loans::{InterestRateModel, JumpModel, Market, MarketState};
use hex_literal::hex;
use parallel_runtime::{
    opaque::SessionKeys, BalancesConfig, CollatorSelectionConfig, CouncilConfig, DemocracyConfig,
    ElectionsConfig, GenesisConfig, LiquidStakingAgentMembershipConfig, LiquidStakingConfig,
    LoansConfig, OracleMembershipConfig, ParachainInfoConfig, SessionConfig, SudoConfig,
    SystemConfig, TechnicalCommitteeConfig, TokensConfig, ValidatorFeedersMembershipConfig,
    VestingConfig, WASM_BINARY,
};
use primitives::*;
use sc_service::ChainType;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::crypto::UncheckedInto;
use sp_core::sr25519;
use sp_runtime::{
    traits::{One, Zero},
    FixedPointNumber,
};

use crate::chain_spec::{get_account_id_from_seed, get_authority_keys_from_seed, Extensions};

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

pub fn development_config(id: ParaId) -> ChainSpec {
    ChainSpec::from_genesis(
        // Name
        "Parallel Development",
        // ID
        "parallel-dev",
        ChainType::Development,
        move || {
            testnet_genesis(
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                vec![
                    get_authority_keys_from_seed("Alice"),
                    get_authority_keys_from_seed("Bob"),
                    get_authority_keys_from_seed("Charlie"),
                ],
                vec![get_account_id_from_seed::<sr25519::Public>("Ferdie")],
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Dave"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
                ],
                vec![get_account_id_from_seed::<sr25519::Public>("Eve")],
                // Multisig account combined by Alice, Bob and Charile, ss58 prefix is 42
                vec!["5DjYJStmdZ2rcqXbXGX7TW85JsrW6uG4y9MUcLq2BoPMpRA7"
                    .parse()
                    .unwrap()],
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

pub fn local_testnet_config(id: ParaId) -> ChainSpec {
    ChainSpec::from_genesis(
        // Name
        "Parallel Testnet",
        // ID
        "parallel-local",
        ChainType::Local,
        move || {
            testnet_genesis(
                "5HHMY7e8UAqR5ZaHGaQnRW5EDR8dP7QpAyjeBu6V7vdXxxbf"
                    .parse()
                    .unwrap(),
                vec![
                    (
                        // 5DFScwjDYWMG7oAcotAaWdxnjZKyBd3PvG7QbzMCisEWPquY
                        hex!["346ca44fd617b87fcd050c4a4bb7ef369a2e2e4d7f44233ab12b9bea56290461"]
                            .into(),
                        hex!["346ca44fd617b87fcd050c4a4bb7ef369a2e2e4d7f44233ab12b9bea56290461"]
                            .unchecked_into(),
                    ),
                    (
                        // 5GEwvVsMiZvLY9TsRYVd9NUuTUuAHCEL7uX1GTrLufXf8pKV
                        hex!["b8c0bd039e40de150100a5c7c7dce7e5e2a3006ff4147cdc7caedb7ef0092b76"]
                            .into(),
                        hex!["b8c0bd039e40de150100a5c7c7dce7e5e2a3006ff4147cdc7caedb7ef0092b76"]
                            .unchecked_into(),
                    ),
                    (
                        // 5EbjqR169aiZibNdzMRMcJGjh8fLyXWtA5RSMJRonpdMjunU
                        hex!["7023bbf7ff4780bef4b34759f6df004a341e6e5893df7d74a83b91af2055203d"]
                            .into(),
                        hex!["7023bbf7ff4780bef4b34759f6df004a341e6e5893df7d74a83b91af2055203d"]
                            .unchecked_into(),
                    ),
                ],
                vec!["5GTb3uLbk9VsyGD6taPyk69p2Hfa21GuzmMF52oJnqTQh2AA"
                    .parse()
                    .unwrap()],
                vec![
                    // Parallel team accounts
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
                ],
                vec!["5FjH9a7RQmihmb7i4UzbNmecjPm9WVLyoJHfsixkrLGEKwsJ"
                    .parse()
                    .unwrap()],
                // Parallel team accounts, ss58 prefix is 42
                vec!["5HHMY7e8UAqR5ZaHGaQnRW5EDR8dP7QpAyjeBu6V7vdXxxbf"
                    .parse()
                    .unwrap()],
                id,
            )
        },
        vec![],
        None,
        None,
        None,
        Extensions {
            relay_chain: "polkadot".into(),
            para_id: id.into(),
        },
    )
}

fn testnet_genesis(
    root_key: AccountId,
    invulnerables: Vec<(AccountId, AuraId)>,
    oracle_accounts: Vec<AccountId>,
    endowed_accounts: Vec<AccountId>,
    validator_feeders: Vec<AccountId>,
    liquid_staking_agents: Vec<AccountId>,
    id: ParaId,
) -> GenesisConfig {
    let vesting_list: Vec<(AccountId, BlockNumber, BlockNumber, u32, Balance)> =
        serde_json::from_str(include_str!(
            "../../../../resources/parallel-vesting-PARA.json"
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
            balances: {
                let mut endowed_accounts = endowed_accounts.clone();
                endowed_accounts.extend_from_slice(&oracle_accounts);
                endowed_accounts.extend_from_slice(&validator_feeders);
                endowed_accounts.extend(
                    invulnerables
                        .iter()
                        .map(|invulnerable| invulnerable.0.clone()),
                );

                endowed_accounts
                    .into_iter()
                    .map(|k| (k, 10_u128.pow(13)))
                    .collect()
            },
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
        sudo: SudoConfig { key: root_key },
        parachain_info: ParachainInfoConfig { parachain_id: id },
        tokens: TokensConfig {
            balances: endowed_accounts
                .iter()
                .flat_map(|x| {
                    vec![
                        (x.clone(), CurrencyId::DOT, 10_u128.pow(13)),
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
                    CurrencyId::DOT,
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
                    CurrencyId::xDOT,
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
