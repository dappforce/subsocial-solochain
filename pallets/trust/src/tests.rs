use crate::{mock::*, TrustLevels};
use frame_support::{assert_ok, assert_noop};
use sp_runtime::DispatchError::BadOrigin;

// Test `fn set_email_verified(..)`

#[test]
fn set_email_verified_should_work() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_set_email_verified_by_account1());
        assert_ok!(_set_email_verified_by_account2());

        let is_account1_email_verified =
            Trust::account_trust_levels_contains(&ACCOUNT1, TrustLevels::EMAIL_VERIFIED);
        let is_account2_email_verified =
            Trust::account_trust_levels_contains(&ACCOUNT1, TrustLevels::EMAIL_VERIFIED);

        assert_eq!(is_account1_email_verified, true);
        assert_eq!(is_account2_email_verified, true);
    })
}

#[test]
fn set_email_verified_should_fail() {
    ExtBuilder::build().execute_with(|| {
        assert_noop!(_set_email_verified_by_account1_bad_origin(), BadOrigin);
    })
}

// Test `fn set_phone_number_verified(..)`

#[test]
fn set_phone_number_verified_should_work() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_set_phone_number_verified_by_account1());
        assert_ok!(_set_phone_number_verified_by_account2());

        let is_account1_phone_number_verified =
            Trust::account_trust_levels_contains(&ACCOUNT1, TrustLevels::PHONE_NUMBER_VERIFIED);
        let is_account2_phone_number_verified =
            Trust::account_trust_levels_contains(&ACCOUNT1, TrustLevels::PHONE_NUMBER_VERIFIED);

        assert_eq!(is_account1_phone_number_verified, true);
        assert_eq!(is_account2_phone_number_verified, true);
    })
}

#[test]
fn set_phone_number_verified_should_fail() {
    ExtBuilder::build().execute_with(|| {
        assert_noop!(_set_phone_number_verified_by_account1_bad_origin(), BadOrigin);
    })
}
