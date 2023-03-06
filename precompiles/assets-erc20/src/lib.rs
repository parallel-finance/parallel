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

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(test, feature(assert_matches))]

use fp_evm::{PrecompileHandle, PrecompileOutput};
use frame_support::traits::fungibles::approvals::Inspect as ApprovalInspect;
use frame_support::traits::fungibles::metadata::Inspect as MetadataInspect;
use frame_support::traits::fungibles::Inspect;
use frame_support::traits::OriginTrait;
use frame_support::{
    dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo},
    sp_runtime::traits::StaticLookup,
};
use pallet_evm::{AddressMapping, PrecompileSet};
use precompile_utils::{
    keccak256, succeed, Address, Bytes, EvmData, EvmDataWriter, EvmResult, FunctionModifier,
    LogExt, LogsBuilder, PrecompileHandleExt, RuntimeHelper,
};
use sp_runtime::traits::Bounded;

use core::fmt::Display;
use sp_core::{H160, U256};
use sp_std::{
    convert::{TryFrom, TryInto},
    marker::PhantomData,
};

mod eip2612;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

/// Solidity selector of the Transfer log, which is the Keccak of the Log signature.
pub const SELECTOR_LOG_TRANSFER: [u8; 32] = keccak256!("Transfer(address,address,uint256)");

/// Solidity selector of the Approval log, which is the Keccak of the Log signature.
pub const SELECTOR_LOG_APPROVAL: [u8; 32] = keccak256!("Approval(address,address,uint256)");

/// Alias for the Balance type for the provided Runtime and Instance.
pub type BalanceOf<Runtime, Instance = ()> = <Runtime as pallet_assets::Config<Instance>>::Balance;

/// Alias for the Asset Id type for the provided Runtime and Instance.
pub type AssetIdOf<Runtime, Instance = ()> = <Runtime as pallet_assets::Config<Instance>>::AssetId;

#[precompile_utils::generate_function_selector]
#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    TotalSupply = "totalSupply()",
    BalanceOf = "balanceOf(address)",
    Allowance = "allowance(address,address)",
    Transfer = "transfer(address,uint256)",
    Approve = "approve(address,uint256)",
    TransferFrom = "transferFrom(address,address,uint256)",
    Name = "name()",
    Symbol = "symbol()",
    Decimals = "decimals()",
    MinimumBalance = "minimumBalance()",
    Mint = "mint(address,uint256)",
    Burn = "burn(address,uint256)",
    // EIP 2612
    Eip2612Permit = "permit(address,address,uint256,uint256,uint8,bytes32,bytes32)",
    Eip2612Nonces = "nonces(address)",
    Eip2612DomainSeparator = "DOMAIN_SEPARATOR()",
}

/// This trait ensure we can convert EVM address to AssetIds
/// We will require Runtime to have this trait implemented
pub trait AddressToAssetId<AssetId> {
    // Get assetId from address
    fn address_to_asset_id(address: H160) -> Option<AssetId>;

    // Get address from AssetId
    fn asset_id_to_address(asset_id: AssetId) -> H160;
}

/// The following distribution has been decided for the precompiles
/// 0-1023: Ethereum Mainnet Precompiles
/// 1024-2047 Precompiles that are not in Ethereum Mainnet but are neither Astar specific
/// 2048-4095 Parallel specific precompiles
/// Asset precompiles can only fall between
///     0xFFFFFFFF00000000000000000000000000000000 - 0xFFFFFFFF000000000000000000000000FFFFFFFF
/// The precompile for AssetId X, where X is a u32 (i.e.4 bytes), if 0XFFFFFFFF + Bytes(AssetId)

/// This means that every address that starts with 0xFFFFFFFF will go through an additional db read,
/// but the probability for this to happen is 2^-32 for random addresses
pub struct Erc20AssetsPrecompileSet<Runtime, Instance: 'static = ()>(
    PhantomData<(Runtime, Instance)>,
);

