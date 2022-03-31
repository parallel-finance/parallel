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

mod benchmarking;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub mod weights;
pub use pallet::*;

use frame_support::{
    dispatch::{DispatchResult, GetDispatchInfo},
    pallet_prelude::*,
    traits::fungibles::{Inspect, Mutate, Transfer},
    transactional, PalletId,
};
use frame_system::pallet_prelude::BlockNumberFor;

use frame_support::traits::ReservableCurrency;
use primitives::{switch_relay, ump::*, Balance, CurrencyId, ParaId};
use sp_io::hashing::blake2_256;
use sp_runtime::traits::Zero;
use sp_runtime::traits::{AccountIdConversion, BlockNumberProvider, Convert, StaticLookup};
use sp_runtime::traits::{Dispatchable, TrailingZeroInput};
use sp_std::prelude::*;
use sp_std::{boxed::Box, vec, vec::Vec};
use xcm::{latest::prelude::*, DoubleEncoded, VersionedMultiLocation, VersionedXcm};
use xcm_executor::traits::InvertLocation;

pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type CallIdOf<T> = <T as pallet_xcm::Config>::Call;
pub type AssetIdOf<T> =
    <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;
pub type BalanceOf<T> =
    <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

/// The parameters under which a particular account has a proxy relationship with some other
/// account.
#[derive(
    Encode,
    Decode,
    Clone,
    Copy,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    RuntimeDebug,
    MaxEncodedLen,
    TypeInfo,
)]
pub struct ProxyDefinition<AccountId, ProxyType, BlockNumber> {
    /// The account which may act on behalf of another.
    pub delegate: AccountId,
    /// A value defining the subset of calls that it is allowed to make.
    pub proxy_type: ProxyType,
    /// The number of blocks that an announcement must be in place for before the corresponding
    /// call may be dispatched. If zero, then no announcement is needed.
    pub delay: BlockNumber,
}

#[frame_support::pallet]
pub mod pallet {
    use crate::weights::WeightInfo;
    use frame_system::pallet_prelude::{BlockNumberFor, OriginFor};

    use super::*;
    use frame_support::traits::InstanceFilter;
    use frame_support::traits::IsSubType;
    use frame_support::traits::OriginTrait;
    use frame_support::traits::UnfilteredDispatchable;
    use frame_support::weights::extract_actual_weight;
    use frame_support::weights::PostDispatchInfo;
    use frame_system::{ensure_root, ensure_signed};
    use sp_runtime::traits::{Convert, Zero};

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_xcm::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// Assets for deposit/withdraw assets to/from crowdloan account
        type Assets: Transfer<AccountIdOf<Self>, AssetId = CurrencyId, Balance = Balance>
            + Inspect<AccountIdOf<Self>, AssetId = CurrencyId, Balance = Balance>
            + Mutate<AccountIdOf<Self>, AssetId = CurrencyId, Balance = Balance>;

        /// XCM message sender
        type XcmSender: SendXcm;

        /// Relay network
        #[pallet::constant]
        type RelayNetwork: Get<NetworkId>;

        /// Pallet account for collecting xcm fees
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// Account on relaychain for receiving refunded fees
        #[pallet::constant]
        type RefundLocation: Get<Self::AccountId>;

        /// Convert `T::AccountId` to `MultiLocation`.
        type AccountIdToMultiLocation: Convert<Self::AccountId, MultiLocation>;

        /// Notify call timeout
        #[pallet::constant]
        type NotifyTimeout: Get<BlockNumberFor<Self>>;

        /// The block number provider
        type BlockNumberProvider: BlockNumberProvider<BlockNumber = BlockNumberFor<Self>>;

        /// The origin which can update reserve_factor, xcm_fees etc
        type UpdateOrigin: EnsureOrigin<<Self as frame_system::Config>::Origin>;

        /// The origin which can call XCM helper functions
        type XCMOrigin: EnsureOrigin<<Self as frame_system::Config>::Origin>;

        /// Weight information
        type WeightInfo: WeightInfo;

        /// The overarching call type.
        type Call: Parameter
            + Dispatchable<
                Origin = <Self as frame_system::Config>::Origin,
                PostInfo = PostDispatchInfo,
            > + GetDispatchInfo
            + From<frame_system::Call<Self>>
            + UnfilteredDispatchable<Origin = <Self as frame_system::Config>::Origin>
            + IsSubType<Call<Self>>
            + IsType<<Self as frame_system::Config>::Call>;

