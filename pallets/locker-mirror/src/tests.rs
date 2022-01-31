#![allow(non_snake_case)]
use frame_benchmarking::account;
use crate::{mock::*, LockedInfoByAccount, BalanceOf, LockedInfo, Config, LockedInfoOf};
use frame_support::{assert_ok, assert_noop, assert_err};
use frame_system::pallet_prelude::OriginFor;
use rand::Rng;
use sp_runtime::DispatchError::BadOrigin;
use subsocial_primitives::Balance;


fn non_root_caller_origin<T: Config>() -> Origin {
    Origin::signed(account("Caller", 0, 0))
}

fn root_caller_origin<T: Config>() -> Origin {
    Origin::root()
}

fn subject_account<T: Config>() -> T::AccountId {
    account("Subject", 1, 1)
}

fn subject_account_n<T: Config>(n: u32) -> T::AccountId {
    account("Subject N", 2 + n, 2 + n)
}

fn random_locked_info() -> LockedInfoOf<Test> {
    let mut rng = rand::thread_rng();
    LockedInfoOf::<Test> {
        locked_amount: rng.gen_range(0..BalanceOf::<Test>::max_value()).into(),
        unlocks_at: rng.gen_range(0..<Test as frame_system::Config>::BlockNumber::max_value()).into(),
        lock_period: rng.gen_range(0..<Test as frame_system::Config>::BlockNumber::max_value()).into(),
    }
}

#[test]
fn set_locked_info__should_fail_when_not_manager_origin() {
    new_test_ext().execute_with(|| {
        assert_err!(
            LockerMirror::set_locked_info(
                non_root_caller_origin::<Test>(),
                subject_account::<Test>(),
                random_locked_info(),
            ),
            BadOrigin,
        );
    });
}

#[test]
fn set_locked_info__should_ok_when_caller_is_manager() {
    new_test_ext().execute_with(|| {
        assert_ok!(
            LockerMirror::set_locked_info(
                root_caller_origin::<Test>(),
                subject_account::<Test>(),
                random_locked_info(),
            ),
        );
    });
}

#[test]
fn set_locked_info__should_change_storage_for_the_subject_account() {
    new_test_ext().execute_with(|| {
        assert_eq!(<LockedInfoByAccount<Test>>::iter().count(), 0);
        let expected_locked_info = random_locked_info();
        assert_ok!(
            LockerMirror::set_locked_info(
                root_caller_origin::<Test>(),
                subject_account::<Test>(),
                expected_locked_info.clone(),
            ),
        );
        assert_eq!(<LockedInfoByAccount<Test>>::iter().count(), 1);
        let (_,found_locked_info) = <LockedInfoByAccount<Test>>::iter().next().unwrap();
        assert_eq!(found_locked_info, expected_locked_info);
    });
}

#[test]
fn clear_locked_info__should_fail_when_not_manager_origin() {
    new_test_ext().execute_with(|| {
        assert_err!(
            LockerMirror::clear_locked_info(
                non_root_caller_origin::<Test>(),
                subject_account::<Test>(),
            ),
            BadOrigin,
        );
    });
}

#[test]
fn clear_locked_info__should_ok_when_caller_is_manager() {
    new_test_ext().execute_with(|| {
        assert_ok!(
            LockerMirror::clear_locked_info(
                root_caller_origin::<Test>(),
                subject_account::<Test>(),
            ),
        );
    });
}


#[test]
fn clear_locked_info__should_remove_from_storage() {
    new_test_ext().execute_with(|| {
        assert_eq!(<LockedInfoByAccount<Test>>::iter().count(), 0);
        <LockedInfoByAccount<Test>>::insert(subject_account_n::<Test>(1), random_locked_info());
        <LockedInfoByAccount<Test>>::insert(subject_account_n::<Test>(2), random_locked_info());
        <LockedInfoByAccount<Test>>::insert(subject_account_n::<Test>(3), random_locked_info());
        <LockedInfoByAccount<Test>>::insert(subject_account_n::<Test>(4), random_locked_info());
        assert_eq!(<LockedInfoByAccount<Test>>::iter().count(), 4);

    });
}