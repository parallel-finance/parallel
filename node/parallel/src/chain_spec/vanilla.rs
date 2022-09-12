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

use primitives::{network::NetworkType, *};
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{sr25519, H160, U256};
use sp_runtime::{traits::Zero, FixedPointNumber};
use std::{collections::BTreeMap, str::FromStr};
use vanilla_runtime::{
    opaque::SessionKeys, BalancesConfig, BaseFeeConfig, BridgeMembershipConfig,
    CollatorSelectionConfig, CrowdloansAutomatorsMembershipConfig, DemocracyConfig, EVMConfig,
    GeneralCouncilConfig, GeneralCouncilMembershipConfig, GenesisConfig,
    LiquidStakingAgentsMembershipConfig, LiquidStakingConfig, OracleMembershipConfig,
    ParachainInfoConfig, ParallelPrecompilesType, PolkadotXcmConfig, SessionConfig, SudoConfig,
    SystemConfig, TechnicalCommitteeMembershipConfig, VestingConfig, WASM_BINARY,
};

use crate::chain_spec::{
    accumulate, as_properties, get_account_id_from_seed, get_authority_keys_from_seed, Extensions,
    TELEMETRY_URL,
};

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

fn evm_accounts() -> Vec<AccountId> {
    vec![
        // the mapping SS58 of Alice
        // Secret seed: 0xe5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a
        // H160: 0x8097c3C354652CB1EEed3E5B65fBa2576470678A
        "hJGTRJgAGeMy9515iQ56VtfGKiGmejMkXmjKDRrxA46b7Z2So"
            .parse()
            .unwrap(),
        // others are all from ganache with MNEMONIC:
        //'into treat head shock search rule sheriff sword problem carpet exercise useful'
        "hJGGzG3onkMCMZbPMpLWe3jo3NQeDLtdHRedes6FtPjwEBjnk"
            .parse()
            .unwrap(),
        "hJJAtKQkAFTJKTgPQoMjAqafQeFWuNQujNUjC7xmtjguQLY8z"
            .parse()
            .unwrap(),
        "hJHkTHBNs2UbaLTYyNdRtGqKJfEt1g6qJTsoFSmnSwCMB1GiH"
            .parse()
            .unwrap(),
        "hJGRnsnw7JMJkbc4Jsa2AJyrEkdtBNLFFSpYgNiwpvsgsGS3s"
            .parse()
            .unwrap(),
        "hJJYGraYTku7dcvPY5eeGoXYDZC6ujRboQosFwMuNSMD9bEok"
            .parse()
            .unwrap(),
    ]
}

fn substrate_accounts() -> Vec<AccountId> {
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
}

pub fn local_development_config(id: ParaId) -> ChainSpec {
    ChainSpec::from_genesis(
        // Name
        "Vanilla Local Dev",
        // ID
        "vanilla-local-dev",
        ChainType::Development,
        move || {
            let root_key = get_account_id_from_seed::<sr25519::Public>("Alice");
            let invulnerables = vec![get_authority_keys_from_seed("Alice")];
            let oracle_accounts = vec![get_account_id_from_seed::<sr25519::Public>("Ferdie")];
            let bridge_accounts = vec![get_account_id_from_seed::<sr25519::Public>("Alice")];
            let liquid_staking_agents = vec![get_account_id_from_seed::<sr25519::Public>("Eve")];
            let crowdloans_automators = vec![get_account_id_from_seed::<sr25519::Public>("Bob")];
            let initial_allocation: Vec<(AccountId, Balance)> = accumulate(
                [substrate_accounts(), evm_accounts()]
                    .concat()
                    .iter()
                    .flat_map(|x| vec![(x.clone(), 10_u128.pow(24))]),
            );
            let vesting_list = vec![];
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
                initial_allocation,
                vesting_list,
                oracle_accounts,
                bridge_accounts,
                liquid_staking_agents,
                crowdloans_automators,
                council,
                technical_committee,
                id,
            )
        },
        vec![],
        TelemetryEndpoints::new(vec![(TELEMETRY_URL.into(), 0)]).ok(),
        Some("vanilla-local-dev"),
        None,
        Some(as_properties(NetworkType::Heiko)),
        Extensions {
            relay_chain: "kusama-local".into(),
            para_id: id.into(),
        },
    )
}

pub fn vanilla_dev_config(id: ParaId) -> ChainSpec {
    ChainSpec::from_genesis(
        // Name
        "Vanilla Dev",
        // ID
        "vanilla-dev",
        ChainType::Development,
        move || {
            let root_key = get_account_id_from_seed::<sr25519::Public>("Dave");
            let invulnerables = vec![get_authority_keys_from_seed("Alice")];
            let oracle_accounts = vec![get_account_id_from_seed::<sr25519::Public>("Ferdie")];
            let bridge_accounts = vec![get_account_id_from_seed::<sr25519::Public>("Alice")];
            let liquid_staking_agents = vec![get_account_id_from_seed::<sr25519::Public>("Eve")];
            let crowdloans_automators = vec![get_account_id_from_seed::<sr25519::Public>("Bob")];
            let initial_allocation: Vec<(AccountId, Balance)> = accumulate(
                [substrate_accounts(), evm_accounts()]
                    .concat()
                    .iter()
                    .flat_map(|x| vec![(x.clone(), 10_u128.pow(24))]),
            );
            let vesting_list = vec![];
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
                initial_allocation,
                vesting_list,
                oracle_accounts,
                bridge_accounts,
                liquid_staking_agents,
                crowdloans_automators,
                council,
                technical_committee,
                id,
            )
        },
        vec![],
        TelemetryEndpoints::new(vec![(TELEMETRY_URL.into(), 0)]).ok(),
        Some("vanilla-dev"),
        None,
        Some(as_properties(NetworkType::Heiko)),
        Extensions {
            relay_chain: "kusama-local".into(),
            para_id: id.into(),
        },
    )
}