        /// A kind of proxy; specified with the proxy and passed in to the `IsProxyable` fitler.
        /// The instance filter determines whether a given call may be proxied under this type.
        ///
        /// IMPORTANT: `Default` must be provided and MUST BE the the *most permissive* value.
        type ProxyType: Parameter
            + Member
            + Ord
            + PartialOrd
            + InstanceFilter<<Self as Config>::Call>
            + Default
            + MaxEncodedLen;

        /// The maximum amount of proxies allowed for a single account.
        #[pallet::constant]
        type MaxProxies: Get<u32>;

        /// The currency mechanism.
        type Currency: ReservableCurrency<Self::AccountId, Balance = Balance>;

        /// The base amount of currency needed to reserve for creating a proxy.
        ///
        /// This is held for an additional storage item whose value size is
        /// `sizeof(Balance)` bytes and whose key size is `sizeof(AccountId)` bytes.
        #[pallet::constant]
        type ProxyDepositBase: Get<BalanceOf<Self>>;

        /// The amount of currency needed per proxy added.
        ///
        /// This is held for adding 32 bytes plus an instance of `ProxyType` more into a
        /// pre-existing storage value. Thus, when configuring `ProxyDepositFactor` one should take
        /// into account `32 + proxy_type.encode().len()` bytes of data.
        #[pallet::constant]
        type ProxyDepositFactor: Get<BalanceOf<Self>>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Xcm fee and weight updated
        XcmWeightFeeUpdated(XcmWeightFeeMisc<Weight, BalanceOf<T>>),
        /// Xcm Withdraw
        XCMWithdrawDone,
        /// Xcm Contribute
        XCMContributeDone,
        /// XCMBonded
        XCMBonded,
        /// XCMBondedExtra
        XCMBondedExtra,
        /// XCMUnBonded
        XCMUnBonded,
        /// XCMReBonded
        XCMReBonded,
        /// XCMWithdrawUnBonded
        XCMWithdrawUnBonded,
        /// XCMNominated
        XCMNominated,
        /// XCM message sent. \[to, message\]
        Sent { to: MultiLocation, message: Xcm<()> },
        /// Batch of dispatches completed fully with no error.
        BatchCompleted,
        /// A single item within a Batch of dispatches has completed with no error.
        ItemCompleted,
        /// A proxy was added.
        ProxyAdded {
            delegator: T::AccountId,
            delegatee: T::AccountId,
            proxy_type: T::ProxyType,
            delay: T::BlockNumber,
        },
        /// A proxy was removed.
        ProxyRemoved {
            delegator: T::AccountId,
            delegatee: T::AccountId,
            proxy_type: T::ProxyType,
            delay: T::BlockNumber,
        },
    }

    #[pallet::storage]
    #[pallet::getter(fn xcm_weight_fee)]
    pub type XcmWeightFee<T: Config> =
        StorageMap<_, Twox64Concat, XcmCall, XcmWeightFeeMisc<Weight, BalanceOf<T>>, ValueQuery>;

    /// The set of account proxies. Maps the account which has delegated to the accounts
    /// which are being delegated to, together with the amount held on deposit.
    #[pallet::storage]
    #[pallet::getter(fn proxies)]
    pub type Proxies<T: Config> = StorageMap<
        _,
        Twox64Concat,
        T::AccountId,
        (
            BoundedVec<ProxyDefinition<T::AccountId, T::ProxyType, T::BlockNumber>, T::MaxProxies>,
            BalanceOf<T>,
        ),
        ValueQuery,
    >;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::error]
    pub enum Error<T> {
        /// `MultiLocation` value ascend more parents than known ancestors of local location.
        MultiLocationNotInvertible,
        /// Xcm message send failure
        SendXcmError,
        /// XcmWeightMisc cannot have zero value
        ZeroXcmWeightMisc,
        /// Xcm fees cannot be zero
        ZeroXcmFees,
        /// Insufficient xcm fees
        InsufficientXcmFees,
        /// The message and destination combination was not recognized as being
        /// reachable.
        Unreachable,
        /// The message and destination was recognized as being reachable but
        /// the operation could not be completed.
        SendFailure,
        /// The version of the `Versioned` value used is not able to be
        /// interpreted.
        BadVersion,
        /// Too many calls batched.
        TooManyCalls,
        /// There are too many proxies registered or too many announcements pending.
        TooMany,
        /// Proxy registration not found.
        NotFound,
        /// Sender is not a proxy of the account to be proxied.
        NotProxy,
        /// A call which is incompatible with the proxy type's filter was attempted.
        Unproxyable,
        /// Account is already a proxy.
        Duplicate,
        /// Call may not be made by proxy because it may escalate its privileges.
        NoPermission,
        /// Announcement, if made at all, was made too recently.
        Unannounced,
        /// Cannot add self as proxy.
        NoSelfProxy,
    }

    // Align the call size to 1KB. As we are currently compiling the runtime for native/wasm
    // the `size_of` of the `Call` can be different. To ensure that this don't leads to
    // mismatches between native/wasm or to different metadata for the same runtime, we
    // algin the call size. The value is choosen big enough to hopefully never reach it.
    const CALL_ALIGN: u32 = 1024;

    #[pallet::extra_constants]
    impl<T: Config> Pallet<T> {
        /// The limit on the number of batched calls.
        fn batched_calls_limit() -> u32 {
            let allocator_limit = sp_core::MAX_POSSIBLE_ALLOCATION;
            let call_size = ((sp_std::mem::size_of::<<T as Config>::Call>() as u32 + CALL_ALIGN
                - 1)
                / CALL_ALIGN)
                * CALL_ALIGN;
            // The margin to take into account vec doubling capacity.
            let margin_factor = 3;

            allocator_limit / margin_factor / call_size
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Update xcm fees amount to be used in xcm.Withdraw message
        #[pallet::weight(<T as Config>::WeightInfo::update_xcm_weight_fee())]
        #[transactional]
        pub fn update_xcm_weight_fee(
            origin: OriginFor<T>,
            xcm_call: XcmCall,
            xcm_weight_fee_misc: XcmWeightFeeMisc<Weight, BalanceOf<T>>,
        ) -> DispatchResult {
            T::UpdateOrigin::ensure_origin(origin)?;

            ensure!(!xcm_weight_fee_misc.fee.is_zero(), Error::<T>::ZeroXcmFees);
            ensure!(
                !xcm_weight_fee_misc.weight.is_zero(),
                Error::<T>::ZeroXcmWeightMisc
            );

            XcmWeightFee::<T>::mutate(xcm_call, |v| *v = xcm_weight_fee_misc);
            Self::deposit_event(Event::<T>::XcmWeightFeeUpdated(xcm_weight_fee_misc));
            Ok(())
        }

        #[pallet::weight(10_000)]
        #[transactional]
        pub fn withdraw(
            origin: OriginFor<T>,
            para_id: ParaId,
            relay_currency: AssetIdOf<T>,
            para_account_id: AccountIdOf<T>,
            notify: Box<CallIdOf<T>>,
        ) -> DispatchResult {
            T::XCMOrigin::ensure_origin(origin)?;

            Self::do_withdraw(para_id, relay_currency, para_account_id, *notify)?;

            Self::deposit_event(Event::<T>::XCMWithdrawDone);
            Ok(())
        }

        #[pallet::weight(10_000)]
        #[transactional]
        pub fn contribute(
            origin: OriginFor<T>,
            para_id: ParaId,
            relay_currency: AssetIdOf<T>,
            amount: BalanceOf<T>,
            who: AccountIdOf<T>,
            notify: Box<CallIdOf<T>>,
        ) -> DispatchResult {
            T::XCMOrigin::ensure_origin(origin)?;

            Self::do_contribute(para_id, relay_currency, amount, &who, *notify)?;

            Self::deposit_event(Event::<T>::XCMContributeDone);
            Ok(())
        }

        #[pallet::weight(10_000)]
        #[transactional]
        pub fn bond(
            origin: OriginFor<T>,
            value: BalanceOf<T>,
            payee: RewardDestination<AccountIdOf<T>>,
            stash: AccountIdOf<T>,
            relay_currency: AssetIdOf<T>,
            index: u16,
            notify: Box<CallIdOf<T>>,
        ) -> DispatchResult {
            T::XCMOrigin::ensure_origin(origin)?;

            Self::do_bond(value, payee, stash, relay_currency, index, *notify)?;

            Self::deposit_event(Event::<T>::XCMBonded);
            Ok(())
        }

        #[pallet::weight(10_000)]
        #[transactional]
        pub fn bond_extra(
            origin: OriginFor<T>,
            value: BalanceOf<T>,
            stash: AccountIdOf<T>,
            relay_currency: AssetIdOf<T>,
            index: u16,
            notify: Box<CallIdOf<T>>,
        ) -> DispatchResult {
            T::XCMOrigin::ensure_origin(origin)?;

            Self::do_bond_extra(value, stash, relay_currency, index, *notify)?;

            Self::deposit_event(Event::<T>::XCMBondedExtra);
            Ok(())
        }

        #[pallet::weight(10_000)]
        #[transactional]
        pub fn unbond(
            origin: OriginFor<T>,
            value: BalanceOf<T>,
            relay_currency: AssetIdOf<T>,
            index: u16,
            notify: Box<CallIdOf<T>>,
        ) -> DispatchResult {
            T::XCMOrigin::ensure_origin(origin)?;

            Self::do_unbond(value, relay_currency, index, *notify)?;

            Self::deposit_event(Event::<T>::XCMUnBonded);
            Ok(())
        }

        #[pallet::weight(10_000)]
        #[transactional]
        pub fn rebond(
            origin: OriginFor<T>,
            value: BalanceOf<T>,
            relay_currency: AssetIdOf<T>,
            index: u16,
            notify: Box<CallIdOf<T>>,
        ) -> DispatchResult {
            T::XCMOrigin::ensure_origin(origin)?;

            Self::do_rebond(value, relay_currency, index, *notify)?;

            Self::deposit_event(Event::<T>::XCMReBonded);
            Ok(())
        }

        #[pallet::weight(10_000)]
        #[transactional]
        pub fn withdraw_unbonded(
            origin: OriginFor<T>,
            num_slashing_spans: u32,
            para_account_id: AccountIdOf<T>,
            relay_currency: AssetIdOf<T>,
            index: u16,
            notify: Box<CallIdOf<T>>,
        ) -> DispatchResult {
            T::XCMOrigin::ensure_origin(origin)?;

            Self::do_withdraw_unbonded(
                num_slashing_spans,
                para_account_id,
                relay_currency,
                index,
                *notify,
            )?;

            Self::deposit_event(Event::<T>::XCMWithdrawUnBonded);
            Ok(())
        }

        #[pallet::weight(10_000)]
        #[transactional]
        pub fn nominate(
            origin: OriginFor<T>,
            targets: Vec<AccountIdOf<T>>,
            relay_currency: AssetIdOf<T>,
            index: u16,
            notify: Box<CallIdOf<T>>,
        ) -> DispatchResult {
            T::XCMOrigin::ensure_origin(origin)?;

            Self::do_nominate(targets, relay_currency, index, *notify)?;

            Self::deposit_event(Event::<T>::XCMNominated);
            Ok(())
        }

        #[pallet::weight(100_000_000)]
        pub fn send_as_sovereign(
            origin: OriginFor<T>,
            dest: Box<VersionedMultiLocation>,
            message: Box<VersionedXcm<()>>,
        ) -> DispatchResult {
            T::XCMOrigin::ensure_origin(origin)?;
            let dest = MultiLocation::try_from(*dest).map_err(|()| Error::<T>::BadVersion)?;
            let message: Xcm<()> = (*message).try_into().map_err(|()| Error::<T>::BadVersion)?;

            pallet_xcm::Pallet::<T>::send_xcm(Here, dest.clone(), message.clone()).map_err(
                |e| match e {
                    SendError::CannotReachDestination(..) => Error::<T>::Unreachable,
                    _ => Error::<T>::SendFailure,
                },
            )?;
            Self::deposit_event(Event::Sent { to: dest, message });
            Ok(())
        }

        #[pallet::weight(10_000)]
        #[transactional]
        pub fn ump_transacts(
            origin: OriginFor<T>,
            call: DoubleEncoded<()>,
            weight: Weight,
            beneficiary: MultiLocation,
            relay_currency: AssetIdOf<T>,
            fees: BalanceOf<T>,
        ) -> DispatchResult {
            T::XCMOrigin::ensure_origin(origin)?;

            Self::ump_transact(call, weight, beneficiary, relay_currency, fees)?;

            Ok(())
        }

        #[pallet::weight({
		let dispatch_info = call.get_dispatch_info();
		(
		T::WeightInfo::as_derivative()
		.saturating_add(dispatch_info.weight)
		// AccountData for inner call origin accountdata.
		.saturating_add(T::DbWeight::get().reads_writes(1, 1)),
		dispatch_info.class,
		)
		})]
        pub fn as_derivative(
            origin: OriginFor<T>,
            index: u16,
            call: Box<<T as Config>::Call>,
        ) -> DispatchResultWithPostInfo {
            let mut origin = origin;
            let who = ensure_signed(origin.clone())?;
            let pseudonym = Self::derivative_account_id(who, index);
            origin.set_caller_from(frame_system::RawOrigin::Signed(pseudonym));
            let info = call.get_dispatch_info();
            let result = call.dispatch(origin);
            // Always take into account the base weight of this call.
            let mut weight = T::WeightInfo::as_derivative()
                .saturating_add(T::DbWeight::get().reads_writes(1, 1));
            // Add the real weight of the dispatch.
            weight = weight.saturating_add(extract_actual_weight(&result, &info));
            result
                .map_err(|mut err| {
                    err.post_info = Some(weight).into();
                    err
                })
                .map(|_| Some(weight).into())
        }

        #[pallet::weight({
		let dispatch_infos = calls.iter().map(|call| call.get_dispatch_info()).collect::<Vec<_>>();
		let dispatch_weight = dispatch_infos.iter()
		.map(|di| di.weight)
		.fold(0, |total: Weight, weight: Weight| total.saturating_add(weight))
		.saturating_add(T::WeightInfo::batch_all(calls.len() as u32));
		let dispatch_class = {
		let all_operational = dispatch_infos.iter()
		.map(|di| di.class)
		.all(|class| class == DispatchClass::Operational);
		if all_operational {
		DispatchClass::Operational
		} else {
		DispatchClass::Normal
		}
		};
		(dispatch_weight, dispatch_class)
		})]
        #[transactional]
        pub fn batch_all(
            origin: OriginFor<T>,
            calls: Vec<<T as Config>::Call>,
        ) -> DispatchResultWithPostInfo {
            let is_root = ensure_root(origin.clone()).is_ok();
            let calls_len = calls.len();
            ensure!(
                calls_len <= Self::batched_calls_limit() as usize,
                Error::<T>::TooManyCalls
            );

            // Track the actual weight of each of the batch calls.
            let mut weight: Weight = 0;
            for (index, call) in calls.into_iter().enumerate() {
                let info = call.get_dispatch_info();
                // If origin is root, bypass any dispatch filter; root can call anything.
                let result = if is_root {
                    call.dispatch_bypass_filter(origin.clone())
                } else {
                    let mut filtered_origin = origin.clone();
                    // Don't allow users to nest `batch_all` calls.
                    filtered_origin.add_filter(move |c: &<T as frame_system::Config>::Call| {
                        let c = <T as Config>::Call::from_ref(c);
                        !matches!(c.is_sub_type(), Some(Call::batch_all { .. }))
                    });
                    call.dispatch(filtered_origin)
                };
                // Add the weight of this call.
                weight = weight.saturating_add(extract_actual_weight(&result, &info));
                result.map_err(|mut err| {
                    // Take the weight of this function itself into account.
                    let base_weight = T::WeightInfo::batch_all(index.saturating_add(1) as u32);
                    // Return the actual used weight + base_weight of this call.
                    err.post_info = Some(base_weight + weight).into();
                    err
                })?;
                Self::deposit_event(Event::ItemCompleted);
            }
            Self::deposit_event(Event::BatchCompleted);
            let base_weight = T::WeightInfo::batch_all(calls_len as u32);
            Ok(Some(base_weight + weight).into())
        }

        #[pallet::weight(10_000)]
        pub fn remove_proxy(
            origin: OriginFor<T>,
            delegate: T::AccountId,
            proxy_type: T::ProxyType,
            delay: T::BlockNumber,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::remove_proxy_delegate(&who, delegate, proxy_type, delay)
        }
    }
}

