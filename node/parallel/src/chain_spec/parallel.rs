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
use parallel_runtime::{
    opaque::SessionKeys, BalancesConfig, CollatorSelectionConfig, DemocracyConfig,
    GeneralCouncilConfig, GeneralCouncilMembershipConfig, GenesisConfig,
    LiquidStakingAgentMembershipConfig, LiquidStakingConfig, OracleMembershipConfig,
    ParachainInfoConfig, PolkadotXcmConfig, SessionConfig, SudoConfig, SystemConfig,
    TechnicalCommitteeMembershipConfig, ValidatorFeedersMembershipConfig, VestingConfig,
    WASM_BINARY,
};
use primitives::{network::NetworkType, *};
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{crypto::UncheckedInto, sr25519};
use sp_runtime::{traits::Zero, FixedPointNumber};

use crate::chain_spec::{
    accumulate, as_properties, get_account_id_from_seed, get_authority_keys_from_seed, Extensions,
    TELEMETRY_URL,
};

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

pub fn parallel_dev_config(id: ParaId) -> ChainSpec {
    ChainSpec::from_genesis(
        // Name
        "Parallel Dev",
        // ID
        "parallel-dev",
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

            parallel_genesis(
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
        Some("parallel-dev"),
        Some(as_properties(NetworkType::Parallel)),
        Extensions {
            relay_chain: "westend-local".into(),
            para_id: id.into(),
        },
    )
}

pub fn parallel_config(id: ParaId) -> Result<ChainSpec, String> {
    // ChainSpec::from_json_bytes(&include_bytes!("../../../../resources/specs/parallel.json")[..])
    Ok(ChainSpec::from_genesis(
        // Name
        "Parallel",
        // ID
        "parallel",
        ChainType::Live,
        move || {
            let root_key: AccountId = "5GpSUyyzqeL55TLDrHLifYcvxshzqQ9c4JaogkEcuVvirE3w"
                .parse()
                .unwrap();
            let invulnerables: Vec<(AccountId, AuraId)> = vec![
                (
                    // 5EA4X7f81kBRVRtVH6qotiQPmdTSw6oqLqNGuRqHxbLyUhAf
                    hex!["5c8e4059d8eeef6e9cd387961c38a0a28a8a713190ca995a0c2e9dd4d926f07e"].into(),
                    hex!["5c8e4059d8eeef6e9cd387961c38a0a28a8a713190ca995a0c2e9dd4d926f07e"]
                        .unchecked_into(),
                ),
                (
                    // 5GNYurfysPCrD8Y1CFd8Lc9CJJRCNPYhJEufeGfLJ3gQe5d5
                    hex!["be8d3d4c7781682236df1e068e1746024c958d12df9f84e025d7e4c3f7126404"].into(),
                    hex!["be8d3d4c7781682236df1e068e1746024c958d12df9f84e025d7e4c3f7126404"]
                        .unchecked_into(),
                ),
                (
                    // 5GzmRkqBrfryUbtwf2n694H9Nm75ubCSPsVQf7zmYDHw6fR5
                    hex!["da2c3477a12743c98e734f14a100fd8aa6885d5d4f9ce32a5ce0f9602500547e"].into(),
                    hex!["da2c3477a12743c98e734f14a100fd8aa6885d5d4f9ce32a5ce0f9602500547e"]
                        .unchecked_into(),
                ),
                (
                    // 5DHu97jdzpTk2anCRc2QRvvZ9d2f2e1SxbATPJsEHfqrAF49
                    hex!["364c685c411c72d90718c71c305a479854bb0d49c439bf51f4ea4f1317d6c969"].into(),
                    hex!["364c685c411c72d90718c71c305a479854bb0d49c439bf51f4ea4f1317d6c969"]
                        .unchecked_into(),
                ),
                (
                    // 5Hp61nKPbaPt3GGxxmMtkfT6t3Tt6K7996RWuCwxbmwmQ7bs
                    hex!["fe434ea4283ee8c49e8aeda990698e3815fc78b0c46529154c7dcba462f7e33a"].into(),
                    hex!["fe434ea4283ee8c49e8aeda990698e3815fc78b0c46529154c7dcba462f7e33a"]
                        .unchecked_into(),
                ),
            ];
            let oracle_accounts = vec![];
            let validator_feeders = vec![];
            let liquid_staking_agents = vec![];
            let initial_allocation: Vec<(AccountId, Balance)> = serde_json::from_str(include_str!(
                "../../../../resources/parallel-allocation-PARA.json"
            ))
            .unwrap();
            let initial_allocation: Vec<(AccountId, Balance)> = accumulate(initial_allocation);
            let council = vec![];
            let technical_committee = vec![];

            parallel_genesis(
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
        vec![
            "/dns/bootnode-0.parallel.fi/tcp/30333/p2p/12D3KooWNngQxhrT19QqK2dCPtCQb5kB92RscWMnPfNxCC1sgr3N".parse().unwrap(),
            "/dns/bootnode-1.parallel.fi/tcp/30333/p2p/12D3KooWMzctxpmtti9dWsPaosh2cPCBZFUGeQhmT6W1ynErwKKB".parse().unwrap(),
            "/dns/bootnode-2.parallel.fi/tcp/30333/p2p/12D3KooWAWRTCjiVo3VoSZYMCwKk6CQCSLTqVVjBnbWhvp71Ey6Y".parse().unwrap(),
            "/dns/bootnode-3.parallel.fi/tcp/30333/p2p/12D3KooWSMKQCs6JXjVdaqBSyoMZLBNWrLjJ3QzTET7Zd7kWoB8G".parse().unwrap(),
            "/dns/bootnode-4.parallel.fi/tcp/30333/p2p/12D3KooWCAhW29HjprkLmQ39gCTJmHsEWSqLXPkCz27qVbsGjpLk".parse().unwrap(),
        ],
        TelemetryEndpoints::new(vec![(TELEMETRY_URL.into(), 0)]).ok(),
        Some("parallel"),
        Some(as_properties(network::NetworkType::Parallel)),
        Extensions {
            relay_chain: "polkadot".into(),
            para_id: id.into(),
        },
    ))
}

fn parallel_genesis(
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
        polkadot_xcm: PolkadotXcmConfig {
            safe_xcm_version: Some(2),
        },
    }
}
