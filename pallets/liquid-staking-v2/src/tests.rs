use frame_support::{assert_err, assert_ok};

use primitives::{EraIndex, Rate};

use crate::mock::*;
use crate::types::{Operation, StakingSettlementKind};
use crate::{Error, UnbondingOperationHistory};

fn t_insert_pending_op(era_index: EraIndex) {
    let block_number = System::block_number();
    UnbondingOperationHistory::<Test>::insert(
        era_index,
        Operation {
            amount: 1u64.into(),
            block_number,
            status: crate::types::ResponseStatus::Pending,
        },
    )
}

#[test]
fn test_record_staking_settlement_ok() {
    new_test_ext().execute_with(|| {
        assert_ok!(LiquidStaking::record_staking_settlement(
            Origin::signed(Alice),
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
            Origin::signed(Alice),
            1,
            100,
            StakingSettlementKind::Reward,
        )
        .unwrap();

        assert_err!(
            LiquidStaking::record_staking_settlement(
                Origin::signed(Alice),
                1,
                100,
                StakingSettlementKind::Reward
            ),
            Error::<Test>::StakeingSettlementAlreadyRecorded
        )
    })
}

#[test]
fn test_set_era_index() {
    new_test_ext().execute_with(|| {
        assert_ok!(LiquidStaking::trigger_new_era(Origin::signed(Alice), 1));
        assert_eq!(LiquidStaking::previous_era(), 0u32);
        assert_eq!(LiquidStaking::current_era(), 1u32);
        assert_err!(
            LiquidStaking::trigger_new_era(Origin::signed(Alice), 1),
            Error::<Test>::EraAlreadyPushed
        );
    })
}

#[test]
fn test_record_unbond_response() {
    new_test_ext().execute_with(|| {
        assert_err!(
            LiquidStaking::record_unbond_response(Origin::signed(Alice), 1u32),
            Error::<Test>::OperationNotReady
        );

        t_insert_pending_op(1u32);
        assert_ok!(LiquidStaking::record_unbond_response(
            Origin::signed(Alice),
            1u32
        ));

        assert_err!(
            LiquidStaking::record_unbond_response(Origin::signed(Alice), 1u32),
            Error::<Test>::OperationNotReady
        );
    })
}
