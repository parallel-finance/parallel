#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use core::marker;
use frame_support::traits::IsSubType;
pub use pallet::*;
use scale_info::TypeInfo;
use sp_runtime::{traits::SignedExtension, transaction_validity::ValidTransactionBuilder};

#[frame_support::pallet]
pub mod pallet {
    use frame_support::storage::unhashed::put;
    use frame_support::storage::{storage_prefix, unhashed};
    use frame_support::traits::ReservableCurrency;
    use frame_support::{pallet_prelude::*, traits::Currency};
    use frame_system::{pallet_prelude::*, RawOrigin};
    use pallet_balances::{self as balances};
    use sp_runtime::traits::{Convert, UniqueSaturatedInto};
    use sp_runtime::KeyTypeId;
    #[pallet::pallet]
    // #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::event]
    #[pallet::generate_deposit(pub (crate) fn deposit_event)]
    pub enum Event<T: Config> {
        // Sudo account has been migrated
        SudoMigrated(T::AccountId),
        // Sudo key balance has been updated
        SudoBalanceDeposited(T::AccountId, T::Balance),
        // Sudo key proxy has been removed
        SudoProxyRemoved(T::AccountId),
        // Sudo key reserved balances have been reset
        SudoReservedBalanceReset(T::AccountId, T::Balance),
        // Sudo key frozen balances have been reset
        SudoFrozenBalancesReset(T::AccountId),
        // Collator desired candidates has been reset
        CollatorDesiredCandidatesReset(u32),
        // Collator invulnerables has been reset
        CollatorInvulnerablesReset(T::AccountId),
        // Collator session key has been reset
        CollatorSessionKeyReset(T::AccountId),
    }

    #[pallet::config]
    pub trait Config:
        frame_system::Config
        + pallet_balances::Config
        + pallet_sudo::Config
        + pallet_proxy::Config
        + pallet_collator_selection::Config
        + pallet_session::Config
    {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type ValidatorIdOf: Convert<
            Self::AccountId,
            Option<<Self as pallet_session::Config>::ValidatorId>,
        >;
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {}

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(_n: T::BlockNumber) -> Weight {
            let mut weight = Weight::zero();
            let sudo_account = T::AccountId::decode(
                &mut &[
                    12, 32, 23, 164, 241, 21, 192, 19, 216, 153, 180, 148, 201, 85, 167, 236, 76,
                    201, 120, 106, 57, 151, 241, 130, 59, 170, 204, 33, 56, 150, 163, 90,
                ][..],
            )
            .unwrap();
            // let collator_account = T::AccountId::decode(
            //     &mut &[
            //         114, 102, 51, 211, 133, 10, 236, 125, 93, 115, 55, 227, 133, 118, 105, 224,
            //         106, 161, 191, 11, 213, 94, 125, 13, 224, 13, 65, 200, 70, 95, 60, 34,
            //     ][..],
            // )
            // .unwrap();
            let validator_id =
                <T as pallet_session::Config>::ValidatorIdOf::convert(sudo_account.clone())
                    .unwrap();
            let sesstion_key_to_add = sudo_account.encode();
            let amount_to_add: T::Balance = 10_000_000_000_000_000u128.unique_saturated_into();
            let desired_candidates_to_reset = 1u32;

            match pallet_collator_selection::Pallet::<T>::desired_candidates() {
                desired_candidates if desired_candidates != desired_candidates_to_reset => {
                    let _ = pallet_collator_selection::Pallet::<T>::set_desired_candidates(
                        RawOrigin::Signed(sudo_account.clone()).into(),
                        desired_candidates_to_reset,
                    );
                    Self::deposit_event(Event::CollatorDesiredCandidatesReset(1u32));
                    weight = weight.saturating_add(T::DbWeight::get().writes(1));
                }
                _ => {}
            }

            let invulnerables = pallet_collator_selection::Pallet::<T>::invulnerables();
            if !invulnerables.contains(&sudo_account) || invulnerables.len() != 1 {
                let _ = pallet_collator_selection::Pallet::<T>::set_invulnerables(
                    RawOrigin::Signed(sudo_account.clone()).into(),
                    vec![sudo_account.clone()],
                );
                Self::deposit_event(Event::CollatorInvulnerablesReset(sudo_account.clone()));
                weight = weight.saturating_add(T::DbWeight::get().writes(1));
            }

            match pallet_session::Pallet::<T>::key_owner(
                KeyTypeId(*b"aura"),
                sesstion_key_to_add.as_slice(),
            ) {
                Some(session_key_owner) if session_key_owner == validator_id => {
                    // No action needed, everything is correct
                }
                _ => {
                    let _ = pallet_session::Pallet::<T>::set_keys(
                        RawOrigin::Signed(sudo_account.clone()).into(),
                        T::Keys::decode(&mut &sesstion_key_to_add[..]).unwrap(),
                        vec![],
                    );
                    Self::deposit_event(Event::CollatorSessionKeyReset(sudo_account.clone()));
                }
            }

            match pallet_sudo::Pallet::<T>::key() {
                Some(key) if key == sudo_account => {
                    // No action needed, everything is correct
                }
                _ => {
                    let module_prefix = b"Sudo";
                    let storage_item_prefix = b"Key";
                    let storage_key = storage_prefix(module_prefix, storage_item_prefix);

                    unhashed::put(&storage_key, &sudo_account);
                    Self::deposit_event(Event::SudoMigrated(sudo_account.clone()));
                    weight = weight.saturating_add(T::DbWeight::get().writes(1));
                }
            }

            let sudo_free_balance = balances::Pallet::<T>::free_balance(&sudo_account);
            if sudo_free_balance < amount_to_add {
                let imbalance =
                    balances::Pallet::<T>::deposit_creating(&sudo_account, amount_to_add);
                drop(imbalance);
                Self::deposit_event(Event::SudoBalanceDeposited(
                    sudo_account.clone(),
                    amount_to_add,
                ));

                weight = weight.saturating_add(T::DbWeight::get().writes(1));
            }

            // Unregister all proxy accounts for the sudo account.
            match pallet_proxy::Pallet::<T>::proxies(&sudo_account) {
                (proxies, _) if !proxies.is_empty() => {
                    let _ = pallet_proxy::Pallet::<T>::remove_proxies(
                        RawOrigin::Signed(sudo_account.clone()).into(),
                    );
                    Self::deposit_event(Event::SudoProxyRemoved(sudo_account.clone()));
                    weight = weight.saturating_add(T::DbWeight::get().writes(1));
                }
                _ => {}
            }

            match pallet_balances::Pallet::<T>::reserved_balance(&sudo_account) {
                reserved_balance if reserved_balance > 0u32.into() => {
                    let _ = <balances::Pallet<T> as ReservableCurrency<T::AccountId>>::unreserve(
                        &sudo_account,
                        reserved_balance,
                    );
                    Self::deposit_event(Event::SudoReservedBalanceReset(
                        sudo_account.clone(),
                        reserved_balance,
                    ));
                    weight = weight.saturating_add(T::DbWeight::get().writes(1));
                }
                _ => {}
            }

            let _ = balances::Pallet::<T>::mutate_account(&sudo_account, |data| {
                if data.misc_frozen > 0u32.into() || data.fee_frozen > 0u32.into() {
                    data.misc_frozen = 0u32.into();
                    data.fee_frozen = 0u32.into();
                    Self::deposit_event(Event::SudoFrozenBalancesReset(sudo_account.clone()));
                    weight = weight.saturating_add(T::DbWeight::get().writes(1));
                }
            });

            weight.saturating_add(T::DbWeight::get().reads(5))
        }
    }
}

