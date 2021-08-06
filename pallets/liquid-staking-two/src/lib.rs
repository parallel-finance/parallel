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

//! # Liquid staking pallet
//!
//! ## Overview
//!
//! This pallet manages the NPoS operations for relay chain asset.
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    pallet_prelude::*, traits::SortedMembers, transactional, BoundedVec, PalletId,
};
use frame_system::pallet_prelude::*;
use orml_traits::XcmTransfer;
use sp_runtime::{traits::AccountIdConversion, ArithmeticError, FixedPointNumber, RuntimeDebug};
use sp_std::convert::TryInto;
use sp_std::prelude::*;
use xcm::v0::{Junction, MultiLocation, NetworkId};

use orml_traits::{MultiCurrency, MultiCurrencyExtended};

pub use pallet::*;
use primitives::{Amount, Balance, CurrencyId, EraIndex, LiquidStakingProtocol, ExchangeRateProvider, Rate, Ratio};

#[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub enum StakingOperationType {
	Bond,
	BondExtra,
	Unbond,
	Rebond,
	TransferToRelaychain,
	RecordReward,
	RecordSlash,
}

#[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub enum Phase {
	Started,
	UpdateEraIndex,
	RecordReward,
	EmitEventToRelaychain,
	RecordStakingOperation,
	Finished,
}

#[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub enum ResponseStatus {
	Ready,
	Processing,
	Successed,
	Failed,
}

#[frame_support::pallet]
pub mod pallet {

    use super::*;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// Currency type used for staking and liquid assets
        type Currency: MultiCurrencyExtended<
            Self::AccountId,
            CurrencyId = CurrencyId,
            Balance = Balance,
            Amount = Amount,
        >;

        /// Currency used for staking
        #[pallet::constant]
        type StakingCurrency: Get<CurrencyId>;

        /// Currency used for liquid voucher
        #[pallet::constant]
        type LiquidCurrency: Get<CurrencyId>;

        /// The pallet id of liquid staking, keeps all the staking assets.
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// The origin which can withdraw staking assets.
        type WithdrawOrigin: EnsureOrigin<Self::Origin>;

        /// XCM transfer
        type XcmTransfer: XcmTransfer<Self::AccountId, Balance, CurrencyId>;

        /// Approved agent list on relaychain
        type Members: SortedMembers<Self::AccountId>;

