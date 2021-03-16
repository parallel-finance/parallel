use parallel_runtime::{
    AccountId, AuraConfig, BalancesConfig, GenesisConfig, GrandpaConfig, LoansConfig, Signature,
    SudoConfig, SystemConfig, TokensConfig, WASM_BINARY,
};
use primitives::{CurrencyId, RATE_DECIMAL, TOKEN_DECIMAL};
use sc_service::ChainType;
use serde_json::map::Map;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{sr25519, Pair, Public};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{IdentifyAccount, Verify};

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
    (get_from_seed::<AuraId>(s), get_from_seed::<GrandpaId>(s))
}

pub fn development_config() -> Result<ChainSpec, String> {
    let mut properties = Map::new();
    // properties.insert("tokenSymbol".into(), "TEST".into());
    properties.insert("tokenDecimals".into(), 18.into());

    let wasm_binary =
        WASM_BINARY.ok_or_else(|| "Development wasm binary not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        // Name
        "Development",
        // ID
        "dev",
        ChainType::Development,
        move || {
            testnet_genesis(
                wasm_binary,
                // Initial PoA authorities
                vec![authority_keys_from_seed("Alice")],
                // Sudo account
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                // Pre-funded accounts
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Pool"),
                ],
                true,
            )
        },
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        None,
        // Properties
        Some(properties),
        // Extensions
        None,
    ))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
    let wasm_binary =
        WASM_BINARY.ok_or_else(|| "Development wasm binary not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        // Name
        "Local Testnet",
        // ID
        "local_testnet",
        ChainType::Local,
        move || {
            testnet_genesis(
                wasm_binary,
                // Initial PoA authorities
                vec![
                    authority_keys_from_seed("Alice"),
                    authority_keys_from_seed("Bob"),
                ],
                // Sudo account
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                // Pre-funded accounts
                vec![
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
                ],
                true,
            )
        },
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        None,
        // Properties
        None,
        // Extensions
        None,
    ))
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
    wasm_binary: &[u8],
    initial_authorities: Vec<(AuraId, GrandpaId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    _enable_println: bool,
) -> GenesisConfig {
    GenesisConfig {
        frame_system: Some(SystemConfig {
            // Add Wasm runtime to storage.
            code: wasm_binary.to_vec(),
            changes_trie_config: Default::default(),
        }),
        pallet_balances: Some(BalancesConfig {
            // Configure endowed accounts with initial balance of 1 << 60.
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, 1 << 60))
                .collect(),
        }),
        pallet_aura: Some(AuraConfig {
            authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
        }),
        pallet_grandpa: Some(GrandpaConfig {
            authorities: initial_authorities
                .iter()
                .map(|x| (x.1.clone(), 1))
                .collect(),
        }),
        pallet_sudo: Some(SudoConfig {
            // Assign network admin rights.
            key: root_key,
        }),
        orml_tokens: Some(TokensConfig {
            endowed_accounts: endowed_accounts
                .iter()
                .flat_map(|x| {
                    vec![
                        (x.clone(), CurrencyId::DOT, 1_000 * TOKEN_DECIMAL),
                        (x.clone(), CurrencyId::KSM, 1_000 * TOKEN_DECIMAL),
                        (x.clone(), CurrencyId::BTC, 1_000 * TOKEN_DECIMAL),
                        (x.clone(), CurrencyId::USDC, 1_000 * TOKEN_DECIMAL),
                        (x.clone(), CurrencyId::xDOT, 1_000 * TOKEN_DECIMAL),
                    ]
                })
                .collect(),
        }),
        pallet_loans: Some(LoansConfig {
            currencies: vec![
                CurrencyId::DOT,
                CurrencyId::KSM,
                CurrencyId::BTC,
                CurrencyId::USDC,
                CurrencyId::xDOT,
            ],
            total_supply: 1000 * TOKEN_DECIMAL, // 1000
            total_borrows: 600 * TOKEN_DECIMAL, // 600

            borrow_index: RATE_DECIMAL,                 // 1
            exchange_rate: 2 * RATE_DECIMAL / 100,      // 0.02
            base_rate: 2 * RATE_DECIMAL / 100,          // 0.02
            multiplier_per_year: 1 * RATE_DECIMAL / 10, // 0.1
            jump_muiltiplier: 11 * RATE_DECIMAL / 10,   // 1.1
            kink: 8 * RATE_DECIMAL / 10,                // 0.8
            collateral_rate: vec![
                (CurrencyId::DOT, 5 * RATE_DECIMAL / 10),
                (CurrencyId::KSM, 5 * RATE_DECIMAL / 10),
                (CurrencyId::BTC, 5 * RATE_DECIMAL / 10),
                (CurrencyId::USDC, 5 * RATE_DECIMAL / 10),
                (CurrencyId::xDOT, 5 * RATE_DECIMAL / 10),
            ],
            liquidation_incentive: vec![
                (CurrencyId::DOT, 9 * RATE_DECIMAL / 10),
                (CurrencyId::KSM, 9 * RATE_DECIMAL / 10),
                (CurrencyId::BTC, 9 * RATE_DECIMAL / 10),
                (CurrencyId::USDC, 9 * RATE_DECIMAL / 10),
                (CurrencyId::xDOT, 9 * RATE_DECIMAL / 10),
            ],
            //FIXME :In fact,"liquidation_threshold" should be higher than "collateral_rate",
            //but for test, let's make it lower
            liquidation_threshold: vec![
                (CurrencyId::DOT, 40 * RATE_DECIMAL / 100),
                (CurrencyId::KSM, 40 * RATE_DECIMAL / 100),
                (CurrencyId::BTC, 40 * RATE_DECIMAL / 100),
                (CurrencyId::USDC, 40 * RATE_DECIMAL / 100),
                (CurrencyId::xDOT, 40 * RATE_DECIMAL / 100),
            ],
            close_factor: vec![
                (CurrencyId::DOT, 5 * RATE_DECIMAL / 10),
                (CurrencyId::KSM, 5 * RATE_DECIMAL / 10),
                (CurrencyId::BTC, 5 * RATE_DECIMAL / 10),
                (CurrencyId::USDC, 5 * RATE_DECIMAL / 10),
                (CurrencyId::xDOT, 5 * RATE_DECIMAL / 10),
            ],
        }),
    }
}
