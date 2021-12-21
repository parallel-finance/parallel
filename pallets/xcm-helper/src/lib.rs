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

//! # Common XCM Helper pallet
//!
//! ## Overview
//! This pallet should be in charge of everything XCM related including callbacks and sending XCM calls.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

use frame_support::{
    dispatch::{DispatchResult, GetDispatchInfo},
    pallet_prelude::*,
    traits::fungibles::{Inspect, Mutate, Transfer},
    PalletId,
};

use primitives::{switch_relay, ump::*, Balance, CurrencyId, ParaId};
use sp_runtime::traits::{AccountIdConversion, BlockNumberProvider, StaticLookup};
use sp_std::{boxed::Box, vec, vec::Vec};
use xcm::{latest::prelude::*, DoubleEncoded};
use xcm_executor::traits::InvertLocation;

pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type AssetIdOf<T> =
    <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;
pub type BalanceOf<T> =
    <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
    use frame_system::pallet_prelude::BlockNumberFor;

    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_xcm::Config {
        /// Assets for deposit/withdraw assets to/from crowdloan account
        type Assets: Transfer<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
            + Inspect<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
            + Mutate<Self::AccountId, AssetId = CurrencyId, Balance = Balance>;

        /// XCM message sender
        type XcmSender: SendXcm;

        /// Relay network
        #[pallet::constant]
        type RelayNetwork: Get<NetworkId>;

        /// Pallet account for collecting xcm fees
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// Notify call timeout
        #[pallet::constant]
        type NotifyTimeout: Get<BlockNumberFor<Self>>;

        /// The block number provider
        type BlockNumberProvider: BlockNumberProvider<BlockNumber = Self::BlockNumber>;
    }

    #[pallet::storage]
    #[pallet::getter(fn xcm_fees)]
    pub type XcmFees<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn xcm_weight)]
    pub type XcmWeight<T: Config> = StorageValue<_, XcmWeightMisc<Weight>, ValueQuery>;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::error]
    pub enum Error<T> {
        /// `MultiLocation` value ascend more parents than known ancestors of local location.
        MultiLocationNotInvertible,
        /// Xcm message send failure
        SendXcmError,
    }
}

pub trait XcmHelper<T: pallet_xcm::Config, Balance, AssetId, AccountId> {
    fn get_xcm_fees() -> Balance;

    fn update_xcm_fees(fees: Balance);

    fn update_xcm_weight(xcm_weight_misc: XcmWeightMisc<Weight>);

    fn add_xcm_fees(relay_currency: AssetId, payer: AccountId, amount: Balance) -> DispatchResult;

    fn ump_transact(
        call: DoubleEncoded<()>,
        weight: Weight,
        beneficiary: MultiLocation,
        currency: AssetId,
    ) -> Result<Xcm<()>, DispatchError>;

    fn do_withdraw(
        para_id: ParaId,
        beneficiary: MultiLocation,
        relay_currency: AssetId,
        para_account_id: AccountId,
        notify: impl Into<<T as pallet_xcm::Config>::Call>,
    ) -> Result<QueryId, DispatchError>;

    fn do_contribute(
        para_id: ParaId,
        beneficiary: MultiLocation,
        relay_currency: AssetId,
        amount: Balance,
        who: &AccountId,
        notify: impl Into<<T as pallet_xcm::Config>::Call>,
    ) -> Result<QueryId, DispatchError>;

    fn do_bond(
        value: Balance,
        payee: RewardDestination<AccountId>,
        stash: AccountId,
        beneficiary: MultiLocation,
        staking_currency: AssetId,
        index: u16,
    ) -> DispatchResult;

    fn do_bond_extra(
        value: Balance,
        stash: AccountId,
        beneficiary: MultiLocation,
        staking_currency: AssetId,
        index: u16,
    ) -> DispatchResult;

    fn do_unbond(
        value: Balance,
        beneficiary: MultiLocation,
        staking_currency: AssetId,
        index: u16,
    ) -> DispatchResult;

    fn do_rebond(
        value: Balance,
        beneficiary: MultiLocation,
        staking_currency: AssetId,
        index: u16,
    ) -> DispatchResult;

