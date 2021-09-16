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
use sp_runtime::traits::One;
use xcm_simulator::TestExt;

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
            (
                vec![Stake(30 * DOT_DECIMAL), Unstake(5 * DOT_DECIMAL)],
                0,
                (25 * DOT_DECIMAL, 0, 0),
                0,
            ),
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
                unbonding_amount
            ));
            Pallet::<Test>::on_idle(0, 10000);
        }
    });
    Relay::execute_with(|| {
        assert_eq!(
            RelayBalances::free_balance(&RelayAgent::get()),
            // FIXME: weight should be take into account
            9999999200000000
        );
    });
}

#[test]
fn test_transact_bond_work() {
    TestNet::reset();

    ParaA::execute_with(|| {
        assert_ok!(LiquidStaking::bond(
            3 * DOT_DECIMAL,
            RewardDestination::Staked
        ));

        ParaSystem::assert_has_event(mock::Event::LiquidStaking(crate::Event::BondCallSent(
            LiquidStaking::derivative_account_id(),
            3 * DOT_DECIMAL,
            RewardDestination::Staked,
        )));
    });

    Relay::execute_with(|| {
        RelaySystem::assert_has_event(RelayEvent::Staking(RelayStakingEvent::Bonded(
            LiquidStaking::derivative_account_id(),
            3 * DOT_DECIMAL,
        )));
        let ledger = RelayStaking::ledger(LiquidStaking::derivative_account_id()).unwrap();
        assert_eq!(ledger.total, 3 * DOT_DECIMAL);
    });
}

#[test]
fn test_transact_bond_extra_work() {
    TestNet::reset();

    ParaA::execute_with(|| {
        assert_ok!(LiquidStaking::bond(
            2 * DOT_DECIMAL,
            RewardDestination::Staked
        ));

        assert_ok!(LiquidStaking::bond_extra(3 * DOT_DECIMAL));
    });

    Relay::execute_with(|| {
        let ledger = RelayStaking::ledger(LiquidStaking::derivative_account_id()).unwrap();
        assert_eq!(ledger.total, 5 * DOT_DECIMAL);
    });
}

#[test]
fn test_transact_unbond_work() {
    TestNet::reset();

    ParaA::execute_with(|| {
        assert_ok!(LiquidStaking::bond(
            5 * DOT_DECIMAL,
            RewardDestination::Staked
        ));
        assert_ok!(LiquidStaking::unbond(2 * DOT_DECIMAL));
    });

    Relay::execute_with(|| {
        RelaySystem::assert_has_event(RelayEvent::Staking(RelayStakingEvent::Bonded(
            LiquidStaking::derivative_account_id(),
            5 * DOT_DECIMAL,
        )));
        RelaySystem::assert_has_event(RelayEvent::Staking(RelayStakingEvent::Unbonded(
            LiquidStaking::derivative_account_id(),
            2 * DOT_DECIMAL,
        )));
        let ledger = RelayStaking::ledger(LiquidStaking::derivative_account_id()).unwrap();
        assert_eq!(ledger.total, 5 * DOT_DECIMAL);
        assert_eq!(ledger.active, 3 * DOT_DECIMAL);
    });
}

#[test]
fn test_transact_withdraw_unbonded_work() {
    TestNet::reset();

    ParaA::execute_with(|| {
        assert_ok!(LiquidStaking::bond(
            5 * DOT_DECIMAL,
            RewardDestination::Staked
        ));
        assert_ok!(LiquidStaking::unbond(2 * DOT_DECIMAL));
    });

    Relay::execute_with(|| {
        let ledger = RelayStaking::ledger(LiquidStaking::derivative_account_id()).unwrap();
        assert_eq!(ledger.total, 5 * DOT_DECIMAL);
        assert_eq!(ledger.active, 3 * DOT_DECIMAL);
        assert_eq!(ledger.unlocking.len(), 1);

        RelaySystem::assert_has_event(RelayEvent::Staking(RelayStakingEvent::Bonded(
            LiquidStaking::derivative_account_id(),
            5 * DOT_DECIMAL,
        )));
        RelaySystem::assert_has_event(RelayEvent::Staking(RelayStakingEvent::Unbonded(
            LiquidStaking::derivative_account_id(),
            2 * DOT_DECIMAL,
        )));

        pallet_staking::CurrentEra::<WestendRuntime>::put(
            <WestendRuntime as pallet_staking::Config>::BondingDuration::get(),
        );
    });

    ParaA::execute_with(|| {
        assert_ok!(LiquidStaking::withdraw_unbonded(0));
    });

    Relay::execute_with(|| {
        let ledger = RelayStaking::ledger(LiquidStaking::derivative_account_id()).unwrap();
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
            10 * DOT_DECIMAL,
            RewardDestination::Staked
        ));
        assert_ok!(LiquidStaking::unbond(5 * DOT_DECIMAL));
        assert_ok!(LiquidStaking::rebond(3 * DOT_DECIMAL));
    });

    Relay::execute_with(|| {
        RelaySystem::assert_has_event(RelayEvent::Staking(RelayStakingEvent::Bonded(
            LiquidStaking::derivative_account_id(),
            10 * DOT_DECIMAL,
        )));
        RelaySystem::assert_has_event(RelayEvent::Staking(RelayStakingEvent::Unbonded(
            LiquidStaking::derivative_account_id(),
            5 * DOT_DECIMAL,
        )));
        RelaySystem::assert_has_event(RelayEvent::Staking(RelayStakingEvent::Bonded(
            LiquidStaking::derivative_account_id(),
            3 * DOT_DECIMAL,
        )));
        let ledger = RelayStaking::ledger(LiquidStaking::derivative_account_id()).unwrap();
        assert_eq!(ledger.total, 10 * DOT_DECIMAL);
        assert_eq!(ledger.active, 8 * DOT_DECIMAL);
    });
}

#[test]
fn test_transact_nominate_work() {
    TestNet::reset();

    ParaA::execute_with(|| {
        assert_ok!(LiquidStaking::bond(
            10 * DOT_DECIMAL,
            RewardDestination::Staked
        ));

        assert_ok!(LiquidStaking::nominate(vec![ALICE, BOB],));
    });

    Relay::execute_with(|| {
        let ledger = RelayStaking::ledger(LiquidStaking::derivative_account_id()).unwrap();
        assert_eq!(ledger.total, 10 * DOT_DECIMAL);
        let nominators = RelayStaking::nominators(LiquidStaking::derivative_account_id()).unwrap();
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
            LiquidStaking::derivative_account_id(),
            exposure,
        );
        RelayStaking::reward_by_ids(vec![(LiquidStaking::derivative_account_id(), 100)]);
    });

    ParaA::execute_with(|| {
        assert_ok!(LiquidStaking::bond(
            1 * DOT_DECIMAL,
            RewardDestination::Account(BOB),
        ));

        // weight is 31701208000
        assert_ok!(LiquidStaking::payout_stakers(
            LiquidStaking::derivative_account_id(),
            0
        ));
    });

    // (33/100) * 500
    Relay::execute_with(|| {
        assert_eq!(RelayBalances::free_balance(BOB), 165 * DOT_DECIMAL);
    });
}
