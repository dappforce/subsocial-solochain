use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};
use sp_runtime::DispatchError::BadOrigin;

use super::*;

// Force Add faucet
// ----------------------------------------------------------------------------
#[test]
fn force_add_faucet_should_work() {
    ExtBuilder::new_test_ext().execute_with(|| {
        assert_ok!(Faucets::force_add_faucet(
                Origin::root(),
                1,
                100,
                50,
                25
        ));
        let faucet = FaucetByAccount::<Test>::get(1).unwrap();
        assert_eq!(faucet.period, 100);
    });
}

#[test]
fn force_add_faucet_should_fail_when_origin_is_not_root() {
    ExtBuilder::new_test_ext().execute_with(|| {
        assert_noop!(Faucets::force_add_faucet(
                Origin::signed(1),
                1,
                100,
                50,
                25
        ), BadOrigin);
    });
}

#[test]
fn force_add_faucet_should_fail_when_faucet_already_added() {
    ExtBuilder::new_test_ext().execute_with(|| {
        assert_ok!(Faucets::force_add_faucet(
                Origin::root(),
                2,
                100,
                50,
                25
        ));

        assert_noop!(Faucets::force_add_faucet(
                Origin::root(),
                2,
                100,
                50,
                25
        ), Error::<Test>::FaucetAlreadyAdded);
    });
}

#[test]
fn force_add_faucet_should_fail_when_no_free_balance_on_account() {
    ExtBuilder::new_test_ext().execute_with(|| {
        assert_noop!(Faucets::force_add_faucet(
                Origin::root(),
                9,
                100,
                50,
                25
        ), Error::<Test>::NoFreeBalanceOnFaucet);
    });
}

#[test]
fn force_add_faucet_should_fail_when_drip_limit_exceeds_period_limit() {
    ExtBuilder::new_test_ext().execute_with(|| {
        assert_noop!(Faucets::force_add_faucet(
                Origin::root(),
                1,
                100,
                50,
                51
        ), Error::<Test>::DripLimitCannotExceedPeriodLimit);
    });
}

// Add faucet
// ----------------------------------------------------------------------------
#[test]
fn add_faucet_should_work() {
    ExtBuilder::new_test_ext().execute_with(|| {
        assert_ok!(Faucets::add_faucet(
                Origin::signed(1),
                100,
                50,
                25
        ));

        let faucet = FaucetByAccount::<Test>::get(1).unwrap();
        assert_eq!(faucet.period, 100);

    });
}

// Update faucet
// ----------------------------------------------------------------------------
#[test]
fn update_faucet_should_work() {
    ExtBuilder::new_test_ext().execute_with(|| {
        assert_ok!(Faucets::force_add_faucet(
                Origin::root(),
                1,
                100,
                50,
                25
        ));

        let update = default_faucet_update(None, Some(7_200), Some(100), Some(50));
        assert_ok!(Faucets::update_faucet(
                Origin::root(), 
                1, 
                update.clone()
        ));

       let faucet = FaucetByAccount::<Test>::get(1).unwrap();
       assert_eq!(faucet.period, update.period.unwrap());
    });
}

#[test]
fn update_faucet_should_fail_when_no_updates_provided() {
    ExtBuilder::new_test_ext().execute_with(|| {
        assert_ok!(Faucets::force_add_faucet(
                Origin::root(),
                1,
                100,
                50,
                25
        ));
        let update = default_faucet_update(None, None, None, None);
        assert_noop!(
            Faucets::update_faucet(
                Origin::root(),
                1,
                update.clone()
        ), Error::<Test>::NoUpdatesProvided);
        let faucet = FaucetByAccount::<Test>::get(1).unwrap();
        assert_eq!(faucet.period, 100);
    });
}

#[test]
fn update_faucet_should_fail_when_faucet_address_in_unknown() {
    ExtBuilder::new_test_ext().execute_with(|| {
        let update = default_faucet_update(None, Some(7_200), Some(100), Some(50));
        assert_noop!(Faucets::update_faucet(
                Origin::root(),
                1,
                update.clone()
        ), Error::<Test>::FaucetNotFound);
    });
}

