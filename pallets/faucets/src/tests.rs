use crate::{Error, mock::*, Faucet, FaucetUpdate};
use frame_support::{assert_ok, assert_noop};
use sp_runtime::DispatchError::BadOrigin;

// Add faucet
// ----------------------------------------------------------------------------

#[test]
fn add_faucet_should_work() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_add_default_faucet());

        let faucet = Faucets::faucet_by_account(FAUCET1).unwrap();
        assert_eq!(faucet, default_faucet());
    });
}

#[test]
fn add_faucet_should_fail_when_origin_is_not_root() {
    ExtBuilder::build().execute_with(|| {
        let not_root = Origin::signed(ACCOUNT1);
        assert_noop!(
            _add_faucet(Some(not_root), None, None),
            BadOrigin
        );
    });
}

#[test]
fn add_faucet_should_fail_when_faucet_already_added() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_add_default_faucet());

        assert_noop!(
            _add_default_faucet(),
            Error::<Test>::FaucetAlreadyAdded
        );
    });
}

#[test]
fn add_faucet_should_fail_when_no_free_balance_on_account() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_add_default_faucet());

        assert_noop!(
            _add_faucet(None, Some(FAUCET9), None),
            Error::<Test>::NoFreeBalanceOnFaucet
        );
    });
}

#[test]
fn add_faucet_should_fail_when_drip_limit_exceeds_period_limit() {
    ExtBuilder::build().execute_with(|| {
        let mut settings = default_faucet();
        settings.drip_limit = settings.period_limit + 1;
        assert_noop!(
            _add_faucet(None, None, Some(settings)),
            Error::<Test>::DripLimitCannotExceedPeriodLimit
        );
    });
}

// Update faucet
// ----------------------------------------------------------------------------

#[test]
fn update_faucet_should_work() {
    ExtBuilder::build_with_faucet().execute_with(|| {
        assert_ok!(_update_default_faucet());
        let update = default_faucet_update();

        let faucet = Faucets::faucet_by_account(FAUCET1).unwrap();
        let updated_faucet = Faucet::<Test>::new(
            update.period.unwrap_or(faucet.period),
            update.period_limit.unwrap_or(faucet.period_limit),
            update.drip_limit.unwrap_or(faucet.drip_limit)
        );

        assert_eq!(faucet.period, updated_faucet.period);
        assert_eq!(faucet.period_limit, updated_faucet.period_limit);
        assert_eq!(faucet.drip_limit, updated_faucet.drip_limit);
    });
}

#[test]
fn update_faucet_should_fail_when_no_updates_provided() {
    ExtBuilder::build_with_faucet().execute_with(|| {
        assert_noop!(
            _update_faucet_settings(
                FaucetUpdate {
                    enabled: None,
                    period: None,
                    period_limit: None,
                    drip_limit: None
                }
            ),
            Error::<Test>::NoUpdatesProvided
        );
    });
}

#[test]
fn update_faucet_should_fail_when_faucet_address_in_unknown() {
    ExtBuilder::build().execute_with(|| {
        assert_noop!(
            _update_default_faucet(),
            Error::<Test>::FaucetNotFound
        );
    });
}

#[test]
fn update_faucet_should_fail_when_same_active_flag_provided() {
    ExtBuilder::build_with_faucet().execute_with(|| {
        assert_noop!(
            _update_faucet_settings(
                FaucetUpdate {
                    enabled: Some(default_faucet().enabled),
                    period: None,
                    period_limit: None,
                    drip_limit: None
                }
            ),
            Error::<Test>::NothingToUpdate
        );
    });
}

#[test]
fn update_faucet_should_fail_when_same_period_provided() {
    ExtBuilder::build_with_faucet().execute_with(|| {
        assert_noop!(
            _update_faucet_settings(
                FaucetUpdate {
                    enabled: None,
                    period: Some(default_faucet().period),
                    period_limit: None,
                    drip_limit: None
                }
            ),
            Error::<Test>::NothingToUpdate
        );
    });
}