pub trait XcmHelper<T: pallet_xcm::Config, Balance, AssetId, AccountId> {
    fn add_xcm_fees(relay_currency: AssetId, payer: &AccountId, amount: Balance) -> DispatchResult;

    fn ump_transact(
        call: DoubleEncoded<()>,
        weight: Weight,
        beneficiary: MultiLocation,
        relay_currency: AssetId,
        fees: Balance,
    ) -> Result<Xcm<()>, DispatchError>;

    fn do_withdraw(
        para_id: ParaId,
        relay_currency: AssetId,
        para_account_id: AccountId,
        notify: impl Into<<T as pallet_xcm::Config>::Call>,
    ) -> Result<QueryId, DispatchError>;

    fn do_contribute(
        para_id: ParaId,
        relay_currency: AssetId,
        amount: Balance,
        who: &AccountId,
        notify: impl Into<<T as pallet_xcm::Config>::Call>,
    ) -> Result<QueryId, DispatchError>;

    fn do_bond(
        value: Balance,
        payee: RewardDestination<AccountId>,
        stash: AccountId,
        relay_currency: AssetId,
        index: u16,
        notify: impl Into<<T as pallet_xcm::Config>::Call>,
    ) -> Result<QueryId, DispatchError>;

    fn do_bond_extra(
        value: Balance,
        stash: AccountId,
        relay_currency: AssetId,
        index: u16,
        notify: impl Into<<T as pallet_xcm::Config>::Call>,
    ) -> Result<QueryId, DispatchError>;

