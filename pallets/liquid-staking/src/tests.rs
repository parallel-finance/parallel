use crate::{
    mock::*,
    types::{MatchingLedger, StakingLedger, UnlockChunk, XcmRequest},
    *,
};

use frame_support::{
    assert_noop, assert_ok, error::BadOrigin, storage::with_transaction, traits::Hooks,
};

use primitives::{
    tokens::{KSM, SKSM},
    ump::RewardDestination,
    Balance, Rate, Ratio,
};
use sp_runtime::traits::BlakeTwo256;
use sp_runtime::{
    traits::{One, Zero},
    MultiAddress::Id,
    TransactionOutcome,
};
use sp_trie::StorageProof;
use xcm_simulator::TestExt;

#[test]
fn stake_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(LiquidStaking::stake(Origin::signed(ALICE), ksm(10f64)));
        // Check storage is correct
        assert_eq!(ExchangeRate::<Test>::get(), Rate::one());
        assert_eq!(
            MatchingPool::<Test>::get(),
            MatchingLedger {
                total_stake_amount: ksm(9.95f64),
                total_unstake_amount: 0,
            }
        );

        // Check balance is correct
        assert_eq!(<Test as Config>::Assets::balance(KSM, &ALICE), ksm(90f64));
        assert_eq!(
            <Test as Config>::Assets::balance(SKSM, &ALICE),
            ksm(109.95f64)
        );

        assert_eq!(
            <Test as Config>::Assets::balance(KSM, &LiquidStaking::account_id()),
            ksm(10f64)
        );

        with_transaction(|| {
            LiquidStaking::do_advance_era(1).unwrap();
            LiquidStaking::notification_received(
                pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
                0,
                Response::ExecutionResult(None),
            )
            .unwrap();
            TransactionOutcome::Commit(0)
        });

        assert_eq!(
            <Test as Config>::Assets::balance(KSM, &LiquidStaking::account_id()),
            ksm(0.05f64)
        );

        assert_eq!(
            MatchingPool::<Test>::get(),
            MatchingLedger {
                total_stake_amount: 0,
                total_unstake_amount: 0,
            }
        );
        let derivative_index = <Test as Config>::DerivativeIndex::get();
        assert_eq!(
            StakingLedgers::<Test>::get(&0).unwrap(),
            StakingLedger {
                stash: LiquidStaking::derivative_sovereign_account_id(derivative_index),
                total: ksm(9.95f64),
                active: ksm(9.95f64),
                unlocking: vec![],
                claimed_rewards: vec![]
            }
        );

        assert_ok!(LiquidStaking::stake(Origin::signed(ALICE), ksm(10f64)));

        with_transaction(|| {
            LiquidStaking::do_advance_era(1).unwrap();
            LiquidStaking::notification_received(
                pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
                1,
                Response::ExecutionResult(None),
            )
            .unwrap();
            TransactionOutcome::Commit(0)
        });

        assert_eq!(
            <Test as Config>::Assets::balance(KSM, &LiquidStaking::account_id()),
            ksm(0.1f64)
        );

        assert_eq!(
            StakingLedgers::<Test>::get(&0).unwrap(),
            StakingLedger {
                stash: LiquidStaking::derivative_sovereign_account_id(derivative_index),
                total: ksm(19.9f64),
                active: ksm(19.9f64),
                unlocking: vec![],
                claimed_rewards: vec![]
            }
        );
    })
}

#[test]
fn unstake_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(LiquidStaking::stake(Origin::signed(ALICE), ksm(10f64)));
        assert_ok!(LiquidStaking::unstake(Origin::signed(ALICE), ksm(6f64)));

        // Check storage is correct
        assert_eq!(ExchangeRate::<Test>::get(), Rate::one());
        assert_eq!(
            MatchingPool::<Test>::get(),
            MatchingLedger {
                total_stake_amount: ksm(9.95f64),
                total_unstake_amount: ksm(6f64),
            }
        );

        assert_eq!(
            Unlockings::<Test>::get(ALICE).unwrap(),
            vec![UnlockChunk {
                value: ksm(6f64),
                era: 4
            }]
        );

        with_transaction(|| {
            LiquidStaking::do_advance_era(1).unwrap();
            LiquidStaking::notification_received(
                pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
                0,
                Response::ExecutionResult(None),
            )
            .unwrap();
            TransactionOutcome::Commit(0)
        });

        assert_eq!(
            MatchingPool::<Test>::get(),
            MatchingLedger {
                total_stake_amount: 0,
                total_unstake_amount: 0,
            }
        );

        let derivative_index = <Test as Config>::DerivativeIndex::get();
        assert_eq!(
            StakingLedgers::<Test>::get(&0).unwrap(),
            StakingLedger {
                stash: LiquidStaking::derivative_sovereign_account_id(derivative_index),
                total: ksm(3.95f64),
                active: ksm(3.95f64),
                unlocking: vec![],
                claimed_rewards: vec![]
            }
        );
        // Just make it 1 to calculate.
        ExchangeRate::<Test>::set(Rate::one());
        assert_ok!(LiquidStaking::unstake(Origin::signed(ALICE), ksm(3.95f64)));

        assert_eq!(
            Unlockings::<Test>::get(ALICE).unwrap(),
            vec![
                UnlockChunk {
                    value: ksm(6f64),
                    era: 4
                },
                UnlockChunk {
                    value: ksm(3.95f64),
                    era: 5
                }
            ]
        );

        with_transaction(|| {
            LiquidStaking::do_advance_era(1).unwrap();
            LiquidStaking::notification_received(
                pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
                1,
                Response::ExecutionResult(None),
            )
            .unwrap();
            TransactionOutcome::Commit(0)
        });

        assert_eq!(
            StakingLedgers::<Test>::get(&0).unwrap(),
            StakingLedger {
                stash: LiquidStaking::derivative_sovereign_account_id(derivative_index),
                total: ksm(3.95),
                active: 0,
                unlocking: vec![UnlockChunk {
                    value: ksm(3.95),
                    era: 5
                }],
                claimed_rewards: vec![]
            }
        );
    })
}

enum StakeOp {
    Stake(Balance),
    Unstake(Balance),
}

impl StakeOp {
    fn execute(self) {
        match self {
            Self::Stake(amount) => LiquidStaking::stake(Origin::signed(ALICE), amount).unwrap(),
            Self::Unstake(amount) => LiquidStaking::unstake(Origin::signed(ALICE), amount).unwrap(),
        };
    }
}

