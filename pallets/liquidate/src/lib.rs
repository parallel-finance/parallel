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

use codec::{Decode, Encode};
use frame_support::{log, pallet_prelude::*};
use frame_system::{
    ensure_signed,
    offchain::{AppCrypto, CreateSignedTransaction, SendSignedTransaction, Signer},
    pallet_prelude::*,
};
pub use module::*;
use primitives::*;
use sp_core::crypto::KeyTypeId;
use sp_runtime::{
    offchain as rt_offchain,
    offchain::storage_lock::{BlockAndTime, StorageLock},
    FixedPointNumber, RuntimeDebug,
};
use sp_std::prelude::*;

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"pool");
pub const LOCK_TIMEOUT_EXPIRATION: u64 = 20000; // in milli-seconds
pub const LOCK_BLOCK_EXPIRATION: u32 = 10; // in block number

type TotalSumPirce = Price;
type LiquidationThreshold = u128;
type DebtAccountBook = (CurrencyId, Balance, Price, TotalSumPirce);
type CollateralsAccountBook = (
    CurrencyId,
    Balance,
    Price,
    TotalSumPirce,
    LiquidationThreshold,
);
// the borrower's debt
type LiquidateToken = CurrencyId;
// the borrower's collateral
type CollateralToken = CurrencyId;
// the amount of liquidate_token the liquidator will repay for borrower
type RepayAmount = Balance;

#[frame_support::pallet]
pub mod module {
    use super::*;
    pub mod crypto {
        use super::KEY_TYPE;
        use sp_core::sr25519::Signature as Sr25519Signature;
        use sp_runtime::{
            app_crypto::{app_crypto, sr25519},
            traits::Verify,
            MultiSignature, MultiSigner,
        };
        app_crypto!(sr25519, KEY_TYPE);

        pub struct TestAuthId;
        impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for TestAuthId {
            type RuntimeAppPublic = Public;
            type GenericSignature = sp_core::sr25519::Signature;
            type GenericPublic = sp_core::sr25519::Public;
        }

        impl
            frame_system::offchain::AppCrypto<
                <Sr25519Signature as Verify>::Signer,
                Sr25519Signature,
            > for TestAuthId
        {
            type RuntimeAppPublic = Public;
            type GenericSignature = sp_core::sr25519::Signature;
            type GenericPublic = sp_core::sr25519::Public;
        }
    }

    /// store info that need to be liquidated
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
    pub struct WaitingForLiquidation<AccountId>(
        AccountId,
        LiquidateToken,
        CollateralToken,
        RepayAmount,
    );

    #[pallet::config]
    pub trait Config:
        frame_system::Config + CreateSignedTransaction<Call<Self>> + pallet_loans::Config
    {
        /// The identifier type for an offchain worker.
        type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
        /// The overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// The overarching dispatch call type.
        type Call: From<Call<Self>>;
        /// The oracle price feeder
        type PriceFeeder: PriceFeeder;
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn offchain_worker(block_number: T::BlockNumber) {
            Self::liquidate(block_number);
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub (crate) fn deposit_event)]
    pub enum Event<T: Config> {
        LiquidationOccur(T::AccountId, u128),
    }

    #[pallet::error]
    pub enum Error<T> {
        CaculateError,
        OracleCurrencyPriceNotReady,
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        fn execute_liquidation(
            origin: OriginFor<T>,
            waiting_for_liquidation_vec: Vec<WaitingForLiquidation<T::AccountId>>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            for (i, waiting_for_liquidation) in waiting_for_liquidation_vec.iter().enumerate() {
                let waiting_for_liquidation = waiting_for_liquidation.clone();
                let borrower = waiting_for_liquidation.0;
                let liquidate_token = waiting_for_liquidation.1;
                let collateral_token = waiting_for_liquidation.2;
                let repay_amount = waiting_for_liquidation.3;
                let r = pallet_loans::Pallet::<T>::liquidate_borrow_internal(
                    who.clone(),
                    borrower.clone(),
                    liquidate_token,
                    repay_amount,
                    collateral_token,
                );
                match r {
                    Ok(_) => log::info!("success liquidate index: {:?}", i),
                    Err(e) => {
                        log::error!("error invoke liquidate call: {:?}", e);
                        continue;
                    }
                }
            }
            // todo trigger event
            Self::deposit_event(Event::<T>::LiquidationOccur(
                who,
                waiting_for_liquidation_vec.len() as u128,
            ));
            Ok(().into())
        }
    }

