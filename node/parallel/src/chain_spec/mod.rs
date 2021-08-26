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

pub mod heiko;
pub mod parallel;

use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::Properties;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{sr25519, Pair, Public};
use sp_runtime::traits::IdentifyAccount;
#[cfg(feature = "std")]
use sp_std::collections::btree_map::BTreeMap;

use primitives::{network::NetworkType, *};

use crate::service::IdentifyVariant;

pub const TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Token symbol of heiko network.
pub const HEIKO_TOKEN: &str = "HKO";
/// Token symbol of parallel network.
pub const PARALLEL_TOKEN: &str = "PARA";

/// accumulate account balances
pub fn accumulate(
    iter: impl IntoIterator<Item = (AccountId, Balance)>,
) -> Vec<(AccountId, Balance)> {
    let acc = BTreeMap::<AccountId, Balance>::new();
    iter.into_iter()
        .fold(acc, |mut acc, (account_id, amount)| {
            if let Some(balance) = acc.get_mut(&account_id) {
                *balance = balance.checked_add(amount).unwrap()
            } else {
                acc.insert(account_id.clone(), amount);
            }
            acc
        })
        .into_iter()
        .collect()
}

/// set default ss58 crypto
pub fn set_default_ss58_version(spec: &Box<dyn sc_service::ChainSpec>) {
    use sp_core::crypto::Ss58AddressFormat;

    let ss58_version = if spec.is_heiko() {
        Ss58AddressFormat::HeikoAccount
    } else if spec.is_parallel() {
        Ss58AddressFormat::ParallelAccount
    } else {
        Ss58AddressFormat::SubstrateAccount
    };

    sp_core::crypto::set_default_ss58_version(ss58_version);
}

/// Generate chain properties for network.
///
/// For fields definition, see https://github.com/polkadot-js/apps/blob/bd78840d2142df121d182e8700b20308880dde0a/packages/react-api/src/Api.tsx#L115
pub(crate) fn as_properties(network: NetworkType) -> Properties {
    let (symbol, decimal) = token_info(&network);
    json!({
        "ss58Format": network.ss58_addr_format_id(),
        "tokenSymbol": symbol,
        "tokenDecimals": decimal,
    })
    .as_object()
    .expect("Network properties are valid; qed")
    .to_owned()
}

/// Return (token_symbol, token_decimal) of this network.
fn token_info(network: &NetworkType) -> (&str, u8) {
    match network {
        NetworkType::Heiko => (HEIKO_TOKEN, 12),
        NetworkType::Parallel => (PARALLEL_TOKEN, 12),
    }
}

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

/// Generate an Aura authority key
pub fn get_authority_keys_from_seed(seed: &str) -> (AccountId, AuraId) {
    (
        get_account_id_from_seed::<sr25519::Public>(seed),
        get_from_seed::<AuraId>(seed),
    )
}

/// The extensions for the [`ChainSpec`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
#[serde(deny_unknown_fields)]
pub struct Extensions {
    /// The relay chain of the Parachain.
    pub relay_chain: String,
    /// The id of the Parachain.
    pub para_id: u32,
}

impl Extensions {
    /// Try to get the extension from the given `ChainSpec`.
    pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
        sc_chain_spec::get_extension(chain_spec.extensions())
    }
}

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_accumulate_test() {
        let balances: Vec<(AccountId, Balance)> = vec![
            (
                "5DJd3duMMEeEo9Gi5az1esvuNRB31V8Fds91VkBMrZUCFyUn"
                    .parse()
                    .unwrap(),
                1000,
            ),
            (
                "5EUmwapW8qScFGh4KGug1xb5Dnm4FYQtzrjTcvjynyRAMRR3"
                    .parse()
                    .unwrap(),
                1000,
            ),
        ];

        assert_eq!(
            accumulate(balances),
            vec![
                (
                    "5DJd3duMMEeEo9Gi5az1esvuNRB31V8Fds91VkBMrZUCFyUn"
                        .parse()
                        .unwrap(),
                    1000,
                ),
                (
                    "5EUmwapW8qScFGh4KGug1xb5Dnm4FYQtzrjTcvjynyRAMRR3"
                        .parse()
                        .unwrap(),
                    1000
                )
            ]
        );
    }

    #[test]
    fn complex_accumulate_test() {
        let balances: Vec<(AccountId, Balance)> = vec![
            (
                "5DJd3duMMEeEo9Gi5az1esvuNRB31V8Fds91VkBMrZUCFyUn"
                    .parse()
                    .unwrap(),
                1000,
            ),
            (
                "5DJd3duMMEeEo9Gi5az1esvuNRB31V8Fds91VkBMrZUCFyUn"
                    .parse()
                    .unwrap(),
                100,
            ),
            (
                "5DJd3duMMEeEo9Gi5az1esvuNRB31V8Fds91VkBMrZUCFyUn"
                    .parse()
                    .unwrap(),
                10,
            ),
            (
                "5DJd3duMMEeEo9Gi5az1esvuNRB31V8Fds91VkBMrZUCFyUn"
                    .parse()
                    .unwrap(),
                1,
            ),
            (
                "5EUmwapW8qScFGh4KGug1xb5Dnm4FYQtzrjTcvjynyRAMRR3"
                    .parse()
                    .unwrap(),
                1000,
            ),
            (
                "5EUmwapW8qScFGh4KGug1xb5Dnm4FYQtzrjTcvjynyRAMRR3"
                    .parse()
                    .unwrap(),
                100,
            ),
            (
                "5EUmwapW8qScFGh4KGug1xb5Dnm4FYQtzrjTcvjynyRAMRR3"
                    .parse()
                    .unwrap(),
                10,
            ),
            (
                "5EUmwapW8qScFGh4KGug1xb5Dnm4FYQtzrjTcvjynyRAMRR3"
                    .parse()
                    .unwrap(),
                1,
            ),
        ];

        assert_eq!(
            accumulate(balances),
            vec![
                (
                    "5DJd3duMMEeEo9Gi5az1esvuNRB31V8Fds91VkBMrZUCFyUn"
                        .parse()
                        .unwrap(),
                    1111,
                ),
                (
                    "5EUmwapW8qScFGh4KGug1xb5Dnm4FYQtzrjTcvjynyRAMRR3"
                        .parse()
                        .unwrap(),
                    1111
                )
            ]
        );
    }
}