    fn do_unbond(
        value: Balance,
        relay_currency: AssetId,
        index: u16,
        notify: impl Into<<T as pallet_xcm::Config>::Call>,
    ) -> Result<QueryId, DispatchError>;

    fn do_rebond(
        value: Balance,
        relay_currency: AssetId,
        index: u16,
        notify: impl Into<<T as pallet_xcm::Config>::Call>,
    ) -> Result<QueryId, DispatchError>;

    fn do_withdraw_unbonded(
        num_slashing_spans: u32,
        para_account_id: AccountId,
        staking_currency: AssetId,
        index: u16,
        notify: impl Into<<T as pallet_xcm::Config>::Call>,
    ) -> Result<QueryId, DispatchError>;

    fn do_nominate(
        targets: Vec<AccountId>,
        relay_currency: AssetId,
        index: u16,
        notify: impl Into<<T as pallet_xcm::Config>::Call>,
    ) -> Result<QueryId, DispatchError>;
}

impl<T: Config> Pallet<T> {
    pub fn account_id() -> AccountIdOf<T> {
        T::PalletId::get().into_account()
    }

    pub fn refund_location() -> MultiLocation {
        T::AccountIdToMultiLocation::convert(T::RefundLocation::get())
    }

    pub fn report_outcome_notify(
        message: &mut Xcm<()>,
        responder: impl Into<MultiLocation>,
        notify: impl Into<<T as pallet_xcm::Config>::Call>,
        timeout: BlockNumberFor<T>,
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

    pub fn derivative_account_id(who: T::AccountId, index: u16) -> T::AccountId {
        let entropy = (b"modlpy/utilisuba", who, index).using_encoded(blake2_256);
        Decode::decode(&mut TrailingZeroInput::new(entropy.as_ref()))
            .expect("infinite length input; no invalid inputs for type; qed")
    }

    /// Register a proxy account for the delegator that is able to make calls on its behalf.
    ///
    /// Parameters:
    /// - `delegator`: The delegator account.
    /// - `delegatee`: The account that the `delegator` would like to make a proxy.
    /// - `proxy_type`: The permissions allowed for this proxy account.
    /// - `delay`: The announcement period required of the initial proxy. Will generally be
    /// zero.
    pub fn add_proxy_delegate(
        delegator: &T::AccountId,
        delegatee: T::AccountId,
        proxy_type: T::ProxyType,
        delay: T::BlockNumber,
    ) -> DispatchResult {
        ensure!(delegator != &delegatee, Error::<T>::NoSelfProxy);
        Proxies::<T>::try_mutate(delegator, |(ref mut proxies, ref mut deposit)| {
            let proxy_def = ProxyDefinition {
                delegate: delegatee.clone(),
                proxy_type: proxy_type.clone(),
                delay,
            };
            let i = proxies
                .binary_search(&proxy_def)
                .err()
                .ok_or(Error::<T>::Duplicate)?;
            proxies
                .try_insert(i, proxy_def)
                .map_err(|_| Error::<T>::TooMany)?;
            let new_deposit = Self::deposit(proxies.len() as u32);
            if new_deposit > *deposit {
                T::Currency::reserve(delegator, new_deposit - *deposit)?;
            } else if new_deposit < *deposit {
                T::Currency::unreserve(delegator, *deposit - new_deposit);
            }
            *deposit = new_deposit;
            Self::deposit_event(Event::<T>::ProxyAdded {
                delegator: delegator.clone(),
                delegatee,
                proxy_type,
                delay,
            });
            Ok(())
        })
    }

    /// Unregister a proxy account for the delegator.
    ///
    /// Parameters:
    /// - `delegator`: The delegator account.
    /// - `delegatee`: The account that the `delegator` would like to make a proxy.
    /// - `proxy_type`: The permissions allowed for this proxy account.
    /// - `delay`: The announcement period required of the initial proxy. Will generally be
    /// zero.
    pub fn remove_proxy_delegate(
        delegator: &T::AccountId,
        delegatee: T::AccountId,
        proxy_type: T::ProxyType,
        delay: T::BlockNumber,
    ) -> DispatchResult {
        Proxies::<T>::try_mutate_exists(delegator, |x| {
            let (mut proxies, old_deposit) = x.take().ok_or(Error::<T>::NotFound)?;
            let proxy_def = ProxyDefinition {
                delegate: delegatee.clone(),
                proxy_type: proxy_type.clone(),
                delay,
            };
            let i = proxies
                .binary_search(&proxy_def)
                .ok()
                .ok_or(Error::<T>::NotFound)?;
            proxies.remove(i);
            let new_deposit = Self::deposit(proxies.len() as u32);
            if new_deposit > old_deposit {
                T::Currency::reserve(delegator, new_deposit - old_deposit)?;
            } else if new_deposit < old_deposit {
                T::Currency::unreserve(delegator, old_deposit - new_deposit);
            }
            if !proxies.is_empty() {
                *x = Some((proxies, new_deposit))
            }
            Self::deposit_event(Event::<T>::ProxyRemoved {
                delegator: delegator.clone(),
                delegatee,
                proxy_type,
                delay,
            });
            Ok(())
        })
    }

    pub fn deposit(num_proxies: u32) -> BalanceOf<T> {
        if num_proxies == 0 {
            Zero::zero()
        } else {
            BalanceOf::<T>::from(num_proxies)
                .saturating_mul(T::ProxyDepositBase::get() + T::ProxyDepositFactor::get())
        }
    }
}

impl<T: Config> XcmHelper<T, BalanceOf<T>, AssetIdOf<T>, AccountIdOf<T>> for Pallet<T> {
    fn add_xcm_fees(
        relay_currency: AssetIdOf<T>,
        payer: &AccountIdOf<T>,
        amount: BalanceOf<T>,
    ) -> DispatchResult {
        T::Assets::transfer(relay_currency, payer, &Self::account_id(), amount, false)?;
        Ok(())
    }