#[test]
fn update_faucet_should_fail_when_same_period_limit_provided() {
    ExtBuilder::build_with_faucet().execute_with(|| {
        assert_noop!(
            _update_faucet_settings(
                FaucetUpdate {
                    enabled: None,
                    period: None,
                    period_limit: Some(default_faucet().period_limit),
                    drip_limit: None
                }
            ),
            Error::<Test>::NothingToUpdate
        );
    });
}

#[test]
fn update_faucet_should_fail_when_same_drip_limit_provided() {
    ExtBuilder::build_with_faucet().execute_with(|| {
        assert_noop!(
            _update_faucet_settings(
                FaucetUpdate {
                    enabled: None,
                    period: None,
                    period_limit: None,
                    drip_limit: Some(default_faucet().drip_limit)
                }
            ),
            Error::<Test>::NothingToUpdate
        );
    });
}

#[test]
fn update_faucet_should_fail_when_new_period_limit_below_drip_limit() {
    ExtBuilder::build_with_faucet().execute_with(|| {
        assert_noop!(
            _update_faucet_settings(
                FaucetUpdate {
                    enabled: None,
                    period: None,
                    period_limit: Some(default_faucet().drip_limit - 1),
                    drip_limit: None
                }
            ),
            Error::<Test>::DripLimitCannotExceedPeriodLimit
        );
    });
}

#[test]
fn update_faucet_should_fail_when_new_drip_limit_exceeds_period_limit() {
    ExtBuilder::build_with_faucet().execute_with(|| {
        assert_noop!(
            _update_faucet_settings(
                FaucetUpdate {
                    enabled: None,
                    period: None,
                    period_limit: None,
                    drip_limit: Some(default_faucet().period_limit + 1)
                }
            ),
            Error::<Test>::DripLimitCannotExceedPeriodLimit
        );
    });
}

// Remove faucets
// ----------------------------------------------------------------------------

#[test]
fn remove_faucets_should_work() {
    ExtBuilder::build().execute_with(|| {
        // This will add faucets with accounts ids [1; 8]
        let mut faucets = Vec::new();
        for account in FAUCET1..=FAUCET8 {
            assert_ok!(_add_faucet(None, Some(account), None));
            faucets.push(account);
        }

        // This should remove only faucets from 1 to 7
        let _ = faucets.pop();
        assert_ok!(
            _remove_faucets(
                None,
                Some(faucets)
            )
        );

        for account in FAUCET1..FAUCET8 {
            assert!(Faucets::faucet_by_account(account).is_none());
        }
        assert!(Faucets::faucet_by_account(FAUCET8).is_some());
    });
}

#[test]
fn remove_faucets_should_handle_duplicate_addresses() {
    ExtBuilder::build().execute_with(|| {
        // This will add faucets with accounts ids [1; 8]
        let mut faucets = Vec::new();
        for account in FAUCET1..=FAUCET8 {
            assert_ok!(_add_faucet(None, Some(account), None));
            faucets.push(account);
        }

        // This should remove only faucets from 1 to 7
        let _ = faucets.pop();
        let mut duplicates = vec![FAUCET1, FAUCET2];
        faucets.append(&mut duplicates);
        assert_ok!(
            _remove_faucets(
                None,
                Some(faucets)
            )
        );

        for account in FAUCET1..FAUCET8 {
            assert!(Faucets::faucet_by_account(account).is_none());
        }
        assert!(Faucets::faucet_by_account(FAUCET8).is_some());
    });
}

#[test]
fn remove_faucets_should_fail_when_no_faucet_addresses_provided() {
    ExtBuilder::build_with_faucet().execute_with(|| {
        assert_noop!(
            _remove_faucets(
                None,
                Some(vec![])
            ),
            Error::<Test>::NoFaucetsProvided
        );
    });
}

// Drip
// ----------------------------------------------------------------------------

#[test]
fn drip_should_work() {
    ExtBuilder::build_with_faucet().execute_with(|| {
        System::set_block_number(INITIAL_BLOCK_NUMBER);

        let Faucet { period, drip_limit, .. } = default_faucet();
        assert_eq!(Balances::free_balance(ACCOUNT1), 0);

        assert_ok!(_do_default_drip());

        assert_eq!(Balances::free_balance(ACCOUNT1), drip_limit);

        let faucet_state = Faucets::faucet_by_account(FAUCET1).unwrap();
        assert_eq!(faucet_state.next_period_at, INITIAL_BLOCK_NUMBER + period);
        assert_eq!(faucet_state.dripped_in_current_period, drip_limit);
    });
}

