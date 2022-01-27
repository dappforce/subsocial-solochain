use frame_benchmarking::account;
use crate::{mock::*, LockedInfoByAccount, BalanceOf};
use frame_support::{assert_ok, assert_noop, assert_err};
use sp_io::KillStorageResult::AllRemoved;
use sp_runtime::DispatchError::BadOrigin;

#[test]
fn set_locked_info__will_fail_when_not_manager_origin() {
    new_test_ext().execute_with(|| {
        let caller = account("Test Account", 1 ,4);
        let subject = account("Test Account", 1, 4);
        assert_err!(
            LockerMirror::set_locked_info(
                Origin::signed(caller),
                subject,
                1_000_000_000_000u64.into(),
                11u32.into(),
                23u32.into(),
            ),
            BadOrigin,
        );
    });
}

#[test]
fn clear_locked_info__will_fail_when_not_manager_origin() {
    new_test_ext().execute_with(|| {
        let caller = account("Test Account", 1 ,4);
        let subject = account("Test Account2", 2, 5);
        assert_err!(
            LockerMirror::clear_locked_info(
                Origin::signed(caller),
                subject,
            ),
            BadOrigin,
        );
    });
}