    fn do_withdraw_unbonded(
        num_slashing_spans: u32,
        amount: Balance,
        beneficiary: MultiLocation,
        staking_currency: AssetId,
        para_account_id: AccountId,
        index: u16,
    ) -> DispatchResult;

    fn do_nominate(
        targets: Vec<AccountId>,
        beneficiary: MultiLocation,
        staking_currency: AssetId,
        index: u16,
    ) -> DispatchResult;
}

impl<T: Config> Pallet<T> {
    pub fn account_id() -> T::AccountId {
        T::PalletId::get().into_account()
    }

    pub fn report_outcome_notify(
        message: &mut Xcm<()>,
        responder: impl Into<MultiLocation>,
        notify: impl Into<<T as pallet_xcm::Config>::Call>,
        timeout: T::BlockNumber,
    ) -> Result<QueryId, DispatchError> {
        let responder = responder.into();
        let dest = <T as pallet_xcm::Config>::LocationInverter::invert_location(&responder)
            .map_err(|()| Error::<T>::MultiLocationNotInvertible)?;
        let notify: <T as pallet_xcm::Config>::Call = notify.into();
        let max_response_weight = notify.get_dispatch_info().weight;
        let query_id = pallet_xcm::Pallet::<T>::new_notify_query(responder, notify, timeout);
        let report_error = Xcm(vec![ReportError {
            dest,
            query_id,
            max_response_weight,
        }]);
        // Prepend SetAppendix(Xcm(vec![ReportError])) wont be able to pass barrier check
        // so we need to insert it after Withdraw, BuyExecution
        message.0.insert(2, SetAppendix(report_error));
        Ok(query_id)
    }
}

impl<T: Config> XcmHelper<T, BalanceOf<T>, AssetIdOf<T>, T::AccountId> for Pallet<T> {
    fn get_xcm_fees() -> BalanceOf<T> {
        Self::xcm_fees()
    }

    fn update_xcm_fees(fees: BalanceOf<T>) {
        XcmFees::<T>::mutate(|v| *v = fees);
    }

    fn update_xcm_weight(xcm_weight_misc: XcmWeightMisc<Weight>) {
        XcmWeight::<T>::mutate(|v| *v = xcm_weight_misc);
    }

    fn add_xcm_fees(
        relay_currency: AssetIdOf<T>,
        payer: T::AccountId,
        amount: BalanceOf<T>,
    ) -> DispatchResult {
        T::Assets::transfer(relay_currency, &payer, &Self::account_id(), amount, false)?;
        Ok(())
    }