#[test]
fn test_matching_should_work() {
    use StakeOp::*;
    TestNet::reset();
    ParaA::execute_with(|| {
        let test_case: Vec<(Vec<StakeOp>, Balance, Balance, (Balance, Balance, Balance))> = vec![
            (
                vec![Stake(ksm(5000f64)), Unstake(ksm(1000f64))],
                0,
                0,
                (ksm(3975f64), 0, 0),
            ),
            // Calculate right here.
            (
                vec![Unstake(ksm(10f64)), Unstake(ksm(5f64)), Stake(ksm(10f64))],
                ksm(3975f64),
                0,
                (0, 0, ksm(5.05f64)),
            ),
            // (vec![], 0, (0, 0, 0)),
        ];
        for (i, (stake_ops, _bonding_amount, unbonding_amount, matching_result)) in
            test_case.into_iter().enumerate()
        {
            stake_ops.into_iter().for_each(StakeOp::execute);
            assert_eq!(
                LiquidStaking::matching_pool().matching(unbonding_amount),
                Ok(matching_result)
            );
            with_transaction(|| {
                LiquidStaking::do_advance_era(1).unwrap();
                LiquidStaking::notification_received(
                    pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
                    i.try_into().unwrap(),
                    Response::ExecutionResult(None),
                )
                .unwrap();
                TransactionOutcome::Commit(0)
            });
        }
    });
}

#[test]
fn test_transact_bond_work() {
    TestNet::reset();
    let derivative_index = <Test as Config>::DerivativeIndex::get();
    ParaA::execute_with(|| {
        assert_ok!(LiquidStaking::stake(Origin::signed(ALICE), ksm(2000f64),));
        assert_ok!(LiquidStaking::bond(
            Origin::signed(ALICE),
            derivative_index,
            ksm(3f64),
            RewardDestination::Staked
        ));

        ParaSystem::assert_has_event(mock::Event::LiquidStaking(crate::Event::Bonding(
            derivative_index,
            LiquidStaking::derivative_sovereign_account_id(derivative_index),
            ksm(3f64),
            RewardDestination::Staked,
        )));
    });

    Relay::execute_with(|| {
        RelaySystem::assert_has_event(RelayEvent::Staking(RelayStakingEvent::Bonded(
            LiquidStaking::derivative_sovereign_account_id(derivative_index),
            ksm(3f64),
        )));
        let ledger = RelayStaking::ledger(LiquidStaking::derivative_sovereign_account_id(
            derivative_index,
        ))
        .unwrap();
        assert_eq!(ledger.total, ksm(3f64));
    });
}

#[test]
fn test_transact_bond_extra_work() {
    TestNet::reset();
    let derivative_index = <Test as Config>::DerivativeIndex::get();
    ParaA::execute_with(|| {
        assert_ok!(LiquidStaking::stake(Origin::signed(ALICE), ksm(4000f64),));

        assert_ok!(LiquidStaking::bond(
            Origin::signed(ALICE),
            derivative_index,
            ksm(2f64),
            RewardDestination::Staked
        ));

        assert_ok!(LiquidStaking::notification_received(
            pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
            0,
            Response::ExecutionResult(None),
        ));

        assert_ok!(LiquidStaking::bond_extra(
            Origin::signed(ALICE),
            derivative_index,
            ksm(3f64)
        ));
    });

    Relay::execute_with(|| {
        let ledger = RelayStaking::ledger(LiquidStaking::derivative_sovereign_account_id(
            derivative_index,
        ))
        .unwrap();
        assert_eq!(ledger.total, ksm(5f64));
    });
}

#[test]
fn test_transact_unbond_work() {
    TestNet::reset();
    let derivative_index = <Test as Config>::DerivativeIndex::get();
    ParaA::execute_with(|| {
        assert_ok!(LiquidStaking::stake(Origin::signed(ALICE), ksm(6000f64),));

        assert_ok!(LiquidStaking::bond(
            Origin::signed(ALICE),
            derivative_index,
            ksm(5f64),
            RewardDestination::Staked
        ));
        assert_ok!(LiquidStaking::notification_received(
            pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
            0,
            Response::ExecutionResult(None),
        ));
        assert_ok!(LiquidStaking::unbond(
            Origin::signed(ALICE),
            derivative_index,
            ksm(2f64)
        ));
    });

    Relay::execute_with(|| {
        RelaySystem::assert_has_event(RelayEvent::Staking(RelayStakingEvent::Bonded(
            LiquidStaking::derivative_sovereign_account_id(derivative_index),
            ksm(5f64),
        )));
        RelaySystem::assert_has_event(RelayEvent::Staking(RelayStakingEvent::Unbonded(
            LiquidStaking::derivative_sovereign_account_id(derivative_index),
            ksm(2f64),
        )));
        let ledger = RelayStaking::ledger(LiquidStaking::derivative_sovereign_account_id(
            derivative_index,
        ))
        .unwrap();
        assert_eq!(ledger.total, ksm(5f64));
        assert_eq!(ledger.active, ksm(3f64));
    });
}

#[test]
fn test_transact_withdraw_unbonded_work() {
    TestNet::reset();
    let derivative_index = <Test as Config>::DerivativeIndex::get();
    ParaA::execute_with(|| {
        assert_ok!(LiquidStaking::stake(Origin::signed(ALICE), ksm(6000f64),));
        assert_ok!(LiquidStaking::unstake(Origin::signed(ALICE), ksm(2000f64),));

        assert_ok!(LiquidStaking::bond(
            Origin::signed(ALICE),
            derivative_index,
            ksm(5f64),
            RewardDestination::Staked
        ));
        assert_ok!(LiquidStaking::notification_received(
            pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
            0,
            Response::ExecutionResult(None),
        ));
        assert_ok!(LiquidStaking::unbond(
            Origin::signed(ALICE),
            derivative_index,
            ksm(2f64)
        ));
        assert_ok!(LiquidStaking::notification_received(
            pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
            1,
            Response::ExecutionResult(None),
        ));
    });

    Relay::execute_with(|| {
        let ledger = RelayStaking::ledger(LiquidStaking::derivative_sovereign_account_id(
            derivative_index,
        ))
        .unwrap();
        assert_eq!(ledger.total, ksm(5f64));
        assert_eq!(ledger.active, ksm(3f64));
        assert_eq!(ledger.unlocking.len(), 1);

        RelaySystem::assert_has_event(RelayEvent::Staking(RelayStakingEvent::Bonded(
            LiquidStaking::derivative_sovereign_account_id(derivative_index),
            ksm(5f64),
        )));
        RelaySystem::assert_has_event(RelayEvent::Staking(RelayStakingEvent::Unbonded(
            LiquidStaking::derivative_sovereign_account_id(derivative_index),
            ksm(2f64),
        )));

        pallet_staking::CurrentEra::<KusamaRuntime>::put(
            <KusamaRuntime as pallet_staking::Config>::BondingDuration::get(),
        );
    });

    ParaA::execute_with(|| {
        assert_ok!(LiquidStaking::force_set_current_era(
            Origin::root(),
            <KusamaRuntime as pallet_staking::Config>::BondingDuration::get(),
        ));

        assert_ok!(LiquidStaking::withdraw_unbonded(
            Origin::signed(BOB),
            derivative_index,
            0
        ));
    });

    Relay::execute_with(|| {
        let ledger = RelayStaking::ledger(LiquidStaking::derivative_sovereign_account_id(
            derivative_index,
        ))
        .unwrap();
        assert_eq!(ledger.total, ksm(3f64));
        assert_eq!(ledger.active, ksm(3f64));
        assert_eq!(ledger.unlocking.len(), 0);
    });
}

