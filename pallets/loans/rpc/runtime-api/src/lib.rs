#![cfg_attr(not(feature = "std"), no_std)]
use codec::Codec;
use sp_runtime::FixedU128;

sp_api::decl_runtime_apis! {
    pub trait LoanApi<AccountId> where
        AccountId: Codec, {
        fn get_account_liquidity(account: AccountId) -> (FixedU128, FixedU128);
    }
}
