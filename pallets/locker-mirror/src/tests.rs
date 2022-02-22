#![allow(non_snake_case)]
use frame_benchmarking::account;
use crate::{mock::*, LockedInfoByAccount, BalanceOf, Config, LockedInfoOf, ProcessedEventInfo, LastProcessedParachainEvent};
use frame_support::{assert_ok, assert_err};
use frame_support::dispatch::DispatchResultWithPostInfo;
use frame_support::weights::{Pays, PostDispatchInfo};
use rand::Rng;
use sp_runtime::DispatchError::BadOrigin;
use sp_runtime::DispatchErrorWithPostInfo;


fn extract_post_info(result: DispatchResultWithPostInfo) -> PostDispatchInfo {
    let post_info = match result {
        Ok(post_info) => post_info,
        Err(DispatchErrorWithPostInfo { post_info, ..}) => post_info,
    };

    post_info
}

fn subject_account<T: Config>() -> T::AccountId {
    account("Subject", 0, 0)
}

fn subject_account_n<T: Config>(n: u32) -> T::AccountId {
    assert_ne!(n, 0);
    account("Subject N", n, n)
}

fn random_subject_account<T: Config>() -> T::AccountId {
    let mut rng = rand::thread_rng();
    subject_account_n::<T>(rng.gen_range(1..1024))
}

fn random_locked_info() -> LockedInfoOf<Test> {
    let mut rng = rand::thread_rng();
    LockedInfoOf::<Test> {
        locked_amount: rng.gen_range(0..BalanceOf::<Test>::max_value()).into(),
        locked_at: rng.gen_range(0..<Test as frame_system::Config>::BlockNumber::max_value()).into(),
        expires_at: Some(rng.gen_range(0..<Test as frame_system::Config>::BlockNumber::max_value()).into()),
    }
}


fn random_parachain_event() -> ProcessedEventInfo {
    let mut rng = rand::thread_rng();
    ProcessedEventInfo {
        event_index: rng.gen(),
        block_number: rng.gen(),
    }
}

////////////////

#[test]
fn set_last_processed_parachain_event__should_fail_when_unsigned() {
    ExtBuilder::default()
        .build().execute_with(|| {
        assert_err!(
            LockerMirror::set_locked_info(
                Origin::none(),
                subject_account::<Test>(),
                random_locked_info(),
            ),
            BadOrigin,
        );
    });
}

#[test]
fn set_last_processed_parachain_event__should_fail_when_not_oracle_origin() {
    let oracle = subject_account_n::<Test>(1);
    let non_oracle = subject_account_n::<Test>(2);
    ExtBuilder::default()
        .oracle_account_id(oracle.clone())
        .build().execute_with(|| {
        assert_err!(
            LockerMirror::set_last_processed_parachain_event(
                Origin::signed(non_oracle.clone()),
                random_parachain_event(),
            ),
            BadOrigin,
        );
    });
}

#[test]
fn set_last_processed_parachain_event__should_ok_when_caller_is_oracle() {
    let oracle = subject_account_n::<Test>(1);
    let non_oracle = subject_account_n::<Test>(2);
    ExtBuilder::default()
        .oracle_account_id(oracle.clone())
        .build().execute_with(|| {
        assert_ok!(
            LockerMirror::set_last_processed_parachain_event(
                Origin::signed(oracle.clone()),
                random_parachain_event(),
            ),
        );
    });
}

#[test]
fn set_last_processed_parachain_event__should_pay_when_caller_is_not_oracle() {
    let oracle = subject_account_n::<Test>(1);
    let non_oracle = subject_account_n::<Test>(2);
    ExtBuilder::default()
        .oracle_account_id(oracle.clone())
        .build().execute_with(|| {
        let res = LockerMirror::set_last_processed_parachain_event(
            Origin::signed(non_oracle.clone()),
            random_parachain_event(),
        );
        assert_err!(
            res,
            BadOrigin,
        );

        assert!(extract_post_info(res).pays_fee == Pays::Yes);
    });
}


