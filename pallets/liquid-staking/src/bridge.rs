use super::pallet::*;
use super::types::{RewardDestination, StakingBondCall, StakingBondExtraCall};
use super::{BalanceOf, Config, Pallet};
use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
use sp_runtime::{traits::StaticLookup, DispatchResult};

use primitives::Balance;
use sp_std::prelude::*;
use xcm::v0::{
    Junction, MultiAsset, MultiLocation, NetworkId,
    Order::{BuyExecution, DepositAsset},
    OriginKind, SendXcm,
    Xcm::{Transact, WithdrawAsset},
};

impl<T: Config> Pallet<T>
where
    [u8; 32]: From<<T as frame_system::Config>::AccountId>,
{
    /// Bond on relaychain via xcm.transact
    pub fn bond(
        origin: OriginFor<T>,
        controller: T::AccountId,
        value: BalanceOf<T>,
        payee: RewardDestination<T::AccountId>,
    ) -> DispatchResult {
        T::BridgeOrigin::ensure_origin(origin)?;
        let source = T::Lookup::unlookup(controller.clone());
        let call = StakingBondCall::<T> {
            call_index: [6, 0],
            controller: source,
            value,
            payee: payee.clone(),
        };

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
    pub fn bond_extra(origin: OriginFor<T>, value: Balance) -> DispatchResult {
        T::BridgeOrigin::ensure_origin(origin)?;
        let call = StakingBondExtraCall::<T> {
            call_index: [6, 1],
            value,
        };

        let msg = WithdrawAsset {
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
                    require_weight_at_most: 1_000_000_000,
                    call: call.encode().into(),
                }],
            }],
        };

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
}