#[test]
fn test_transact_rebond_work() {
    TestNet::reset();
    let derivative_index = <Test as Config>::DerivativeIndex::get();
    ParaA::execute_with(|| {
        assert_ok!(LiquidStaking::stake(Origin::signed(ALICE), ksm(6000f64),));

        assert_ok!(LiquidStaking::bond(
            Origin::signed(ALICE),
            derivative_index,
            ksm(10f64),
            RewardDestination::Staked
        ));
        assert_ok!(LiquidStaking::notification_received(
            pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
            0,
            Response::ExecutionResult(None),
        ));
        assert_ok!(LiquidStaking::unbond(
            Origin::signed(ALICE),
            derivative_index,
            ksm(5f64)
        ));
        assert_ok!(LiquidStaking::rebond(
            Origin::signed(ALICE),
            derivative_index,
            ksm(3f64)
        ));
    });

    Relay::execute_with(|| {
        RelaySystem::assert_has_event(RelayEvent::Staking(RelayStakingEvent::Bonded(
            LiquidStaking::derivative_sovereign_account_id(derivative_index),
            ksm(10f64),
        )));
        RelaySystem::assert_has_event(RelayEvent::Staking(RelayStakingEvent::Unbonded(
            LiquidStaking::derivative_sovereign_account_id(derivative_index),
            ksm(5f64),
        )));
        RelaySystem::assert_has_event(RelayEvent::Staking(RelayStakingEvent::Bonded(
            LiquidStaking::derivative_sovereign_account_id(derivative_index),
            ksm(3f64),
        )));
        let ledger = RelayStaking::ledger(LiquidStaking::derivative_sovereign_account_id(
            derivative_index,
        ))
        .unwrap();
        assert_eq!(ledger.total, ksm(10f64));
        assert_eq!(ledger.active, ksm(8f64));
    });
}

#[test]
fn test_transact_nominate_work() {
    TestNet::reset();
    let derivative_index = <Test as Config>::DerivativeIndex::get();
    ParaA::execute_with(|| {
        assert_ok!(LiquidStaking::stake(Origin::signed(ALICE), ksm(4000f64),));

        assert_ok!(LiquidStaking::bond(
            Origin::signed(ALICE),
            derivative_index,
            ksm(10f64),
            RewardDestination::Staked
        ));

        assert_ok!(LiquidStaking::notification_received(
            pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
            0,
            Response::ExecutionResult(None),
        ));

        assert_ok!(LiquidStaking::nominate(
            Origin::signed(ALICE),
            derivative_index,
            vec![ALICE, BOB],
        ));
    });

    Relay::execute_with(|| {
        let ledger = RelayStaking::ledger(LiquidStaking::derivative_sovereign_account_id(
            derivative_index,
        ))
        .unwrap();
        assert_eq!(ledger.total, ksm(10f64));
        let nominators = RelayStaking::nominators(LiquidStaking::derivative_sovereign_account_id(
            derivative_index,
        ))
        .unwrap();
        assert_eq!(nominators.targets, vec![ALICE, BOB]);
    });
}

#[test]
fn test_transfer_bond() {
    TestNet::reset();
    let xcm_transfer_amount = ksm(10f64);
    let derivative_index = <Test as Config>::DerivativeIndex::get();
    ParaA::execute_with(|| {
        assert_ok!(LiquidStaking::stake(Origin::signed(ALICE), ksm(2000f64),));
        assert_ok!(LiquidStaking::bond(
            Origin::signed(ALICE),
            derivative_index,
            xcm_transfer_amount,
            RewardDestination::Staked
        ));
        // print_events::<Test>("ParaA");
    });
    Relay::execute_with(|| {
        // print_events::<kusama_runtime::Runtime>("Relay");
        let ledger = RelayStaking::ledger(LiquidStaking::derivative_sovereign_account_id(
            derivative_index,
        ))
        .unwrap();
        assert_eq!(ledger.total, xcm_transfer_amount);
        assert_eq!(
            RelayBalances::free_balance(LiquidStaking::derivative_sovereign_account_id(
                derivative_index
            )),
            xcm_transfer_amount
        );
        assert_eq!(
            RelayBalances::usable_balance(LiquidStaking::derivative_sovereign_account_id(
                derivative_index
            )),
            0
        );
    });
}

#[test]
fn update_market_cap_should_not_work_if_with_invalid_param() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            LiquidStaking::update_market_cap(Origin::root(), Zero::zero()),
            Error::<Test>::InvalidCap
        );
    })
}

#[test]
fn update_reserve_factor_should_not_work_if_with_invalid_param() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            LiquidStaking::update_reserve_factor(Origin::root(), Ratio::zero()),
            Error::<Test>::InvalidFactor
        );
        assert_noop!(
            LiquidStaking::update_reserve_factor(Origin::root(), Ratio::one()),
            Error::<Test>::InvalidFactor
        );
    })
}

#[test]
fn claim_for_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(LiquidStaking::stake(Origin::signed(ALICE), ksm(10f64)));
        assert_eq!(<Test as Config>::Assets::balance(KSM, &ALICE), ksm(90f64));

        assert_ok!(LiquidStaking::unstake(Origin::signed(ALICE), ksm(1f64)));
        assert_ok!(LiquidStaking::unstake(Origin::signed(ALICE), ksm(3.95f64)));
        assert_eq!(
            Unlockings::<Test>::get(ALICE).unwrap(),
            vec![UnlockChunk {
                value: ksm(4.95f64),
                era: 4
            },]
        );

        assert_noop!(
            LiquidStaking::claim_for(Origin::signed(BOB), Id(ALICE)),
            Error::<Test>::NothingToClaim
        );

        let derivative_index = <Test as Config>::DerivativeIndex::get();
        with_transaction(|| {
            assert_ok!(LiquidStaking::do_advance_era(4));
            TransactionOutcome::Commit(0)
        });
        assert_ok!(LiquidStaking::notification_received(
            pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
            0,
            Response::ExecutionResult(None),
        ));
        assert_ok!(LiquidStaking::withdraw_unbonded(
            Origin::signed(BOB),
            derivative_index,
            0
        ));
        assert_ok!(LiquidStaking::notification_received(
            pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
            1,
            Response::ExecutionResult(None),
        ));

        assert_ok!(LiquidStaking::claim_for(Origin::signed(BOB), Id(ALICE)));
        assert_eq!(
            <Test as Config>::Assets::balance(KSM, &ALICE),
            ksm(90f64) + ksm(4.95f64)
        );

        assert!(Unlockings::<Test>::get(ALICE).is_none());
    })
}

