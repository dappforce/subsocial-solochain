// Tests to be written here

use crate::{Error, mock::*, FaucetSettings, FaucetSettingsUpdate};
use frame_support::{assert_ok, assert_noop};
use sp_runtime::DispatchError::BadOrigin;

#[test]
fn add_faucet_should_work() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_add_default_faucet());

        let faucet_settings = Faucet::faucet_settings_by_account(FAUCET1).unwrap();
        assert_eq!(faucet_settings, default_faucet_settings());
    });
}

#[test]
fn add_faucet_should_fail_bad_origin() {
    ExtBuilder::build().execute_with(|| {
        assert_noop!(
            _add_faucet(
                Some(Origin::signed(FAUCET1)),
                None,
                None
            ),
            BadOrigin
        );
    });
}

#[test]
fn add_faucet_should_fail_with_faucet_already_exists() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_add_default_faucet());

        assert_noop!(
            _add_default_faucet(),
            Error::<Test>::FaucetAlreadyAdded
        );
    });
}

#[test]
fn add_faucet_should_fail_with_no_free_balance_on_account() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_add_default_faucet());

        assert_noop!(
            _add_faucet(
                None,
                Some(FAUCET9),
                None
            ),
            Error::<Test>::NoFreeBalanceOnAccount
        );
    });
}

#[test]
fn update_faucet_should_work() {
    ExtBuilder::build_with_faucet().execute_with(|| {
        assert_ok!(_update_default_faucet());
        const SETTINGS_UPDATE: FaucetSettingsUpdate<BlockNumber, Balance>
            = default_faucet_settings_update();

        let faucet_settings = Faucet::faucet_settings_by_account(FAUCET1).unwrap();
        let updated_faucet_settings = FaucetSettings {
            period: SETTINGS_UPDATE.period.unwrap_or(faucet_settings.period),
            period_limit: SETTINGS_UPDATE.period_limit.unwrap_or(faucet_settings.period_limit)
        };

        assert_eq!(faucet_settings.period, updated_faucet_settings.period);
        assert_eq!(faucet_settings.period_limit, updated_faucet_settings.period_limit);
    });
}

#[test]
fn update_faucet_should_fail_with_nothing_to_update() {
    ExtBuilder::build_with_faucet().execute_with(|| {
        assert_noop!(
            _update_faucet(
                None,
                None,
                Some(FaucetSettingsUpdate {
                    period: None,
                    period_limit: None
                })
            ),
            Error::<Test>::NothingToUpdate
        );
    });
}

#[test]
fn update_faucet_should_fail_with_faucet_not_found() {
    ExtBuilder::build().execute_with(|| {
        assert_noop!(
            _update_default_faucet(),
            Error::<Test>::FaucetNotFound
        );
    });
}

#[test]
fn update_faucet_should_fail_with_period_not_differ() {
    ExtBuilder::build_with_faucet().execute_with(|| {
        assert_noop!(
            _update_faucet(
                None,
                None,
                Some(FaucetSettingsUpdate {
                    period: Some(default_faucet_settings().period),
                    period_limit: None
                })
            ),
            Error::<Test>::NothingToUpdate
        );
    });
}

#[test]
fn update_faucet_should_fail_with_period_limit_not_differ() {
    ExtBuilder::build_with_faucet().execute_with(|| {
        assert_noop!(
            _update_faucet(
                None,
                None,
                Some(FaucetSettingsUpdate {
                    period: None,
                    period_limit: Some(default_faucet_settings().period_limit)
                })
            ),
            Error::<Test>::NothingToUpdate
        );
    });
}

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
            assert!(Faucet::faucet_settings_by_account(account).is_none());
        }
        assert!(Faucet::faucet_settings_by_account(FAUCET8).is_some());
    });
}

#[test]
fn remove_faucets_should_work_with_duplicates() {
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
            assert!(Faucet::faucet_settings_by_account(account).is_none());
        }
        assert!(Faucet::faucet_settings_by_account(FAUCET8).is_some());
    });
}

