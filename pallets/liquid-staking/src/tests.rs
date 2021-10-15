use crate::{
    mock::*,
    types::{MatchingLedger, RewardDestination, StakingSettlementKind},
    *,
};
use frame_support::{assert_err, assert_ok, traits::Hooks};
use pallet_staking::{Exposure, IndividualExposure};
use primitives::{
    tokens::{DOT, XDOT},
    Balance, Rate,
};
use sp_runtime::traits::{AccountIdLookup, One, StaticLookup};
use xcm::latest::prelude::ExecuteXcm;
use xcm_simulator::TestExt;

use crate::types::WestendCall as RelaychainCall;
use codec::Encode;
use types::*;
#[test]
fn stake_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(LiquidStaking::stake(Origin::signed(ALICE), 10));
        // Check storage is correct
        assert_eq!(ExchangeRate::<Test>::get(), Rate::one());
        assert_eq!(StakingPool::<Test>::get(), 10);
        assert_eq!(
            MatchingPool::<Test>::get(),
            MatchingLedger {
                total_stake_amount: 10,
                total_unstake_amount: 0,
            }
        );

        // Check balance is correct
        assert_eq!(<Test as Config>::Assets::balance(DOT, &ALICE), 90);
        assert_eq!(<Test as Config>::Assets::balance(XDOT, &ALICE), 110);
        assert_eq!(
            <Test as Config>::Assets::balance(DOT, &LiquidStaking::account_id()),
            10
        );
    })
}

#[test]
fn unstake_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(LiquidStaking::stake(Origin::signed(ALICE), 10));
        assert_ok!(LiquidStaking::unstake(Origin::signed(ALICE), 6));

        // Check storage is correct
        assert_eq!(ExchangeRate::<Test>::get(), Rate::one());
        assert_eq!(StakingPool::<Test>::get(), 4);
        assert_eq!(
            MatchingPool::<Test>::get(),
            MatchingLedger {
                total_stake_amount: 10,
                total_unstake_amount: 6,
            }
        );

        // Check balance is correct
        assert_eq!(<Test as Config>::Assets::balance(DOT, &ALICE), 96);
        assert_eq!(<Test as Config>::Assets::balance(XDOT, &ALICE), 104);
        assert_eq!(
            <Test as Config>::Assets::balance(DOT, &LiquidStaking::account_id()),
            4
        );
    })
}

#[test]
fn test_record_staking_settlement_ok() {
    new_test_ext().execute_with(|| {
        assert_ok!(LiquidStaking::record_staking_settlement(
            Origin::signed(ALICE),
            1,
            100,
            StakingSettlementKind::Reward
        ));

        assert_eq!(LiquidStaking::exchange_rate(), Rate::from(1));
    })
}