#[test]
fn test_on_initialize_work() {
    new_test_ext().execute_with(|| {
        let derivative_index = <Test as Config>::DerivativeIndex::get();
        let xcm_fees = XcmFees::get();
        let reserve_factor = LiquidStaking::reserve_factor();

        // 1.1 stake
        let bond_amount = ksm(10f64);
        assert_ok!(LiquidStaking::stake(Origin::signed(ALICE), bond_amount));
        let total_stake_amount = bond_amount - xcm_fees - reserve_factor.mul_floor(bond_amount);

        // 1.2 on_initialize_bond
        let total_era_blocknumbers = <Test as Config>::EraLength::get();
        assert_eq!(total_era_blocknumbers, 10);
        RelayChainValidationDataProvider::set(total_era_blocknumbers);
        LiquidStaking::on_initialize(System::block_number());
        assert_eq!(EraStartBlock::<Test>::get(), total_era_blocknumbers);
        assert_eq!(CurrentEra::<Test>::get(), 1);
        assert_eq!(LiquidStaking::staking_ledgers(derivative_index), None);
        assert_eq!(
            LiquidStaking::matching_pool(),
            MatchingLedger {
                total_stake_amount,
                total_unstake_amount: 0,
            }
        );

        // 1.3 notification_received bond
        assert_ok!(LiquidStaking::notification_received(
            pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
            0,
            Response::ExecutionResult(None),
        ));

        let staking_ledger = <StakingLedger<AccountId, BalanceOf<Test>>>::new(
            LiquidStaking::derivative_sovereign_account_id(derivative_index),
            total_stake_amount,
        );
        assert_eq!(
            LiquidStaking::staking_ledgers(derivative_index).unwrap(),
            staking_ledger
        );

        assert_eq!(LiquidStaking::matching_pool(), MatchingLedger::default());
    })
}

// TODO: add more test
// #[test]
// fn test_set_staking_ledger_work() {
//     new_test_ext().execute_with(|| {
//         let derivative_index = <Test as Config>::DerivativeIndex::get();
//         let bond_amount = 100;
//         let bond_extra_amount = 50;
//         let mut staking_ledger = <StakingLedger<AccountId, BalanceOf<Test>>>::new(
//             LiquidStaking::derivative_sovereign_account_id(derivative_index),
//             bond_amount,
//         );
//         assert_noop!(
//             LiquidStaking::set_staking_ledger(
//                 Origin::signed(ALICE),
//                 derivative_index,
//                 staking_ledger.clone()
//             ),
//             Error::<Test>::NotBonded
//         );
//         StakingLedgers::<Test>::insert(derivative_index, staking_ledger.clone());
//         assert_eq!(
//             LiquidStaking::staking_ledgers(derivative_index).unwrap(),
//             staking_ledger.clone()
//         );
//         staking_ledger.bond_extra(bond_extra_amount);
//         assert_ok!(LiquidStaking::set_staking_ledger(
//             Origin::signed(ALICE),
//             derivative_index,
//             staking_ledger.clone()
//         ));

//         assert_noop!(
//             LiquidStaking::set_staking_ledger(
//                 Origin::signed(ALICE),
//                 derivative_index,
//                 staking_ledger.clone()
//             ),
//             Error::<Test>::StakingLedgerLocked
//         );

//         LiquidStaking::on_finalize(1);

//         assert_ok!(LiquidStaking::set_staking_ledger(
//             Origin::signed(ALICE),
//             derivative_index,
//             staking_ledger.clone()
//         ));

//         let new_staking_ledger = <StakingLedger<AccountId, BalanceOf<Test>>>::new(
//             LiquidStaking::derivative_sovereign_account_id(derivative_index),
//             bond_amount + bond_extra_amount,
//         );
//         assert_eq!(
//             LiquidStaking::staking_ledgers(derivative_index).unwrap(),
//             new_staking_ledger
//         );
//     })
// }

#[test]
fn test_force_set_era_start_block_work() {
    new_test_ext().execute_with(|| {
        assert_eq!(EraStartBlock::<Test>::get(), 0);
        assert_ok!(LiquidStaking::force_set_era_start_block(Origin::root(), 11));
        assert_eq!(EraStartBlock::<Test>::get(), 11);
    })
}

#[test]
fn test_force_set_current_era_work() {
    new_test_ext().execute_with(|| {
        assert_eq!(CurrentEra::<Test>::get(), 0);
        assert_ok!(LiquidStaking::force_set_current_era(Origin::root(), 12));
        assert_eq!(CurrentEra::<Test>::get(), 12);
    })
}

#[test]
fn test_force_notification_received_work() {
    new_test_ext().execute_with(|| {
        let derivative_index = <Test as Config>::DerivativeIndex::get();
        let bond_amount = ksm(10f64);
        assert_ok!(LiquidStaking::stake(Origin::signed(ALICE), ksm(20f64),));

        assert_ok!(LiquidStaking::bond(
            Origin::signed(ALICE),
            derivative_index,
            bond_amount,
            RewardDestination::Staked
        ));
        let query_id = 0;
        assert_eq!(
            XcmRequests::<Test>::get(query_id),
            Some(XcmRequest::Bond {
                index: derivative_index,
                amount: bond_amount,
            })
        );
        assert_noop!(
            LiquidStaking::notification_received(
                Origin::signed(ALICE),
                query_id,
                Response::ExecutionResult(None),
            ),
            BadOrigin
        );
        assert_ok!(LiquidStaking::notification_received(
            Origin::root(),
            query_id,
            Response::ExecutionResult(None),
        ));
        assert_eq!(XcmRequests::<Test>::get(query_id), None);
    })
}