#[test]
fn set_last_processed_parachain_event__should_not_pay_when_caller_is_oracle() {
    let oracle = subject_account_n::<Test>(1);
    let non_oracle = subject_account_n::<Test>(2);
    ExtBuilder::default()
        .oracle_account_id(oracle.clone())
        .build().execute_with(|| {
        let res = LockerMirror::set_last_processed_parachain_event(
            Origin::signed(oracle.clone()),
            random_parachain_event(),
        );
        assert_ok!(res);

        assert!(extract_post_info(res).pays_fee == Pays::No);
    });
}

#[test]
fn set_last_processed_parachain_event__should_change_storage() {
    let oracle = subject_account_n::<Test>(1);
    let non_oracle = subject_account_n::<Test>(2);
    ExtBuilder::default()
        .oracle_account_id(oracle.clone())
        .build().execute_with(|| {
        assert_eq!(<LastProcessedParachainEvent<Test>>::get(), None);
        let expected_event_info = random_parachain_event();
        assert_ok!(
            LockerMirror::set_last_processed_parachain_event(
                Origin::signed(oracle.clone()),
                expected_event_info.clone(),
            ),
        );
        assert_eq!(<LastProcessedParachainEvent<Test>>::get().unwrap(), expected_event_info);
    });
}


///////////////

#[test]
fn set_locked_info__should_fail_when_unsigned() {
    ExtBuilder::default()
        .build().execute_with(|| {
        assert_err!(
            LockerMirror::set_locked_info(
                Origin::none(),
                subject_account::<Test>(),
                random_locked_info(),
            ),
            BadOrigin,
        );
    });
}

#[test]
fn set_locked_info__should_fail_when_not_oracle_origin() {
    let oracle = subject_account_n::<Test>(1);
    let non_oracle = subject_account_n::<Test>(2);
    ExtBuilder::default()
        .oracle_account_id(oracle.clone())
        .build().execute_with(|| {
        assert_err!(
            LockerMirror::set_locked_info(
                Origin::signed(non_oracle.clone()),
                subject_account::<Test>(),
                random_locked_info(),
            ),
            BadOrigin,
        );
    });
}

#[test]
fn set_locked_info__should_ok_when_caller_is_oracle() {
    let oracle = subject_account_n::<Test>(1);
    let non_oracle = subject_account_n::<Test>(2);
    ExtBuilder::default()
        .oracle_account_id(oracle.clone())
        .build().execute_with(|| {
        assert_ok!(
            LockerMirror::set_locked_info(
                Origin::signed(oracle.clone()),
                subject_account::<Test>(),
                random_locked_info(),
            ),
        );
    });
}

#[test]
fn set_locked_info__should_pay_when_caller_is_not_oracle() {
    let oracle = subject_account_n::<Test>(1);
    let non_oracle = subject_account_n::<Test>(2);
    ExtBuilder::default()
        .oracle_account_id(oracle.clone())
        .build().execute_with(|| {
        let res = LockerMirror::set_locked_info(
            Origin::signed(non_oracle.clone()),
            subject_account::<Test>(),
            random_locked_info(),
        );
        assert_err!(
            res,
            BadOrigin,
        );

        assert!(extract_post_info(res).pays_fee == Pays::Yes);
    });
}


#[test]
fn set_locked_info__should_not_pay_when_caller_is_oracle() {
    let oracle = subject_account_n::<Test>(1);
    let non_oracle = subject_account_n::<Test>(2);
    ExtBuilder::default()
        .oracle_account_id(oracle.clone())
        .build().execute_with(|| {
        let res = LockerMirror::set_locked_info(
            Origin::signed(oracle.clone()),
            subject_account::<Test>(),
            random_locked_info(),
        );
        assert_ok!(res);

        assert!(extract_post_info(res).pays_fee == Pays::No);
    });
}

