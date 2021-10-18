use crate::{
    mock::*,
    types::{MatchingLedger, RewardDestination, StakingSettlementKind},
    *,
};

use frame_support::{assert_err, assert_ok, traits::Hooks};

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
fn stake_fails_due_to_exceed_capacity() {
    new_test_ext().execute_with(|| {
        assert_err!(
            LiquidStaking::stake(Origin::signed(BOB), dot(10053f64)),
            Error::<Test>::ExceededStakingPoolCapacity
        );
    })
}

#[test]
fn stake_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(LiquidStaking::stake(Origin::signed(ALICE), dot(10f64)));
        // Check storage is correct
        assert_eq!(ExchangeRate::<Test>::get(), Rate::one());
        assert_eq!(StakingPool::<Test>::get(), dot(9.95f64));
        assert_eq!(
            MatchingPool::<Test>::get(),
            MatchingLedger {
                total_stake_amount: dot(9.95f64),
                total_unstake_amount: 0,
            }
        );

        // Check balance is correct
        assert_eq!(<Test as Config>::Assets::balance(DOT, &ALICE), dot(90f64));
        assert_eq!(
            <Test as Config>::Assets::balance(XDOT, &ALICE),
            dot(109.95f64)
        );
        assert_eq!(
            <Test as Config>::Assets::balance(DOT, &LiquidStaking::account_id()),
            dot(10f64)
        );
    })
}

#[test]
fn unstake_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(LiquidStaking::stake(Origin::signed(ALICE), dot(10f64)));
        assert_ok!(LiquidStaking::unstake(Origin::signed(ALICE), dot(6f64)));

        // Check storage is correct
        assert_eq!(ExchangeRate::<Test>::get(), Rate::one());
        assert_eq!(StakingPool::<Test>::get(), dot(3.95f64));
        assert_eq!(
            MatchingPool::<Test>::get(),
            MatchingLedger {
                total_stake_amount: dot(9.95f64),
                total_unstake_amount: dot(6f64),
            }
        );

        // Check balance is correct
        assert_eq!(<Test as Config>::Assets::balance(DOT, &ALICE), dot(96f64));
        assert_eq!(
            <Test as Config>::Assets::balance(XDOT, &ALICE),
            dot(103.95f64)
        );
        assert_eq!(
            <Test as Config>::Assets::balance(DOT, &LiquidStaking::account_id()),
            dot(4f64)
        );
    })
}

#[test]
fn test_record_staking_settlement_ok() {
    new_test_ext().execute_with(|| {
        assert_ok!(LiquidStaking::record_staking_settlement(
            Origin::signed(ALICE),
            dot(100f64),
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
            100,
            StakingSettlementKind::Reward,
        )
        .unwrap();
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
            (
                vec![Stake(dot(5000f64)), Unstake(dot(1000f64))],
                0,
                (dot(3975f64), 0, 0),
                dot(25f64),
            ),
            // Calculate right here.
            (
                vec![Unstake(dot(10f64)), Unstake(dot(5f64)), Stake(dot(10f64))],
                0,
                (0, 0, dot(5.05f64)),
                dot(15.05f64),
            ),
            (vec![], 0, (0, 0, 0), dot(5.05f64)),
        ];

        for (stake_ops, unbonding_amount, matching_result, insurance_pool) in test_case.into_iter()
        {
            stake_ops.into_iter().for_each(StakeOp::execute);
            assert_eq!(LiquidStaking::insurance_pool(), insurance_pool);
            assert_eq!(
                LiquidStaking::matching_pool().matching(unbonding_amount),
                matching_result
            );
            assert_ok!(LiquidStaking::settlement(
                Origin::signed(ALICE),
                true,
                unbonding_amount,
            ));
            Pallet::<Test>::on_idle(0, 10000);
        }
    });
    Relay::execute_with(|| {
        assert_eq!(
            RelayBalances::free_balance(&LiquidStaking::para_account_id()),
            // FIXME: weight should be take into account
            9999989117112000
        );
    });
}

#[test]
fn test_transact_bond_work() {
    TestNet::reset();

    ParaA::execute_with(|| {
        assert_ok!(LiquidStaking::stake(
            Origin::signed(ALICE),
            2000 * DOT_DECIMAL,
        ));

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
        assert_ok!(LiquidStaking::stake(
            Origin::signed(ALICE),
            4000 * DOT_DECIMAL,
        ));

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
        assert_ok!(LiquidStaking::stake(
            Origin::signed(ALICE),
            6000 * DOT_DECIMAL,
        ));

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
        assert_ok!(LiquidStaking::stake(
            Origin::signed(ALICE),
            6000 * DOT_DECIMAL,
        ));

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
        assert_ok!(LiquidStaking::stake(
            Origin::signed(ALICE),
            6000 * DOT_DECIMAL,
        ));

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
        assert_ok!(LiquidStaking::stake(
            Origin::signed(ALICE),
            4000 * DOT_DECIMAL,
        ));

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
fn stake_should_correctly_add_insurance_pool() {
    new_test_ext().execute_with(|| {
        LiquidStaking::stake(Origin::signed(ALICE), 1000).unwrap();
        assert_eq!(InsurancePool::<Test>::get(), 5);
    })
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
        assert_ok!(LiquidStaking::stake(
            Origin::signed(ALICE),
            2000 * DOT_DECIMAL,
        ));
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

#[test]
fn test_update_xcm_weight_work() {
    new_test_ext().execute_with(|| {
        assert_eq!(XcmWeight::<Test>::get(), XcmWeightMisc::default());
        let misc = XcmWeightMisc::<u64> {
            bond_weight: 1,
            bond_extra_weight: 2,
            unbond_weight: 3,
            rebond_weight: 4,
            withdraw_unbonded_weight: 5,
            nominate_weight: 6,
        };
        assert_ok!(LiquidStaking::update_xcm_weight(
            Origin::signed(ALICE),
            misc
        ));
        assert_eq!(XcmWeight::<Test>::get(), misc);
    })
}
