use crate::{
    mock::*,
    types::{MatchingLedger, StakingLedger},
    *,
};

use frame_support::{assert_noop, assert_ok};

use primitives::{
    tokens::{KSM, XKSM},
    ump::RewardDestination,
    Balance, Rate, Ratio,
};
use sp_runtime::traits::{One, Zero};
use sp_runtime::MultiAddress::Id;
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
            <Test as Config>::Assets::balance(XKSM, &ALICE),
            ksm(109.95f64)
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

        // Check balance is correct
        assert_eq!(<Test as Config>::Assets::balance(KSM, &ALICE), ksm(90f64));
        assert_eq!(
            <Test as Config>::Assets::balance(XKSM, &ALICE),
            ksm(103.95f64)
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
fn test_settlement_should_work() {
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
        for (i, (stake_ops, bonding_amount, unbonding_amount, matching_result)) in
            test_case.into_iter().enumerate()
        {
            stake_ops.into_iter().for_each(StakeOp::execute);
            assert_eq!(
                LiquidStaking::matching_pool().matching(unbonding_amount),
                Ok(matching_result)
            );
            assert_ok!(LiquidStaking::settlement(
                Origin::signed(ALICE),
                bonding_amount,
                unbonding_amount,
            ));
            LiquidStaking::notification_received(
                pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
                i.try_into().unwrap(),
                Response::ExecutionResult(None),
            )
            .unwrap();
        }
    });
}

#[test]
fn test_transact_bond_work() {
    TestNet::reset();

    ParaA::execute_with(|| {
        assert_ok!(LiquidStaking::stake(Origin::signed(ALICE), ksm(2000f64),));

        assert_ok!(LiquidStaking::bond(
            Origin::signed(ALICE),
            ksm(3f64),
            RewardDestination::Staked
        ));

        ParaSystem::assert_has_event(mock::Event::LiquidStaking(crate::Event::Bonding(
            LiquidStaking::derivative_para_account_id(),
            ksm(3f64),
            RewardDestination::Staked,
        )));
    });

    Relay::execute_with(|| {
        RelaySystem::assert_has_event(RelayEvent::Staking(RelayStakingEvent::Bonded(
            LiquidStaking::derivative_para_account_id(),
            ksm(3f64),
        )));
        let ledger = RelayStaking::ledger(LiquidStaking::derivative_para_account_id()).unwrap();
        assert_eq!(ledger.total, ksm(3f64));
    });
}

#[test]
fn test_transact_bond_extra_work() {
    TestNet::reset();

    ParaA::execute_with(|| {
        assert_ok!(LiquidStaking::stake(Origin::signed(ALICE), ksm(4000f64),));

        assert_ok!(LiquidStaking::bond(
            Origin::signed(ALICE),
            ksm(2f64),
            RewardDestination::Staked
        ));

        assert_ok!(LiquidStaking::bond_extra(Origin::signed(ALICE), ksm(3f64)));
    });

    Relay::execute_with(|| {
        let ledger = RelayStaking::ledger(LiquidStaking::derivative_para_account_id()).unwrap();
        assert_eq!(ledger.total, ksm(5f64));
    });
}

#[test]
fn test_transact_unbond_work() {
    TestNet::reset();

    ParaA::execute_with(|| {
        assert_ok!(LiquidStaking::stake(Origin::signed(ALICE), ksm(6000f64),));

        assert_ok!(LiquidStaking::bond(
            Origin::signed(ALICE),
            ksm(5f64),
            RewardDestination::Staked
        ));
        assert_ok!(LiquidStaking::unbond(Origin::signed(ALICE), ksm(2f64)));
    });

    Relay::execute_with(|| {
        RelaySystem::assert_has_event(RelayEvent::Staking(RelayStakingEvent::Bonded(
            LiquidStaking::derivative_para_account_id(),
            ksm(5f64),
        )));
        RelaySystem::assert_has_event(RelayEvent::Staking(RelayStakingEvent::Unbonded(
            LiquidStaking::derivative_para_account_id(),
            ksm(2f64),
        )));
        let ledger = RelayStaking::ledger(LiquidStaking::derivative_para_account_id()).unwrap();
        assert_eq!(ledger.total, ksm(5f64));
        assert_eq!(ledger.active, ksm(3f64));
    });
}

