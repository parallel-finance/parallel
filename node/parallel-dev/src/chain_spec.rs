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

use hex_literal::hex;
use primitives::*;
use sc_service::ChainType;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::{
    traits::{IdentifyAccount, One, Verify},
    FixedPointNumber,
};
use vanilla_runtime::{
    pallet_loans::{InterestRateModel, JumpModel, Market, MarketState},
    AuraConfig, BalancesConfig, CouncilConfig, DemocracyConfig, GenesisConfig, GrandpaConfig,
    LiquidStakingConfig, LoansConfig, OracleMembershipConfig, SudoConfig, SystemConfig,
    TechnicalCommitteeConfig, TokensConfig, ValidatorFeedersMembershipConfig, WASM_BINARY,
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
                vec![get_account_id_from_seed::<sr25519::Public>("Ferdie")],
                vec![get_account_id_from_seed::<sr25519::Public>("Eve")],
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie"),
                    get_account_id_from_seed::<sr25519::Public>("Dave"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
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
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Live wasm not available".to_string())?;
    Ok(ChainSpec::from_genesis(
        // Name
        "Vanilla Live",
        // ID
        "vanilla-live",
        ChainType::Local,
        move || {
            testnet_genesis(
                wasm_binary,
                "5HHMY7e8UAqR5ZaHGaQnRW5EDR8dP7QpAyjeBu6V7vdXxxbf"
                    .parse()
                    .unwrap(),
                vec![(
                    hex!["a213e66d76b8b4b7f8da0343bbb658cb22b77c5f4b6bf87eb20be5618a61577b"]
                        .unchecked_into(),
                    hex!["90992c3f6fade153e1bf5a2856ec6983648b339a5c158238d6dd7c4e16832b12"]
                        .unchecked_into(),
                )],
                vec!["5GTb3uLbk9VsyGD6taPyk69p2Hfa21GuzmMF52oJnqTQh2AA"
                    .parse()
                    .unwrap()],
                vec!["5FjH9a7RQmihmb7i4UzbNmecjPm9WVLyoJHfsixkrLGEKwsJ"
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
        vec!["/dns/35.246.154.195/tcp/30333/p2p/12D3KooWMRN3wVhcijAB7H6M7wo48KW3CJSpyHSWZcCzNwYW3KVF".parse().unwrap()],
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
    validator_feeders: Vec<AccountId>,
    endowed_accounts: Vec<AccountId>,
) -> GenesisConfig {
    GenesisConfig {
        system: SystemConfig {
            code: wasm_binary.to_vec(),
            changes_trie_config: Default::default(),
        },
        aura: AuraConfig {
            authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
        },
        grandpa: GrandpaConfig {
            authorities: initial_authorities
                .iter()
                .map(|x| (x.1.clone(), 1))
                .collect(),
        },
        balances: BalancesConfig {
            balances: {
                let mut endowed_accounts = endowed_accounts.clone();
                endowed_accounts.extend_from_slice(&oracle_accounts);
                endowed_accounts.extend_from_slice(&validator_feeders);

                endowed_accounts
                    .into_iter()
                    .map(|k| (k, 10_u128.pow(16)))
                    .collect()
            },
        },
        sudo: SudoConfig { key: root_key },
        tokens: TokensConfig {
            balances: endowed_accounts
                .iter()
                .flat_map(|x| {
                    if x == &"5HHMY7e8UAqR5ZaHGaQnRW5EDR8dP7QpAyjeBu6V7vdXxxbf"
                        .parse()
                        .unwrap()
                    {
                        vec![
                            (x.clone(), CurrencyId::KSM, 10_u128.pow(20)),
                            (x.clone(), CurrencyId::USDT, 10_u128.pow(14)),
                        ]
                    } else {
                        vec![
                            (x.clone(), CurrencyId::KSM, 10_u128.pow(15)),
                            (x.clone(), CurrencyId::USDT, 10_u128.pow(9)),
                        ]
                    }
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
        validator_feeders_membership: ValidatorFeedersMembershipConfig {
            members: validator_feeders,
            phantom: Default::default(),
        },
    }
}
