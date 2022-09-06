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

pub use pallet::*;

/// Ethereum-compatible signatures (eth_sign API call).
pub mod ethereum;
pub mod weights;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{
        pallet_prelude::*,
        traits::{
            tokens::{
                fungible::Transfer,
                fungibles::{Inspect as Inspects, Mutate as Mutates, Transfer as Transfers},
            },
            Currency, ExistenceRequirement, Get, OnUnbalanced, UnfilteredDispatchable,
            WithdrawReasons,
        },
        transactional,
        weights::GetDispatchInfo,
    };
    use frame_system::{ensure_none, pallet_prelude::*};
    use pallet_evm::{AddressMapping, EnsureAddressOrigin};
    use primitives::{Balance, CurrencyId};
    use sp_core::H160;
    use sp_runtime::traits::{IdentifyAccount, Verify};
    use sp_std::{convert::TryFrom, prelude::*};
    use weights::WeightInfo;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// The balance type of this pallet.
    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    pub type AssetBalanceOf<T> =
        <<T as Config>::Assets as Inspects<<T as frame_system::Config>::AccountId>>::Balance;

    pub type AssetIdOf<T> =
        <<T as Config>::Assets as Inspects<<T as frame_system::Config>::AccountId>>::AssetId;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// A signable call.
        type Call: Parameter + UnfilteredDispatchable<Origin = Self::Origin> + GetDispatchInfo;

        /// User defined signature type.
        type Signature: Parameter + Verify<Signer = Self::Signer> + TryFrom<Vec<u8>>;

        /// User defined signer type.
        type Signer: IdentifyAccount<AccountId = Self::AccountId>;

        /// The currency trait.
        type Currency: Currency<Self::AccountId> + Transfer<Self::AccountId, Balance = Balance>;

        /// The call fee destination.
        type OnChargeTransaction: OnUnbalanced<
            <Self::Currency as Currency<Self::AccountId>>::NegativeImbalance,
        >;

        /// The call processing fee amount.
        #[pallet::constant]
        type CallFee: Get<BalanceOf<Self>>;

        /// The call magic number.
        #[pallet::constant]
        type CallMagicNumber: Get<u16>;

        /// A configuration for base priority of unsigned transactions.
        ///
        /// This is exposed so that it can be tuned for particular runtime, when
        /// multiple pallets send unsigned transactions.
        type UnsignedPriority: Get<TransactionPriority>;

        /// Allow the origin to withdraw on behalf of given address.
        type WithdrawOrigin: EnsureAddressOrigin<Self::Origin, Success = Self::AccountId>;

        /// Enable signature verify or not
        #[pallet::constant]
        type VerifySignature: Get<bool>;

        type Assets: Transfers<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
            + Inspects<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
            + Mutates<Self::AccountId, AssetId = CurrencyId, Balance = Balance>;

        #[pallet::constant]
        type GetNativeCurrencyId: Get<AssetIdOf<Self>>;

        /// Mapping from address to account id.
        type AddressMapping: AddressMapping<Self::AccountId>;

        /// Weight information
        type WeightInfo: WeightInfo;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Signature decode fails.
        DecodeFailure,
        /// Signature and account mismatched.
        InvalidSignature,
        /// Bad nonce parameter.
        BadNonce,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A call just executed. \[result\]
        Executed(T::AccountId, DispatchResult),
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// # <weight>
        /// - O(1).
        /// - Limited storage reads.
        /// - One DB write (event).
        /// - Weight of derivative `call` execution + 10_000_000.
        /// # </weight>
        #[pallet::weight({
            let dispatch_info = call.get_dispatch_info();
            (dispatch_info.weight.saturating_add(10_000_000), dispatch_info.class)
        })]
        pub fn call(
            origin: OriginFor<T>,
            call: Box<<T as Config>::Call>,
            signer: T::AccountId,
            signature: Vec<u8>,
            #[pallet::compact] nonce: T::Index,
        ) -> DispatchResultWithPostInfo {
            ensure_none(origin)?;

            // Ensure that transaction isn't stale
            ensure!(
                nonce == frame_system::Pallet::<T>::account_nonce(signer.clone()),
                Error::<T>::BadNonce,
            );

            let signature = <T as Config>::Signature::try_from(signature)
                .map_err(|_| Error::<T>::DecodeFailure)?;

            // Ensure that transaction signature is valid
            ensure!(
                Self::valid_signature(&call, &signer, &signature, &nonce),
                Error::<T>::InvalidSignature
            );

            // Increment account nonce
            frame_system::Pallet::<T>::inc_account_nonce(signer.clone());

            // Processing fee
            let tx_fee = T::Currency::withdraw(
                &signer,
                T::CallFee::get(),
                WithdrawReasons::FEE,
                ExistenceRequirement::AllowDeath,
            )?;
            T::OnChargeTransaction::on_unbalanced(tx_fee);

            // Dispatch call
            let new_origin = frame_system::RawOrigin::Signed(signer.clone()).into();
            let res = call.dispatch_bypass_filter(new_origin).map(|_| ());
            Self::deposit_event(Event::Executed(signer, res.map_err(|e| e.error)));

            // Fee already charged
            Ok(Pays::No.into())
        }

        #[pallet::weight(<T as Config>::WeightInfo::withdraw())]
        #[transactional]
        pub fn withdraw(
            origin: OriginFor<T>,
            asset: AssetIdOf<T>,
            address: H160,
            value: AssetBalanceOf<T>,
        ) -> DispatchResult {
            let destination = T::WithdrawOrigin::ensure_address_origin(&address, origin)?;
            let address_account_id = T::AddressMapping::into_account_id(address);

            Self::transfer(asset, &address_account_id, &destination, value)?;

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Verify custom signature and returns `true` if correct.
        pub fn valid_signature(
            call: &<T as Config>::Call,
            signer: &T::AccountId,
            signature: &T::Signature,
            nonce: &T::Index,
        ) -> bool {
            let payload = (T::CallMagicNumber::get(), *nonce, call.clone());
            //temporarily disable
            if T::VerifySignature::get() {
                signature.verify(&payload.encode()[..], signer)
            } else {
                true
            }
        }

        fn transfer(
            asset: AssetIdOf<T>,
            source: &T::AccountId,
            dest: &T::AccountId,
            amount: AssetBalanceOf<T>,
        ) -> Result<AssetBalanceOf<T>, DispatchError> {
            if asset == T::GetNativeCurrencyId::get() {
                <<T as pallet::Config>::Currency as Transfer<T::AccountId>>::transfer(
                    source, dest, amount, true,
                )
            } else {
                T::Assets::transfer(asset, source, dest, amount, true)
            }
        }
    }

    pub(crate) const SIGNATURE_DECODE_FAILURE: u8 = 1;

    #[pallet::validate_unsigned]
    impl<T: Config> frame_support::unsigned::ValidateUnsigned for Pallet<T> {
        type Call = Call<T>;

        fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
            // Call decomposition (we have only one possible value here)
            let (call, signer, signature, nonce) = match call {
                Call::call {
                    call,
                    signer,
                    signature,
                    nonce,
                } => (call, signer, signature, nonce),
                _ => return InvalidTransaction::Call.into(),
            };

            // Check that tx isn't stale
            if *nonce != frame_system::Pallet::<T>::account_nonce(signer.clone()) {
                return InvalidTransaction::Stale.into();
            }

            // Check signature encoding
            if let Ok(signature) = <T as Config>::Signature::try_from(signature.clone()) {
                // Verify signature
                if Self::valid_signature(call, signer, &signature, nonce) {
                    ValidTransaction::with_tag_prefix("EVMSignatures")
                        .priority(T::UnsignedPriority::get())
                        .and_provides((call, signer, nonce))
                        .longevity(64_u64)
                        .propagate(true)
                        .build()
                } else {
                    // Signature mismatched to given signer
                    InvalidTransaction::BadProof.into()
                }
            } else {
                // Signature encoding broken
                InvalidTransaction::Custom(SIGNATURE_DECODE_FAILURE).into()
            }
        }

        fn pre_dispatch(_call: &Self::Call) -> Result<(), TransactionValidityError> {
            Ok(())
        }
    }
}