    fn ump_transact(
        call: DoubleEncoded<()>,
        weight: Weight,
        beneficiary: MultiLocation,
        relay_currency: AssetIdOf<T>,
        fees: BalanceOf<T>,
    ) -> Result<Xcm<()>, DispatchError> {
        let asset: MultiAsset = (MultiLocation::here(), fees).into();
        T::Assets::burn_from(relay_currency, &Self::account_id(), fees)
            .map_err(|_| Error::<T>::InsufficientXcmFees)?;

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
        relay_currency: AssetIdOf<T>,
        para_account_id: AccountIdOf<T>,
        notify: impl Into<<T as pallet_xcm::Config>::Call>,
    ) -> Result<QueryId, DispatchError> {
        let xcm_weight_fee_misc = Self::xcm_weight_fee(XcmCall::Withdraw);
        Ok(switch_relay!({
            let call =
                RelaychainCall::<T>::Crowdloans(CrowdloansCall::Withdraw(CrowdloansWithdrawCall {
                    who: para_account_id,
                    index: para_id,
                }));

            let mut msg = Self::ump_transact(
                call.encode().into(),
                xcm_weight_fee_misc.weight,
                Self::refund_location(),
                relay_currency,
                xcm_weight_fee_misc.fee,
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
        relay_currency: AssetIdOf<T>,
        amount: BalanceOf<T>,
        _who: &AccountIdOf<T>,
        notify: impl Into<<T as pallet_xcm::Config>::Call>,
    ) -> Result<QueryId, DispatchError> {
        let xcm_weight_fee_misc = Self::xcm_weight_fee(XcmCall::Contribute);
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
                xcm_weight_fee_misc.weight,
                Self::refund_location(),
                relay_currency,
                xcm_weight_fee_misc.fee,
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
        payee: RewardDestination<AccountIdOf<T>>,
        stash: AccountIdOf<T>,
        relay_currency: AssetIdOf<T>,
        index: u16,
        notify: impl Into<<T as pallet_xcm::Config>::Call>,
    ) -> Result<QueryId, DispatchError> {
        let controller = stash.clone();
        let xcm_weight_fee_misc = Self::xcm_weight_fee(XcmCall::Bond);
        Ok(switch_relay!({
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

            let mut msg = Self::ump_transact(
                call.encode().into(),
                xcm_weight_fee_misc.weight,
                Self::refund_location(),
                relay_currency,
                xcm_weight_fee_misc.fee,
            )?;

            let query_id = Self::report_outcome_notify(
                &mut msg,
                MultiLocation::parent(),
                notify,
                T::NotifyTimeout::get(),
            )?;

            if let Err(_err) = T::XcmSender::send_xcm(MultiLocation::parent(), msg) {
                return Err(Error::<T>::SendXcmError.into());
            }

            query_id
        }))
    }

    fn do_bond_extra(
        value: BalanceOf<T>,
        stash: AccountIdOf<T>,
        relay_currency: AssetIdOf<T>,
        index: u16,
        notify: impl Into<<T as pallet_xcm::Config>::Call>,
    ) -> Result<QueryId, DispatchError> {
        let xcm_weight_fee_misc = Self::xcm_weight_fee(XcmCall::BondExtra);
        Ok(switch_relay!({
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

            let mut msg = Self::ump_transact(
                call.encode().into(),
                xcm_weight_fee_misc.weight,
                Self::refund_location(),
                relay_currency,
                xcm_weight_fee_misc.fee,
            )?;

            let query_id = Self::report_outcome_notify(
                &mut msg,
                MultiLocation::parent(),
                notify,
                T::NotifyTimeout::get(),
            )?;

            if let Err(_err) = T::XcmSender::send_xcm(MultiLocation::parent(), msg) {
                return Err(Error::<T>::SendXcmError.into());
            }

            query_id
        }))
    }

    fn do_unbond(
        value: BalanceOf<T>,
        relay_currency: AssetIdOf<T>,
        index: u16,
        notify: impl Into<<T as pallet_xcm::Config>::Call>,
    ) -> Result<QueryId, DispatchError> {
        let xcm_weight_fee_misc = Self::xcm_weight_fee(XcmCall::Unbond);
        Ok(switch_relay!({
            let call = RelaychainCall::Utility(Box::new(UtilityCall::AsDerivative(
                UtilityAsDerivativeCall {
                    index,
                    call: RelaychainCall::Staking::<T>(StakingCall::Unbond(StakingUnbondCall {
                        value,
                    })),
                },
            )));

            let mut msg = Self::ump_transact(
                call.encode().into(),
                xcm_weight_fee_misc.weight,
                Self::refund_location(),
                relay_currency,
                xcm_weight_fee_misc.fee,
            )?;

            let query_id = Self::report_outcome_notify(
                &mut msg,
                MultiLocation::parent(),
                notify,
                T::NotifyTimeout::get(),
            )?;

            if let Err(_err) = T::XcmSender::send_xcm(MultiLocation::parent(), msg) {
                return Err(Error::<T>::SendXcmError.into());
            }

            query_id
        }))
    }

    fn do_rebond(
        value: BalanceOf<T>,
        relay_currency: AssetIdOf<T>,
        index: u16,
        notify: impl Into<<T as pallet_xcm::Config>::Call>,
    ) -> Result<QueryId, DispatchError> {
        let xcm_weight_fee_misc = Self::xcm_weight_fee(XcmCall::Rebond);
        Ok(switch_relay!({
            let call = RelaychainCall::Utility(Box::new(UtilityCall::AsDerivative(
                UtilityAsDerivativeCall {
                    index,
                    call: RelaychainCall::Staking::<T>(StakingCall::Rebond(StakingRebondCall {
                        value,
                    })),
                },
            )));

            let mut msg = Self::ump_transact(
                call.encode().into(),
                xcm_weight_fee_misc.weight,
                Self::refund_location(),
                relay_currency,
                xcm_weight_fee_misc.fee,
            )?;

            let query_id = Self::report_outcome_notify(
                &mut msg,
                MultiLocation::parent(),
                notify,
                T::NotifyTimeout::get(),
            )?;

            if let Err(_err) = T::XcmSender::send_xcm(MultiLocation::parent(), msg) {
                return Err(Error::<T>::SendXcmError.into());
            }

            query_id
        }))
    }

    fn do_withdraw_unbonded(
        num_slashing_spans: u32,
        para_account_id: AccountIdOf<T>,
        relay_currency: AssetIdOf<T>,
        index: u16,
        notify: impl Into<<T as pallet_xcm::Config>::Call>,
    ) -> Result<QueryId, DispatchError> {
        let xcm_weight_fee_misc = Self::xcm_weight_fee(XcmCall::WithdrawUnbonded);
        Ok(switch_relay!({
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

            let mut msg = Self::ump_transact(
                call.encode().into(),
                xcm_weight_fee_misc.weight,
                Self::refund_location(),
                relay_currency,
                xcm_weight_fee_misc.fee,
            )?;

            let query_id = Self::report_outcome_notify(
                &mut msg,
                MultiLocation::parent(),
                notify,
                T::NotifyTimeout::get(),
            )?;

            if let Err(_err) = T::XcmSender::send_xcm(MultiLocation::parent(), msg) {
                return Err(Error::<T>::SendXcmError.into());
            }

            query_id
        }))
    }

    fn do_nominate(
        targets: Vec<AccountIdOf<T>>,
        relay_currency: AssetIdOf<T>,
        index: u16,
        notify: impl Into<<T as pallet_xcm::Config>::Call>,
    ) -> Result<QueryId, DispatchError> {
        let targets_source = targets.into_iter().map(T::Lookup::unlookup).collect();
        let xcm_weight_fee_misc = Self::xcm_weight_fee(XcmCall::Nominate);
        Ok(switch_relay!({
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

            let mut msg = Self::ump_transact(
                call.encode().into(),
                xcm_weight_fee_misc.weight,
                Self::refund_location(),
                relay_currency,
                xcm_weight_fee_misc.fee,
            )?;

            let query_id = Self::report_outcome_notify(
                &mut msg,
                MultiLocation::parent(),
                notify,
                T::NotifyTimeout::get(),
            )?;

            if let Err(_err) = T::XcmSender::send_xcm(MultiLocation::parent(), msg) {
                return Err(Error::<T>::SendXcmError.into());
            }

            query_id
        }))
    }
}