#[test]
fn drip_should_work_multiple_times_in_same_period() {
    ExtBuilder::build_with_one_default_drip().execute_with(|| {
        let Faucet { period, drip_limit, .. } = default_faucet();
        
        // Do the second drip
        assert_ok!(_drip(None, None, Some(drip_limit)));
        assert_eq!(Balances::free_balance(ACCOUNT1), drip_limit * 2);

        let faucet_state = Faucets::faucet_by_account(FAUCET1).unwrap();
        assert_eq!(faucet_state.next_period_at, INITIAL_BLOCK_NUMBER + period);
        assert_eq!(faucet_state.dripped_in_current_period, drip_limit * 2);
    });
}

#[test]
fn drip_should_work_for_same_recipient_in_next_period() {
    ExtBuilder::build_with_faucet().execute_with(|| {
        System::set_block_number(INITIAL_BLOCK_NUMBER);

        let Faucet { period, drip_limit, .. } = default_faucet();

        // Drip to the same recipient twice in the same period to reach perdiod limit
        assert_ok!(_do_default_drip());
        assert_ok!(_do_default_drip());
        assert_eq!(Balances::free_balance(ACCOUNT1), drip_limit * 2);

        // Move to the next period
        System::set_block_number(INITIAL_BLOCK_NUMBER + period);

        // Repeat the same drip as we did a few line above but now it will be in the next period
        assert_ok!(_do_default_drip());
        assert_eq!(Balances::free_balance(ACCOUNT1), drip_limit * 3);

        let faucet_state = Faucets::faucet_by_account(FAUCET1).unwrap();
        assert_eq!(faucet_state.next_period_at, INITIAL_BLOCK_NUMBER + period * 2);
        assert_eq!(faucet_state.dripped_in_current_period, drip_limit);
    });
}

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
fn drip_should_fail_when_recipient_equals_faucet() {
    ExtBuilder::build_with_faucet().execute_with(|| {
        assert_noop!(
            _drip(None, Some(FAUCET1), None),
            Error::<Test>::RecipientEqualsFaucet
        );
        
        // Account should have no tokens if drip failed
        assert_eq!(Balances::free_balance(ACCOUNT1), 0);
    });
}

#[test]
fn drip_should_fail_when_amount_is_bigger_than_free_balance_on_faucet() {
    ExtBuilder::build_with_faucet().execute_with(|| {

        // Let's transfer most of tokens from the default faucet to another one
        assert_ok!(Balances::transfer(
            Origin::signed(FAUCET1),
            FAUCET2,
            FAUCET_INITIAL_BALANCE - 1 // Leave one token on the Faucet number 1.
        ));

        assert_noop!(
            _do_default_drip(),
            Error::<Test>::NotEnoughFreeBalanceOnFaucet
        );
        
        // Account should have no tokens if drip failed
        assert_eq!(Balances::free_balance(ACCOUNT1), 0);
    });
}

#[test]
fn drip_should_fail_when_zero_amount_provided() {
    ExtBuilder::build_with_faucet().execute_with(|| {
        assert_noop!(
            _drip(None, None, Some(0)),
            Error::<Test>::ZeroDripAmountProvided
        );
        
        // Account should have no tokens if drip failed
        assert_eq!(Balances::free_balance(ACCOUNT1), 0);
    });
}

#[test]
fn drip_should_fail_when_too_big_amount_provided() {
    ExtBuilder::build_with_faucet().execute_with(|| {
        let too_big_amount = default_faucet().drip_limit + 1;
        assert_noop!(
            _drip(None, None, Some(too_big_amount)),
            Error::<Test>::DripLimitReached
        );

        // Account should have no tokens if drip failed
        assert_eq!(Balances::free_balance(ACCOUNT1), 0);
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