impl<Runtime, Instance> Erc20AssetsPrecompileSet<Runtime, Instance> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<Runtime, Instance> Default for Erc20AssetsPrecompileSet<Runtime, Instance> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<Runtime, Instance> PrecompileSet for Erc20AssetsPrecompileSet<Runtime, Instance>
where
    Instance: eip2612::InstanceToPrefix + 'static,
    Runtime: pallet_assets::Config<Instance> + pallet_evm::Config + frame_system::Config,
    Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
    Runtime::RuntimeCall: From<pallet_assets::Call<Runtime, Instance>>,
    <Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
    BalanceOf<Runtime, Instance>: TryFrom<U256> + Into<U256> + EvmData,
    Runtime: AddressToAssetId<AssetIdOf<Runtime, Instance>>,
    <<Runtime as frame_system::Config>::RuntimeCall as Dispatchable>::RuntimeOrigin: OriginTrait,
    <Runtime as pallet_timestamp::Config>::Moment: Into<U256>,
    AssetIdOf<Runtime, Instance>: Display,
{
    fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<EvmResult<PrecompileOutput>> {
        let address = handle.code_address();

        if let Some(asset_id) = Runtime::address_to_asset_id(address) {
            // We check maybe_total_supply. This function returns Some if the asset exists,
            // which is all we care about at this point
            if pallet_assets::Pallet::<Runtime, Instance>::maybe_total_supply(asset_id).is_some() {
                let result = {
                    let selector = match handle.read_selector() {
                        Ok(selector) => selector,
                        Err(e) => return Some(Err(e)),
                    };

                    if let Err(err) = handle.check_function_modifier(match selector {
                        Action::Approve
                        | Action::Transfer
                        | Action::TransferFrom
                        | Action::Mint
                        | Action::Burn => FunctionModifier::NonPayable,
                        _ => FunctionModifier::View,
                    }) {
                        return Some(Err(err));
                    }

                    match selector {
                        // XC20
                        Action::TotalSupply => Self::total_supply(asset_id, handle),
                        Action::BalanceOf => Self::balance_of(asset_id, handle),
                        Action::Allowance => Self::allowance(asset_id, handle),
                        Action::Approve => Self::approve(asset_id, handle),
                        Action::Transfer => Self::transfer(asset_id, handle),
                        Action::TransferFrom => Self::transfer_from(asset_id, handle),
                        Action::Name => Self::name(asset_id, handle),
                        Action::Symbol => Self::symbol(asset_id, handle),
                        Action::Decimals => Self::decimals(asset_id, handle),
                        // XC20+
                        Action::MinimumBalance => Self::minimum_balance(asset_id, handle),
                        Action::Mint => Self::mint(asset_id, handle),
                        Action::Burn => Self::burn(asset_id, handle),
                        // EIP2612
                        Action::Eip2612Permit => {
                            eip2612::Eip2612::<Runtime, Instance>::permit(asset_id, handle)
                        }
                        Action::Eip2612Nonces => {
                            eip2612::Eip2612::<Runtime, Instance>::nonces(asset_id, handle)
                        }
                        Action::Eip2612DomainSeparator => {
                            eip2612::Eip2612::<Runtime, Instance>::domain_separator(
                                asset_id, handle,
                            )
                        }
                    }
                };
                return Some(result);
            }
        }
        None
    }

    fn is_precompile(&self, address: H160) -> bool {
        if let Some(asset_id) = Runtime::address_to_asset_id(address) {
            pallet_assets::Pallet::<Runtime, Instance>::maybe_total_supply(asset_id).is_some()
        } else {
            false
        }
    }
}