#[test]
fn set_locked_info__should_change_storage_for_the_subject_account() {
    let oracle = subject_account_n::<Test>(1);
    let non_oracle = subject_account_n::<Test>(2);
    ExtBuilder::default()
        .oracle_account_id(oracle.clone())
        .build().execute_with(|| {
        assert_eq!(<LockedInfoByAccount<Test>>::iter().count(), 0);
        let expected_locked_info = random_locked_info();
        assert_ok!(
            LockerMirror::set_locked_info(
                Origin::signed(oracle.clone()),
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
fn clear_locked_info__should_fail_when_unsigned() {
    ExtBuilder::default()
        .build().execute_with(|| {
        assert_err!(
            LockerMirror::clear_locked_info(
                Origin::none(),
                subject_account::<Test>(),
            ),
            BadOrigin,
        );
    });
}

#[test]
fn clear_locked_info__should_fail_when_not_oracle_origin() {
    let oracle = subject_account_n::<Test>(1);
    let non_oracle = subject_account_n::<Test>(2);
    ExtBuilder::default()
        .oracle_account_id(oracle.clone())
        .build().execute_with(|| {
        assert_err!(
            LockerMirror::clear_locked_info(
                Origin::signed(non_oracle.clone()),
                subject_account::<Test>(),
            ),
            BadOrigin,
        );
    });
}

#[test]
fn clear_locked_info__should_ok_when_caller_is_oracle() {
    let oracle = subject_account_n::<Test>(1);
    let non_oracle = subject_account_n::<Test>(2);
    ExtBuilder::default()
        .oracle_account_id(oracle.clone())
        .build().execute_with(|| {
        assert_ok!(
            LockerMirror::clear_locked_info(
                Origin::signed(oracle.clone()),
                subject_account::<Test>(),
            ),
        );
    });
}

#[test]
fn clear_locked_info__should_pay_when_caller_is_not_oracle() {
    let oracle = subject_account_n::<Test>(1);
    let non_oracle = subject_account_n::<Test>(2);
    ExtBuilder::default()
        .oracle_account_id(oracle.clone())
        .build().execute_with(|| {
        let res = LockerMirror::clear_locked_info(
            Origin::signed(non_oracle.clone()),
            subject_account::<Test>(),
        );
        assert_err!(
            res,
            BadOrigin,
        );

        assert!(extract_post_info(res).pays_fee == Pays::Yes);
    });
}


#[test]
fn clear_locked_info__should_not_pay_when_caller_is_oracle() {
    let oracle = subject_account_n::<Test>(1);
    let non_oracle = subject_account_n::<Test>(2);
    ExtBuilder::default()
        .oracle_account_id(oracle.clone())
        .build().execute_with(|| {
        let res = LockerMirror::clear_locked_info(
            Origin::signed(oracle.clone()),
            subject_account::<Test>(),
        );
        assert_ok!(res);

        assert!(extract_post_info(res).pays_fee == Pays::No);
    });
}

#[test]
fn clear_locked_info__should_clear_storage() {
    let oracle = subject_account_n::<Test>(1);
    let non_oracle = subject_account_n::<Test>(2);
    ExtBuilder::default()
        .oracle_account_id(oracle.clone())
        .build().execute_with(|| {
        assert_eq!(<LockedInfoByAccount<Test>>::iter().count(), 0);

        assert_ok!(
            LockerMirror::clear_locked_info(
                Origin::signed(oracle.clone()),
                subject_account_n::<Test>(11),
            ),
        );
        // nothing is changed
        assert_eq!(<LockedInfoByAccount<Test>>::iter().count(), 0);

        let account = subject_account::<Test>();
        let info = random_locked_info();

        <LockedInfoByAccount<Test>>::insert(account.clone(), info.clone());
        assert_eq!(<LockedInfoByAccount<Test>>::iter().count(), 1);

        assert_ok!(
            LockerMirror::clear_locked_info(
                Origin::signed(oracle.clone()),
                subject_account_n::<Test>(12),
            ),
        );
        // nothing will change
        assert_eq!(<LockedInfoByAccount<Test>>::iter().count(), 1);

        assert_ok!(
            LockerMirror::clear_locked_info(
                Origin::signed(oracle.clone()),
                subject_account::<Test>(),
            ),
        );
        // now since the account is found, the storage is cleared
        assert_eq!(<LockedInfoByAccount<Test>>::iter().count(), 0);
    });
}

fn compare_ignore_order<T: PartialEq>(a: &Vec<T>, b: &Vec<T>) -> bool {
    if a.len() != b.len() {
        return false;
    }

    for item_a in a {
        if !b.contains(item_a) {
            return false;
        }
    }

    return true;
}

#[test]
fn sequence_of_set_clear() {
    let oracle = subject_account_n::<Test>(1);
    let non_oracle = subject_account_n::<Test>(2);
    ExtBuilder::default()
        .oracle_account_id(oracle.clone())
        .build().execute_with(|| {
        assert_eq!(<LockedInfoByAccount<Test>>::iter().count(), 0);

        let mut expected = vec![
            (subject_account_n::<Test>(1), random_locked_info()),
            (subject_account_n::<Test>(2), random_locked_info()),
            (subject_account_n::<Test>(3), random_locked_info()),
            (subject_account_n::<Test>(4), random_locked_info()),
        ];

        for (account, info) in expected.iter() {
            assert_ok!(
                LockerMirror::set_locked_info(
                    Origin::signed(oracle.clone()),
                    account.clone(),
                    info.clone(),
                ),
            );
        }

        assert_eq!(<LockedInfoByAccount<Test>>::iter().count(), 4);
        let infos: Vec<_> = <LockedInfoByAccount<Test>>::iter().collect();


        assert!(compare_ignore_order(&infos, &expected));


        // nothing should happen since account 55 don't have any locked_info
        assert_ok!(
            LockerMirror::clear_locked_info(
                Origin::signed(oracle.clone()),
                subject_account_n::<Test>(55),
            ),
        );
        let infos: Vec<_> = <LockedInfoByAccount<Test>>::iter().collect();
        assert!(compare_ignore_order(&infos, &expected));



        // remove account 4
        assert_ok!(
            LockerMirror::clear_locked_info(
                Origin::signed(oracle.clone()),
                subject_account_n::<Test>(4),
            ),
        );
        let infos: Vec<_> = <LockedInfoByAccount<Test>>::iter().collect();
        assert!(!compare_ignore_order(&infos, &expected));
        expected.retain(|(account, _)| account != &subject_account_n::<Test>(4));
        assert!(compare_ignore_order(&infos, &expected));

        // nothing should happen since account 1312 don't have any locked_info
        assert_ok!(
            LockerMirror::clear_locked_info(
                Origin::signed(oracle.clone()),
                subject_account_n::<Test>(1312),
            ),
        );
        let infos: Vec<_> = <LockedInfoByAccount<Test>>::iter().collect();
        assert!(compare_ignore_order(&infos, &expected));


        // remove account 1
        assert_ok!(
            LockerMirror::clear_locked_info(
                Origin::signed(oracle.clone()),
                subject_account_n::<Test>(1),
            ),
        );
        let infos: Vec<_> = <LockedInfoByAccount<Test>>::iter().collect();
        assert!(!compare_ignore_order(&infos, &expected));
        expected.retain(|(account, _)| account != &subject_account_n::<Test>(1));
        assert!(compare_ignore_order(&infos, &expected));


        // Add a new account
        let acc_221122 = subject_account_n::<Test>(221122);
        let acc_221122_info = random_locked_info();
        expected.push((acc_221122.clone(), acc_221122_info.clone()));
        assert_ok!(
            LockerMirror::set_locked_info(
                Origin::signed(oracle.clone()),
                acc_221122,
                acc_221122_info.clone(),
            ),
        );
        let infos: Vec<_> = <LockedInfoByAccount<Test>>::iter().collect();
        assert!(compare_ignore_order(&infos, &expected));


        // remove account 221122
        assert_ok!(
            LockerMirror::clear_locked_info(
                Origin::signed(oracle.clone()),
                subject_account_n::<Test>(221122),
            ),
        );
        let infos: Vec<_> = <LockedInfoByAccount<Test>>::iter().collect();
        assert!(!compare_ignore_order(&infos, &expected));
        expected.retain(|(account, _)| account != &subject_account_n::<Test>(221122));
        assert!(compare_ignore_order(&infos, &expected));
    });
}