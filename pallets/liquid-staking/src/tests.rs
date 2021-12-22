use crate::{mock::*, types::MatchingLedger, *};

use frame_support::{assert_noop, assert_ok, traits::Hooks};

use primitives::{
    tokens::{DOT, XDOT},
    ump::RewardDestination,
    Balance, Rate, Ratio,
};
use sp_runtime::traits::{One, Zero};
use xcm_simulator::TestExt;

#[test]
fn stake_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(LiquidStaking::stake(Origin::signed(ALICE), dot(10f64)));
        // Check storage is correct
        assert_eq!(ExchangeRate::<Test>::get(), Rate::one());
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
    })
}

#[test]
fn unstake_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(LiquidStaking::stake(Origin::signed(ALICE), dot(10f64)));
        assert_ok!(LiquidStaking::unstake(Origin::signed(ALICE), dot(6f64)));

        // Check storage is correct
        assert_eq!(ExchangeRate::<Test>::get(), Rate::one());
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
        let test_case: Vec<(Vec<StakeOp>, Balance, (Balance, Balance, Balance))> = vec![
            (
                vec![Stake(dot(5000f64)), Unstake(dot(1000f64))],
                0,
                (dot(3975f64), 0, 0),
            ),
            // Calculate right here.
            (
                vec![Unstake(dot(10f64)), Unstake(dot(5f64)), Stake(dot(10f64))],
                0,
                (0, 0, dot(5.05f64)),
            ),
            (vec![], 0, (0, 0, 0)),
        ];

        for (stake_ops, unbonding_amount, matching_result) in test_case.into_iter() {
            stake_ops.into_iter().for_each(StakeOp::execute);
            assert_eq!(
                LiquidStaking::matching_pool().matching::<LiquidStaking>(unbonding_amount),
                Ok(matching_result)
            );
            assert_ok!(LiquidStaking::settlement(
                Origin::signed(ALICE),
                dot(0f64),
                unbonding_amount,
            ));
            Pallet::<Test>::on_idle(0, 10000);
        }
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

        ParaSystem::assert_has_event(mock::Event::LiquidStaking(crate::Event::Bonding(
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

        pallet_staking::CurrentEra::<KusamaRuntime>::put(
            <KusamaRuntime as pallet_staking::Config>::BondingDuration::get(),
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
fn test_add_insurances_work() {
    new_test_ext().execute_with(|| {
        assert_eq!(InsurancePool::<Test>::get(), 0);
        assert_ok!(LiquidStaking::add_insurances(Origin::signed(BOB), 123));
        assert_eq!(InsurancePool::<Test>::get(), 123);
    })
}

#[test]
fn update_staking_pool_capacity_should_not_work_if_with_invalid_param() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            LiquidStaking::update_staking_pool_capacity(Origin::root(), Zero::zero()),
            Error::<Test>::ZeroCap
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
