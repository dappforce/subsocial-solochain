#![allow(non_snake_case)]
use frame_benchmarking::account;
use crate::{mock::*, LockedInfoByAccount, BalanceOf, Config, LockedInfoOf, ProcessedEventInfo, LastProcessedParachainEvent, LockedInfo};
use frame_support::{assert_ok, assert_err, assert_noop, assert_storage_noop};
use frame_support::dispatch::DispatchResultWithPostInfo;
use frame_support::weights::{Pays, PostDispatchInfo};
use rand::Rng;
use sp_runtime::DispatchError::BadOrigin;
use sp_runtime::DispatchErrorWithPostInfo;
use rstest::*;
use rstest::rstest;
use rstest_reuse::{self, *};

fn extract_post_info(result: DispatchResultWithPostInfo) -> PostDispatchInfo {
    let post_info = match result {
        Ok(post_info) => post_info,
        Err(DispatchErrorWithPostInfo { post_info, ..}) => post_info,
    };

    post_info
}

fn random_locked_info() -> LockedInfoOf<Test> {
    let mut rng = rand::thread_rng();
    LockedInfoOf::<Test> {
        locked_amount: rng.gen_range(0..BalanceOf::<Test>::max_value()).into(),
        locked_at: rng.gen_range(0..<Test as frame_system::Config>::BlockNumber::max_value()).into(),
        expires_at: Some(rng.gen_range(0..<Test as frame_system::Config>::BlockNumber::max_value()).into()),
    }
}


fn subject_account_n(n: u32) -> <Test as frame_system::Config>::AccountId {
    account("Subject N", n, n)
}

macro_rules! CallFixtureType {
    () => {
        (
            impl FnOnce(), // initialization
            impl FnOnce(Origin) -> DispatchResultWithPostInfo,
            impl FnOnce(), // assertion
        )
    }
}

macro_rules! CallFixture {
    {
        initialization: $initialization:expr,
        call: $call:expr,
        assertion: $assertion:expr,

    } => {
        ($initialization, $call, $assertion)
    };
}

#[fixture]
fn set_locked_info_call_with_origin_fixture() -> CallFixtureType!() {
    static LOCKED_INFO: LockedInfoOf<Test> = LockedInfoOf::<Test> {
        locked_amount: 12231232u64,
        locked_at: 34222u64,
        expires_at: Some(132u64),
    };
    CallFixture!(
        initialization: || {
            assert_eq!(<LockedInfoByAccount<Test>>::iter().count(), 0);
        },
        call: |o| {
            LockerMirror::set_locked_info(
                o,
                subject_account_n(23),
                LOCKED_INFO.clone(),
            )
        },
        assertion: || {
            assert_eq!(<LockedInfoByAccount<Test>>::iter().count(), 1);
            assert_eq!(
                <LockedInfoByAccount<Test>>::get(subject_account_n(23)).unwrap(),
                LOCKED_INFO.clone(),
            );
        },
    )
}

#[fixture]
fn clear_locked_info_call_with_origin_fixture() -> CallFixtureType!() {
    CallFixture!(
        initialization: || {
            assert_eq!(<LockedInfoByAccount<Test>>::iter().count(), 0);
            <LockedInfoByAccount<Test>>::insert(
                subject_account_n(32),
                LockedInfoOf::<Test> {
                    locked_amount: 2132u64,
                    locked_at: 3232u64,
                    expires_at: Some(112234u64),
                },
            );
            assert_eq!(<LockedInfoByAccount<Test>>::iter().count(), 1);
        },
        call: |o| {
            LockerMirror::clear_locked_info(
                o,
                subject_account_n(32),
            )
        },
        assertion: || {
            assert_eq!(<LockedInfoByAccount<Test>>::iter().count(), 0);
        },
    )
}

#[fixture]
fn set_last_processed_parachain_event_call_with_origin_fixture() -> CallFixtureType!() {
    static EVENT: ProcessedEventInfo = ProcessedEventInfo {
        block_number: 11u32,
        event_index: 34u32,
    };
    CallFixture!(
        initialization: || {
            assert_eq!(<LastProcessedParachainEvent<Test>>::get(), None);
        },
        call: |o| {
            LockerMirror::set_last_processed_parachain_event(
                o,
                EVENT.clone(),
            )
        },
        assertion: || {
            assert_eq!(<LastProcessedParachainEvent<Test>>::get().unwrap(), EVENT.clone());
        },
    )
}

#[template]
#[rstest]
#[case::set_locked_info(set_locked_info_call_with_origin_fixture())]
#[case::clear_locked_info(clear_locked_info_call_with_origin_fixture())]
#[case::set_last_processed_parachain_event(set_last_processed_parachain_event_call_with_origin_fixture())]
fn call_cases(
    #[case]
    call_fixture: CallFixtureType!(),
) {}

////////////////


#[apply(call_cases)]
fn should_fail_noop_when_unsigned(
    #[case]
    call_fixture: CallFixtureType!(),
) {
    let (_, call, _) = call_fixture;
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(call(Origin::none()), BadOrigin);
    });
}

