use crate::{
    mock::{Loans, Origin, Test, ALICE, DOT, MARKET_MOCK, NATIVE, XKSM},
    tests::ExtBuilder,
    Error, InterestRateModel, Market, MarketState,
};
use frame_support::{assert_noop, assert_ok, error::BadOrigin};
use primitives::{Rate, Ratio};
use sp_runtime::FixedPointNumber;

const PENDING_MARKET_MOCK: Market = {
    let mut market = MARKET_MOCK;
    market.state = MarketState::Pending;
    market
};

macro_rules! rate_model_sanity_check {
    ($call:ident) => {
        ExtBuilder::default().build().execute_with(|| {
            // Invalid base_rate
            assert_noop!(
                Loans::$call(Origin::root(), XKSM, {
                    let mut market = PENDING_MARKET_MOCK;
                    market.rate_model = InterestRateModel::new_jump_model(
                        Rate::saturating_from_rational(36, 100),
                        Rate::saturating_from_rational(15, 100),
                        Rate::saturating_from_rational(35, 100),
                        Ratio::from_percent(80),
                    );
                    market
                }),
                Error::<Test>::InvalidRateModelParam
            );
            // Invalid jump_rate
            assert_noop!(
                Loans::$call(Origin::root(), XKSM, {
                    let mut market = PENDING_MARKET_MOCK;
                    market.rate_model = InterestRateModel::new_jump_model(
                        Rate::saturating_from_rational(5, 100),
                        Rate::saturating_from_rational(36, 100),
                        Rate::saturating_from_rational(37, 100),
                        Ratio::from_percent(80),
                    );
                    market
                }),
                Error::<Test>::InvalidRateModelParam
            );
            // Invalid full_rate
            assert_noop!(
                Loans::$call(Origin::root(), XKSM, {
                    let mut market = PENDING_MARKET_MOCK;
                    market.rate_model = InterestRateModel::new_jump_model(
                        Rate::saturating_from_rational(5, 100),
                        Rate::saturating_from_rational(15, 100),
                        Rate::saturating_from_rational(57, 100),
                        Ratio::from_percent(80),
                    );
                    market
                }),
                Error::<Test>::InvalidRateModelParam
            );
            // base_rate greater than jump_rate
            assert_noop!(
                Loans::$call(Origin::root(), XKSM, {
                    let mut market = PENDING_MARKET_MOCK;
                    market.rate_model = InterestRateModel::new_jump_model(
                        Rate::saturating_from_rational(10, 100),
                        Rate::saturating_from_rational(9, 100),
                        Rate::saturating_from_rational(14, 100),
                        Ratio::from_percent(80),
                    );
                    market
                }),
                Error::<Test>::InvalidRateModelParam
            );
            // jump_rate greater than full_rate
            assert_noop!(
                Loans::$call(Origin::root(), XKSM, {
                    let mut market = PENDING_MARKET_MOCK;
                    market.rate_model = InterestRateModel::new_jump_model(
                        Rate::saturating_from_rational(5, 100),
                        Rate::saturating_from_rational(15, 100),
                        Rate::saturating_from_rational(14, 100),
                        Ratio::from_percent(80),
                    );
                    market
                }),
                Error::<Test>::InvalidRateModelParam
            );
        })
    };
}

#[test]
fn active_market_sets_state_to_active() {
    ExtBuilder::default().build().execute_with(|| {
        Loans::add_market(Origin::root(), XKSM, PENDING_MARKET_MOCK).unwrap();
        assert_eq!(Loans::market(&XKSM).unwrap().state, MarketState::Pending);
        Loans::active_market(Origin::root(), XKSM).unwrap();
        assert_eq!(Loans::market(&XKSM).unwrap().state, MarketState::Active);
    })
}

#[test]
fn active_market_does_not_modify_unknown_market_currencies() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            Loans::active_market(Origin::root(), NATIVE),
            Error::<Test>::MarketDoesNotExist
        );
    })
}

#[test]
fn add_market_can_only_be_used_by_root() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            Loans::add_market(Origin::signed(ALICE), DOT, PENDING_MARKET_MOCK),
            BadOrigin
        );
    })
}

#[test]
fn add_market_ensures_that_market_state_must_be_pending() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            Loans::add_market(Origin::root(), XKSM, MARKET_MOCK),
            Error::<Test>::NewMarketMustHavePendingState
        );
    })
}

#[test]
fn add_market_has_sanity_checks_for_rate_models() {
    rate_model_sanity_check!(add_market);
}

#[test]
fn add_market_successfully_stores_a_new_market() {
    ExtBuilder::default().build().execute_with(|| {
        Loans::add_market(Origin::root(), XKSM, PENDING_MARKET_MOCK).unwrap();
        assert_eq!(Loans::market(&XKSM).unwrap(), PENDING_MARKET_MOCK);
    })
}

#[test]
fn add_market_ensures_that_market_does_not_exist() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Loans::add_market(Origin::root(), XKSM, PENDING_MARKET_MOCK));
        assert_noop!(
            Loans::add_market(Origin::root(), XKSM, PENDING_MARKET_MOCK),
            Error::<Test>::MarketAlredyExists
        );
    })
}

#[test]
fn update_market_can_only_be_used_by_root() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            Loans::update_market(Origin::signed(ALICE), DOT, PENDING_MARKET_MOCK),
            BadOrigin
        );
    })
}

#[test]
fn update_market_does_not_modify_state() {
    ExtBuilder::default().build().execute_with(|| {
        Loans::update_market(Origin::root(), DOT, PENDING_MARKET_MOCK).unwrap();
        assert_eq!(Loans::market(&DOT).unwrap().state, MarketState::Active);
    })
}

#[test]
fn update_market_ensures_that_it_is_not_possible_to_modify_unknown_market_currencies() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            Loans::update_market(Origin::root(), NATIVE, MARKET_MOCK),
            Error::<Test>::MarketDoesNotExist
        );
    })
}

#[test]
fn update_market_successfully_modifies_a_stored_market() {
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(
            Loans::market(&DOT).unwrap().close_factor,
            Ratio::from_percent(50)
        );
        Loans::update_market(Origin::root(), DOT, {
            let mut market = MARKET_MOCK;
            market.close_factor = Default::default();
            market
        })
        .unwrap();
        assert_eq!(
            Loans::market(&DOT).unwrap().close_factor,
            Default::default()
        );
    })
}

#[test]
fn update_market_has_sanity_checks_for_rate_models() {
    rate_model_sanity_check!(update_market);
}