        /// Base xcm weight to use for cross chain transfer
        type BaseXcmWeight: Get<Weight>;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// ExchangeRate is invalid
        InvalidExchangeRate,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// The assets get staked successfully
        Staked(T::AccountId, Balance),
        /// The voucher get unstaked successfully
        Unstaked(T::AccountId, Balance, Balance),
        /// The withdraw request is successful
        Claimed(T::AccountId, Balance),
        /// The rewards are recorded
        RewardsRecorded(T::AccountId, Balance),
        /// The slash is recorded
        SlashRecorded(T::AccountId, Balance),
    }

    /// The exchange rate converts staking native token to voucher.
    #[pallet::storage]
    #[pallet::getter(fn exchange_rate)]
    pub type ExchangeRate<T: Config> = StorageValue<_, Rate, ValueQuery>;

    /// Fraction of staking currency currently set aside for insurance pool
    #[pallet::storage]
    #[pallet::getter(fn reserve_factor)]
    pub type ReserveFactor<T: Config> = StorageValue<_, Ratio, ValueQuery>;

    /// The total amount of insurance pool.
    #[pallet::storage]
    #[pallet::getter(fn insurance_pool)]
    pub type InsurancePool<T: Config> = StorageValue<_, Balance, ValueQuery>;

    /// The total amount of staking pool.
    #[pallet::storage]
    #[pallet::getter(fn staking_pool)]
    pub type StakingPool<T: Config> = StorageValue<_, Balance, ValueQuery>;

    /// The queue stores all the pending unstaking requests.
    /// Key is the owner of assets.
    #[pallet::storage]
    #[pallet::getter(fn account_pending_unstake)]
    pub type Unstakes<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, Vec<(EraIndex,Balance)>>;

    /// Current era index on Relaychain.
	///
	/// CurrentEra: EraIndex
	#[pallet::storage]
	#[pallet::getter(fn current_era)]
	pub type CurrentEra<T: Config> = StorageValue<_, EraIndex, ValueQuery>;

    // #[pallet::storage]
    // #[pallet::getter(fn staking_operation_history)]
    // pub type StakingOperationHistory<T: Config> = 
    //     StorageMap<_, Blake2_128Concat, EraIndex, BTreeMap<StakingOperationType,(Balance,ResponseStatus)>>;

    // #[pallet::storage]
	// #[pallet::getter(fn current_phase)]
	// pub type CurrentPhase<T: Config> = StorageValue<_, Phase, ValueQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig {
        pub exchange_rate: Rate,
        pub reserve_factor: Ratio,
    }

    #[cfg(feature = "std")]
    impl Default for GenesisConfig {
        fn default() -> Self {
            Self {
                exchange_rate: Rate::default(),
                reserve_factor: Ratio::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig {
        fn build(&self) {
            ExchangeRate::<T>::put(self.exchange_rate);
            ReserveFactor::<T>::put(self.reserve_factor);
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        
        #[pallet::weight(10_000)]
        #[transactional]
        pub fn trigger_new_era(origin: OriginFor<T>,era_index: EraIndex) -> DispatchResultWithPostInfo {
            Ok(().into())
        }
        
        //todo，record reward on each era, invoked by stake-client
        #[pallet::weight(10_000)]
        #[transactional]
        pub fn record_reward(origin: OriginFor<T>,) -> DispatchResultWithPostInfo {
            Ok(().into())
        }
    
        //todo invoked by stake-client, considering insurrance pool
        #[pallet::weight(10_000)]
        #[transactional]
        pub fn record_slash(origin: OriginFor<T>,) -> DispatchResultWithPostInfo {
            Ok(().into())
        }
    
        // bond/unbond/rebond/bond_extra may be merge into one
        #[pallet::weight(10_000)]
        #[transactional]
        pub fn record_bond_response(origin: OriginFor<T>,) -> DispatchResultWithPostInfo {
            Ok(().into())
        }
    
        #[pallet::weight(10_000)]
        #[transactional]
        pub fn record_bond_extra_response(origin: OriginFor<T>,) -> DispatchResultWithPostInfo {
            Ok(().into())
        }
    
        #[pallet::weight(10_000)]
        #[transactional]
        pub fn record_rebond_response(origin: OriginFor<T>,) -> DispatchResultWithPostInfo {
            Ok(().into())
        }
    
        #[pallet::weight(10_000)]
        #[transactional]
        pub fn record_unbond_response(origin: OriginFor<T>,) -> DispatchResultWithPostInfo {
            Ok(().into())
        }
    
        #[pallet::weight(10_000)]
        #[transactional]
        pub fn transfer_to_relaychain(origin: OriginFor<T>, amount: Balance) -> DispatchResultWithPostInfo {
            // todo xcm transfer
            // maybe multiple in one era
            Ok(().into())
        }


        // todo below three method should be remove while stablity pool is ready
        #[pallet::weight(10_000)]
        #[transactional]
        pub fn stake(
            origin: OriginFor<T>, 
            #[pallet::compact] amount: Balance
        ) -> DispatchResultWithPostInfo {
            <Self as LiquidStakingProtocol>::stake();
            Ok(().into())
        }

        #[pallet::weight(10_000)]
        #[transactional]
        pub fn unstake(
            origin: OriginFor<T>,
            #[pallet::compact] amount: Balance,
        ) -> DispatchResultWithPostInfo {
            <Self as LiquidStakingProtocol>::unstake();
            Ok(().into())
        }

        #[pallet::weight(10_000)]
        #[transactional]
        pub fn claim(
            origin: OriginFor<T>,
            #[pallet::compact] amount: Balance,
        ) -> DispatchResultWithPostInfo {
            <Self as LiquidStakingProtocol>::claim();
            Ok(().into())
        }
    }
}

impl<T: Config> Pallet<T> {
    pub fn account_id() -> T::AccountId {
        T::PalletId::get().into_account()
    }
}

impl<T: Config> ExchangeRateProvider for Pallet<T> {
    fn get_exchange_rate() -> Rate {
        ExchangeRate::<T>::get()
    }
}

impl<T: Config> LiquidStakingProtocol for Pallet<T> {
    fn stake() -> () {
        // Ok(().into())
    }

    fn unstake() -> () {
        // Ok(().into())
    }

    fn claim() -> () {
        // Ok(().into())
    }
}
