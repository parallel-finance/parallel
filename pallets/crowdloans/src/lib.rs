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

//! # Crowdloans
//!

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

use frame_support::{
    dispatch::DispatchResult,
    pallet_prelude::*,
    traits::{
        fungibles::{Inspect, Mutate, Transfer},
        Get,
    },
    Blake2_128Concat, PalletId,
};

mod crowdloan_structs;
use crowdloan_structs::{
    ContributionStrategy, ContributionStrategyExecutor, ParaId, Vault, VaultPhase,
};

use frame_system::ensure_signed;
use frame_system::pallet_prelude::OriginFor;

// pub use pallet::*;

use sp_runtime::{
    traits::{AccountIdConversion, Zero},
    DispatchError,
};

pub use pallet::*;

macro_rules! switch_relay {
    ({ $( $code:tt )* }) => {
        if T::RelayNetwork::get() == NetworkId::Polkadot {
            use crate::crowdloan_structs::PolkadotCall as RelaychainCall;

            $( $code )*
        } else if T::RelayNetwork::get() == NetworkId::Kusama {
            use crate::crowdloan_structs::KusamaCall as RelaychainCall;

            $( $code )*
        } else if T::RelayNetwork::get() == NetworkId::Named("westend".into()) {
            use crate::crowdloan_structs::WestendCall as RelaychainCall;

            $( $code )*
        } else {
            unreachable!()
        }
    }
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use crate::crowdloan_structs::*;
    use frame_system::ensure_root;
    use xcm::latest::prelude::*;
    use xcm::DoubleEncoded;

    use primitives::{Balance, CurrencyId};

    pub type AssetIdOf<T> =
        <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;
    pub type BalanceOf<T> =
        <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        // /// Currency type for deposit/withdraw assets to/from crowdloan
        // /// module
        // type Assets: Transfer<Self::AccountId> + Inspect<Self::AccountId> + Mutate<Self::AccountId>;
        /// Assets for deposit/withdraw assets to/from pallet account
        type Assets: Transfer<Self::AccountId, AssetId = CurrencyId>
            + Inspect<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
            + Mutate<Self::AccountId, Balance = Balance>;

        /// XCM message sender
        type XcmSender: SendXcm;

        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// Relay network
        #[pallet::constant]
        type RelayNetwork: Get<NetworkId>;

        /// Account derivative index
        #[pallet::constant]
        type DerivativeIndex: Get<u16>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    // #[pallet::metadata(T::AccountId = "AccountId", BalanceOf<T> = "Balance")]
    // pub enum Event<T: Config: 'static = ()> {
    pub enum Event<T: Config> {
        /// Create new vault
        /// [token, crowdloan, project_shares, currency_shares]
        VaultCreated(AssetIdOf<T>, ParaId, AssetIdOf<T>, AssetIdOf<T>),

        /// Send participate with amount to relaychain
        ParticipteCallSent(BalanceOf<T>),
    }

    #[pallet::error]
    pub enum Error<T> {
        // pub enum Error<T = ()> {
        /// Vault is not in correct phase
        IncorrectVaultPhase,
        /// Vault shares are not new
        SharesNotNew,
        // Crowdload ParaId aready exists
        CrowdloanAlreadyExists,
        // Crowdload ParaId does not exist
        CrowdloanDoesNotExists,
        // Amount is not enough
        InsufficientAmount,
        // Vault does not exist
        VaultDoesNotExist,
        // Vault contributed greater than issuance
        ContributedGreaterThanIssuance,

        /// Failed to send participate call
        ParticipateCallFailed,
        /// Xcm fees given are too low to execute on relaychain
        XcmFeesCompensationTooLow,
    }

    #[pallet::storage]
    #[pallet::getter(fn vaults)]
    pub type Vaults<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        ParaId,
        Vault<ParaId, AssetIdOf<T>, BalanceOf<T>>,
        OptionQuery,
    >;

    #[pallet::genesis_config]
    pub struct GenesisConfig {
        pub exchange_rate: usize,
        pub reserve_factor: usize,
    }

    #[cfg(feature = "std")]
    impl Default for GenesisConfig {
        fn default() -> Self {
            Self {
                exchange_rate: 1,
                reserve_factor: 1,
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig {
        fn build(&self) {}
    }

    #[pallet::call]
    impl<T: Config> Pallet<T>
    where
        [u8; 32]: From<<T as frame_system::Config>::AccountId>,
        // [u8; 32]: From<<T as frame_system::Config>::AccountId>,
        // u128: From<
        //     <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance,
        // >,
        // BalanceOf<T>: FixedPointOperand,
        // AssetIdOf<T>: AtLeast32BitUnsigned,
    {
        //// 1. Vaults Management

        /// Create a new vault via a governance decision
        /// - `currency` is the currency or token which needs to be deposited to fill
        ///   the vault and later participate in the crowdloans
        /// - `crowdloan` represents which crowdloan we are supporting on the relay
        ///   chain
        /// - `ctoken` is a new asset created for this vault to represent the shares
        ///   of the vault's contributors which will later be used for refunding their
        ///   contributions
        /// - `contribution_strategy` represents how we can contribute coins to the
        ///   crowdloan on the relay chain
        #[pallet::weight(10_000)]
        #[allow(unused)]
        pub fn create_vault(
            origin: OriginFor<T>,
            currency: AssetIdOf<T>,
            crowdloan: ParaId,
            ctoken: AssetIdOf<T>,
            contribution_strategy: ContributionStrategy<
                ParaId,
                primitives::CurrencyId,
                primitives::Balance,
            >,
        ) -> DispatchResult {
            // 1. EnsureOrigin
            ensure_root(origin)?;

            // 2. make sure both ctoken is a new asset (total_issuance == Zero::zero())
            let ctoken_issuance = T::Assets::total_issuance(ctoken);

            // make sure both project_shares and currency_shares are new assets
            ensure!(ctoken_issuance == Zero::zero(), Error::<T>::SharesNotNew);

            // 3. make sure no similar vault already exists as identified by crowdloan
            // add new vault to vaults storage
            Vaults::<T>::try_mutate(&crowdloan, |vault| -> Result<_, DispatchError> {
                // make sure no similar vault already exists as identified by crowdloan
                ensure!(vault.is_none(), Error::<T>::CrowdloanAlreadyExists);

                // 4. mutate our storage to register a new vault
                // inialize new vault
                *vault = Some(crowdloan_structs::Vault {
                    ctoken,
                    currency,
                    phase: VaultPhase::CollectingContributions,
                    contribution_strategy: ContributionStrategy::XCM,
                    contributed: Zero::zero(),
                });

                Ok(())
            })
        }

        ////
        //// 2. Contribution to Vaults

        /// Contribute `amount` to the vault of `crowdloan` and receive some
        /// shares from it
        #[pallet::weight(10_000)]
        #[allow(unused)]
        pub fn contribute(
            origin: OriginFor<T>,
            crowdloan: ParaId,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 1. Make sure crowdloan has a vault linked to it
            let vault = Self::vault(crowdloan)?;

            // 2. Make sure the vault.phase == CollectingContributions
            ensure!(
                vault.phase == VaultPhase::CollectingContributions,
                Error::<T>::IncorrectVaultPhase
            );

            // 3. Make sure origin has at least amount of vault.currency
            // get amount origin has
            let origin_currency_amount = T::Assets::balance(vault.currency, &who);

            ensure!(
                origin_currency_amount >= amount,
                Error::<T>::InsufficientAmount
            );

            // 4. Wire amount of vault.currency to the pallet's account id
            T::Assets::transfer(vault.currency, &who, &Self::account_id(), amount, true)?;

            // 5. Create amount of vault.ctoken to origin
            T::Assets::mint_into(vault.ctoken, &who, amount)?;

            Ok(())
        }

        ////
        //// 3. Triggering participation in a crowdloan

        /// Once a auction loan vault is expired, move the coins to the relay chain
        /// and participate in a relay chain crowdloan by using the call `call`.
        #[pallet::weight(10_000)]
        #[allow(unused)]
        pub fn participate(origin: OriginFor<T>, crowdloan: ParaId) -> DispatchResult {
            // 1. EnsureOrigin
            ensure_root(origin)?;

            Vaults::<T>::try_mutate(&crowdloan, |vault| -> Result<_, DispatchError> {
                // make sure there's a vault
                ensure!(vault.is_some(), Error::<T>::CrowdloanDoesNotExists);

                if let Some(vault_contents) = vault {
                    // 2. Make sure vault.contributed is less than total_issuance(vault.currency_shares)
                    // let vault_currency_issuance =
                    //     T::Assets::total_issuance(vault_contents.currency);

                    let vault_ctoken_issuance = T::Assets::total_issuance(vault_contents.ctoken);

                    ensure!(
                        vault_contents.contributed < vault_ctoken_issuance,
                        Error::<T>::ContributedGreaterThanIssuance
                    );

                    // 3. Execute vault.contribution_strategy with parameters crowdloan,
                    // vault.currency and total_issuance(vault.ctoken) - vault.contributed
                    let amount = vault_ctoken_issuance - vault_contents.contributed;

                    println!("vault_ctoken_issuance:\t{:?}", vault_ctoken_issuance);
                    println!(
                        "vault_contents.contributed:\t{:?}",
                        vault_contents.contributed
                    );
                    println!("amount:\t{:?}", amount);

                    // this logic should be movec into the execute function on the executor state
                    switch_relay!({
                        let call = RelaychainCall::Utility(Box::new(UtilityCall::AsDerivative(
                            UtilityAsDerivativeCall {
                                index: T::DerivativeIndex::get(),
                                call: RelaychainCall::Crowdloan::<T>(CrowdloanCall::Participate(
                                    CrowdloanParticipateCall { value: amount },
                                )),
                            },
                        )));

                        let msg = Self::ump_transact(call.encode().into(), 1_000_000)?;

                        match T::XcmSender::send_xcm(MultiLocation::parent(), msg) {
                            Ok(()) => {
                                Self::deposit_event(Event::<T>::ParticipteCallSent(amount));
                            }
                            Err(_e) => {
                                return Err(Error::<T>::ParticipateCallFailed.into());
                            }
                        }
                    });

                    vault_contents.contribution_strategy.execute(
                        crowdloan,
                        vault_contents.currency,
                        amount,
                    );

                    // 4. Set vault.contributed to total_issuance(vault.currency_shares)
                    vault_contents.contributed = vault_ctoken_issuance;

                    // update storage
                    *vault = Some(*vault_contents);
                }

                Ok(())
            })
        }

        ////
        //// 4. Handling Auction Closure

        /// Mark the associated vault as closed and stop accepting contributions for it
        #[pallet::weight(10_000)]
        #[allow(unused)]

        pub fn close(origin: OriginFor<T>, crowdloan: ParaId) -> DispatchResult {
            // 1. EnsureOrigin
            ensure_root(origin)?;

            Vaults::<T>::try_mutate(&crowdloan, |vault| -> Result<_, DispatchError> {
                // make sure there's a vault
                ensure!(vault.is_some(), Error::<T>::CrowdloanDoesNotExists);

                if let Some(vault_contents) = vault {
                    // 2. Make sure vault.phase == VaultPhase::CollectingContributions
                    ensure!(
                        vault_contents.phase == VaultPhase::CollectingContributions,
                        Error::<T>::IncorrectVaultPhase
                    );

                    // 3. Change vault.phase to Closed
                    vault_contents.phase = VaultPhase::Closed;

                    // update storage
                    *vault = Some(*vault_contents);
                }
                Ok(())
            })
        }

        ////
        //// 5. Handling Failed Auctions

        /// If a `crowdloan` failed, get the coins back and mark the vault as ready
        /// for distribution
        #[pallet::weight(10_000)]
        #[allow(unused)]
        pub fn auction_failed(origin: OriginFor<T>, crowdloan: ParaId) -> DispatchResult {
            // 1. `EnsureOrigin`
            ensure_root(origin)?;

            Vaults::<T>::try_mutate(&crowdloan, |vault| -> Result<_, DispatchError> {
                // make sure there's a vault
                ensure!(vault.is_some(), Error::<T>::CrowdloanDoesNotExists);

                if let Some(vault_contents) = vault {
                    // 2. Make sure `vault.phase == Closed`
                    ensure!(
                        vault_contents.phase == VaultPhase::Closed,
                        Error::<T>::IncorrectVaultPhase
                    );

                    // 3. Execute the `refund` function of the `contribution_strategy`
                    vault_contents
                        .contribution_strategy
                        .refund(crowdloan, vault_contents.currency);

                    // 4. Set `vault.phase` to `Failed`
                    vault_contents.phase = VaultPhase::Failed;

                    // update storage
                    *vault = Some(*vault_contents);
                }
                Ok(())
            })
        }

        /// If a `crowdloan` failed, claim back your share of the assets you
        /// contributed
        #[pallet::weight(10_000)]
        #[allow(unused)]
        pub fn claim_refund(
            origin: OriginFor<T>,
            crowdloan: ParaId,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Vaults::<T>::try_mutate(&crowdloan, |vault| -> Result<_, DispatchError> {
                // make sure there's a vault
                ensure!(vault.is_some(), Error::<T>::CrowdloanDoesNotExists);

                if let Some(vault_contents) = vault {
                    // 1. Make sure `vault.phase == Failed` **or `Expired`** (more on that later)
                    ensure!(
                        vault_contents.phase == VaultPhase::Failed
                            || vault_contents.phase == VaultPhase::Expired,
                        Error::<T>::IncorrectVaultPhase
                    );

                    // 2. Make sure `origin` has at least `amount` of `vault.ctoken`
                    // get amount origin has
                    let origin_ctoken_amount =
                        <T as Config>::Assets::balance(vault_contents.ctoken, &who);

                    ensure!(
                        origin_ctoken_amount >= amount,
                        Error::<T>::InsufficientAmount
                    );

                    // 3. Burns `amount` from `vault.ctoken`
                    T::Assets::burn_from(vault_contents.ctoken, &who, amount)?;

                    // 4. Wire `amount` of `vault.currency` from our account id to the caller
                    T::Assets::transfer(
                        vault_contents.currency,
                        &Self::account_id(),
                        &who,
                        amount,
                        false,
                    )?;
                }
                Ok(())
            })
        }

        ////
        //// 6. Refunding the contributed assets after auction success

        /// If a `crowdloan` succeeded and its slot expired, use `call` to
        /// claim back the funds lent to the parachain
        #[pallet::weight(10_000)]
        #[allow(unused)]
        pub fn slot_expired(origin: OriginFor<T>, crowdloan: ParaId) -> DispatchResult {
            // 1. `EnsureOrigin`
            ensure_root(origin)?;

            Vaults::<T>::try_mutate(&crowdloan, |vault| -> Result<_, DispatchError> {
                // make sure there's a vault
                ensure!(vault.is_some(), Error::<T>::CrowdloanDoesNotExists);

                if let Some(vault_contents) = vault {
                    // 2. Execute the `withdraw` function of our `contribution_strategy`
                    vault_contents
                        .contribution_strategy
                        .withdraw(crowdloan, vault_contents.currency);

                    // 3. Modify `vault.phase` to `Expired
                    vault_contents.phase = VaultPhase::Expired;

                    // update storage
                    *vault = Some(*vault_contents);
                }
                Ok(())
            })
        }
    }

    impl<T: Config> Pallet<T>
    where
        [u8; 32]: From<<T as frame_system::Config>::AccountId>,
    {
        /// Crowdloan pool account
        pub fn account_id() -> T::AccountId {
            T::PalletId::get().into_account()
        }

        /// Parachain sovereign account
        pub fn para_account_id() -> T::AccountId {
            // T::SelfParaId::get().into_account()
            T::PalletId::get().into_account()
        }

        // Returns a stored Vault.
        //
        // Returns `Err` if market does not exist.
        pub fn vault(
            crowdloan: ParaId,
        ) -> Result<Vault<ParaId, AssetIdOf<T>, BalanceOf<T>>, DispatchError> {
            Vaults::<T>::try_get(crowdloan).map_err(|_err| Error::<T>::VaultDoesNotExist.into())
        }

        fn ump_transact(call: DoubleEncoded<()>, weight: Weight) -> Result<Xcm<()>, DispatchError> {
            // let fees = Self::xcm_fees_compensation();
            let fees = 10;
            ensure!(!fees.is_zero(), Error::<T>::XcmFeesCompensationTooLow);

            // let account_id = Self::account_id();
            let asset: MultiAsset = (MultiLocation::here(), fees).into();

            Ok(Xcm(vec![
                WithdrawAsset(MultiAssets::from(asset.clone())),
                BuyExecution {
                    fees: asset.clone(),
                    weight_limit: Unlimited,
                },
                Transact {
                    origin_type: OriginKind::SovereignAccount,
                    require_weight_at_most: weight,
                    call,
                },
                RefundSurplus,
                DepositAsset {
                    assets: asset.into(),
                    max_assets: 1,
                    beneficiary: MultiLocation {
                        parents: 1,
                        interior: X1(AccountId32 {
                            network: NetworkId::Any,
                            id: Self::para_account_id().into(),
                        }),
                    },
                },
            ]))
        }
    }
}