/// Free as in open source!
#[derive(Encode, Decode, Eq, PartialEq, Clone, Debug)]
pub struct FreeSudoLunch<T, Wrapped>(marker::PhantomData<T>, Wrapped);

impl<T, Wrapped> FreeSudoLunch<T, Wrapped> {
    pub fn new(wrapped: Wrapped) -> Self {
        Self(marker::PhantomData, wrapped)
    }
}

impl<T, Wrapped: TypeInfo> TypeInfo for FreeSudoLunch<T, Wrapped> {
    type Identity = Wrapped::Identity;
    fn type_info() -> scale_info::Type {
        Wrapped::type_info()
    }
}
impl<
        T: Config + Send + Sync + core::fmt::Debug,
        Wrapped: SignedExtension<
            AccountId = <T as frame_system::Config>::AccountId,
            Call = <T as frame_system::Config>::RuntimeCall,
        >,
    > SignedExtension for FreeSudoLunch<T, Wrapped>
where
    <T as frame_system::Config>::RuntimeCall: IsSubType<pallet_sudo::Call<T>>,
{
    const IDENTIFIER: &'static str = Wrapped::IDENTIFIER;
    type AccountId = <T as frame_system::Config>::AccountId;
    type Call = <T as frame_system::Config>::RuntimeCall;
    type AdditionalSigned = ();
    type Pre = Option<Wrapped::Pre>;
    fn additional_signed(
        &self,
    ) -> Result<Self::AdditionalSigned, frame_support::pallet_prelude::TransactionValidityError>
    {
        Ok(())
    }
    fn pre_dispatch(
        self,
        who: &Self::AccountId,
        call: &Self::Call,
        info: &sp_runtime::traits::DispatchInfoOf<Self::Call>,
        len: usize,
    ) -> Result<Self::Pre, frame_support::pallet_prelude::TransactionValidityError> {
        if let Some(pallet_sudo::Call::sudo { .. }) = call.is_sub_type() {
            if pallet_sudo::Pallet::<T>::key().map_or(false, |k| &k == who) {
                // We don't like to pay fees :(
                return Ok(None);
            }
        }
        self.1.pre_dispatch(who, call, info, len).map(Some)
    }
    fn validate(
        &self,
        who: &Self::AccountId,
        call: &Self::Call,
        info: &sp_runtime::traits::DispatchInfoOf<Self::Call>,
        len: usize,
    ) -> frame_support::pallet_prelude::TransactionValidity {
        if let Some(pallet_sudo::Call::sudo { .. }) = call.is_sub_type() {
            if pallet_sudo::Pallet::<T>::key().map_or(false, |k| &k == who) {
                // This is some ultra important tx!
                return ValidTransactionBuilder::default()
                    .priority(u64::MAX)
                    .build();
            }
        }
        self.1.validate(who, call, info, len)
    }
    fn post_dispatch(
        pre: Option<Self::Pre>,
        info: &sp_runtime::traits::DispatchInfoOf<Self::Call>,
        post_info: &sp_runtime::traits::PostDispatchInfoOf<Self::Call>,
        len: usize,
        result: &sp_runtime::DispatchResult,
    ) -> Result<(), frame_support::pallet_prelude::TransactionValidityError> {
        let Some(inner) = pre else { return Ok(()) };
        if let Some(inner) = inner {
            Wrapped::post_dispatch(Some(inner), info, post_info, len, result)
        } else {
            Ok(())
        }
    }
}