#[test]
fn test_storage_proof_approach_should_work() {
    // block_hash on Kusama
    // 0x5a5bc2c15e160df11a7468cb91aca2f6b9db8faa87354099674e955e180b8ee2
    let relay_root = sp_core::hash::H256::from_slice(
        &hex::decode("6f5c11cf6bfe2721697af3cecd0a6c5e5a0a6e1bf0671dfd5b68abd433f09764").unwrap(),
    );

    // Get proof_bytes
    // await api.rpc.state.getReadProof(["0x5f3e4907f716ac89b6347d15ececedca422adb579f1dbf4f3886c5cfa3bb8cc405aae5fc2c15c1fd7a2b6d9562c689875d199b535508990c59f411757617904ce65c905fced6878bacfbf26d3b4a1e97"],"0x5a5bc2c15e160df11a7468cb91aca2f6b9db8faa87354099674e955e180b8ee2");
    let proof_bytes = [
        hex::decode("800c02809497c8d51db26995948a33a004fca43442ba654fdedb15f5e123059896c634b080ef30f3b11273df6aa7dd4c3cf02c1249822edf25b66854fb98fb269ad0dd066280cd249b8479cf1cc37f8a0fe68fcb610f75503ee68180134814020c92c43ffcb8").unwrap(),
        hex::decode("7f1de5fc2c15c1fd7a2b6d9562c689875d199b535508990c59f411757617904ce65c905fced6878bacfbf26d3b4a1e970d065d199b535508990c59f411757617904ce65c905fced6878bacfbf26d3b4a1e970f1157e968fea1010f1157e968fea101005101310d0000320d0000330d0000340d0000350d0000360d0000370d0000380d0000390d00003a0d00003b0d00003c0d00003d0d00003e0d00003f0d0000400d0000410d0000420d0000430d0000440d0000450d0000460d0000470d0000480d0000490d00004a0d00004b0d00004c0d00004d0d00004e0d00004f0d0000500d0000510d0000520d0000530d0000540d0000550d0000560d0000570d0000580d0000590d00005a0d00005b0d00005c0d00005d0d00005e0d00005f0d0000600d0000610d0000620d0000630d0000640d0000650d0000660d0000670d0000680d0000690d00006a0d00006b0d00006c0d00006d0d00006e0d00006f0d0000700d0000710d0000720d0000730d0000740d0000750d0000760d0000770d0000780d0000790d00007a0d00007b0d00007c0d00007d0d00007e0d00007f0d0000800d0000810d0000820d0000830d0000840d0000").unwrap(),
        hex::decode("8004418026367743456425703e1ce51c51ba1742a59c02411b89b06196477363461e23de785e7df464e44a534ba6b0cbb32407b58734a40d0000019933f2aa7f0100004c5e7b9012096b41c4eb3aaf947f6ea429080000").unwrap(),
        hex::decode("80ffff80232944b9808759b768ba9bbee33bcecc6bb678a845cee7cade527ab250dffee380590491ed9db2709fccd5896bd45755ff6fdaaab133b04af77c1579279e4b68e180005623d575e3acef57307bf4482475f56f6ee5a86c0b1a7163b6b547ea2b893080594ddb56a8bce7f68a3629232fc9e6216126541b96c355d087f4c692bac8de98806a805dd205ac73d747f2432e85c49a151081bbdad5704ea2c92316b09507a0fd8000dffd90f405477faa114057decbb3393717f023f7a854d77b13e1a4b7bbd0c78038a7486fe59f2289b44702b92c2c10e0a098ed325fa9259bc123f6a2e0a52e5180aa131c6196a12fa526137db11fcf1ba5d06c4cf909e720bd9946e830bd41e3cb806bdef693878cb5be73af25d494153e00a8a8507b6152f52ef908b8c29d4a543d80706d1ca2892e685610e4ed6ccd7565a6efe9f76e25c99077b4b831fd11144caa80adc203db51d5ddb3a87e24a416f842946d574a4fcb8893e5f4b898548e8e3d6080b25987b0de9667f44fa319ef2886eabddc49e057c58bf6a66f2efe8db6fbccac804b48b1cee9ba183e6b8414092a41f92f095474465e13bd59e887121e7bf4de8580ee3e1870838e81990e8dd2348a8d1f4cc01507a9e33bc0099b3c5ffedb7ffdd780a3d43738dca1ff59f4ed0ab18099c610544dbb81139c950aace1a39793f626b3801b88e87aec4d02ac33d60c2e2b10dd29c9b9c735b51a25d70a91961b8f300dd6").unwrap(),
        hex::decode("80f7ff80038713baa7e6bb357c33e159ebb93fcf652aa7ce34aa7ad2adbe593aff2e918a80daead2ede3b3393205a686158d62003f19351ea300356e545ae80a31f7ae8f9780238fd143e6c5dc2129e07f85a027269e030242202437d1c869c81fa42fd7bd79806bac257abc7bfe0a91f65b048e201e8ea1de778e5df0e4d9d7de6babdf8e441a8008acfe2a42dd0ae9b9d7726c9cc2c954d4a53d7d5dc543630026b2bd809aafd7805f852336bb8f25b5e9729223f27b4c380f538082ded97c54ea735d46245ddd11804b86acd2e1972801654ddfe795e20ca59d48d69d64ef32ae6bad6af8b9770fee80184b10002d45521f502b344e06793bf3cf77934d58588a024d9dbd8041b015f7802717ad2e00b2bcfea59a0179e69072f05fc65884d1c0b42cd29ce34d17b2062d80888bbb1ca4a539482087f2ed338227526bf9f518bf4dd67130469a264b95e69e807a698f5008c02bc7dcbf2ea0e974914b56924d5440fef85179b2a08f1cb73add80ff5db327160b6d5edbbf01c548f5841f8325ca55954c99a54e22aa9ea39cba528042194d5944a25c37885e422c647092b97a3ae70e1ad247f0292164da7862303f80bd7fa002fc6cc9960365833889016a48d7ef9e32df363b59d76cfb39b9f53d1a800f6e6ea64b62457b0e9ddacbe978bd79cd6fa1dbb422eec5a223f49ae2bb85bc").unwrap(),
        hex::decode("80ffff80a9ed75e03215b907b35e5fb98346cac4b9ca9a19a182d6ccdba36bada8fef05b80a972ea9a27023f0935df491962b994e6f6c4ee3a7805cf27aab8611d2f06ed6b8060b390cbb48376bb848cad68fb6fcff10f687b7045088dfc5d0699a0db15b4ea804313dfe81d8f811456223a5fc72cd1be994e51218e72dfca4f7cd9331ec247eb802515e2ae8b12222edb90600fb147c7459a7c265c753619c9014cc5aecd04ed3a807ce6a9911f470062b2772fee859f7b9e4739c0867a3746b5d0b5dcc324f6a1ad806538ed1ff3452f5a690624e1541b0466ca3f452ad60b70e21c27b1dec3d1be1c80136ac180b31bc0e4f3b798262f5eb618d82aca5c85a0d0e2620324e23d288bd8809594c690f7fbc4db8db681f5da18df3226897b17c20fd8002897d2bcc840e135800bac1a3ccd071e8c6b04c6b96e16cc002286da0b0cadbc79d5cacb2ce7b2b44a803981f53505969f281ce224234791c8db7e7d0377ceb1d24ce29fdba27a2aa55c809824f0eed4dd3fddc2aa20c9c9b0199278beaee525fa2ddc6c8cf7e23f9d59c480a8c945b098ebc6fdb5b073147642d4deef487b016c8b27054a00280c638384a6802f3b98d55b2ad4fcea5c405dc5d343afecb007c057b8a60ae4100545042aa729805e56350f28813b295c870c0003b0130cbe66f5d11881387114003564ba8c78578065e8ef797423da01cee91b0b790e6a9c81b5504ea2786be972d5c961447dd4d2").unwrap(),
        hex::decode("800180807c2c31b3747d7767c40b41f0e75d60d8045f32dfa2ce397b25b641205c77e57580c1df2a29725bf95921b5f4cee86c913b58118f6dfc59c9822b16712dcbf84122").unwrap(),
        hex::decode("80040280c56aedca768a4e919df533325b6fc6a368455fb599d7ad9271a2249e49e65c82803560432335d6e87f5c6c0da59797aee0423cecc207c3a37304511c3a8d031455").unwrap(),
        hex::decode("9d0e4907f716ac89b6347d15ececedcaffff585f0b6a45321efae92aea15e0740ec7afe710a40d0000585f038e71612491192d68deab7e6f563fe110e8030000806ec80d7afc89a3ccebe10fcdf04ca16518ff88d6b29462ec861b00d31239db3580c04da45c5e5cddac29c9e21d75662d40adfabd88bf3010dda8cf5a13f22e32d980f981a50341cdae9bd105be756196698e91d8be2fb0dc6397a0aaf82cddf0fc5d80f42d10621daacf0ea0e3268a8bc2fe3121d9e250a88b3024ed7c56cba35b7bb980bfa6614065f0fadaa9d7f71840a3d29f63a189f2f3aef8241fe5b06b3b9f0cd180c7da650afb99c538063c05d6c02504e4d6555ad682412479d189e85e994fd6cd80a3aaade6ba17d9833da300f436bc3985fb2fe6c61670fea253d5d58d5edd5bfd80f11d7ff327f21936e99db2f172c95d464b81c557d6662f46a0991d01f5683be7803eb67870b1c418c6c7a26103aa386a17ecda2e481f97c8ff37f390b98e4bd6bc585f049a2738eeb30896aacb8b3fb46471bd100400000080009dacd73537f8a2f5933ded58c0102b9f9af9d0379079b8f6b1f63027e77ef9585f0642c00af119adf30dc11d32e9f0886d10204e0000809bee97f441e7fe63916b3923c1279e7f401558ab1df7ffb4b51e864e820b48e1585f099b25852d3d69419882da651375cdb310861c0000").unwrap(),
        hex::decode("9d0adb579f1dbf4f3886c5cfa3bb8cc4ffff80a7b28f89318d1fd3eb7a23c78ba24134b89cdd332efec5615cf2139e87777b0c8020766326bb8cfa97296157a7cc3d502dcfc606fc21c823ff98f5dc6062559d3380f46475b1c59ce606753e77b74d1448ffab5a35b853a6d6f49795b027019325f380c2e52883c83b9b71a2ad00a0500eab314fc3c22a3af221075e5f57ebbe53b02180bab606d8bc597e34133306bf3cf8b6da31d745053bdc02a9352f3910562f368f801f8d5f3e91bc4195b0f2ce3c8f762d92288f099a3cff19544383cf5ee2b138f680f88564d53bfc2e0a524bf7d288f6736b5e6d5eb8ccb9dd253fed49954775070e80f67ae62ea2d122fd11ad9af0db42d7a087062679856b83f7f1fec3ff07d9a95d8082260fc9c9d39d941f8647142b6fa9c59c7cbbd1a62713dede381d011a7f1535807eaca35e2da21f58d204ee089e4e0a758a03fa8352f421c80dac4b932e78e53080a922540b71ca78d22691f4d5b35de64e2cc86c1016fbf964a883e5be2aa2718c802dc3aebb89be3c7f47f75b0e1af2ca818953f8001715b9ad50ef62d64831d176807d725bdd58f12b77f22953139d49cd785292a6576aaff12d7204fc42dfbd363780c0fbc5bfc21820c07c09ab52a7f7b7d113062c861dc772bb8fd480a48eac90fd80a753133ae3c9ea1ec743a363e1908f10511a7457cecddd836c3ebcc9d272b09a800174ba5e2235e5f210c26e1ca3ddd7b98b5a375626be5c54307597bbcfe70514").unwrap(),
        hex::decode("804c148041384dfbc07ad3401c8d4464793fa06360e54d7c21358998cc3c4cf848899145802bbf85831deea0e456e8061b4112ad00ffea89e53b6f8d0a4ea1b7ff4b83ff4b80fb786dead294471fc3c0f868831403b0ca753840d73cf92d4e52c6b9090b1ee080399b822502d71b5bdc3fe2de3f1773269f3c259ce65d89d008392bf394395302807927cf5a1a4859edd77a2be0cd98010622e7ccff6b5e8281667f60016d8f68ca").unwrap(),
    ];

    let key = hex::decode("5f3e4907f716ac89b6347d15ececedca422adb579f1dbf4f3886c5cfa3bb8cc405aae5fc2c15c1fd7a2b6d9562c689875d199b535508990c59f411757617904ce65c905fced6878bacfbf26d3b4a1e97").unwrap();
    let value = hex::decode("5d199b535508990c59f411757617904ce65c905fced6878bacfbf26d3b4a1e970f1157e968fea1010f1157e968fea101005101310d0000320d0000330d0000340d0000350d0000360d0000370d0000380d0000390d00003a0d00003b0d00003c0d00003d0d00003e0d00003f0d0000400d0000410d0000420d0000430d0000440d0000450d0000460d0000470d0000480d0000490d00004a0d00004b0d00004c0d00004d0d00004e0d00004f0d0000500d0000510d0000520d0000530d0000540d0000550d0000560d0000570d0000580d0000590d00005a0d00005b0d00005c0d00005d0d00005e0d00005f0d0000600d0000610d0000620d0000630d0000640d0000650d0000660d0000670d0000680d0000690d00006a0d00006b0d00006c0d00006d0d00006e0d00006f0d0000700d0000710d0000720d0000730d0000740d0000750d0000760d0000770d0000780d0000790d00007a0d00007b0d00007c0d00007d0d00007e0d00007f0d0000800d0000810d0000820d0000830d0000840d0000").unwrap();
    let relay_proof = StorageProof::new(proof_bytes.to_vec());
    let result = sp_state_machine::read_proof_check::<BlakeTwo256, _>(
        relay_root,
        relay_proof.clone(),
        [key.clone()],
    )
    .unwrap();
    assert_eq!(
        result.into_iter().collect::<Vec<_>>(),
        vec![(key, Some(value))],
    );
}