    impl<T: Config> Pallet<T> {
        fn liquidate(_block_number: T::BlockNumber) {
            let mut lock = StorageLock::<BlockAndTime<Self>>::with_block_and_time_deadline(
                b"offchain-liquidate::lock",
                LOCK_BLOCK_EXPIRATION,
                rt_offchain::Duration::from_millis(LOCK_TIMEOUT_EXPIRATION),
            );
            if let Ok(_guard) = lock.try_lock() {
                // 1 get all borrowers
                let mut borrowers: Vec<T::AccountId> = vec![];
                for currency_id in pallet_loans::Currencies::<T>::get().iter() {
                    let mut v = pallet_loans::AccountBorrows::<T>::iter_prefix(currency_id)
                        .map(|(x, _)| x)
                        .filter(|x| !borrowers.contains(x))
                        .collect::<Vec<T::AccountId>>();
                    borrowers.append(&mut v);
                }
                // Execute liquidation with borrowers one by one
                'outer: for borrower in borrowers.iter() {
                    // 2.1 get debts by currency
                    let mut classify_debts: Vec<DebtAccountBook> = vec![];
                    // 2.2 get collaterals by currency
                    let mut classify_collaterals: Vec<CollateralsAccountBook> = vec![];

                    for currency_id in pallet_loans::Currencies::<T>::get().iter() {
                        let currency_price =
                            match <T as module::Config>::PriceFeeder::get_price(currency_id)
                                .ok_or(Error::<T>::OracleCurrencyPriceNotReady)
                            {
                                Ok((v, _)) => v,
                                Err(e) => {
                                    log::error!("error msg: {:?}", e);
                                    Price::MIN
                                }
                            };
                        // 2.1.1 insert debt by currency
                        let borrow_currency_amount =
                            match pallet_loans::Pallet::<T>::borrow_balance_stored(
                                borrower,
                                currency_id,
                            ) {
                                Ok(v) => v,
                                Err(e) => {
                                    log::error!("error get borrow balance: {:?}", e);
                                    continue 'outer;
                                }
                            };
                        if borrow_currency_amount > 0 {
                            if currency_price == Price::MIN {
                                continue 'outer;
                            }
                            let borrow_currency_sum_price = match borrow_currency_amount
                                .checked_mul(currency_price)
                                .ok_or(Error::<T>::CaculateError)
                            {
                                Ok(v) => v,
                                Err(e) => {
                                    log::error!("error calculate borrow: {:?}", e);
                                    continue 'outer;
                                }
                            };

                            classify_debts.push((
                                *currency_id,
                                borrow_currency_amount,
                                currency_price,
                                borrow_currency_sum_price,
                            ));
                        }

                        // 2.2.1 insert collateral by currency
                        let collateral_ctoken_amount =
                            pallet_loans::AccountCollateral::<T>::get(currency_id, &borrower);
                        //the total amount of borrower's collateral token
                        if collateral_ctoken_amount > 0 {
                            if currency_price == Price::MIN {
                                continue 'outer;
                            }
                            let exchange_rate = pallet_loans::ExchangeRate::<T>::get(currency_id);
                            let collateral_currency_amount = match exchange_rate
                                .checked_mul_int(collateral_ctoken_amount)
                                .ok_or(Error::<T>::CaculateError)
                            {
                                Ok(v) => v,
                                Err(e) => {
                                    log::error!("error calculate collateral amount: {:?}", e);
                                    continue 'outer;
                                }
                            };

                            //the total price of borrower's collateral token
                            let collateral_currency_sum_price = match collateral_currency_amount
                                .checked_mul(currency_price)
                                .ok_or(Error::<T>::CaculateError)
                            {
                                Ok(v) => v,
                                Err(e) => {
                                    log::error!("error calculate collateral sum price: {:?}", e);
                                    continue 'outer;
                                }
                            };
                            let liquidation_threshold =
                                pallet_loans::LiquidationThreshold::<T>::get(currency_id);
                            classify_collaterals.push((
                                *currency_id,
                                collateral_currency_amount,
                                currency_price,
                                collateral_currency_sum_price,
                                liquidation_threshold,
                            ));
                        }
                    }

                    if classify_debts.is_empty() || classify_collaterals.is_empty() {
                        continue;
                    }
                    // 3 check liquidation threshold
                    // if (collateral_total_value * liquidation_threshold)/(debt_total_value) < 1 ,execute liquidation
                    let mut processing = true;
                    let collateral_liquidation_threshold_value = classify_collaterals.iter().fold(
                        Price::MIN,
                        |acc,&(_,_,_,total_sum_price,liquidation_threshold)|
							// acc + total_sum_price * liquidation_threshold
							match total_sum_price
								.checked_mul(liquidation_threshold)
								.and_then(|r| r.checked_div(RATE_DECIMAL))
								.and_then(|r| r.checked_add(acc))
								.ok_or(Error::<T>::CaculateError)
							{
								Ok(v) => v,
								Err(e) => {
									log::error!("error calculate liquidation threshold: {:?}",e);
									processing = false;
									acc
								}
							},
                    );
                    if !processing {
                        continue;
                    }

                    let debt_total_value = classify_debts.iter().fold(
                        Price::MIN,
                        |acc, &(_,_,_,total_sum_price)|
							// acc + total_sum_price
							match acc
								.checked_add(total_sum_price)
								.ok_or(Error::<T>::CaculateError)
							{
								Ok(v) => v,
								Err(e) => {
									log::error!("error calculate debt toal: {:?}",e);
									processing = false;
									acc
								}
							},
                    );
                    if !processing {
                        continue;
                    }

                    log::info!(
                        "_threshold_value: {:?}",
                        collateral_liquidation_threshold_value
                    );
                    log::info!("debt_total_value: {:?}", debt_total_value);

                    // 4 no need liquidate
                    if collateral_liquidation_threshold_value > debt_total_value {
                        continue;
                    }

                    // 5 liquidation strategy
                    let mut waiting_for_liquidation_vec: Vec<WaitingForLiquidation<T::AccountId>> =
                        vec![];

                    let collateral_total_value = classify_collaterals.iter().fold(
                        Price::MIN,
                        |acc,&(_,_,_,total_sum_price,_)|
							// acc + total_sum_price
							match acc
								.checked_add(total_sum_price)
								.ok_or(Error::<T>::CaculateError)
							{
								Ok(v) => v,
								Err(e) => {
									log::error!("error calculate collateral toal: {:?}",e);
									processing = false;
									acc
								}
							},
                    );
                    if !processing {
                        continue;
                    }
                    for &(liquidate_token, debt_repay_amount, _, _debt_total_sum_price) in
                        classify_debts.iter()
                    {
                        let close_factor = pallet_loans::CloseFactor::<T>::get(liquidate_token);
                        //CollateralsAccountBook = (CurrencyId, Balance, Price, TotalSumPirce, LiquidationThreshold);
                        for &(collateral_token, _, _, single_collateral_total_sum_pirce, _) in
                            classify_collaterals.iter()
                        {
                            // let repay_amount = (single_collateral_total_sum_pirce / collateral_total_value) * (debt_repay_amount * close_factor);
                            let m: Price = 100;
                            let repay_amount = match (close_factor
                                * single_collateral_total_sum_pirce)
                                .checked_mul(m)
                                .and_then(|r| r.checked_div(collateral_total_value))
                                .and_then(|r| r.checked_mul(debt_repay_amount))
                                .and_then(|r| r.checked_div(m))
                                .ok_or(Error::<T>::CaculateError)
                            {
                                Ok(v) => v,
                                Err(e) => {
                                    log::error!(
                                        "error calculate waiting_for_liquidation_vec: {:?}",
                                        e
                                    );
                                    processing = false;
                                    Price::MIN
                                }
                            };
                            waiting_for_liquidation_vec.push(WaitingForLiquidation(
                                borrower.clone(),
                                liquidate_token,
                                collateral_token,
                                repay_amount,
                            ));
                        }
                    }
                    if waiting_for_liquidation_vec.is_empty() || !processing {
                        continue;
                    }
                    //liquidate a single user every time
                    Self::offchain_signed_tx(borrower.clone(), waiting_for_liquidation_vec);
                }
                return;
            }
            log::error!("offchain_worker error: get lock failed");
        }

        fn offchain_signed_tx(
            _borrower: T::AccountId,
            waiting_for_liquidation_vec: Vec<WaitingForLiquidation<T::AccountId>>,
        ) {
            // Get signer from ocw
            //TODO get special pool account
            let signer = Signer::<T, <T as module::Config>::AuthorityId>::any_account();
            let result = signer.send_signed_transaction(|_acct|
				// This is the on-chain function
				Call::execute_liquidation(waiting_for_liquidation_vec.clone()));

            // Display error if the signed tx fails.
            if let Some((acc, res)) = result {
                if res.is_err() {
                    log::error!("failure: offchain_signed_tx: tx sent: {:?}", acc.id);
                } else {
                    log::info!(
                        "successful: offchain_signed_tx: tx sent: {:?} index is {:?}",
                        acc.id,
                        acc.index
                    );
                }
            } else {
                log::error!("No local account available");
            }
        }
    }

    impl<T: Config> rt_offchain::storage_lock::BlockNumberProvider for Pallet<T> {
        type BlockNumber = T::BlockNumber;
        fn current_block_number() -> Self::BlockNumber {
            <frame_system::Pallet<T>>::block_number()
        }
    }
}