#[test]
fn test_duplicated_record_staking_settlement() {
    new_test_ext().execute_with(|| {
        LiquidStaking::record_staking_settlement(
            Origin::signed(ALICE),
            1,
            100,
            StakingSettlementKind::Reward,
        )
        .unwrap();

        assert_err!(
            LiquidStaking::record_staking_settlement(
                Origin::signed(ALICE),
                1,
                100,
                StakingSettlementKind::Reward
            ),
            Error::<Test>::StakingSettlementAlreadyRecorded
        )
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
fn test_settlement_should_work() {
    use StakeOp::*;
    TestNet::reset();
    ParaA::execute_with(|| {
        let test_case: Vec<(Vec<StakeOp>, Balance, (Balance, Balance, Balance), Balance)> = vec![
            (vec![Stake(3000), Unstake(500)], 0, (2485, 0, 0), 0),
            // Calculate right here.
            (vec![Unstake(10), Unstake(5), Stake(10)], 0, (0, 0, 5), 10),
            (vec![], 0, (0, 0, 0), 0),
        ];

        for (stake_ops, unbonding_amount, matching_result, _pallet_balance) in test_case.into_iter()
        {
            stake_ops.into_iter().for_each(StakeOp::execute);
            assert_eq!(
                LiquidStaking::matching_pool().matching(unbonding_amount),
                matching_result
            );
            assert_ok!(LiquidStaking::settlement(
                Origin::signed(ALICE),
                0,
                unbonding_amount,
                0
            ));
            Pallet::<Test>::on_idle(0, 10000);
        }
    });
    Relay::execute_with(|| {
        assert_eq!(
            RelayBalances::free_balance(&LiquidStaking::para_account_id()),
            // FIXME: weight should be take into account
            9999983330792000
        );
    });
}

#[test]
fn test_transact_bond_work() {
    TestNet::reset();

    ParaA::execute_with(|| {
        assert_ok!(LiquidStaking::bond(
            Origin::signed(ALICE),
            3 * DOT_DECIMAL,
            RewardDestination::Staked
        ));

        ParaSystem::assert_has_event(mock::Event::LiquidStaking(crate::Event::BondCallSent(
            LiquidStaking::derivative_para_account_id(),
            3 * DOT_DECIMAL,
            RewardDestination::Staked,
        )));
    });

    Relay::execute_with(|| {
        RelaySystem::assert_has_event(RelayEvent::Staking(RelayStakingEvent::Bonded(
            LiquidStaking::derivative_para_account_id(),
            3 * DOT_DECIMAL,
        )));
        let ledger = RelayStaking::ledger(LiquidStaking::derivative_para_account_id()).unwrap();
        assert_eq!(ledger.total, 3 * DOT_DECIMAL);
    });
}

#[test]
fn test_transact_bond_extra_work() {
    TestNet::reset();

    ParaA::execute_with(|| {
        assert_ok!(LiquidStaking::bond(
            Origin::signed(ALICE),
            2 * DOT_DECIMAL,
            RewardDestination::Staked
        ));

        assert_ok!(LiquidStaking::bond_extra(
            Origin::signed(ALICE),
            3 * DOT_DECIMAL
        ));
    });

    Relay::execute_with(|| {
        let ledger = RelayStaking::ledger(LiquidStaking::derivative_para_account_id()).unwrap();
        assert_eq!(ledger.total, 5 * DOT_DECIMAL);
    });
}

#[test]
fn test_transact_unbond_work() {
    TestNet::reset();

    ParaA::execute_with(|| {
        assert_ok!(LiquidStaking::bond(
            Origin::signed(ALICE),
            5 * DOT_DECIMAL,
            RewardDestination::Staked
        ));
        assert_ok!(LiquidStaking::unbond(
            Origin::signed(ALICE),
            2 * DOT_DECIMAL
        ));
    });

    Relay::execute_with(|| {
        RelaySystem::assert_has_event(RelayEvent::Staking(RelayStakingEvent::Bonded(
            LiquidStaking::derivative_para_account_id(),
            5 * DOT_DECIMAL,
        )));
        RelaySystem::assert_has_event(RelayEvent::Staking(RelayStakingEvent::Unbonded(
            LiquidStaking::derivative_para_account_id(),
            2 * DOT_DECIMAL,
        )));
        let ledger = RelayStaking::ledger(LiquidStaking::derivative_para_account_id()).unwrap();
        assert_eq!(ledger.total, 5 * DOT_DECIMAL);
        assert_eq!(ledger.active, 3 * DOT_DECIMAL);
    });
}

#[test]
fn test_transact_withdraw_unbonded_work() {
    TestNet::reset();

    ParaA::execute_with(|| {
        assert_ok!(LiquidStaking::bond(
            Origin::signed(ALICE),
            5 * DOT_DECIMAL,
            RewardDestination::Staked
        ));
        assert_ok!(LiquidStaking::unbond(
            Origin::signed(ALICE),
            2 * DOT_DECIMAL
        ));
    });

    Relay::execute_with(|| {
        let ledger = RelayStaking::ledger(LiquidStaking::derivative_para_account_id()).unwrap();
        assert_eq!(ledger.total, 5 * DOT_DECIMAL);
        assert_eq!(ledger.active, 3 * DOT_DECIMAL);
        assert_eq!(ledger.unlocking.len(), 1);

        RelaySystem::assert_has_event(RelayEvent::Staking(RelayStakingEvent::Bonded(
            LiquidStaking::derivative_para_account_id(),
            5 * DOT_DECIMAL,
        )));
        RelaySystem::assert_has_event(RelayEvent::Staking(RelayStakingEvent::Unbonded(
            LiquidStaking::derivative_para_account_id(),
            2 * DOT_DECIMAL,
        )));

        pallet_staking::CurrentEra::<WestendRuntime>::put(
            <WestendRuntime as pallet_staking::Config>::BondingDuration::get(),
        );
    });

    ParaA::execute_with(|| {
        assert_ok!(LiquidStaking::withdraw_unbonded(
            Origin::signed(ALICE),
            0,
            0
        ));
    });

    Relay::execute_with(|| {
        let ledger = RelayStaking::ledger(LiquidStaking::derivative_para_account_id()).unwrap();
        assert_eq!(ledger.total, 3 * DOT_DECIMAL);
        assert_eq!(ledger.active, 3 * DOT_DECIMAL);
        assert_eq!(ledger.unlocking.len(), 0);
    });
}

#[test]
fn test_transact_rebond_work() {
    TestNet::reset();

    ParaA::execute_with(|| {
        assert_ok!(LiquidStaking::bond(
            Origin::signed(ALICE),
            10 * DOT_DECIMAL,
            RewardDestination::Staked
        ));
        assert_ok!(LiquidStaking::unbond(
            Origin::signed(ALICE),
            5 * DOT_DECIMAL
        ));
        assert_ok!(LiquidStaking::rebond(
            Origin::signed(ALICE),
            3 * DOT_DECIMAL
        ));
    });

    Relay::execute_with(|| {
        RelaySystem::assert_has_event(RelayEvent::Staking(RelayStakingEvent::Bonded(
            LiquidStaking::derivative_para_account_id(),
            10 * DOT_DECIMAL,
        )));
        RelaySystem::assert_has_event(RelayEvent::Staking(RelayStakingEvent::Unbonded(
            LiquidStaking::derivative_para_account_id(),
            5 * DOT_DECIMAL,
        )));
        RelaySystem::assert_has_event(RelayEvent::Staking(RelayStakingEvent::Bonded(
            LiquidStaking::derivative_para_account_id(),
            3 * DOT_DECIMAL,
        )));
        let ledger = RelayStaking::ledger(LiquidStaking::derivative_para_account_id()).unwrap();
        assert_eq!(ledger.total, 10 * DOT_DECIMAL);
        assert_eq!(ledger.active, 8 * DOT_DECIMAL);
    });
}

#[test]
fn test_transact_nominate_work() {
    TestNet::reset();

    ParaA::execute_with(|| {
        assert_ok!(LiquidStaking::bond(
            Origin::signed(ALICE),
            10 * DOT_DECIMAL,
            RewardDestination::Staked
        ));

        assert_ok!(LiquidStaking::nominate(
            Origin::signed(ALICE),
            vec![ALICE, BOB],
        ));
    });

    Relay::execute_with(|| {
        let ledger = RelayStaking::ledger(LiquidStaking::derivative_para_account_id()).unwrap();
        assert_eq!(ledger.total, 10 * DOT_DECIMAL);
        let nominators =
            RelayStaking::nominators(LiquidStaking::derivative_para_account_id()).unwrap();
        assert_eq!(nominators.targets, vec![ALICE, BOB]);
    });
}

#[test]
fn test_transact_payout_stakers_work() {
    TestNet::reset();

    Relay::execute_with(|| {
        let exposure = Exposure {
            total: 100 * DOT_DECIMAL,
            own: 33 * DOT_DECIMAL,
            others: vec![IndividualExposure {
                who: CHARILE,
                value: 67 * DOT_DECIMAL,
            }],
        };
        pallet_babe::Pallet::<WestendRuntime>::on_initialize(1);
        pallet_staking::ErasStartSessionIndex::<WestendRuntime>::insert(0, 1);
        pallet_session::Pallet::<WestendRuntime>::rotate_session();
        pallet_staking::CurrentEra::<WestendRuntime>::put(0);
        pallet_staking::ErasValidatorReward::<WestendRuntime>::insert(0, 500 * DOT_DECIMAL);
        pallet_staking::ErasStakersClipped::<WestendRuntime>::insert(
            0,
            LiquidStaking::derivative_para_account_id(),
            exposure,
        );
        RelayStaking::reward_by_ids(vec![(LiquidStaking::derivative_para_account_id(), 100)]);
    });

    ParaA::execute_with(|| {
        assert_ok!(LiquidStaking::bond(
            Origin::signed(ALICE),
            1 * DOT_DECIMAL,
            RewardDestination::Account(BOB),
        ));

        // weight is 31701208000
        assert_ok!(LiquidStaking::payout_stakers(
            Origin::signed(ALICE),
            LiquidStaking::derivative_para_account_id(),
            0
        ));
    });

    // (33/100) * 500
    Relay::execute_with(|| {
        assert_eq!(RelayBalances::free_balance(BOB), 165 * DOT_DECIMAL);
    });
}

#[test]
fn test_transfer_and_then_bond() {
    TestNet::reset();
    let xcm_transfer_amount = 30 * DOT_DECIMAL;
    let relay_transfer_amount = 12 * DOT_DECIMAL;
    ParaA::execute_with(|| {
        let stash = LiquidStaking::derivative_para_account_id();
        let controller = stash.clone();
        let payee = RewardDestination::<AccountId>::Staked;
        let bond_call =
            RelaychainCall::Utility(Box::new(UtilityCall::BatchAll(UtilityBatchAllCall {
                calls: vec![
                    RelaychainCall::Balances(BalancesCall::TransferKeepAlive(
                        BalancesTransferKeepAliveCall {
                            dest: AccountIdLookup::<AccountId, ()>::unlookup(stash.clone()),
                            value: relay_transfer_amount,
                        },
                    )),
                    RelaychainCall::Utility(Box::new(UtilityCall::AsDerivative(
                        UtilityAsDerivativeCall {
                            index: 0,
                            call: RelaychainCall::Staking::<Test>(StakingCall::Bond(
                                StakingBondCall {
                                    controller: AccountIdLookup::<AccountId, ()>::unlookup(
                                        controller.clone(),
                                    ),
                                    value: relay_transfer_amount,
                                    payee: payee.clone(),
                                },
                            )),
                        },
                    ))),
                ],
            })));
        let bond_transact_xcm = Transact {
            origin_type: OriginKind::SovereignAccount,
            require_weight_at_most: u64::MAX,
            call: bond_call.encode().into(),
        };

        let asset: MultiAsset = (MultiLocation::parent(), xcm_transfer_amount).into();
        let reserve = MultiLocation::parent();
        let recipient = MultiLocation::new(
            0,
            X1(Junction::AccountId32 {
                network: NetworkId::Any,
                id: LiquidStaking::derivative_para_account_id().into(),
            }),
        );
        let fees: MultiAsset = (MultiLocation::here(), xcm_transfer_amount).into();
        let msg = WithdrawAsset {
            assets: asset.clone().into(),
            effects: vec![InitiateReserveWithdraw {
                assets: All.into(),
                reserve: reserve.clone(),
                effects: vec![
                    BuyExecution {
                        fees,
                        weight: 0,
                        debt: 30,
                        halt_on_error: false,
                        instructions: vec![bond_transact_xcm],
                    },
                    DepositAsset {
                        assets: All.into(),
                        max_assets: u32::max_value(),
                        beneficiary: recipient,
                    },
                ],
            }],
        };
        let origin_location = MultiLocation::new(
            0,
            X1(Junction::AccountId32 {
                network: NetworkId::Any,
                id: ALICE.into(),
            }),
        );
        let weight = 2;
        let _ = xcm_executor::XcmExecutor::<XcmConfig>::execute_xcm_in_credit(
            origin_location,
            msg,
            weight,
            weight,
        )
        .ensure_complete();
        print_events::<Test>("ParaA");
    });

    Relay::execute_with(|| {
        print_events::<westend_runtime::Runtime>("Relay");
        assert_eq!(
            RelayBalances::free_balance(&LiquidStaking::derivative_para_account_id()),
            xcm_transfer_amount + relay_transfer_amount - 240
        );

        let ledger = RelayStaking::ledger(LiquidStaking::derivative_para_account_id()).unwrap();
        assert_eq!(ledger.total, relay_transfer_amount);
    });
}

#[test]
fn test_transfer_bond() {
    TestNet::reset();
    let xcm_transfer_amount = 10 * DOT_DECIMAL;
    ParaA::execute_with(|| {
        assert_ok!(LiquidStaking::bond(
            Origin::signed(ALICE),
            xcm_transfer_amount,
            RewardDestination::Staked
        ));
        print_events::<Test>("ParaA");
    });
    Relay::execute_with(|| {
        print_events::<westend_runtime::Runtime>("Relay");
        let ledger = RelayStaking::ledger(LiquidStaking::derivative_para_account_id()).unwrap();
        assert_eq!(ledger.total, xcm_transfer_amount);
        assert_eq!(
            RelayBalances::free_balance(LiquidStaking::derivative_para_account_id()),
            xcm_transfer_amount
        );
        assert_eq!(
            RelayBalances::usable_balance(LiquidStaking::derivative_para_account_id()),
            0
        );
    });
}

fn print_events<T: frame_system::Config>(context: &str) {
    println!("------ {:?} events ------", context);
    frame_system::Pallet::<T>::events().iter().for_each(|r| {
        println!("{:?}", r.event);
    });
}