#[test]
fn test_transact_withdraw_unbonded_work() {
    TestNet::reset();

    ParaA::execute_with(|| {
        assert_ok!(LiquidStaking::stake(Origin::signed(ALICE), ksm(6000f64),));

        assert_ok!(LiquidStaking::bond(
            Origin::signed(ALICE),
            ksm(5f64),
            RewardDestination::Staked
        ));
        assert_ok!(LiquidStaking::unbond(Origin::signed(ALICE), ksm(2f64)));
    });

    Relay::execute_with(|| {
        let ledger = RelayStaking::ledger(LiquidStaking::derivative_para_account_id()).unwrap();
        assert_eq!(ledger.total, ksm(5f64));
        assert_eq!(ledger.active, ksm(3f64));
        assert_eq!(ledger.unlocking.len(), 1);

        RelaySystem::assert_has_event(RelayEvent::Staking(RelayStakingEvent::Bonded(
            LiquidStaking::derivative_para_account_id(),
            ksm(5f64),
        )));
        RelaySystem::assert_has_event(RelayEvent::Staking(RelayStakingEvent::Unbonded(
            LiquidStaking::derivative_para_account_id(),
            ksm(2f64),
        )));

        pallet_staking::CurrentEra::<KusamaRuntime>::put(
            <KusamaRuntime as pallet_staking::Config>::BondingDuration::get(),
        );
    });

    ParaA::execute_with(|| {
        assert_ok!(LiquidStaking::withdraw_unbonded(Origin::signed(BOB), 0));
    });

    Relay::execute_with(|| {
        let ledger = RelayStaking::ledger(LiquidStaking::derivative_para_account_id()).unwrap();
        assert_eq!(ledger.total, ksm(3f64));
        assert_eq!(ledger.active, ksm(3f64));
        assert_eq!(ledger.unlocking.len(), 0);
    });
}

#[test]
fn test_transact_rebond_work() {
    TestNet::reset();

    ParaA::execute_with(|| {
        assert_ok!(LiquidStaking::stake(Origin::signed(ALICE), ksm(6000f64),));

        assert_ok!(LiquidStaking::bond(
            Origin::signed(ALICE),
            ksm(10f64),
            RewardDestination::Staked
        ));
        assert_ok!(LiquidStaking::unbond(Origin::signed(ALICE), ksm(5f64)));
        assert_ok!(LiquidStaking::rebond(Origin::signed(ALICE), ksm(3f64)));
    });

    Relay::execute_with(|| {
        RelaySystem::assert_has_event(RelayEvent::Staking(RelayStakingEvent::Bonded(
            LiquidStaking::derivative_para_account_id(),
            ksm(10f64),
        )));
        RelaySystem::assert_has_event(RelayEvent::Staking(RelayStakingEvent::Unbonded(
            LiquidStaking::derivative_para_account_id(),
            ksm(5f64),
        )));
        RelaySystem::assert_has_event(RelayEvent::Staking(RelayStakingEvent::Bonded(
            LiquidStaking::derivative_para_account_id(),
            ksm(3f64),
        )));
        let ledger = RelayStaking::ledger(LiquidStaking::derivative_para_account_id()).unwrap();
        assert_eq!(ledger.total, ksm(10f64));
        assert_eq!(ledger.active, ksm(8f64));
    });
}

#[test]
fn test_transact_nominate_work() {
    TestNet::reset();

    ParaA::execute_with(|| {
        assert_ok!(LiquidStaking::stake(Origin::signed(ALICE), ksm(4000f64),));

        assert_ok!(LiquidStaking::bond(
            Origin::signed(ALICE),
            ksm(10f64),
            RewardDestination::Staked
        ));

        assert_ok!(LiquidStaking::nominate(
            Origin::signed(ALICE),
            vec![ALICE, BOB],
        ));
    });

    Relay::execute_with(|| {
        let ledger = RelayStaking::ledger(LiquidStaking::derivative_para_account_id()).unwrap();
        assert_eq!(ledger.total, ksm(10f64));
        let nominators =
            RelayStaking::nominators(LiquidStaking::derivative_para_account_id()).unwrap();
        assert_eq!(nominators.targets, vec![ALICE, BOB]);
    });
}

#[test]
fn test_transfer_bond() {
    TestNet::reset();
    let xcm_transfer_amount = ksm(10f64);
    ParaA::execute_with(|| {
        assert_ok!(LiquidStaking::stake(Origin::signed(ALICE), ksm(2000f64),));
        assert_ok!(LiquidStaking::bond(
            Origin::signed(ALICE),
            xcm_transfer_amount,
            RewardDestination::Staked
        ));
        // print_events::<Test>("ParaA");
    });
    Relay::execute_with(|| {
        // print_events::<kusama_runtime::Runtime>("Relay");
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

        let unstake_amount = ksm(6f64);
        assert_ok!(LiquidStaking::unstake(
            Origin::signed(ALICE),
            unstake_amount
        ));
        assert_eq!(PendingUnstake::<Test>::get(0, ALICE), unstake_amount);

        assert_noop!(
            LiquidStaking::claim_for(Origin::signed(BOB), 0, Id(ALICE)),
            Error::<Test>::NothingToClaim
        );

        CurrentUnbondIndex::<Test>::put(3);

        assert_noop!(
            LiquidStaking::claim_for(Origin::signed(BOB), 0, Id(ALICE)),
            Error::<Test>::InsufficientAsset
        );

        Ledger::<Test>::put(StakingLedger {
            withdrawable: unstake_amount,
            unlocking: vec![],
        });

        assert_ok!(LiquidStaking::claim_for(Origin::signed(BOB), 0, Id(ALICE)));
        assert_eq!(
            <Test as Config>::Assets::balance(KSM, &ALICE),
            ksm(90f64) + unstake_amount
        );
    })
}
