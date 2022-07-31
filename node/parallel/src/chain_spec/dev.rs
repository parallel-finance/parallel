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
#[allow(dead_code, unused)]
use dev_runtime::{
    opaque::SessionKeys, AuraConfig, BalancesConfig, BaseFeeConfig, BridgeMembershipConfig,
    CollatorSelectionConfig, CrowdloansAutomatorsMembershipConfig, DemocracyConfig, EVMConfig,
    GeneralCouncilConfig, GeneralCouncilMembershipConfig, GenesisConfig, GrandpaConfig, GrandpaId,
    LiquidStakingAgentsMembershipConfig, LiquidStakingConfig, OracleMembershipConfig,
    ParachainInfoConfig, PolkadotXcmConfig, Precompiles, SessionConfig, SudoConfig, SystemConfig,
    TechnicalCommitteeMembershipConfig, VestingConfig, WASM_BINARY,
};
use primitives::{network::NetworkType, *};
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{sr25519, Pair, Public};
use sp_runtime::traits::Zero;
use sp_runtime::FixedPointNumber;

use crate::chain_spec::{
    accumulate, as_properties, get_account_id_from_seed, Extensions, TELEMETRY_URL,
};

/// Helper function to generate a crypto pair from seed
fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> (AccountId, AuraId, GrandpaId) {
    (
        get_account_id_from_seed::<sr25519::Public>(s),
        get_from_seed::<AuraId>(s),
        get_from_seed::<GrandpaId>(s),
    )
}

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

pub fn development_config(id: ParaId) -> ChainSpec {
    ChainSpec::from_genesis(
        // Name
        "Vanilla Local Dev",
        // ID
        "vanilla-local-dev",
        ChainType::Development,
        move || {
            let root_key = get_account_id_from_seed::<sr25519::Public>("Alice");
            let invulnerables = vec![authority_keys_from_seed("Alice")];
            let oracle_accounts = vec![get_account_id_from_seed::<sr25519::Public>("Ferdie")];
            let bridge_accounts = vec![get_account_id_from_seed::<sr25519::Public>("Alice")];
            let liquid_staking_agents = vec![get_account_id_from_seed::<sr25519::Public>("Eve")];
            let crowdloans_automators = vec![get_account_id_from_seed::<sr25519::Public>("Bob")];
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

            development_genesis(
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

fn development_genesis(
    root_key: AccountId,
    invulnerables: Vec<(AccountId, AuraId, GrandpaId)>,
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
            invulnerables: invulnerables
                .iter()
                .cloned()
                .map(|(acc, _, _)| acc)
                .collect(),
            candidacy_bond: Zero::zero(),
            desired_candidates: 16,
        },
        session: SessionConfig {
            keys: invulnerables
                .iter()
                .cloned()
                .map(|(acc, aura, grandpa)| {
                    (
                        acc.clone(),                   // account id
                        acc,                           // validator id
                        SessionKeys { aura, grandpa }, // session keys
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
        grandpa: Default::default(),
        evm: EVMConfig {
            // We need _some_ code inserted at the precompile address so that
            // the evm will actually call the address.
            accounts: Precompiles::used_addresses()
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
                .collect(),
        },
        base_fee: BaseFeeConfig::new(
            sp_core::U256::from(1_000_000_000),
            false,
            sp_runtime::Permill::from_parts(125_000),
        ),
        ethereum: Default::default(),
    }
}
