#![allow(non_snake_case)]
use frame_benchmarking::account;
use crate::{mock::*, LockedInfoByAccount, BalanceOf, LockedInfo, Config};
use frame_support::{assert_ok, assert_noop, assert_err};
use sp_runtime::DispatchError::BadOrigin;


fn caller_account<T: Config>() -> T::AccountId {
    account("Caller", 0, 0)
}

fn subject_account<T: Config>() -> T::AccountId {
    account("Subject", 1, 1)
}

#[test]
fn set_locked_info__should_fail_when_not_manager_origin() {
    new_test_ext().execute_with(|| {
        assert_err!(
            LockerMirror::set_locked_info(
                Origin::signed(caller_account::<Test>()),
                subject_account::<Test>(),
                LockedInfo::<<Test as frame_system::Config>::BlockNumber, BalanceOf<Test>> {
                    locked_amount: 1_000_000_000_000u64.into(),
                    unlocks_at: 11u32.into(),
                    lock_period: 23u32.into(),
                }
            ),
            BadOrigin,
        );
    });
}

#[test]
fn clear_locked_info__should_fail_when_not_manager_origin() {
    new_test_ext().execute_with(|| {
        assert_err!(
            LockerMirror::clear_locked_info(
                Origin::signed(caller_account::<Test>()),
                subject_account::<Test>(),
            ),
            BadOrigin,
        );
    });
}