#[apply(call_cases)]
fn should_fail_noop_when_non_oracle(
    #[case]
    call_fixture: CallFixtureType!(),
) {
    let (_, call, _) = call_fixture;

    let oracle = subject_account_n(1);
    let non_oracle = subject_account_n(2);

    ExtBuilder::default()
        .oracle_account_id(oracle.clone())
        .build()
        .execute_with(|| {
            assert_noop!(call(Origin::signed(non_oracle)), BadOrigin);
        });
}

#[apply(call_cases)]
fn should_ok_if_when_oracle(
    #[case]
    call_fixture: CallFixtureType!(),
) {
    let (_, call, _) = call_fixture;

    let oracle = subject_account_n(1);

    ExtBuilder::default()
        .oracle_account_id(oracle.clone())
        .build()
        .execute_with(|| {
            assert_ok!(call(Origin::signed(oracle)));
        });
}

#[apply(call_cases)]
fn should_pay_when_caller_is_not_oracle(
    #[case]
    call_fixture: CallFixtureType!(),
) {
    let (_, call, _) = call_fixture;

    let oracle = subject_account_n(1);
    let non_oracle = subject_account_n(2);

    ExtBuilder::default()
        .oracle_account_id(oracle.clone())
        .build()
        .execute_with(|| {
            assert_storage_noop!({
                let res = call(Origin::signed(non_oracle));
                assert_err!(res, BadOrigin);
                assert_eq!(extract_post_info(res).pays_fee, Pays::Yes);
            });
        });
}


#[apply(call_cases)]
fn should_not_pay_when_caller_is_oracle(
    #[case]
    call_fixture: CallFixtureType!(),
) {
    let (_, call, _) = call_fixture;

    let oracle = subject_account_n(1);

    ExtBuilder::default()
        .oracle_account_id(oracle.clone())
        .build()
        .execute_with(|| {
            let res = call(Origin::signed(oracle));
            assert_ok!(res);
            assert_eq!(extract_post_info(res).pays_fee, Pays::No);
        });
}

#[apply(call_cases)]
fn check_storage_is_mutated_correctly(
    #[case]
    call_fixture: CallFixtureType!(),
) {
    let (initialization, call, assertion) = call_fixture;

    let oracle = subject_account_n(1);

    ExtBuilder::default()
        .oracle_account_id(oracle.clone())
        .build()
        .execute_with(|| {
            initialization();
            
            assert_ok!(call(Origin::signed(oracle.clone())));
            
            assertion();
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
    let oracle = subject_account_n(1);
    ExtBuilder::default()
        .oracle_account_id(oracle.clone())
        .build().execute_with(|| {
        assert_eq!(<LockedInfoByAccount<Test>>::iter().count(), 0);

        assert_ok!(LockerMirror::clear_locked_info(
            Origin::signed(oracle.clone()),
            subject_account_n(23)
        ));

        // no change because account 23 not found
        assert_eq!(<LockedInfoByAccount<Test>>::iter().count(), 0);

        let mut expected = vec![
            (subject_account_n(1), random_locked_info()),
            (subject_account_n(2), random_locked_info()),
            (subject_account_n(3), random_locked_info()),
            (subject_account_n(4), random_locked_info()),
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
                subject_account_n(55),
            ),
        );
        let infos: Vec<_> = <LockedInfoByAccount<Test>>::iter().collect();
        assert!(compare_ignore_order(&infos, &expected));



        // remove account 4
        assert_ok!(
            LockerMirror::clear_locked_info(
                Origin::signed(oracle.clone()),
                subject_account_n(4),
            ),
        );
        let infos: Vec<_> = <LockedInfoByAccount<Test>>::iter().collect();
        assert!(!compare_ignore_order(&infos, &expected));
        expected.retain(|(account, _)| account != &subject_account_n(4));
        assert!(compare_ignore_order(&infos, &expected));

        // nothing should happen since account 1312 don't have any locked_info
        assert_ok!(
            LockerMirror::clear_locked_info(
                Origin::signed(oracle.clone()),
                subject_account_n(1312),
            ),
        );
        let infos: Vec<_> = <LockedInfoByAccount<Test>>::iter().collect();
        assert!(compare_ignore_order(&infos, &expected));


        // remove account 1
        assert_ok!(
            LockerMirror::clear_locked_info(
                Origin::signed(oracle.clone()),
                subject_account_n(1),
            ),
        );
        let infos: Vec<_> = <LockedInfoByAccount<Test>>::iter().collect();
        assert!(!compare_ignore_order(&infos, &expected));
        expected.retain(|(account, _)| account != &subject_account_n(1));
        assert!(compare_ignore_order(&infos, &expected));


        // Add a new account
        let acc_221122 = subject_account_n(221122);
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
                subject_account_n(221122),
            ),
        );
        let infos: Vec<_> = <LockedInfoByAccount<Test>>::iter().collect();
        assert!(!compare_ignore_order(&infos, &expected));
        expected.retain(|(account, _)| account != &subject_account_n(221122));
        assert!(compare_ignore_order(&infos, &expected));
    });
}