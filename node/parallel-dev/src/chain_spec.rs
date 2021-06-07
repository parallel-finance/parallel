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

use primitives::*;
use sc_service::ChainType;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{sr25519, Pair, Public};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::{
    traits::{IdentifyAccount, One, Verify},
    FixedPointNumber,
};
use vanilla_runtime::{
    constants::currency::DOLLARS, AuraConfig, BalancesConfig, CouncilConfig, DemocracyConfig,
    ElectionsConfig, GenesisConfig, GrandpaConfig, LiquidStakingConfig, LoansConfig,
    OracleMembershipConfig, SudoConfig, SystemConfig, TechnicalCommitteeConfig, TokensConfig,
    WASM_BINARY,
};

pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
    (get_from_seed::<AuraId>(s), get_from_seed::<GrandpaId>(s))
}

pub fn development_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;
    Ok(ChainSpec::from_genesis(
        // Name
        "Development",
        // ID
        "dev",
        ChainType::Development,
        move || {
            testnet_genesis(
                wasm_binary,
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                vec![authority_keys_from_seed("Alice")],
                vec![
                    "5GTb3uLbk9VsyGD6taPyk69p2Hfa21GuzmMF52oJnqTQh2AA"
                        .parse()
                        .unwrap(),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                ],
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie"),
                    get_account_id_from_seed::<sr25519::Public>("Dave"),
                    get_account_id_from_seed::<sr25519::Public>("Eve"),
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
            )
        },
        vec![],
        None,
        None,
        None,
        None,
    ))
}

pub fn live_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Testnet wasm not available".to_string())?;
    Ok(ChainSpec::from_genesis(
        // Name
        "Vanilla Testnet",
        // ID
        "vanilla-local",
        ChainType::Local,
        move || {
            testnet_genesis(
                wasm_binary,
                "5HHMY7e8UAqR5ZaHGaQnRW5EDR8dP7QpAyjeBu6V7vdXxxbf"
                    .parse()
                    .unwrap(),
                vec![authority_keys_from_seed("Alice")],
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
            )
        },
        vec![],
        None,
        None,
        None,
        None,
    ))
}

fn testnet_genesis(
    wasm_binary: &[u8],
    root_key: AccountId,
    initial_authorities: Vec<(AuraId, GrandpaId)>,
    oracle_accounts: Vec<AccountId>,
    endowed_accounts: Vec<AccountId>,
) -> GenesisConfig {
    let num_endowed_accounts = endowed_accounts.len();
    const ENDOWMENT: Balance = 10_000_000 * DOLLARS;
    const STASH: Balance = ENDOWMENT / 1000;
    GenesisConfig {
        frame_system: SystemConfig {
            code: wasm_binary.to_vec(),
            changes_trie_config: Default::default(),
        },
        pallet_aura: AuraConfig {
            authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
        },
        pallet_grandpa: GrandpaConfig {
            authorities: initial_authorities
                .iter()
                .map(|x| (x.1.clone(), 1))
                .collect(),
        },
        pallet_balances: BalancesConfig {
            balances: {
                let mut endowed_accounts = endowed_accounts.clone();
                endowed_accounts.extend_from_slice(&oracle_accounts);

                endowed_accounts
                    .into_iter()
                    .map(|k| (k, 10_u128.pow(21)))
                    .collect()
            },
        },
        pallet_sudo: SudoConfig { key: root_key },
        orml_tokens: TokensConfig {
            endowed_accounts: endowed_accounts
                .iter()
                .flat_map(|x| {
                    vec![
                        (x.clone(), CurrencyId::USDT, 10_u128.pow(21)),
                        (x.clone(), CurrencyId::xKSM, 10_u128.pow(21)),
                    ]
                })
                .collect(),
        },
        pallet_loans: LoansConfig {
            currencies: vec![CurrencyId::KSM, CurrencyId::USDT, CurrencyId::xKSM],
            borrow_index: Rate::one(),                             // 1
            exchange_rate: Rate::saturating_from_rational(2, 100), // 0.02
            base_rate: Rate::saturating_from_rational(2, 100),     // 2%
            kink_rate: Rate::saturating_from_rational(10, 100),    // 10%
            full_rate: Rate::saturating_from_rational(32, 100),    // 32%
            kink_utilization: Ratio::from_percent(80),             // 80%
            collateral_factor: vec![
                (CurrencyId::KSM, Ratio::from_percent(50)),
                (CurrencyId::USDT, Ratio::from_percent(50)),
                (CurrencyId::xKSM, Ratio::from_percent(90)),
            ],
            liquidation_incentive: vec![
                (CurrencyId::KSM, Ratio::from_percent(90)),
                (CurrencyId::USDT, Ratio::from_percent(90)),
                (CurrencyId::xKSM, Ratio::from_percent(90)),
            ],
            close_factor: vec![
                (CurrencyId::KSM, Ratio::from_percent(50)),
                (CurrencyId::USDT, Ratio::from_percent(50)),
                (CurrencyId::xKSM, Ratio::from_percent(50)),
            ],
            reserve_factor: vec![
                (CurrencyId::KSM, Ratio::from_percent(15)),
                (CurrencyId::USDT, Ratio::from_percent(15)),
                (CurrencyId::xKSM, Ratio::from_percent(15)),
            ],
            last_block_timestamp: 0,
        },
        pallet_liquid_staking: LiquidStakingConfig {
            exchange_rate: Rate::saturating_from_rational(2, 100), // 0.02
        },
        pallet_democracy: DemocracyConfig::default(),
        pallet_elections_phragmen: ElectionsConfig {
            members: endowed_accounts
                .iter()
                .take((num_endowed_accounts + 1) / 2)
                .cloned()
                .map(|member| (member, STASH))
                .collect(),
        },
        pallet_collective_Instance1: CouncilConfig::default(),
        pallet_collective_Instance2: TechnicalCommitteeConfig {
            members: endowed_accounts
                .iter()
                .take((num_endowed_accounts + 1) / 2)
                .cloned()
                .collect(),
            phantom: Default::default(),
        },
        pallet_membership_Instance1: Default::default(),
        pallet_treasury: Default::default(),
        pallet_membership_Instance2: OracleMembershipConfig {
            members: oracle_accounts,
            phantom: Default::default(),
        },
    }
}
