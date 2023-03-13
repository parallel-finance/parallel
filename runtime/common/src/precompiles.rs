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

use frame_support::dispatch::GetDispatchInfo;
use frame_support::dispatch::PostDispatchInfo;
use pallet_evm::{
    ExitRevert, Precompile, PrecompileFailure, PrecompileHandle, PrecompileResult, PrecompileSet,
};
use sp_core::H160;
use sp_runtime::traits::Dispatchable;
use sp_std::fmt::Debug;
use sp_std::marker::PhantomData;

use pallet_evm_precompile_assets_erc20::{AddressToAssetId, Erc20AssetsPrecompileSet};
use pallet_evm_precompile_balances_erc20::Erc20BalancesPrecompile;
use pallet_evm_precompile_balances_erc20::Erc20Metadata;
use pallet_evm_precompile_blake2::Blake2F;
use pallet_evm_precompile_bn128::{Bn128Add, Bn128Mul, Bn128Pairing};
use pallet_evm_precompile_dispatch::Dispatch;
use pallet_evm_precompile_ed25519::Ed25519Verify;
use pallet_evm_precompile_modexp::Modexp;
use pallet_evm_precompile_sha3fips::Sha3FIPS256;
use pallet_evm_precompile_simple::{ECRecover, ECRecoverPublicKey, Identity, Ripemd160, Sha256};

/// The asset precompile address prefix. Addresses that match against this prefix will be routed
/// to Erc20AssetsPrecompileSet
pub const ASSET_PRECOMPILE_ADDRESS_PREFIX: &[u8] = &[255u8; 4];

#[derive(Debug, Default, Clone, Copy)]
pub struct ParallelPrecompiles<R, M>(PhantomData<(R, M)>);

impl<R, M> ParallelPrecompiles<R, M>
where
    R: pallet_evm::Config,
    M: Erc20Metadata,
{
    pub fn new() -> Self {
        Self(Default::default())
    }
    pub fn used_addresses() -> impl Iterator<Item = H160> {
        sp_std::vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 1024, 1025, 1026, 1027, 2050]
            .into_iter()
            .map(hash)
    }
}

impl<R, M> PrecompileSet for ParallelPrecompiles<R, M>
where
    Erc20AssetsPrecompileSet<R>: PrecompileSet,
    Erc20BalancesPrecompile<R, M>: Precompile,
    Dispatch<R>: Precompile,
    R: pallet_evm::Config
        + AddressToAssetId<<R as pallet_assets::Config>::AssetId>
        + pallet_assets::Config
        + pallet_balances::Config,
    R::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
    <R as frame_system::Config>::RuntimeCall: From<polkadot_runtime_common::BalancesCall<R>>,
    <<R as frame_system::Config>::RuntimeCall as Dispatchable>::RuntimeOrigin:
        From<Option<<R as frame_system::Config>::AccountId>>,
    <R as pallet_balances::Config>::Balance: TryFrom<sp_core::U256>,
    <R as pallet_balances::Config>::Balance: Into<sp_core::U256>,
    <R as pallet_timestamp::Config>::Moment: Into<sp_core::U256>,
    M: Erc20Metadata,
{
    fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<PrecompileResult> {
        let address = handle.code_address();
        if self.is_precompile(address) && address > hash(9) && handle.context().address != address {
            return Some(Err(PrecompileFailure::Revert {
                exit_status: ExitRevert::Reverted,
                output: b"cannot be called with DELEGATECALL or CALLCODE".to_vec(),
            }));
        }
        match address {
            // Ethereum precompiles :
            a if a == hash(1) => Some(ECRecover::execute(handle)),
            a if a == hash(2) => Some(Sha256::execute(handle)),
            a if a == hash(3) => Some(Ripemd160::execute(handle)),
            a if a == hash(4) => Some(Identity::execute(handle)),
            a if a == hash(5) => Some(Modexp::execute(handle)),
            a if a == hash(6) => Some(Bn128Add::execute(handle)),
            a if a == hash(7) => Some(Bn128Mul::execute(handle)),
            a if a == hash(8) => Some(Bn128Pairing::execute(handle)),
            a if a == hash(9) => Some(Blake2F::execute(handle)),
            // Non-Frontier specific nor Ethereum precompiles :
            a if a == hash(1024) => Some(Sha3FIPS256::execute(handle)),
            a if a == hash(1025) => Some(ECRecoverPublicKey::execute(handle)),
            a if a == hash(1026) => Some(ECRecoverPublicKey::execute(handle)),
            a if a == hash(1027) => Some(Ed25519Verify::execute(handle)),
            //Parallel precompiles:
            a if a == hash(2050) => Some(Erc20BalancesPrecompile::<R, M>::execute(handle)),
            a if &a.to_fixed_bytes()[0..4] == ASSET_PRECOMPILE_ADDRESS_PREFIX => {
                Erc20AssetsPrecompileSet::<R>::new().execute(handle)
            }
            _ => None,
        }
    }

    fn is_precompile(&self, address: H160) -> bool {
        Self::used_addresses().any(|x| x == address)
            || Erc20AssetsPrecompileSet::<R>::new().is_precompile(address)
    }
}

fn hash(a: u64) -> H160 {
    H160::from_low_u64_be(a)
}