#[test]
fn update_faucet_should_fail_when_same_active_flag_provided() {
    ExtBuilder::new_test_ext().execute_with(|| {
        assert_ok!(Faucets::force_add_faucet(
                Origin::root(),
                1,
                100,
                50,
                25
        ));
        let update = default_faucet_update(Some(true), Some(7_200), Some(100), Some(50));
        assert_noop!(Faucets::update_faucet(
                Origin::root(),
                1,
                update.clone()
        ), Error::<Test>::InvalidUpdate);
    });
}

#[test]
fn update_faucet_should_fail_when_same_period_provided() {
    ExtBuilder::new_test_ext().execute_with(|| {
        assert_ok!(Faucets::force_add_faucet(
                Origin::root(),
                1,
                100,
                50,
                25
        ));
        let update = default_faucet_update(None, Some(100), Some(100), Some(50));
        assert_noop!(Faucets::update_faucet(
                Origin::root(),
                1,
                update.clone()
        ), Error::<Test>::InvalidUpdate);
    });
}

#[test]
fn update_faucet_should_fail_when_same_period_limit_provided() {
    ExtBuilder::new_test_ext().execute_with(|| {
        assert_ok!(Faucets::force_add_faucet(
                Origin::root(),
                1,
                100,
                50,
                25
        ));
        let update = default_faucet_update(None, Some(7_200), Some(50), Some(30));
        assert_noop!(Faucets::update_faucet(
                Origin::root(),
                1,
                update.clone()
        ), Error::<Test>::InvalidUpdate);
    });
}

#[test]
fn update_faucet_should_fail_when_same_drip_limit_provided() {
    ExtBuilder::new_test_ext().execute_with(|| {
        assert_ok!(Faucets::force_add_faucet(
                Origin::root(),
                1,
                100,
                50,
                25
        ));
        let update = default_faucet_update(None, Some(7_200), Some(100), Some(25));
        assert_noop!(Faucets::update_faucet(
                Origin::root(),
                1,
                update.clone()
        ), Error::<Test>::InvalidUpdate);
    });
}

#[test]
fn update_faucet_should_fail_when_new_period_limit_below_drip_limit() {
    ExtBuilder::new_test_ext().execute_with(|| {
        assert_ok!(Faucets::force_add_faucet(
                Origin::root(),
                1,
                100,
                50,
                25
        ));
        let update = default_faucet_update(None, Some(7_200), Some(100), Some(110));
        assert_noop!(Faucets::update_faucet(
                Origin::root(),
                1,
                update.clone()
        ), Error::<Test>::DripLimitCannotExceedPeriodLimit);
    });
}

// Remove faucets
// ----------------------------------------------------------------------------
#[test]
fn remove_faucet_should_work() {
    ExtBuilder::new_test_ext().execute_with(|| {
        let mut faucets = Vec::new();
        for account in 1..=6 {
            assert_ok!(Faucets::force_add_faucet(
                    Origin::root(),
                    account,
                    100,
                    50,
                    25
            ));
            faucets.push(account);
        }

        // This should remove only faucet 6
        let _ = faucets.pop();
        assert_ok!(Faucets::remove_faucets(
                Origin::root(),
                faucets
        ));

        for account in 1..6 {
            assert!(FaucetByAccount::<Test>::get(account).is_none());
        }
        assert!(FaucetByAccount::<Test>::get(6).is_some());
    });
}

#[test]
fn remove_faucets_should_handle_duplicate_addresses() {
    ExtBuilder::new_test_ext().execute_with(|| {
        let mut faucets = Vec::new();
        for account in 1..=6 {
            assert_ok!(Faucets::force_add_faucet(
                    Origin::root(),
                    account,
                    100,
                    50,
                    25
            ));
            faucets.push(account)
        }

        let _ = faucets.pop();
        let mut duplicates = vec![1, 4];
        faucets.append(&mut duplicates);
        assert_ok!(Faucets::remove_faucets(
                Origin::root(),
                faucets
        ));

        for account in 1..6 {
            assert!(FaucetByAccount::<Test>::get(account).is_none());
        }
        assert!(FaucetByAccount::<Test>::get(6).is_some());
    });
}