#[test]
fn remove_faucets_should_fail_no_faucets_specified() {
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

#[test]
fn drip_should_work() {
    ExtBuilder::build_with_faucet().execute_with(|| {
        System::set_block_number(INITIAL_BLOCK_NUMBER);

        assert_ok!(_default_drip()); // DropId 1

        assert_eq!(Faucet::next_drop_id(), DROP2);
        assert_eq!(Faucet::drop_id_by_recipient(ACCOUNT1), Some(DROP1));
        assert_eq!(Faucet::drop_id_by_alias(valid_sha_1()), Some(DROP1));

        let drop = Faucet::drop_by_id(DROP1).unwrap();
        assert_eq!(drop.id, DROP1);
        assert_eq!(drop.last_drop_at, INITIAL_BLOCK_NUMBER);
        assert_eq!(drop.total_dropped, default_faucet_settings().period_limit);
    });
}

#[test]
fn drip_should_work_twice() {
    ExtBuilder::build_with_faucet().execute_with(|| {
        System::set_block_number(INITIAL_BLOCK_NUMBER);

        assert_ok!(_default_drip()); // DropId 1

        // Default faucet must have a limit for testing purposes
        let period = default_faucet_settings().period.unwrap();
        System::set_block_number(INITIAL_BLOCK_NUMBER + period);

        assert_ok!(_default_drip()); // Should reuse DropId 1

        let drop = Faucet::drop_by_id(DROP1).unwrap();
        assert_eq!(drop.id, DROP1);
        assert_eq!(drop.last_drop_at, INITIAL_BLOCK_NUMBER + period);
        assert_eq!(drop.total_dropped, default_faucet_settings().period_limit);
    });
}

#[test]
fn drip_should_work_with_two_drips() {
    ExtBuilder::build_with_partial_drip().execute_with(|| {
        assert_ok!(_default_drip());

        assert_eq!(Faucet::next_drop_id(), DROP2);
        assert_eq!(Faucet::drop_id_by_recipient(ACCOUNT1), Some(DROP1));
        assert_eq!(Faucet::drop_id_by_alias(valid_sha_1()), Some(DROP1));

        let drop = Faucet::drop_by_id(DROP1).unwrap();
        assert_eq!(drop.id, DROP1);
        assert_eq!(drop.last_drop_at, INITIAL_BLOCK_NUMBER);
        assert_eq!(drop.total_dropped, 2);
    });
}

#[test]
fn drip_should_fail_zero_amount() {
    ExtBuilder::build_with_faucet().execute_with(|| {
        assert_noop!(
            _drip(None, Some(0), None, None),
            Error::<Test>::ZeroAmount
        );
    });
}

#[test]
fn drip_should_fail_faucet_limit_reached_with_same_recipient() {
    ExtBuilder::build_with_faucet().execute_with(|| {
        System::set_block_number(INITIAL_BLOCK_NUMBER);

        assert_ok!(_default_drip()); // Drip to ACCOUNT1 with `valid_sha_1` alias
        assert_noop!(
            _drip(None, None, None, Some(self::valid_sha_2())),
            Error::<Test>::FaucetLimitReached
        );
    });
}

#[test]
fn drip_should_fail_faucet_limit_reached_with_same_alias() {
    ExtBuilder::build_with_faucet().execute_with(|| {
        System::set_block_number(INITIAL_BLOCK_NUMBER);

        assert_ok!(_default_drip()); // Drip to ACCOUNT1 with `valid_sha_1` alias
        assert_noop!(
            _drip(None, None, Some(ACCOUNT2), None),
            Error::<Test>::FaucetLimitReached
        );
    });
}

#[test]
fn drip_should_fail_faucet_limit_reached_with_three_drips() {
    ExtBuilder::build().execute_with(|| {
        let mut faucet_settings = default_faucet_settings();
        faucet_settings.period_limit = 2;
        assert_ok!(
            _add_faucet(
                None,
                None,
                Some(faucet_settings)
            )
        );

        assert_ok!(_default_drip());
        assert_ok!(_default_drip());
        assert_noop!(
            _default_drip(),
            Error::<Test>::FaucetLimitReached
        );
    });
}