#![cfg_attr(not(feature = "std"), no_std)]

pub mod precompiles;

pub use fp_rpc;
pub use fp_self_contained;
pub use pallet_evm_precompile_assets_erc20::AddressToAssetId;
pub use pallet_evm_precompile_balances_erc20::Erc20Metadata;