#[test]
fn remove_faucets_should_fail_when_no_faucet_addresses_provided() {
    ExtBuilder::new_test_ext().execute_with(|| {
        assert_noop!(Faucets::remove_faucets(
                Origin::root(),
                vec![]
            ),
            Error::<Test>::NoFaucetsProvided
        );
    });
}

// Drip
// ----------------------------------------------------------------------------
#[test]
fn drip_should_work() {
    ExtBuilder::new_test_ext().execute_with(|| {
        assert_ok!(Faucets::force_add_faucet(
                Origin::root(),
                1,
                100,
                50,
                25
        ));
        let faucet = FaucetByAccount::<Test>::get(1).unwrap();
        assert_eq!(faucet.next_period_at, 0);
        assert_eq!(faucet.dripped_in_current_period, 0);
        assert_eq!(Balances::free_balance(5), 50);
        assert_ok!(Faucets::drip(
                Origin::signed(1),
                5,
                5
        ));
        assert_eq!(Balances::free_balance(5), 55);

        let faucet_state = FaucetByAccount::<Test>::get(1).unwrap();
        assert_eq!(faucet_state.next_period_at, 101);
        assert_eq!(faucet_state.dripped_in_current_period, 5);
    });
}

#[test]
fn drip_should_work_multiple_times_in_the_same_period() {
    ExtBuilder::new_test_ext().execute_with(|| {
        assert_ok!(Faucets::force_add_faucet(
                Origin::root(),
                1,
                100,
                50,
                25
        ));
        let faucet = FaucetByAccount::<Test>::get(1).unwrap();
        assert_eq!(faucet.next_period_at, 0);
        assert_eq!(faucet.dripped_in_current_period, 0);
        assert_eq!(Balances::free_balance(5), 50);
        assert_ok!(Faucets::drip(
                Origin::signed(1),
                5,
                5
        ));
        assert_eq!(Balances::free_balance(5), 55);

        let faucet_state = FaucetByAccount::<Test>::get(1).unwrap();
        assert_eq!(faucet_state.next_period_at, 101);
        assert_eq!(faucet_state.dripped_in_current_period, 5);
        assert_ok!(Faucets::drip(
                Origin::signed(1),
                5,
                1
        ));
        assert_eq!(Balances::free_balance(5), 56);

        let next_faucet_state = FaucetByAccount::<Test>::get(1).unwrap();
        assert_eq!(next_faucet_state.next_period_at, 101);
        assert_eq!(next_faucet_state.dripped_in_current_period, 6);
    });
}

#[test]
fn drip_should_work_for_same_recipient_in_next_period() {
    ExtBuilder::new_test_ext().execute_with(|| {
        assert_ok!(Faucets::force_add_faucet(
                Origin::root(),
                1,
                100,
                50,
                25
        ));
        let faucet = FaucetByAccount::<Test>::get(1).unwrap();
        assert_eq!(faucet.next_period_at, 0);
        assert_eq!(faucet.dripped_in_current_period, 0);
        assert_eq!(Balances::free_balance(5), 50);
        assert_ok!(Faucets::drip(
                Origin::signed(1),
                5,
                5
        ));
        assert_eq!(Balances::free_balance(5), 55);

        let faucet_state = FaucetByAccount::<Test>::get(1).unwrap();
        assert_eq!(faucet_state.next_period_at, 101);
        assert_eq!(faucet_state.dripped_in_current_period, 5);

        // Move to the next period
        System::set_block_number(faucet_state.next_period_at);
        assert_ok!(Faucets::drip(
                Origin::signed(1),
                5,
                1
        ));
        assert_eq!(Balances::free_balance(5), 56);

        let next_faucet_state = FaucetByAccount::<Test>::get(1).unwrap();
        assert_eq!(next_faucet_state.next_period_at, 201);
        assert_eq!(next_faucet_state.dripped_in_current_period, 1);
    });
}

