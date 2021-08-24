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
    currency::EXISTENTIAL_DEPOSIT, opaque::SessionKeys, BalancesConfig, CollatorSelectionConfig,
    DemocracyConfig, GeneralCouncilConfig, GeneralCouncilMembershipConfig, GenesisConfig,
    LiquidStakingAgentMembershipConfig, LiquidStakingConfig, LoansConfig, OracleMembershipConfig,
    ParachainInfoConfig, SessionConfig, SudoConfig, SystemConfig,
    TechnicalCommitteeMembershipConfig, TokensConfig, ValidatorFeedersMembershipConfig,
    VestingConfig, WASM_BINARY,
};
use hex_literal::hex;
use primitives::*;
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{crypto::UncheckedInto, sr25519};
use sp_runtime::{
    traits::{One, Zero},
    FixedPointNumber,
};
#[cfg(feature = "std")]
use sp_std::collections::btree_map::BTreeMap;

use crate::chain_spec::{
    as_properties, get_account_id_from_seed, get_authority_keys_from_seed, Extensions,
    TELEMETRY_URL,
};

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

pub fn heiko_dev_config(id: ParaId) -> ChainSpec {
    ChainSpec::from_genesis(
        // Name
        "Parallel Heiko Dev",
        // ID
        "heiko-dev",
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
            let initial_allocation: Vec<(AccountId, Balance)> = vec![
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
            })
            .chain(
                invulnerables
                    .iter()
                    .cloned()
                    .map(|k| (k.0, EXISTENTIAL_DEPOSIT)),
            )
            .fold(
                BTreeMap::<AccountId, Balance>::new(),
                |mut acc, (account_id, amount)| {
                    if let Some(balance) = acc.get_mut(&account_id) {
                        *balance = balance.checked_add(amount).unwrap()
                    } else {
                        acc.insert(account_id.clone(), amount);
                    }
                    acc
                },
            )
            .into_iter()
            .collect::<Vec<(AccountId, Balance)>>();
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

            heiko_genesis(
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
        Some("heiko-dev"),
        Some(as_properties(network::NetworkType::Heiko)),
        Extensions {
            relay_chain: "rococo-local".into(),
            para_id: id.into(),
        },
    )
}

pub fn heiko_config(id: ParaId) -> ChainSpec {
    ChainSpec::from_genesis(
        // Name
        "Heiko",
        // ID
        "heiko",
        ChainType::Live,
        move || {
            let root_key: AccountId = "5CfaMb7d21Zh5wSthPXxLLj4D6sdb9YpdFKW8kM8cAdQ22fF"
                .parse()
                .unwrap();
            let invulnerables: Vec<(AccountId, AuraId)> = vec![(
                // 5GuwhbAaZd8bdkzSqSw1bpT9E86GH62DjLXaA55AdRtqFLG2
                hex!["d67e8f550de6438476394ba0908a711fffbdfeb7f2cfb5bcc0ff0a834160100a"].into(),
                hex!["d67e8f550de6438476394ba0908a711fffbdfeb7f2cfb5bcc0ff0a834160100a"]
                    .unchecked_into(),
            )];
            let oracle_accounts = vec![];
            let validator_feeders = vec![];
            let liquid_staking_agents = vec![];
            let initial_allocation: Vec<(AccountId, Balance)> = serde_json::from_str(include_str!(
                "../../../../resources/heiko-allocation-HKO.json"
            ))
            .unwrap();
            let initial_allocation: Vec<(AccountId, Balance)> = initial_allocation
                .iter()
                .cloned()
                .chain(
                    invulnerables
                        .iter()
                        .cloned()
                        .map(|k| (k.0, EXISTENTIAL_DEPOSIT)),
                )
                .fold(
                    BTreeMap::<AccountId, Balance>::new(),
                    |mut acc, (account_id, amount)| {
                        if let Some(balance) = acc.get_mut(&account_id) {
                            *balance = balance.checked_add(amount).unwrap()
                        } else {
                            acc.insert(account_id.clone(), amount);
                        }
                        acc
                    },
                )
                .into_iter()
                .collect::<Vec<(AccountId, Balance)>>();
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
            ];
            let technical_committee = vec!["1Gu7GSgLSPrhc1Wci9wAGP6nvzQfaUCYqbfXxjYjMG9bob6"
                .parse()
                .unwrap()];

            heiko_genesis(
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
        Some("heiko"),
        Some(as_properties(network::NetworkType::Heiko)),
        Extensions {
            relay_chain: "kusama".into(),
            para_id: id.into(),
        },
    )
}

fn heiko_genesis(
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
        serde_json::from_str(include_str!("../../../../resources/heiko-vesting-HKO.json")).unwrap();
    GenesisConfig {
        system: SystemConfig {
            code: WASM_BINARY
                .expect("WASM binary was not build, please build it!")
                .to_vec(),
            changes_trie_config: Default::default(),
        },
        balances: BalancesConfig {
            balances: initial_allocation,
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
        tokens: TokensConfig { balances: vec![] },
        loans: LoansConfig {
            borrow_index: Rate::one(),                             // 1
            exchange_rate: Rate::saturating_from_rational(2, 100), // 0.02
            last_block_timestamp: 0,
            markets: vec![],
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
