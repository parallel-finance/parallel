use frame_support::{assert_err, assert_ok};

use primitives::Rate;

use crate::types::StakeingSettlementKind;
use crate::Error;
use crate::{mock::*, EraIndexPair};

#[test]
fn test_record_staking_settlement_ok() {
    new_test_ext().execute_with(|| {
        assert_ok!(LiquidStaking::record_staking_settlement(
            Origin::signed(Alice),
            1,
            100,
            StakeingSettlementKind::Reward
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
            StakeingSettlementKind::Reward,
        )
        .unwrap();

        assert_err!(
            LiquidStaking::record_staking_settlement(
                Origin::signed(Alice),
                1,
                100,
                StakeingSettlementKind::Reward
            ),
            Error::<Test>::StakeingSettlementAlreadyRecorded
        )
    })
}

#[test]
fn test_set_era_index() {
    new_test_ext().execute_with(|| {
        assert_ok!(LiquidStaking::set_era_index(Origin::signed(Alice), 1));
        assert_eq!(EraIndexPair::<Test>::get(), (Some(0u32), 1u32));
        assert_err!(
            LiquidStaking::set_era_index(Origin::signed(Alice), 1),
            Error::<Test>::EraAlreadyPushed
        );
    })
}