#[test]
fn drip_should_fail_when_too_big_amount_provided() {
    ExtBuilder::new_test_ext().execute_with(|| {
        assert_ok!(Faucets::force_add_faucet(
                Origin::root(),
                6,
                100,
                50,
                25
        ));
        assert_noop!(Faucets::drip(
                Origin::signed(6),
                5,
                30
        ), Error::<Test>::DripLimitReached);
    });
}

#[test]
fn drip_should_fail_when_zero_amount_provided() {
    ExtBuilder::new_test_ext().execute_with(|| {
        assert_ok!(Faucets::force_add_faucet(
                Origin::root(),
                1,
                100,
                50,
                25
        ));
        assert_noop!(
            Faucets::drip(Origin::signed(1), 5, 0),
            Error::<Test>::ZeroDripAmountProvided
        );

        // Account should have no tokens if drip failed
        assert_eq!(Balances::free_balance(5), 50);
    });
}

#[test]
fn drip_should_fail_when_amount_is_bigger_than_free_balance_on_faucet() {
    ExtBuilder::new_test_ext().execute_with(|| {
        assert_ok!(Faucets::force_add_faucet(
                Origin::root(),
                1,
                100,
                50,
                25
        ));
        assert_noop!(
            Faucets::drip(Origin::signed(1), 5, 11),
            Error::<Test>::NotEnoughFreeBalanceOnFaucet
        );

        // Account should have no tokens if drip failed
        assert_eq!(Balances::free_balance(5), 50);
    });
}

#[test]
fn drip_should_fail_when_recipient_equals_faucet() {
ExtBuilder::new_test_ext().execute_with(|| {
        assert_ok!(Faucets::force_add_faucet(
                Origin::root(),
                1,
                100,
                50,
                25
        ));
        assert_noop!(
            Faucets::drip(Origin::signed(1), 1, 11),
            Error::<Test>::RecipientEqualsFaucet
        );
    });
}

/*
#[test]
fn drip_should_fail_when_period_limit_reached() {
    ExtBuilder::build_with_one_default_drip().execute_with(|| {
        System::set_block_number(INITIAL_BLOCK_NUMBER);

        // Do the second drip
        assert_ok!(_do_default_drip());

        // The third drip should fail, b/c it exceeds the period limit of this faucet
        assert_noop!(
            _do_default_drip(),
            Error::<Test>::PeriodLimitReached
        );

        let drip_limit = default_faucet().drip_limit;

        // Balance should be unchanged and equal to two drip
        assert_eq!(Balances::free_balance(ACCOUNT1), drip_limit * 2);
    });
}

#[test]
fn drip_should_fail_when_faucet_is_disabled_and_work_again_after_faucet_enabled() {
    ExtBuilder::build_with_faucet().execute_with(|| {
        
        // Account should have no tokens by default
        assert_eq!(Balances::free_balance(ACCOUNT1), 0);

        // Disable the faucet, so it will be not possible to drip
        assert_ok!(_update_faucet_settings(
            FaucetUpdate {
                enabled: Some(false),
                period: None,
                period_limit: None,
                drip_limit: None
            }
        ));

        // Faucet should not drip tokens if it is disabled
        assert_noop!(
            _do_default_drip(),
            Error::<Test>::FaucetDisabled
        );

        // Account should not receive any tokens
        assert_eq!(Balances::free_balance(ACCOUNT1), 0);

        // Make the faucet enabled again
        assert_ok!(_update_faucet_settings(
            FaucetUpdate {
                enabled: Some(true),
                period: None,
                period_limit: None,
                drip_limit: None
            }
        ));

        // Should be able to drip again
        assert_ok!(_do_default_drip());

        // Account should receive the tokens
        assert_eq!(Balances::free_balance(ACCOUNT1), default_faucet().drip_limit);
    });
}
*/