#[test]
fn test1() {
    use codec::Encode;
    // block_hash on Kusama
    // 0x5a5bc2c15e160df11a7468cb91aca2f6b9db8faa87354099674e955e180b8ee2
    let _relay_root = sp_core::hash::H256::from_slice(
        &hex::decode("6f5c11cf6bfe2721697af3cecd0a6c5e5a0a6e1bf0671dfd5b68abd433f09764").unwrap(),
    );

    let data = "5d199b535508990c59f411757617904ce65c905fced6878bacfbf26d3b4a1e970f1157e968fea1010f1157e968fea101005101310d0000320d0000330d0000340d0000350d0000360d0000370d0000380d0000390d00003a0d00003b0d00003c0d00003d0d00003e0d00003f0d0000400d0000410d0000420d0000430d0000440d0000450d0000460d0000470d0000480d0000490d00004a0d00004b0d00004c0d00004d0d00004e0d00004f0d0000500d0000510d0000520d0000530d0000540d0000550d0000560d0000570d0000580d0000590d00005a0d00005b0d00005c0d00005d0d00005e0d00005f0d0000600d0000610d0000620d0000630d0000640d0000650d0000660d0000670d0000680d0000690d00006a0d00006b0d00006c0d00006d0d00006e0d00006f0d0000700d0000710d0000720d0000730d0000740d0000750d0000760d0000770d0000780d0000790d00007a0d00007b0d00007c0d00007d0d00007e0d00007f0d0000800d0000810d0000820d0000830d0000840d0000";
    let _key = hex::decode("5f3e4907f716ac89b6347d15ececedca422adb579f1dbf4f3886c5cfa3bb8cc405aae5fc2c15c1fd7a2b6d9562c689875d199b535508990c59f411757617904ce65c905fced6878bacfbf26d3b4a1e97").unwrap();
    let _value = hex::decode(data).unwrap();

    // Get proof_bytes
    // await api.rpc.state.getReadProof(["0x5f3e4907f716ac89b6347d15ececedca422adb579f1dbf4f3886c5cfa3bb8cc405aae5fc2c15c1fd7a2b6d9562c689875d199b535508990c59f411757617904ce65c905fced6878bacfbf26d3b4a1e97"],"0x5a5bc2c15e160df11a7468cb91aca2f6b9db8faa87354099674e955e180b8ee2");
    let proof_bytes = [
        hex::decode("800c02809497c8d51db26995948a33a004fca43442ba654fdedb15f5e123059896c634b080ef30f3b11273df6aa7dd4c3cf02c1249822edf25b66854fb98fb269ad0dd066280cd249b8479cf1cc37f8a0fe68fcb610f75503ee68180134814020c92c43ffcb8").unwrap(),
        hex::decode("7f1de5fc2c15c1fd7a2b6d9562c689875d199b535508990c59f411757617904ce65c905fced6878bacfbf26d3b4a1e970d065d199b535508990c59f411757617904ce65c905fced6878bacfbf26d3b4a1e970f1157e968fea1010f1157e968fea101005101310d0000320d0000330d0000340d0000350d0000360d0000370d0000380d0000390d00003a0d00003b0d00003c0d00003d0d00003e0d00003f0d0000400d0000410d0000420d0000430d0000440d0000450d0000460d0000470d0000480d0000490d00004a0d00004b0d00004c0d00004d0d00004e0d00004f0d0000500d0000510d0000520d0000530d0000540d0000550d0000560d0000570d0000580d0000590d00005a0d00005b0d00005c0d00005d0d00005e0d00005f0d0000600d0000610d0000620d0000630d0000640d0000650d0000660d0000670d0000680d0000690d00006a0d00006b0d00006c0d00006d0d00006e0d00006f0d0000700d0000710d0000720d0000730d0000740d0000750d0000760d0000770d0000780d0000790d00007a0d00007b0d00007c0d00007d0d00007e0d00007f0d0000800d0000810d0000820d0000830d0000840d0000").unwrap(),
        hex::decode("8004418026367743456425703e1ce51c51ba1742a59c02411b89b06196477363461e23de785e7df464e44a534ba6b0cbb32407b58734a40d0000019933f2aa7f0100004c5e7b9012096b41c4eb3aaf947f6ea429080000").unwrap(),
        hex::decode("80ffff80232944b9808759b768ba9bbee33bcecc6bb678a845cee7cade527ab250dffee380590491ed9db2709fccd5896bd45755ff6fdaaab133b04af77c1579279e4b68e180005623d575e3acef57307bf4482475f56f6ee5a86c0b1a7163b6b547ea2b893080594ddb56a8bce7f68a3629232fc9e6216126541b96c355d087f4c692bac8de98806a805dd205ac73d747f2432e85c49a151081bbdad5704ea2c92316b09507a0fd8000dffd90f405477faa114057decbb3393717f023f7a854d77b13e1a4b7bbd0c78038a7486fe59f2289b44702b92c2c10e0a098ed325fa9259bc123f6a2e0a52e5180aa131c6196a12fa526137db11fcf1ba5d06c4cf909e720bd9946e830bd41e3cb806bdef693878cb5be73af25d494153e00a8a8507b6152f52ef908b8c29d4a543d80706d1ca2892e685610e4ed6ccd7565a6efe9f76e25c99077b4b831fd11144caa80adc203db51d5ddb3a87e24a416f842946d574a4fcb8893e5f4b898548e8e3d6080b25987b0de9667f44fa319ef2886eabddc49e057c58bf6a66f2efe8db6fbccac804b48b1cee9ba183e6b8414092a41f92f095474465e13bd59e887121e7bf4de8580ee3e1870838e81990e8dd2348a8d1f4cc01507a9e33bc0099b3c5ffedb7ffdd780a3d43738dca1ff59f4ed0ab18099c610544dbb81139c950aace1a39793f626b3801b88e87aec4d02ac33d60c2e2b10dd29c9b9c735b51a25d70a91961b8f300dd6").unwrap(),
        hex::decode("80f7ff80038713baa7e6bb357c33e159ebb93fcf652aa7ce34aa7ad2adbe593aff2e918a80daead2ede3b3393205a686158d62003f19351ea300356e545ae80a31f7ae8f9780238fd143e6c5dc2129e07f85a027269e030242202437d1c869c81fa42fd7bd79806bac257abc7bfe0a91f65b048e201e8ea1de778e5df0e4d9d7de6babdf8e441a8008acfe2a42dd0ae9b9d7726c9cc2c954d4a53d7d5dc543630026b2bd809aafd7805f852336bb8f25b5e9729223f27b4c380f538082ded97c54ea735d46245ddd11804b86acd2e1972801654ddfe795e20ca59d48d69d64ef32ae6bad6af8b9770fee80184b10002d45521f502b344e06793bf3cf77934d58588a024d9dbd8041b015f7802717ad2e00b2bcfea59a0179e69072f05fc65884d1c0b42cd29ce34d17b2062d80888bbb1ca4a539482087f2ed338227526bf9f518bf4dd67130469a264b95e69e807a698f5008c02bc7dcbf2ea0e974914b56924d5440fef85179b2a08f1cb73add80ff5db327160b6d5edbbf01c548f5841f8325ca55954c99a54e22aa9ea39cba528042194d5944a25c37885e422c647092b97a3ae70e1ad247f0292164da7862303f80bd7fa002fc6cc9960365833889016a48d7ef9e32df363b59d76cfb39b9f53d1a800f6e6ea64b62457b0e9ddacbe978bd79cd6fa1dbb422eec5a223f49ae2bb85bc").unwrap(),
        hex::decode("80ffff80a9ed75e03215b907b35e5fb98346cac4b9ca9a19a182d6ccdba36bada8fef05b80a972ea9a27023f0935df491962b994e6f6c4ee3a7805cf27aab8611d2f06ed6b8060b390cbb48376bb848cad68fb6fcff10f687b7045088dfc5d0699a0db15b4ea804313dfe81d8f811456223a5fc72cd1be994e51218e72dfca4f7cd9331ec247eb802515e2ae8b12222edb90600fb147c7459a7c265c753619c9014cc5aecd04ed3a807ce6a9911f470062b2772fee859f7b9e4739c0867a3746b5d0b5dcc324f6a1ad806538ed1ff3452f5a690624e1541b0466ca3f452ad60b70e21c27b1dec3d1be1c80136ac180b31bc0e4f3b798262f5eb618d82aca5c85a0d0e2620324e23d288bd8809594c690f7fbc4db8db681f5da18df3226897b17c20fd8002897d2bcc840e135800bac1a3ccd071e8c6b04c6b96e16cc002286da0b0cadbc79d5cacb2ce7b2b44a803981f53505969f281ce224234791c8db7e7d0377ceb1d24ce29fdba27a2aa55c809824f0eed4dd3fddc2aa20c9c9b0199278beaee525fa2ddc6c8cf7e23f9d59c480a8c945b098ebc6fdb5b073147642d4deef487b016c8b27054a00280c638384a6802f3b98d55b2ad4fcea5c405dc5d343afecb007c057b8a60ae4100545042aa729805e56350f28813b295c870c0003b0130cbe66f5d11881387114003564ba8c78578065e8ef797423da01cee91b0b790e6a9c81b5504ea2786be972d5c961447dd4d2").unwrap(),
        hex::decode("800180807c2c31b3747d7767c40b41f0e75d60d8045f32dfa2ce397b25b641205c77e57580c1df2a29725bf95921b5f4cee86c913b58118f6dfc59c9822b16712dcbf84122").unwrap(),
        hex::decode("80040280c56aedca768a4e919df533325b6fc6a368455fb599d7ad9271a2249e49e65c82803560432335d6e87f5c6c0da59797aee0423cecc207c3a37304511c3a8d031455").unwrap(),
        hex::decode("9d0e4907f716ac89b6347d15ececedcaffff585f0b6a45321efae92aea15e0740ec7afe710a40d0000585f038e71612491192d68deab7e6f563fe110e8030000806ec80d7afc89a3ccebe10fcdf04ca16518ff88d6b29462ec861b00d31239db3580c04da45c5e5cddac29c9e21d75662d40adfabd88bf3010dda8cf5a13f22e32d980f981a50341cdae9bd105be756196698e91d8be2fb0dc6397a0aaf82cddf0fc5d80f42d10621daacf0ea0e3268a8bc2fe3121d9e250a88b3024ed7c56cba35b7bb980bfa6614065f0fadaa9d7f71840a3d29f63a189f2f3aef8241fe5b06b3b9f0cd180c7da650afb99c538063c05d6c02504e4d6555ad682412479d189e85e994fd6cd80a3aaade6ba17d9833da300f436bc3985fb2fe6c61670fea253d5d58d5edd5bfd80f11d7ff327f21936e99db2f172c95d464b81c557d6662f46a0991d01f5683be7803eb67870b1c418c6c7a26103aa386a17ecda2e481f97c8ff37f390b98e4bd6bc585f049a2738eeb30896aacb8b3fb46471bd100400000080009dacd73537f8a2f5933ded58c0102b9f9af9d0379079b8f6b1f63027e77ef9585f0642c00af119adf30dc11d32e9f0886d10204e0000809bee97f441e7fe63916b3923c1279e7f401558ab1df7ffb4b51e864e820b48e1585f099b25852d3d69419882da651375cdb310861c0000").unwrap(),
        hex::decode("9d0adb579f1dbf4f3886c5cfa3bb8cc4ffff80a7b28f89318d1fd3eb7a23c78ba24134b89cdd332efec5615cf2139e87777b0c8020766326bb8cfa97296157a7cc3d502dcfc606fc21c823ff98f5dc6062559d3380f46475b1c59ce606753e77b74d1448ffab5a35b853a6d6f49795b027019325f380c2e52883c83b9b71a2ad00a0500eab314fc3c22a3af221075e5f57ebbe53b02180bab606d8bc597e34133306bf3cf8b6da31d745053bdc02a9352f3910562f368f801f8d5f3e91bc4195b0f2ce3c8f762d92288f099a3cff19544383cf5ee2b138f680f88564d53bfc2e0a524bf7d288f6736b5e6d5eb8ccb9dd253fed49954775070e80f67ae62ea2d122fd11ad9af0db42d7a087062679856b83f7f1fec3ff07d9a95d8082260fc9c9d39d941f8647142b6fa9c59c7cbbd1a62713dede381d011a7f1535807eaca35e2da21f58d204ee089e4e0a758a03fa8352f421c80dac4b932e78e53080a922540b71ca78d22691f4d5b35de64e2cc86c1016fbf964a883e5be2aa2718c802dc3aebb89be3c7f47f75b0e1af2ca818953f8001715b9ad50ef62d64831d176807d725bdd58f12b77f22953139d49cd785292a6576aaff12d7204fc42dfbd363780c0fbc5bfc21820c07c09ab52a7f7b7d113062c861dc772bb8fd480a48eac90fd80a753133ae3c9ea1ec743a363e1908f10511a7457cecddd836c3ebcc9d272b09a800174ba5e2235e5f210c26e1ca3ddd7b98b5a375626be5c54307597bbcfe70514").unwrap(),
        hex::decode("804c148041384dfbc07ad3401c8d4464793fa06360e54d7c21358998cc3c4cf848899145802bbf85831deea0e456e8061b4112ad00ffea89e53b6f8d0a4ea1b7ff4b83ff4b80fb786dead294471fc3c0f868831403b0ca753840d73cf92d4e52c6b9090b1ee080399b822502d71b5bdc3fe2de3f1773269f3c259ce65d89d008392bf394395302807927cf5a1a4859edd77a2be0cd98010622e7ccff6b5e8281667f60016d8f68ca").unwrap(),
    ];

    let derivative_index = <Test as Config>::DerivativeIndex::get();
    let mut staking_ledger = <StakingLedger<AccountId, BalanceOf<Test>>>::new(
        LiquidStaking::derivative_sovereign_account_id(derivative_index),
        459589030598417,
    );
    staking_ledger.claimed_rewards = vec![
        3377, 3378, 3379, 3380, 3381, 3382, 3383, 3384, 3385, 3386, 3387, 3388, 3389, 3390, 3391,
        3392, 3393, 3394, 3395, 3396, 3397, 3398, 3399, 3400, 3401, 3402, 3403, 3404, 3405, 3406,
        3407, 3408, 3409, 3410, 3411, 3412, 3413, 3414, 3415, 3416, 3417, 3418, 3419, 3420, 3421,
        3422, 3423, 3424, 3425, 3426, 3427, 3428, 3429, 3430, 3431, 3432, 3433, 3434, 3435, 3436,
        3437, 3438, 3439, 3440, 3441, 3442, 3443, 3444, 3445, 3446, 3447, 3448, 3449, 3450, 3451,
        3452, 3453, 3454, 3455, 3456, 3457, 3458, 3459, 3460,
    ];
    assert_eq!(hex::encode(&staking_ledger.encode()), data);
    assert!(LiquidStaking::verify_merkle_proof(
        derivative_index,
        staking_ledger,
        proof_bytes.to_vec()
    ));
}