impl<Runtime, Instance> Erc20AssetsPrecompileSet<Runtime, Instance>
where
    Instance: eip2612::InstanceToPrefix + 'static,
    Runtime: pallet_assets::Config<Instance> + pallet_evm::Config + frame_system::Config,
    Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
    Runtime::RuntimeCall: From<pallet_assets::Call<Runtime, Instance>>,
    <Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
    BalanceOf<Runtime, Instance>: TryFrom<U256> + Into<U256> + EvmData,
    Runtime: AddressToAssetId<AssetIdOf<Runtime, Instance>>,
    <<Runtime as frame_system::Config>::RuntimeCall as Dispatchable>::RuntimeOrigin: OriginTrait,
{
    fn total_supply(
        asset_id: AssetIdOf<Runtime, Instance>,
        handle: &mut impl PrecompileHandle,
    ) -> EvmResult<PrecompileOutput> {
        handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

        // Fetch info.
        let amount: U256 =
            pallet_assets::Pallet::<Runtime, Instance>::total_issuance(asset_id).into();

        Ok(succeed(EvmDataWriter::new().write(amount).build()))
    }

    fn balance_of(
        asset_id: AssetIdOf<Runtime, Instance>,
        handle: &mut impl PrecompileHandle,
    ) -> EvmResult<PrecompileOutput> {
        handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

        let mut input = handle.read_input()?;
        input.expect_arguments(1)?;

        let owner: H160 = input.read::<Address>()?.into();

        // Fetch info.
        let amount: U256 = {
            let owner: Runtime::AccountId = Runtime::AddressMapping::into_account_id(owner);
            pallet_assets::Pallet::<Runtime, Instance>::balance(asset_id, &owner).into()
        };

        Ok(succeed(EvmDataWriter::new().write(amount).build()))
    }

    fn allowance(
        asset_id: AssetIdOf<Runtime, Instance>,
        handle: &mut impl PrecompileHandle,
    ) -> EvmResult<PrecompileOutput> {
        handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

        let mut input = handle.read_input()?;
        input.expect_arguments(2)?;

        let owner: H160 = input.read::<Address>()?.into();
        let spender: H160 = input.read::<Address>()?.into();

        // Fetch info.
        let amount: U256 = {
            let owner: Runtime::AccountId = Runtime::AddressMapping::into_account_id(owner);
            let spender: Runtime::AccountId = Runtime::AddressMapping::into_account_id(spender);

            // Fetch info.
            pallet_assets::Pallet::<Runtime, Instance>::allowance(asset_id, &owner, &spender).into()
        };

        Ok(succeed(EvmDataWriter::new().write(amount).build()))
    }

    fn approve(
        asset_id: AssetIdOf<Runtime, Instance>,
        handle: &mut impl PrecompileHandle,
    ) -> EvmResult<PrecompileOutput> {
        handle.record_log_costs_manual(3, 32)?;

        let mut input = handle.read_input()?;
        input.expect_arguments(2)?;

        let spender: H160 = input.read::<Address>()?.into();
        let amount: U256 = input.read()?;

        Self::approve_inner(asset_id, handle, handle.context().caller, spender, amount)?;

        LogsBuilder::new(handle.context().address)
            .log3(
                SELECTOR_LOG_APPROVAL,
                handle.context().caller,
                spender,
                EvmDataWriter::new().write(amount).build(),
            )
            .record(handle)?;

        Ok(succeed(EvmDataWriter::new().write(true).build()))
    }

    fn approve_inner(
        asset_id: AssetIdOf<Runtime, Instance>,
        handle: &mut impl PrecompileHandle,
        owner: H160,
        spender: H160,
        value: U256,
    ) -> EvmResult {
        let owner = Runtime::AddressMapping::into_account_id(owner);
        let spender: Runtime::AccountId = Runtime::AddressMapping::into_account_id(spender);
        // Amount saturate if too high.
        let amount: BalanceOf<Runtime, Instance> =
            value.try_into().unwrap_or_else(|_| Bounded::max_value());

        // Allowance read
        handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

        // If previous approval exists, we need to clean it
        if pallet_assets::Pallet::<Runtime, Instance>::allowance(asset_id, &owner, &spender)
            != 0u32.into()
        {
            RuntimeHelper::<Runtime>::try_dispatch(
                handle,
                Some(owner.clone()).into(),
                pallet_assets::Call::<Runtime, Instance>::cancel_approval {
                    id: asset_id.into(),
                    delegate: Runtime::Lookup::unlookup(spender.clone()),
                },
            )?;
        }
        // Dispatch call (if enough gas).
        RuntimeHelper::<Runtime>::try_dispatch(
            handle,
            Some(owner).into(),
            pallet_assets::Call::<Runtime, Instance>::approve_transfer {
                id: asset_id.into(),
                delegate: Runtime::Lookup::unlookup(spender),
                amount,
            },
        )
    }

    fn transfer(
        asset_id: AssetIdOf<Runtime, Instance>,
        handle: &mut impl PrecompileHandle,
    ) -> EvmResult<PrecompileOutput> {
        handle.record_log_costs_manual(3, 32)?;

        let mut input = handle.read_input()?;
        input.expect_arguments(2)?;

        let to: H160 = input.read::<Address>()?.into();
        let amount = input.read::<BalanceOf<Runtime, Instance>>()?;

        // Build call with origin.
        {
            let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
            let to = Runtime::AddressMapping::into_account_id(to);

            // Dispatch call (if enough gas).
            RuntimeHelper::<Runtime>::try_dispatch(
                handle,
                Some(origin).into(),
                pallet_assets::Call::<Runtime, Instance>::transfer {
                    id: asset_id.into(),
                    target: Runtime::Lookup::unlookup(to),
                    amount,
                },
            )?;
        }

        LogsBuilder::new(handle.context().address)
            .log3(
                SELECTOR_LOG_TRANSFER,
                handle.context().caller,
                to,
                EvmDataWriter::new().write(amount).build(),
            )
            .record(handle)?;

        Ok(succeed(EvmDataWriter::new().write(true).build()))
    }

    fn transfer_from(
        asset_id: AssetIdOf<Runtime, Instance>,
        handle: &mut impl PrecompileHandle,
    ) -> EvmResult<PrecompileOutput> {
        handle.record_log_costs_manual(3, 32)?;

        let mut input = handle.read_input()?;
        input.expect_arguments(3)?;

        let from: H160 = input.read::<Address>()?.into();
        let to: H160 = input.read::<Address>()?.into();
        let amount = input.read::<BalanceOf<Runtime, Instance>>()?;

        {
            let caller: Runtime::AccountId =
                Runtime::AddressMapping::into_account_id(handle.context().caller);
            let from: Runtime::AccountId = Runtime::AddressMapping::into_account_id(from);
            let to: Runtime::AccountId = Runtime::AddressMapping::into_account_id(to);

            // If caller is "from", it can spend as much as it wants from its own balance.
            if caller != from {
                // Dispatch call (if enough gas).
                RuntimeHelper::<Runtime>::try_dispatch(
                    handle,
                    Some(caller).into(),
                    pallet_assets::Call::<Runtime, Instance>::transfer_approved {
                        id: asset_id.into(),
                        owner: Runtime::Lookup::unlookup(from),
                        destination: Runtime::Lookup::unlookup(to),
                        amount,
                    },
                )?;
            } else {
                // Dispatch call (if enough gas).
                RuntimeHelper::<Runtime>::try_dispatch(
                    handle,
                    Some(from).into(),
                    pallet_assets::Call::<Runtime, Instance>::transfer {
                        id: asset_id.into(),
                        target: Runtime::Lookup::unlookup(to),
                        amount,
                    },
                )?;
            }
        }

        LogsBuilder::new(handle.context().address)
            .log3(
                SELECTOR_LOG_TRANSFER,
                from,
                to,
                EvmDataWriter::new().write(amount).build(),
            )
            .record(handle)?;

        Ok(succeed(EvmDataWriter::new().write(true).build()))
    }

    fn name(
        asset_id: AssetIdOf<Runtime, Instance>,
        handle: &mut impl PrecompileHandle,
    ) -> EvmResult<PrecompileOutput> {
        handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

        Ok(succeed(
            EvmDataWriter::new()
                .write::<Bytes>(
                    pallet_assets::Pallet::<Runtime, Instance>::name(asset_id)
                        .as_slice()
                        .into(),
                )
                .build(),
        ))
    }

    fn symbol(
        asset_id: AssetIdOf<Runtime, Instance>,
        handle: &mut impl PrecompileHandle,
    ) -> EvmResult<PrecompileOutput> {
        handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

        // Build output.
        Ok(succeed(
            EvmDataWriter::new()
                .write::<Bytes>(
                    pallet_assets::Pallet::<Runtime, Instance>::symbol(asset_id)
                        .as_slice()
                        .into(),
                )
                .build(),
        ))
    }

    fn decimals(
        asset_id: AssetIdOf<Runtime, Instance>,
        handle: &mut impl PrecompileHandle,
    ) -> EvmResult<PrecompileOutput> {
        handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

        // Build output.
        Ok(succeed(
            EvmDataWriter::new()
                .write::<u8>(pallet_assets::Pallet::<Runtime, Instance>::decimals(
                    asset_id,
                ))
                .build(),
        ))
    }

    fn minimum_balance(
        asset_id: AssetIdOf<Runtime, Instance>,
        handle: &mut impl PrecompileHandle,
    ) -> EvmResult<PrecompileOutput> {
        handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

        let min_balance: U256 =
            pallet_assets::Pallet::<Runtime, Instance>::minimum_balance(asset_id).into();

        Ok(succeed(EvmDataWriter::new().write(min_balance).build()))
    }

    fn mint(
        asset_id: AssetIdOf<Runtime, Instance>,
        handle: &mut impl PrecompileHandle,
    ) -> EvmResult<PrecompileOutput> {
        let mut input = handle.read_input()?;
        input.expect_arguments(2)?;

        let beneficiary: H160 = input.read::<Address>()?.into();
        let amount = input.read::<BalanceOf<Runtime, Instance>>()?;

        let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
        let beneficiary = Runtime::AddressMapping::into_account_id(beneficiary);

        // Dispatch call (if enough gas).
        RuntimeHelper::<Runtime>::try_dispatch(
            handle,
            Some(origin).into(),
            pallet_assets::Call::<Runtime, Instance>::mint {
                id: asset_id.into(),
                beneficiary: Runtime::Lookup::unlookup(beneficiary),
                amount,
            },
        )?;

        Ok(succeed(EvmDataWriter::new().write(true).build()))
    }

    fn burn(
        asset_id: AssetIdOf<Runtime, Instance>,
        handle: &mut impl PrecompileHandle,
    ) -> EvmResult<PrecompileOutput> {
        let mut input = handle.read_input()?;
        input.expect_arguments(2)?;

        let who: H160 = input.read::<Address>()?.into();
        let amount = input.read::<BalanceOf<Runtime, Instance>>()?;

        let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
        let who = Runtime::AddressMapping::into_account_id(who);

        // Dispatch call (if enough gas).
        RuntimeHelper::<Runtime>::try_dispatch(
            handle,
            Some(origin).into(),
            pallet_assets::Call::<Runtime, Instance>::burn {
                id: asset_id.into(),
                who: Runtime::Lookup::unlookup(who),
                amount,
            },
        )?;

        Ok(succeed(EvmDataWriter::new().write(true).build()))
    }
}
