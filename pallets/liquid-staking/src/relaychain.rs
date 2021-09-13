#![allow(dead_code)]
use super::{pallet::*, types::*, BalanceOf, Config, Pallet};
use frame_support::pallet_prelude::*;
use sp_runtime::{traits::StaticLookup, DispatchResult};

use primitives::Balance;
use sp_std::prelude::*;
use xcm::v0::{
    Junction, MultiAsset, MultiLocation, NetworkId,
    Order::{BuyExecution, DepositAsset},
    OriginKind, SendXcm,
    Xcm::{self, Transact, WithdrawAsset},
};
use xcm::DoubleEncoded;

impl<T: Config> Pallet<T>
where
    [u8; 32]: From<<T as frame_system::Config>::AccountId>,
{
    /// Bond on relaychain via xcm.transact
    pub(crate) fn bond(
        controller: T::AccountId,
        value: BalanceOf<T>,
        payee: RewardDestination<T::AccountId>,
    ) -> DispatchResult {
        let source = T::Lookup::unlookup(controller.clone());
        let call = RelaychainCall::Staking::<T>(StakingCall::Bond(StakingBondCall {
            controller: source,
            value,
            payee: payee.clone(),
        }));

        let msg = WithdrawAsset {
            assets: vec![MultiAsset::ConcreteFungible {
                id: MultiLocation::Null,
                amount: 1_000_000_000_000,
            }],
            effects: vec![
                BuyExecution {
                    fees: MultiAsset::All,
                    weight: 800_000_000,
                    debt: 600_000_000,
                    halt_on_error: false,
                    xcm: vec![Transact {
                        origin_type: OriginKind::SovereignAccount,
                        require_weight_at_most: 1_000_000_000,
                        call: call.encode().into(),
                    }],
                },
                DepositAsset {
                    assets: vec![MultiAsset::All],
                    dest: MultiLocation::X1(Junction::AccountId32 {
                        network: NetworkId::Any,
                        id: controller.clone().into(),
                    }),
                },
            ],
        };

        match T::XcmSender::send_xcm(MultiLocation::X1(Junction::Parent), msg) {
            Ok(()) => {
                Self::deposit_event(Event::<T>::BondCallSent(controller, value, payee));
            }
            Err(_e) => {
                return Err(Error::<T>::BondCallFailed.into());
            }
        }
        Ok(())
    }

    /// Bond_extra on relaychain via xcm.transact
    pub(crate) fn bond_extra(value: Balance) -> DispatchResult {
        let call =
            RelaychainCall::Staking::<T>(StakingCall::BondExtra(StakingBondExtraCall { value }));

        let msg = Self::xcm_message(call.encode().into());

        match T::XcmSender::send_xcm(MultiLocation::X1(Junction::Parent), msg) {
            Ok(()) => {
                Self::deposit_event(Event::<T>::BondExtraCallSent(value));
            }
            Err(_e) => {
                return Err(Error::<T>::BondExtraCallFailed.into());
            }
        }
        Ok(())
    }

    /// unbond on relaychain via xcm.transact
    pub(crate) fn unbond(value: Balance) -> DispatchResult {
        let call = RelaychainCall::Staking::<T>(StakingCall::Unbond(StakingUnbondCall { value }));

        let msg = Self::xcm_message(call.encode().into());

        match T::XcmSender::send_xcm(MultiLocation::X1(Junction::Parent), msg) {
            Ok(()) => {
                Self::deposit_event(Event::<T>::UnbondCallSent(value));
            }
            Err(_e) => {
                return Err(Error::<T>::UnbondCallFailed.into());
            }
        }
        Ok(())
    }

    /// rebond on relaychain via xcm.transact
    pub(crate) fn rebond(value: Balance) -> DispatchResult {
        let call = RelaychainCall::Staking::<T>(StakingCall::Rebond(StakingRebondCall { value }));

        let msg = Self::xcm_message(call.encode().into());

        match T::XcmSender::send_xcm(MultiLocation::X1(Junction::Parent), msg) {
            Ok(()) => {
                Self::deposit_event(Event::<T>::RebondCallSent(value));
            }
            Err(_e) => {
                return Err(Error::<T>::RebondCallFailed.into());
            }
        }
        Ok(())
    }

    /// withdraw unbonded on relaychain via xcm.transact
    pub(crate) fn withdraw_unbonded(num_slashing_spans: u32) -> DispatchResult {
        let call = RelaychainCall::Staking::<T>(StakingCall::WithdrawUnbonded(
            StakingWithdrawUnbondedCall { num_slashing_spans },
        ));

        let msg = Self::xcm_message(call.encode().into());

        match T::XcmSender::send_xcm(MultiLocation::X1(Junction::Parent), msg) {
            Ok(()) => {
                Self::deposit_event(Event::<T>::WithdrawUnbondedCallSent(num_slashing_spans));
            }
            Err(_e) => {
                return Err(Error::<T>::WithdrawUnbondedCallFailed.into());
            }
        }
        Ok(())
    }

    /// Nominate on relaychain via xcm.transact
    pub(crate) fn nominate(targets: Vec<T::AccountId>) -> DispatchResult {
        let targets_source = targets
            .clone()
            .into_iter()
            .map(T::Lookup::unlookup)
            .collect();
        let call = RelaychainCall::Staking::<T>(StakingCall::Nominate(StakingNominateCall {
            targets: targets_source,
        }));
        let msg = Self::xcm_message(call.encode().into());

        match T::XcmSender::send_xcm(MultiLocation::X1(Junction::Parent), msg) {
            Ok(()) => {
                Self::deposit_event(Event::<T>::NominateCallSent(targets));
            }
            Err(_e) => {
                return Err(Error::<T>::NominateCallFailed.into());
            }
        }
        Ok(())
    }

    /// Payout_stakers on relaychain via xcm.transact
    pub(crate) fn payout_stakers(validator_stash: T::AccountId, era: u32) -> DispatchResult {
        let call =
            RelaychainCall::Staking::<T>(StakingCall::PayoutStakers(StakingPayoutStakersCall {
                validator_stash: validator_stash.clone(),
                era,
            }));

        let msg = Self::xcm_message(call.encode().into());

        match T::XcmSender::send_xcm(MultiLocation::X1(Junction::Parent), msg) {
            Ok(()) => {
                Self::deposit_event(Event::<T>::PayoutStakersCallSent(validator_stash, era));
            }
            Err(_e) => {
                return Err(Error::<T>::PayoutStakersCallFailed.into());
            }
        }
        Ok(())
    }

    fn xcm_message(call: DoubleEncoded<()>) -> Xcm<()> {
        WithdrawAsset {
            assets: vec![MultiAsset::ConcreteFungible {
                id: MultiLocation::Null,
                amount: 1_000_000_000_000,
            }],
            effects: vec![BuyExecution {
                fees: MultiAsset::All,
                weight: 800_000_000,
                debt: 600_000_000,
                halt_on_error: true,
                xcm: vec![Transact {
                    origin_type: OriginKind::SovereignAccount,
                    require_weight_at_most: 100_000_000_000,
                    call,
                }],
            }],
        }
    }
}