    fn ump_transact(
        call: DoubleEncoded<()>,
        weight: Weight,
        beneficiary: MultiLocation,
        currency: AssetIdOf<T>,
    ) -> Result<Xcm<()>, DispatchError> {
        let fees = Self::xcm_fees();
        let asset: MultiAsset = (MultiLocation::here(), fees).into();

        T::Assets::burn_from(currency, &Self::account_id(), fees)?;

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
                beneficiary,
            },
        ]))
    }

    fn do_withdraw(
        para_id: ParaId,
        beneficiary: MultiLocation,
        relay_currency: AssetIdOf<T>,
        para_account_id: T::AccountId,
        notify: impl Into<<T as pallet_xcm::Config>::Call>,
    ) -> Result<QueryId, DispatchError> {
        Ok(switch_relay!({
            let call =
                RelaychainCall::<T>::Crowdloans(CrowdloansCall::Withdraw(CrowdloansWithdrawCall {
                    who: para_account_id,
                    index: para_id,
                }));

            let mut msg = Self::ump_transact(
                call.encode().into(),
                Self::xcm_weight().withdraw_weight,
                beneficiary,
                relay_currency,
            )?;

            let query_id = Self::report_outcome_notify(
                &mut msg,
                MultiLocation::parent(),
                notify,
                T::NotifyTimeout::get(),
            )?;

            if let Err(_e) = T::XcmSender::send_xcm(MultiLocation::parent(), msg) {
                return Err(Error::<T>::SendXcmError.into());
            }

            query_id
        }))
    }

    fn do_contribute(
        para_id: ParaId,
        beneficiary: MultiLocation,
        relay_currency: AssetIdOf<T>,
        amount: BalanceOf<T>,
        _who: &T::AccountId,
        notify: impl Into<<T as pallet_xcm::Config>::Call>,
    ) -> Result<QueryId, DispatchError> {
        Ok(switch_relay!({
            let call = RelaychainCall::<T>::Crowdloans(CrowdloansCall::Contribute(
                CrowdloansContributeCall {
                    index: para_id,
                    value: amount,
                    signature: None,
                },
            ));

            let mut msg = Self::ump_transact(
                call.encode().into(),
                Self::xcm_weight().contribute_weight,
                beneficiary,
                relay_currency,
            )?;

            let query_id = Self::report_outcome_notify(
                &mut msg,
                MultiLocation::parent(),
                notify,
                T::NotifyTimeout::get(),
            )?;

            if let Err(_e) = T::XcmSender::send_xcm(MultiLocation::parent(), msg) {
                return Err(Error::<T>::SendXcmError.into());
            }

            query_id
        }))
    }

    fn do_bond(
        value: BalanceOf<T>,
        payee: RewardDestination<T::AccountId>,
        stash: T::AccountId,
        beneficiary: MultiLocation,
        staking_currency: AssetIdOf<T>,
        index: u16,
    ) -> DispatchResult {
        let controller = stash.clone();

        switch_relay!({
            let call =
                RelaychainCall::Utility(Box::new(UtilityCall::BatchAll(UtilityBatchAllCall {
                    calls: vec![
                        RelaychainCall::Balances(BalancesCall::TransferKeepAlive(
                            BalancesTransferKeepAliveCall {
                                dest: T::Lookup::unlookup(stash),
                                value,
                            },
                        )),
                        RelaychainCall::Utility(Box::new(UtilityCall::AsDerivative(
                            UtilityAsDerivativeCall {
                                index,
                                call: RelaychainCall::Staking::<T>(StakingCall::Bond(
                                    StakingBondCall {
                                        controller: T::Lookup::unlookup(controller),
                                        value,
                                        payee,
                                    },
                                )),
                            },
                        ))),
                    ],
                })));

            let msg = Self::ump_transact(
                call.encode().into(),
                Self::xcm_weight().bond_weight,
                beneficiary,
                staking_currency,
            )?;

            if let Err(_err) = T::XcmSender::send_xcm(MultiLocation::parent(), msg) {
                return Err(Error::<T>::SendXcmError.into());
            }
        });

        Ok(())
    }

    fn do_bond_extra(
        value: BalanceOf<T>,
        stash: T::AccountId,
        beneficiary: MultiLocation,
        staking_currency: AssetIdOf<T>,
        index: u16,
    ) -> DispatchResult {
        switch_relay!({
            let call =
                RelaychainCall::Utility(Box::new(UtilityCall::BatchAll(UtilityBatchAllCall {
                    calls: vec![
                        RelaychainCall::Balances(BalancesCall::TransferKeepAlive(
                            BalancesTransferKeepAliveCall {
                                dest: T::Lookup::unlookup(stash),
                                value,
                            },
                        )),
                        RelaychainCall::Utility(Box::new(UtilityCall::AsDerivative(
                            UtilityAsDerivativeCall {
                                index,
                                call: RelaychainCall::Staking::<T>(StakingCall::BondExtra(
                                    StakingBondExtraCall { value },
                                )),
                            },
                        ))),
                    ],
                })));

            let msg = Self::ump_transact(
                call.encode().into(),
                Self::xcm_weight().bond_extra_weight,
                beneficiary,
                staking_currency,
            )?;

            if let Err(_err) = T::XcmSender::send_xcm(MultiLocation::parent(), msg) {
                return Err(Error::<T>::SendXcmError.into());
            }
        });
        Ok(())
    }

    fn do_unbond(
        value: BalanceOf<T>,
        beneficiary: MultiLocation,
        staking_currency: AssetIdOf<T>,
        index: u16,
    ) -> DispatchResult {
        switch_relay!({
            let call = RelaychainCall::Utility(Box::new(UtilityCall::AsDerivative(
                UtilityAsDerivativeCall {
                    index,
                    call: RelaychainCall::Staking::<T>(StakingCall::Unbond(StakingUnbondCall {
                        value,
                    })),
                },
            )));

            let msg = Self::ump_transact(
                call.encode().into(),
                Self::xcm_weight().unbond_weight,
                beneficiary,
                staking_currency,
            )?;
            if let Err(_err) = T::XcmSender::send_xcm(MultiLocation::parent(), msg) {
                return Err(Error::<T>::SendXcmError.into());
            }
        });

        Ok(())
    }

    fn do_rebond(
        value: BalanceOf<T>,
        beneficiary: MultiLocation,
        staking_currency: AssetIdOf<T>,
        index: u16,
    ) -> DispatchResult {
        switch_relay!({
            let call = RelaychainCall::Utility(Box::new(UtilityCall::AsDerivative(
                UtilityAsDerivativeCall {
                    index,
                    call: RelaychainCall::Staking::<T>(StakingCall::Rebond(StakingRebondCall {
                        value,
                    })),
                },
            )));

            let msg = Self::ump_transact(
                call.encode().into(),
                Self::xcm_weight().rebond_weight,
                beneficiary,
                staking_currency,
            )?;

            if let Err(_err) = T::XcmSender::send_xcm(MultiLocation::parent(), msg) {
                return Err(Error::<T>::SendXcmError.into());
            }
        });

        Ok(())
    }

    fn do_withdraw_unbonded(
        num_slashing_spans: u32,
        amount: BalanceOf<T>,
        beneficiary: MultiLocation,
        staking_currency: AssetIdOf<T>,
        para_account_id: T::AccountId,
        index: u16,
    ) -> DispatchResult {
        T::Assets::mint_into(staking_currency, &Self::account_id(), amount)?;

        switch_relay!({
            let call =
                RelaychainCall::Utility(Box::new(UtilityCall::BatchAll(UtilityBatchAllCall {
                    calls: vec![
                        RelaychainCall::Utility(Box::new(UtilityCall::AsDerivative(
                            UtilityAsDerivativeCall {
                                index,
                                call: RelaychainCall::Staking::<T>(StakingCall::WithdrawUnbonded(
                                    StakingWithdrawUnbondedCall { num_slashing_spans },
                                )),
                            },
                        ))),
                        RelaychainCall::Utility(Box::new(UtilityCall::AsDerivative(
                            UtilityAsDerivativeCall {
                                index,
                                call: RelaychainCall::Balances::<T>(BalancesCall::TransferAll(
                                    BalancesTransferAllCall {
                                        dest: T::Lookup::unlookup(para_account_id),
                                        keep_alive: true,
                                    },
                                )),
                            },
                        ))),
                    ],
                })));

            let msg = Self::ump_transact(
                call.encode().into(),
                Self::xcm_weight().withdraw_unbonded_weight,
                beneficiary,
                staking_currency,
            )?;

            if let Err(_err) = T::XcmSender::send_xcm(MultiLocation::parent(), msg) {
                return Err(Error::<T>::SendXcmError.into());
            }
        });

        Ok(())
    }

    fn do_nominate(
        targets: Vec<T::AccountId>,
        beneficiary: MultiLocation,
        staking_currency: AssetIdOf<T>,
        index: u16,
    ) -> DispatchResult {
        let targets_source = targets.into_iter().map(T::Lookup::unlookup).collect();

        switch_relay!({
            let call = RelaychainCall::Utility(Box::new(UtilityCall::AsDerivative(
                UtilityAsDerivativeCall {
                    index,
                    call: RelaychainCall::Staking::<T>(StakingCall::Nominate(
                        StakingNominateCall {
                            targets: targets_source,
                        },
                    )),
                },
            )));

            let msg = Self::ump_transact(
                call.encode().into(),
                Self::xcm_weight().nominate_weight,
                beneficiary,
                staking_currency,
            )?;

            if let Err(_err) = T::XcmSender::send_xcm(MultiLocation::parent(), msg) {
                return Err(Error::<T>::SendXcmError.into());
            }
        });

        Ok(())
    }
}