fn vanilla_genesis(
    root_key: AccountId,
    invulnerables: Vec<(AccountId, AuraId)>,
    initial_allocation: Vec<(AccountId, Balance)>,
    vesting_list: Vec<(AccountId, BlockNumber, BlockNumber, u32, Balance)>,
    oracle_accounts: Vec<AccountId>,
    bridge_accounts: Vec<AccountId>,
    liquid_staking_agents: Vec<AccountId>,
    crowdloans_automators: Vec<AccountId>,
    council: Vec<AccountId>,
    technical_committee: Vec<AccountId>,
    id: ParaId,
) -> GenesisConfig {
    // This is supposed the be the simplest bytecode to revert without returning any data.
    // We will pre-deploy it under all of our precompiles to ensure they can be called from
    // within contracts.
    // (PUSH1 0x00 PUSH1 0x00 REVERT)
    let revert_bytecode = vec![0x60, 0x00, 0x60, 0x00, 0xFD];
    GenesisConfig {
        system: SystemConfig {
            code: WASM_BINARY
                .expect("WASM binary was not build, please build it!")
                .to_vec(),
        },
        balances: BalancesConfig {
            balances: initial_allocation,
        },
        collator_selection: CollatorSelectionConfig {
            invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
            candidacy_bond: Zero::zero(),
            desired_candidates: 16,
        },
        session: SessionConfig {
            keys: invulnerables
                .iter()
                .cloned()
                .map(|(acc, aura)| {
                    (
                        acc.clone(),          // account id
                        acc,                  // validator id
                        SessionKeys { aura }, // session keys
                    )
                })
                .collect(),
        },
        aura: Default::default(),
        aura_ext: Default::default(),
        parachain_system: Default::default(),
        sudo: SudoConfig {
            key: Some(root_key),
        },
        parachain_info: ParachainInfoConfig { parachain_id: id },
        liquid_staking: LiquidStakingConfig {
            exchange_rate: Rate::saturating_from_rational(100u32, 100u32), // 1
            reserve_factor: Ratio::from_rational(1u32, 10_000u32),         // 0.01%
        },
        democracy: DemocracyConfig::default(),
        general_council: GeneralCouncilConfig::default(),
        general_council_membership: GeneralCouncilMembershipConfig {
            members: council.try_into().unwrap(),
            phantom: Default::default(),
        },
        technical_committee: Default::default(),
        technical_committee_membership: TechnicalCommitteeMembershipConfig {
            members: technical_committee.try_into().unwrap(),
            phantom: Default::default(),
        },
        treasury: Default::default(),
        oracle_membership: OracleMembershipConfig {
            members: oracle_accounts.try_into().unwrap(),
            phantom: Default::default(),
        },
        bridge_membership: BridgeMembershipConfig {
            members: bridge_accounts.try_into().unwrap(),
            phantom: Default::default(),
        },
        liquid_staking_agents_membership: LiquidStakingAgentsMembershipConfig {
            members: liquid_staking_agents.try_into().unwrap(),
            phantom: Default::default(),
        },
        crowdloans_automators_membership: CrowdloansAutomatorsMembershipConfig {
            members: crowdloans_automators.try_into().unwrap(),
            phantom: Default::default(),
        },
        vesting: VestingConfig {
            vesting: vesting_list.try_into().unwrap(),
        },
        polkadot_xcm: PolkadotXcmConfig {
            safe_xcm_version: Some(2),
        },
        evm: EVMConfig {
            // We need _some_ code inserted at the precompile address so that
            // the evm will actually call the address.
            accounts: {
                let mut map: BTreeMap<H160, fp_evm::GenesisAccount> =
                    ParallelPrecompilesType::used_addresses()
                        .map(|addr| {
                            (
                                addr,
                                fp_evm::GenesisAccount {
                                    nonce: Default::default(),
                                    balance: Default::default(),
                                    storage: Default::default(),
                                    code: revert_bytecode.clone(),
                                },
                            )
                        })
                        .collect();
                map.insert(
                    // the mapping H160 of Alice
                    // Secret seed: 0xe5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a
                    // SS58: 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
                    // hex: 0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d
                    // Truncating to the first 20 bytes (the first 40 hex chars)
                    H160::from_str("d43593c715fdd31c61141abd04a99fd6822c8558")
                        .expect("internal H160 is valid; qed"),
                    fp_evm::GenesisAccount {
                        balance: U256::from_str("0xffffffffffffffffffffffffffffffff")
                            .expect("internal U256 is valid; qed"),
                        code: Default::default(),
                        nonce: Default::default(),
                        storage: Default::default(),
                    },
                );
                map
            },
        },
        base_fee: BaseFeeConfig::new(
            sp_core::U256::from(1_000_000_000),
            false,
            sp_runtime::Permill::from_parts(125_000),
        ),
        ethereum: Default::default(),
    }
}
