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
    opaque::SessionKeys, BalancesConfig, CollatorSelectionConfig, DemocracyConfig,
    GeneralCouncilConfig, GeneralCouncilMembershipConfig, GenesisConfig,
    LiquidStakingAgentMembershipConfig, LiquidStakingConfig, LoansConfig, OracleMembershipConfig,
    ParachainInfoConfig, SessionConfig, SudoConfig, SystemConfig,
    TechnicalCommitteeMembershipConfig, TokensConfig, ValidatorFeedersMembershipConfig,
    VestingConfig, WASM_BINARY,
};

// use hex_literal::hex;
use primitives::*;
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
// use sp_core::crypto::UncheckedInto;

use sp_core::sr25519;
use sp_runtime::{
    traits::{One, Zero},
    FixedPointNumber,
};

use crate::chain_spec::{
    accumulate, as_properties, get_account_id_from_seed, get_authority_keys_from_seed, Extensions,
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
            relay_chain: "westend-local".into(),
            para_id: id.into(),
        },
    )
}

pub fn heiko_config(_id: ParaId) -> Result<ChainSpec, String> {
    ChainSpec::from_json_bytes(&include_bytes!("../../../../resources/specs/heiko.json")[..])
    // Ok(ChainSpec::from_genesis(
    //     // Name
    //     "Parallel Heiko",
    //     // ID
    //     "heiko",
    //     ChainType::Live,
    //     move || {
    //         let root_key: AccountId = "5CLbxwBcUf8PG4zzf56w27YwwJzkyGv4ULsBNfkCBGEdRGKv"
    //             .parse()
    //             .unwrap();
    //         let invulnerables: Vec<(AccountId, AuraId)> = vec![
    //             (
    //                 // 5DfKxDtYyHkWnXkoc8Ek9KaPZE3FBD5kDByDziiRtHsd8D1x
    //                 hex!["46a4161c87a0c6d58dec1e01b8c360123e1373ffafcf100efd1a9999fbacf161"].into(),
    //                 hex!["46a4161c87a0c6d58dec1e01b8c360123e1373ffafcf100efd1a9999fbacf161"]
    //                     .unchecked_into(),
    //             ),
    //             (
    //                 // 5EUmwapW8qScFGh4KGug1xb5Dnm4FYQtzrjTcvjynyRAMRR3
    //                 hex!["6ad41b69e5ff9ec7fa541b9e61f56bc9dd5761e8ab69cf82a3c0722ba227dc5e"].into(),
    //                 hex!["6ad41b69e5ff9ec7fa541b9e61f56bc9dd5761e8ab69cf82a3c0722ba227dc5e"]
    //                     .unchecked_into(),
    //             ),
    //             (
    //                 // 5DJd3duMMEeEo9Gi5az1esvuNRB31V8Fds91VkBMrZUCFyUn
    //                 hex!["36d97965e462e9ca63079c1102db04f4293e59bca83713703a9a772d0017894d"].into(),
    //                 hex!["36d97965e462e9ca63079c1102db04f4293e59bca83713703a9a772d0017894d"]
    //                     .unchecked_into(),
    //             ),
    //         ];
    //         let oracle_accounts = vec![];
    //         let validator_feeders = vec![];
    //         let liquid_staking_agents = vec![];
    //         let initial_allocation: Vec<(AccountId, Balance)> = serde_json::from_str(include_str!(
    //             "../../../../resources/heiko-allocation-HKO.json"
    //         ))
    //         .unwrap();
    //         let initial_allocation: Vec<(AccountId, Balance)> = accumulate(initial_allocation);
    //         let council = vec![];
    //         let technical_committee = vec![];
    //
    //         heiko_genesis(
    //             root_key,
    //             invulnerables,
    //             oracle_accounts,
    //             initial_allocation,
    //             validator_feeders,
    //             liquid_staking_agents,
    //             council,
    //             technical_committee,
    //             id,
    //         )
    //     },
    //     vec![
    //         "/dns/heiko-bootnode-0.parallel.fi/tcp/30333/p2p/12D3KooWLUTzbrJJDowUKMPfEZrDY6eH8HXvm8hrG6YrdUmdrKPz".parse().unwrap(),
    //         "/dns/heiko-bootnode-1.parallel.fi/tcp/30333/p2p/12D3KooWEckTASdnkQC8MfBNnzKGfQJmdmzCBWrwra26nTqY4Hmu".parse().unwrap(),
    //         "/dns/heiko-bootnode-2.parallel.fi/tcp/30333/p2p/12D3KooWFJe4LfS15nTBUduq3cMKmHEWwKYrJFmMnAa7wT5W1eZE".parse().unwrap(),
    //     ],
    //     TelemetryEndpoints::new(vec![(TELEMETRY_URL.into(), 0)]).ok(),
    //     Some("heiko"),
    //     Some(as_properties(network::NetworkType::Heiko)),
    //     Extensions {
    //         relay_chain: "kusama".into(),
    //         para_id: id.into(),
    //     },
    // ))
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
