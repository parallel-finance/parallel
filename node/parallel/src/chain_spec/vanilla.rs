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
use primitives::{network::NetworkType, *};
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
// use hex_literal::hex;
// use sp_core::crypto::UncheckedInto;
use sp_core::sr25519;
use sp_runtime::{traits::Zero, FixedPointNumber};
use vanilla_runtime::{
    opaque::SessionKeys, BalancesConfig, CollatorSelectionConfig, DemocracyConfig,
    GeneralCouncilConfig, GeneralCouncilMembershipConfig, GenesisConfig, LiquidStakingConfig,
    OracleMembershipConfig, ParachainInfoConfig, PolkadotXcmConfig, SessionConfig, SudoConfig,
    SystemConfig, TechnicalCommitteeMembershipConfig, ValidatorFeedersMembershipConfig,
    VestingConfig, WASM_BINARY,
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

pub fn vanilla_config(_id: ParaId) -> Result<ChainSpec, String> {
    ChainSpec::from_json_bytes(&include_bytes!("../../../../resources/specs/vanilla.json")[..])
    // Ok(ChainSpec::from_genesis(
    //     // Name
    //     "Vanilla",
    //     // ID
    //     "vanilla",
    //     ChainType::Live,
    //     move || {
    //         let root_key = "5E5BxCjexvzgH9LsYUzMjD6gJaWiKkmadvjsHFPZmrXrK7Rf"
    //             .parse()
    //             .unwrap();
    //         let invulnerables: Vec<(AccountId, AuraId)> = vec![
    //             (
    //                 // 5E5BxCjexvzgH9LsYUzMjD6gJaWiKkmadvjsHFPZmrXrK7Rf//collator1
    //                 hex!["1a5dd54d1cef45e6140b54f3b83fdbbf41fec82645ad826d4f8cf106c88dd00e"].into(),
    //                 hex!["1a5dd54d1cef45e6140b54f3b83fdbbf41fec82645ad826d4f8cf106c88dd00e"]
    //                     .unchecked_into(),
    //             ),
    //             (
    //                 // 5E5BxCjexvzgH9LsYUzMjD6gJaWiKkmadvjsHFPZmrXrK7Rf//collator2
    //                 hex!["c6b255117d87f959c4e564888dc4987e0c3c35a60872a7fac4c38d771b39b70c"].into(),
    //                 hex!["c6b255117d87f959c4e564888dc4987e0c3c35a60872a7fac4c38d771b39b70c"]
    //                     .unchecked_into(),
    //             ),
    //         ];
    //         // 5E5BxCjexvzgH9LsYUzMjD6gJaWiKkmadvjsHFPZmrXrK7Rf//oracle1
    //         let oracle_accounts = vec!["5EUHNqqv9DTieD5582b1MWuVhfztzsCLjcawp6mBDYxL2sb6"
    //             .parse()
    //             .unwrap()];
    //         // 5E5BxCjexvzgH9LsYUzMjD6gJaWiKkmadvjsHFPZmrXrK7Rf//validator_feeder1
    //         let validator_feeders = vec!["5CwJrAMQdQsihzGBeSWik8GTZuCcogKo1AkXuaoFBQbmnHjJ"
    //             .parse()
    //             .unwrap()];
    //         let initial_allocation = accumulate(
    //             vec![
    //                 // Faucet accounts
    //                 "5HHMY7e8UAqR5ZaHGaQnRW5EDR8dP7QpAyjeBu6V7vdXxxbf"
    //                     .parse()
    //                     .unwrap(),
    //                 "5E5BxCjexvzgH9LsYUzMjD6gJaWiKkmadvjsHFPZmrXrK7Rf"
    //                     .parse()
    //                     .unwrap(),
    //                 // Team members accounts
    //                 "5DhZeTQqotvntGtrg69T2VK9pzUPXHiVyGUTmp5XFTDTT7ME"
    //                     .parse()
    //                     .unwrap(),
    //                 "5GBykvvrUz3vwTttgHzUEPdm7G1FND1reBfddQLdiaCbhoMd"
    //                     .parse()
    //                     .unwrap(),
    //                 "5G9eFoXB95fdwFJK9utBf1AgiLvhPUvzArYR2knzXKrKtZPZ"
    //                     .parse()
    //                     .unwrap(),
    //                 "1Gu7GSgLSPrhc1Wci9wAGP6nvzQfaUCYqbfXxjYjMG9bob6"
    //                     .parse()
    //                     .unwrap(),
    //                 "5G9eFoXB95fdwFJK9utBf1AgiLvhPUvzArYR2knzXKrKtZPZ"
    //                     .parse()
    //                     .unwrap(),
    //                 "5CzR4NFben6n7uk3jZCVCoZbpA9fpdwrJdE1rznQXuFxMkTn"
    //                     .parse()
    //                     .unwrap(),
    //             ]
    //             .iter()
    //             .flat_map(|x: &AccountId| {
    //                 if x == &"5HHMY7e8UAqR5ZaHGaQnRW5EDR8dP7QpAyjeBu6V7vdXxxbf"
    //                     .parse()
    //                     .unwrap()
    //                 {
    //                     vec![(x.clone(), 10_u128.pow(20))]
    //                 } else {
    //                     vec![(x.clone(), 10_u128.pow(16))]
    //                 }
    //             }),
    //         );
    //         let council = vec![
    //             "5GBykvvrUz3vwTttgHzUEPdm7G1FND1reBfddQLdiaCbhoMd"
    //                 .parse()
    //                 .unwrap(),
    //             "5DhZeTQqotvntGtrg69T2VK9pzUPXHiVyGUTmp5XFTDTT7ME"
    //                 .parse()
    //                 .unwrap(),
    //             "1Gu7GSgLSPrhc1Wci9wAGP6nvzQfaUCYqbfXxjYjMG9bob6"
    //                 .parse()
    //                 .unwrap(),
    //             "5CzR4NFben6n7uk3jZCVCoZbpA9fpdwrJdE1rznQXuFxMkTn"
    //                 .parse()
    //                 .unwrap(),
    //             "5G9eFoXB95fdwFJK9utBf1AgiLvhPUvzArYR2knzXKrKtZPZ"
    //                 .parse()
    //                 .unwrap(),
    //         ];
    //         let technical_committee = vec![
    //             "5GBykvvrUz3vwTttgHzUEPdm7G1FND1reBfddQLdiaCbhoMd"
    //                 .parse()
    //                 .unwrap(),
    //             "5DhZeTQqotvntGtrg69T2VK9pzUPXHiVyGUTmp5XFTDTT7ME"
    //                 .parse()
    //                 .unwrap(),
    //             "5G9eFoXB95fdwFJK9utBf1AgiLvhPUvzArYR2knzXKrKtZPZ"
    //                 .parse()
    //                 .unwrap(),
    //             "5CzR4NFben6n7uk3jZCVCoZbpA9fpdwrJdE1rznQXuFxMkTn"
    //                 .parse()
    //                 .unwrap(),
    //         ];
    //
    //         vanilla_genesis(
    //             root_key,
    //             invulnerables,
    //             oracle_accounts,
    //             initial_allocation,
    //             validator_feeders,
    //             council,
    //             technical_committee,
    //             id,
    //         )
    //     },
    //     vec![
    //         "/dns/vanilla-bootnode-0.parallel.fi/tcp/30333/p2p/12D3KooWP3xQ2EzF9stuQTNsHw7DPY4CYjdBuABiN7VShfJ6phia".parse().unwrap(),
    //         "/dns/vanilla-bootnode-1.parallel.fi/tcp/30333/p2p/12D3KooWK984FD65FoNMDS6EMdgmjKKJPKvzwXroPRNjg7eLFSz7".parse().unwrap(),
    //     ],
    //     TelemetryEndpoints::new(vec![(TELEMETRY_URL.into(), 0)]).ok(),
    //     Some("vanilla"),
    //     Some(as_properties(NetworkType::Heiko)),
    //     Extensions {
    //         relay_chain: "westend-local".into(),
    //         para_id: id.into(),
    //     },
    // ))
}

fn vanilla_genesis(
    root_key: AccountId,
    invulnerables: Vec<(AccountId, AuraId)>,
    oracle_accounts: Vec<AccountId>,
    initial_allocation: Vec<(AccountId, Balance)>,
    validator_feeders: Vec<AccountId>,
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
