use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::convert::TryInto;
use frame_benchmarking::account;
use frame_support::{assert_err, assert_noop, assert_ok, assert_storage_noop, BoundedVec, debug};
use frame_support::log::info;
use frame_support::weights::{extract_actual_weight, Pays, PostDispatchInfo};
use frame_system::{EventRecord};
use pallet_locker_mirror::{BalanceOf, LockedInfoByAccount, LockedInfoOf};
use crate::mock::*;
use rand::{Rng, thread_rng};
use sp_core::crypto::UncheckedInto;
use sp_runtime::testing::H256;
use subsocial_primitives::Block;
use crate::{ConsumerStats, ConsumerStatsVec, FreeCallsPrevalidation, FreeCallsValidityError, NumberOfCalls, pallet as free_calls, Pallet, QuotaToWindowRatio, test_pallet, WindowConfig};
use crate::WindowStatsByConsumer;
use frame_support::weights::GetDispatchInfo;
use crate::test_pallet::Something;
use crate::weights::WeightInfo;
pub use sp_io::{self, storage::root as storage_root};
use test_pallet::Call as TestPalletCall;
use frame_system::Call as SystemCall;
use sp_runtime::traits::{Dispatchable, SignedExtension};
use sp_runtime::transaction_validity::{InvalidTransaction, TransactionValidity, TransactionValidityError, ValidTransaction};
use GrantedScenario::*;
use FreeCallScenario::*;
use DeclinedScenario::*;

#[derive(Debug)]
pub enum GrantedScenario {
    /// The granted call errored out
    Errored,
    /// The granted call did succeed
    Succeeded
}

#[derive(Debug)]
pub enum DeclinedScenario {
    /// declined because user don't have quota
    OutOfQuota,
    /// declined from the filter
    CallCannotBeFree,
}

#[derive(Debug)]
pub enum FreeCallScenario {
    /// The free call have been granted.
    Granted(GrantedScenario),
    /// The consumer cannot have this free call
    Declined(DeclinedScenario),
}

pub struct TestUtils;
impl TestUtils {
    pub fn set_block_number(n: BlockNumber) {
        <frame_system::Pallet<Test>>::set_block_number(n)
    }

    pub fn system_events() -> Vec<EventRecord<Event, H256>> {
        <frame_system::Pallet<Test>>::events()
    }

    pub fn clear_system_events() -> Vec<Event> {
        let events: Vec<Event> = Self::system_events().into_iter().map(|e| e.event).collect();
        <frame_system::Pallet<Test>>::reset_events();
        events
    }

    pub fn capture_stats_storage() -> Vec<(AccountId, Vec<ConsumerStats<BlockNumber>>)> {
        <WindowStatsByConsumer<Test>>::iter().map(|x| (x.0, x.1.into_inner())).collect()
    }

    pub fn set_stats_for_consumer(consumer: AccountId, stats: Vec<(BlockNumber, NumberOfCalls)>) {
        let mapped_stats: Vec<_> = stats.iter().map(|(timeline_index, used_calls)| {
            ConsumerStats::<BlockNumber> {
                timeline_index: timeline_index.clone(),
                used_calls: used_calls.clone(),
            }
        }).collect();

        let mapped_stats: ConsumerStatsVec<Test> = mapped_stats.try_into().unwrap();

        <WindowStatsByConsumer<Test>>::insert(
            consumer.clone(),
            mapped_stats,
        );

        TestUtils::assert_stats_equal(consumer.clone(), stats);
    }

    pub fn assert_stats_equal(consumer: AccountId, expected_stats: Vec<(BlockNumber, NumberOfCalls)>) {
        let found_stats = <WindowStatsByConsumer<Test>>::get(consumer.clone());

        let found_stats: Vec<_> = found_stats.iter().map(|x| (x.timeline_index, x.used_calls)).collect();

        assert_eq!(found_stats, expected_stats);
    }

    pub fn random_locked_info() -> LockedInfoOf<Test> {
        let mut rng = rand::thread_rng();
        LockedInfoOf::<Test> {
            locked_amount: rng.gen_range(0..BalanceOf::<Test>::max_value()).into(),
            locked_at: rng.gen_range(0..<Test as frame_system::Config>::BlockNumber::max_value()).into(),
            expires_at: Some(rng.gen_range(0..<Test as frame_system::Config>::BlockNumber::max_value()).into()),
        }
    }

    pub fn assert_stats_storage_have_no_change(old_storage: Vec<(AccountId, Vec<ConsumerStats<BlockNumber>>)>) {
        assert!(TestUtils::compare_ignore_order(&old_storage, &TestUtils::capture_stats_storage()))
    }

    pub fn assert_stats_storage_have_change(old_storage: Vec<(AccountId, Vec<ConsumerStats<BlockNumber>>)>) {
        assert!(!TestUtils::compare_ignore_order(&old_storage, &TestUtils::capture_stats_storage()))
    }

    pub fn assert_no_new_events() {
        assert!(TestUtils::system_events().is_empty());
    }

    pub fn compare_ignore_order<T: PartialEq>(a: &Vec<T>, b: &Vec<T>) -> bool {
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

    /// Run multiple assertion on the result of try_free_call using the TestPallet call.
    ///
    /// if scenario is Granted(Errored), we will try a call that is granted to fail, so we can
    /// test how try_free_call will behave when the boxed call errors out.
    pub fn assert_try_free_call_works(
        consumer: <Test as frame_system::Config>::AccountId,
        scenario: FreeCallScenario,
    ) {
        let old_something = <Something<Test>>::get();
        let something = match scenario {
            Granted(Errored) => 0, // zero should cause an error
            _ => rand::thread_rng().gen_range(1..1024), // other values should be okay
        };

        println!(
            "Block number: {}, scenario: {:?}",
            <frame_system::Pallet<Test>>::block_number(),
            scenario,
        );

        let call: Box<Call> = Box::new(Call::TestPallet(TestPalletCall::<Test>::store_value { something }));

        // test signed extension
        match &scenario {
            Granted(_) => {
                TestUtils::assert_signed_extension_works(
                    consumer.clone(),
                    call.clone(),
                    None,
                );
            },
            Declined(declined_scenario) => {
                TestUtils::assert_signed_extension_works(
                    consumer.clone(),
                    call.clone(),
                    Some(match declined_scenario {
                        OutOfQuota => FreeCallsValidityError::OutOfQuota,
                        CallCannotBeFree => FreeCallsValidityError::CallCannotBeFree,
                    }),
                );
            }
        }


        TestUtils::clear_system_events();

        let stats_storage = TestUtils::capture_stats_storage();
        let storage = storage_root();

        let result = <Pallet<Test>>::try_free_call(Origin::signed(consumer), call.clone());

        // The free call should not return any error, regardless of the boxed call result
        assert_ok!(result);

        let result: PostDispatchInfo = result.unwrap();

        assert_eq!(
            result.pays_fee,
            Pays::No,
            "The caller should not pay",
        );

        // storage should only change if call is granted and it did succeed
        match &scenario {
            Granted(Succeeded) => assert_eq!(
                <Something<Test>>::get(),
                Some(something),
                "Something storage should have mutated to have {}", something,
            ),
            _ => assert_eq!(
                <Something<Test>>::get(),
                old_something,
                "nothing should be changed",
            ),
        };

        match scenario {
            Granted(Succeeded) => {
                let events: Vec<Event> = TestUtils::clear_system_events();
                assert!(TestUtils::system_events().is_empty(), "Only 2 events should be emitted");

                assert_eq!(
                    events,
                    vec![
                        Event::from(test_pallet::Event::ValueStored(something, consumer.clone())),
                        Event::from(pallet_free_calls::Event::FreeCallResult(consumer.clone(), Ok(()))),
                    ],
                );

                assert_ne!(storage, storage_root());
                TestUtils::assert_stats_storage_have_change(stats_storage)
            },
            Granted(Errored) => {
                let events: Vec<Event> = TestUtils::clear_system_events();

                assert_eq!(
                    events,
                    vec![
                        Event::from(pallet_free_calls::Event::FreeCallResult(consumer.clone(), Err(test_pallet::Error::<Test>::DoNotSendZero.into()))),
                    ],
                );

                assert_ne!(storage, storage_root());
                TestUtils::assert_stats_storage_have_change(stats_storage);
            },
            Declined(_) => {
                assert!(TestUtils::system_events().is_empty(), "No events should be emitted");
                TestUtils::assert_stats_storage_have_no_change(stats_storage);
                assert_eq!(storage, storage_root(), "if call is declined there should be noop storage");
            }
        };
    }

    fn assert_signed_extension_works(
        consumer: <Test as frame_system::Config>::AccountId,
        boxed_call: Box<Call>,
        expected_error: Option<FreeCallsValidityError>,
    ) {
        let validator = FreeCallsPrevalidation::<Test>::new();
        let call: <Test as frame_system::Config>::Call = free_calls::Call::<Test>::try_free_call { call: boxed_call }.into();
        let di = call.get_dispatch_info();
        assert_eq!(di.pays_fee, Pays::No);
        let res = validator.validate(
            &consumer,
            &call,
            &di,
            20,
        );
        assert_eq!(
            res,
            match expected_error {
                Some(error) => TransactionValidity::Err(TransactionValidityError::Invalid(InvalidTransaction::Custom(error.into()))),
                None => TransactionValidity::Ok(ValidTransaction::default()),
            },
        );
    }
}

////////////////// Begin Testing ///////////////////////

#[test]
fn dummy() {
    // just make sure everything is okay
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            assert_eq!(1 + 1, 2);

            // events are empty at the start
            assert!(TestUtils::system_events().is_empty());
        });

    ExtBuilder::default()
        .windows_config(vec![
            WindowConfig::new(1, QuotaToWindowRatio::new(1)),
        ])
        .quota_calculation(|_, _| 100.into())
        .build().execute_with(|| {
        let consumer: AccountId = account("Consumer", 2, 1);

        let can_have_free_call = <Pallet<Test>>::can_make_free_call(&consumer).is_some();
        assert!(can_have_free_call);

        TestUtils::assert_try_free_call_works(consumer.clone(), Granted(Succeeded));
        TestUtils::assert_try_free_call_works(consumer.clone(), Granted(Errored));
    });

    ExtBuilder::default()
        .windows_config(vec![
            WindowConfig::new(1, QuotaToWindowRatio::new(1)),
        ])
        .quota_calculation(|_, _| None)
        .build().execute_with(|| {
        let consumer: AccountId = account("Consumer", 2, 1);

        let can_have_free_call = <Pallet<Test>>::can_make_free_call(&consumer).is_some();
        assert_eq!(can_have_free_call, false);

        TestUtils::assert_try_free_call_works(consumer.clone(), Declined(OutOfQuota));
    });
}


#[test]
fn locked_token_info_and_current_block_number_will_be_passed_to_the_calculation_strategy() {
    thread_local! {
        static CAPTURED_LOCKED_TOKENS: RefCell<Option<LockedInfoOf<Test>>> = RefCell::new(None);
        static CAPTURED_CURRENT_BLOCK: RefCell<Option<BlockNumber>> = RefCell::new(None);
    }

    let get_captured_locked_tokens = || CAPTURED_LOCKED_TOKENS.with(|x| x.borrow().clone());
    let get_captured_current_block = || CAPTURED_CURRENT_BLOCK.with(|x| x.borrow().clone());

    ExtBuilder::default()
        .windows_config(vec![WindowConfig::new(1, QuotaToWindowRatio::new(1))])
        .quota_calculation(|current_block, locked_tokens| {
            CAPTURED_LOCKED_TOKENS.with(|x| *x.borrow_mut() = locked_tokens.clone());
            CAPTURED_CURRENT_BLOCK.with(|x| *x.borrow_mut() = Some(current_block));

            locked_tokens.and_then(|_| Some(1))
        })
        .build()
        .execute_with(|| {
            let consumer: AccountId = account("Consumer", 0, 0);

            assert_eq!(get_captured_locked_tokens(), None);
            assert_eq!(get_captured_current_block(), None);

            TestUtils::set_block_number(11);

            TestUtils::assert_try_free_call_works(consumer.clone(), Declined(OutOfQuota));

            assert_eq!(get_captured_locked_tokens(), None);
            assert_eq!(get_captured_current_block(), Some(11));


            ///// try again but

            let locked_info = TestUtils::random_locked_info();
            <LockedInfoByAccount<Test>>::insert(consumer.clone(), locked_info.clone());

            TestUtils::set_block_number(55);

            TestUtils::assert_try_free_call_works(consumer.clone(), Granted(Succeeded));

            assert_eq!(get_captured_locked_tokens(), Some(locked_info.clone()));
            assert_eq!(get_captured_current_block(), Some(55));


            //// change locked info and try again

            let new_locked_info = TestUtils::random_locked_info();
            <LockedInfoByAccount<Test>>::insert(consumer.clone(), new_locked_info.clone());

            // Block number is still 55 and quota is 1
            TestUtils::assert_try_free_call_works(consumer.clone(), Declined(OutOfQuota));

            assert_eq!(get_captured_locked_tokens(), Some(new_locked_info));
            assert_ne!(get_captured_locked_tokens(), Some(locked_info));
            assert_eq!(get_captured_current_block(), Some(55));
        });
}


#[test]
fn boxed_call_will_be_passed_to_the_call_filter() {
    thread_local! {
        static CAPTURED_CALL: RefCell<Option<Call>> = RefCell::new(None);
    }

    let get_captured_call = || CAPTURED_CALL.with(|x| x.borrow().clone());

    ExtBuilder::default()
        .windows_config(vec![WindowConfig::new(1, QuotaToWindowRatio::new(1))])
        .quota_calculation(|_, _| Some(10))
        .call_filter(|call| {
            CAPTURED_CALL.with(|x| *x.borrow_mut() = call.clone().into());
            true
        })
        .build()
        .execute_with(|| {
            let consumer: AccountId = account("Consumer", 0, 0);

            assert_eq!(get_captured_call(), None);

            assert_ok!(<Pallet<Test>>::try_free_call(
                Origin::signed(consumer),
                Box::new(TestPalletCall::<Test>::call_a {}.into()),
            ));

            assert_eq!(get_captured_call(), Some(TestPalletCall::<Test>::call_a {}.into()));

            //////////

            assert_ok!(<Pallet<Test>>::try_free_call(
                Origin::signed(consumer),
                Box::new(TestPalletCall::<Test>::cause_error {}.into()),
            ));

            assert_eq!(get_captured_call(), Some(TestPalletCall::<Test>::cause_error {}.into()));

            //////////

            assert_ok!(<Pallet<Test>>::try_free_call(
                Origin::signed(consumer),
                Box::new(TestPalletCall::<Test>::call_b {}.into()),
            ));

            assert_eq!(get_captured_call(), Some(TestPalletCall::<Test>::call_b {}.into()));

            //////////

            assert_ok!(<Pallet<Test>>::try_free_call(
                Origin::signed(consumer),
                Box::new(TestPalletCall::<Test>::store_value {something: 12}.into()),
            ));

            assert_ne!(get_captured_call(), Some(TestPalletCall::<Test>::store_value {something: 21}.into()));
            assert_eq!(get_captured_call(), Some(TestPalletCall::<Test>::store_value {something: 12}.into()));
        });
}

#[test]
fn denied_if_call_filter_returns_false() {
    thread_local! {
        static ALLOW_CALLS: RefCell<bool> = RefCell::new(false);
    }

    let set_filter = |allow| ALLOW_CALLS.with(|x| *x.borrow_mut() = allow);

    ExtBuilder::default()
        .windows_config(vec![WindowConfig::new(1, QuotaToWindowRatio::new(1))])
        .call_filter(|_| ALLOW_CALLS.with(|b| b.borrow().clone()))
        .quota_calculation(|_,_| Some(1000))
        .build()
        .execute_with(|| {
            let consumer: AccountId = account("Consumer", 0, 0);

            set_filter(false);
            TestUtils::assert_try_free_call_works(consumer.clone(), Declined(CallCannotBeFree));
            TestUtils::assert_try_free_call_works(consumer.clone(), Declined(CallCannotBeFree));

            set_filter(true);
            TestUtils::assert_try_free_call_works(consumer.clone(), Granted(Succeeded));
            TestUtils::assert_try_free_call_works(consumer.clone(), Granted(Errored));
            TestUtils::assert_try_free_call_works(consumer.clone(), Granted(Errored));
            TestUtils::assert_try_free_call_works(consumer.clone(), Granted(Succeeded));

            set_filter(false);
            TestUtils::assert_try_free_call_works(consumer.clone(), Declined(CallCannotBeFree));

            set_filter(true);
            TestUtils::assert_try_free_call_works(consumer.clone(), Granted(Errored));

            set_filter(false);
            TestUtils::assert_try_free_call_works(consumer.clone(), Declined(CallCannotBeFree));

            set_filter(true);
            TestUtils::assert_try_free_call_works(consumer.clone(), Granted(Succeeded));
            TestUtils::assert_try_free_call_works(consumer.clone(), Granted(Errored));
        });
}


#[test]
fn denied_if_configs_are_empty() {
    ExtBuilder::default()
        .windows_config(vec![])
        .build()
        .execute_with(|| {
            let consumer: AccountId = account("Consumer", 0, 0);

            TestUtils::assert_try_free_call_works(consumer.clone(), Declined(OutOfQuota));
        });
}


//// Disallow zero period

#[test]
fn denied_if_configs_have_one_zero_period() {
    ExtBuilder::default()
        .windows_config(vec![
            WindowConfig::new(0, QuotaToWindowRatio::new(1)),
        ])
        .build()
        .execute_with(|| {
            let storage = TestUtils::capture_stats_storage();

            let consumer: AccountId = account("Consumer", 0, 0);

            TestUtils::assert_try_free_call_works(consumer.clone(), Declined(OutOfQuota));
        });
}


#[test]
fn denied_if_configs_have_one_zero_period_and_other_non_zero() {
    ExtBuilder::default()
        .windows_config(vec![
            WindowConfig::new(0, QuotaToWindowRatio::new(1)),
            WindowConfig::new(100, QuotaToWindowRatio::new(2)),
            WindowConfig::new(32, QuotaToWindowRatio::new(3)),
            WindowConfig::new(22, QuotaToWindowRatio::new(3)),
        ])
        .build()
        .execute_with(|| {
            let storage = TestUtils::capture_stats_storage();

            let consumer: AccountId = account("Consumer", 0, 0);

            TestUtils::assert_try_free_call_works(consumer.clone(), Declined(OutOfQuota));
        });


    ExtBuilder::default()
        .windows_config(vec![
            WindowConfig::new(100, QuotaToWindowRatio::new(2)),
            WindowConfig::new(32, QuotaToWindowRatio::new(3)),
            WindowConfig::new(22, QuotaToWindowRatio::new(3)),
            WindowConfig::new(0, QuotaToWindowRatio::new(1)),
        ])
        .build()
        .execute_with(|| {
            let storage = TestUtils::capture_stats_storage();

            let consumer: AccountId = account("Consumer", 0, 0);

            TestUtils::assert_try_free_call_works(consumer.clone(), Declined(OutOfQuota));
        });


    ExtBuilder::default()
        .windows_config(vec![
            WindowConfig::new(100, QuotaToWindowRatio::new(2)),
            WindowConfig::new(32, QuotaToWindowRatio::new(3)),
            WindowConfig::new(0, QuotaToWindowRatio::new(1)),
            WindowConfig::new(22, QuotaToWindowRatio::new(3)),
        ])
        .build()
        .execute_with(|| {
            let storage = TestUtils::capture_stats_storage();

            let consumer: AccountId = account("Consumer", 0, 0);

            TestUtils::assert_try_free_call_works(consumer.clone(), Declined(OutOfQuota));
        });
}

////////


#[test]
fn donot_exceed_the_allowed_quota_with_one_window() {
    ExtBuilder::default()
        .windows_config(vec![
            WindowConfig::new(20, QuotaToWindowRatio::new(1)),
        ])
        .quota_calculation(|_, _| 5.into())
        .build()
        .execute_with(|| {
            let storage = TestUtils::capture_stats_storage();
            assert!(storage.is_empty());

            let consumer: AccountId = account("Consumer", 0, 0);

            // consumer have 5 quotas so consuming one request for the next
            // 5 blocks can be granted
            for i in 1..=5 {
                TestUtils::set_block_number(i);
                TestUtils::assert_try_free_call_works(consumer.clone(), Granted(Succeeded));
            }

            // consumer is now out of quota and trying to get free calls until
            // block number 19 will fail
            for i in 5..20 {
                TestUtils::set_block_number(i);
                TestUtils::assert_try_free_call_works(consumer.clone(), Declined(OutOfQuota));
            }


            // from block number 30 to 34 user can have more 5 calls since we are at a new window
            for i in 30..35 {
                TestUtils::set_block_number(i);
                let granted_scenario = if i % 2 == 0 {
                    Succeeded
                } else {
                    Errored
                };
                TestUtils::assert_try_free_call_works(consumer.clone(), Granted(granted_scenario));
            }
        });
}


#[test]
fn consumer_with_quota_but_no_previous_usages() {
    ExtBuilder::default()
        .windows_config(vec![ WindowConfig::new(100, QuotaToWindowRatio::new(1)) ])
        .quota_calculation(|_, _| Some(100))
        .build()
        .execute_with(|| {
            TestUtils::set_block_number(315);

            assert!(TestUtils::capture_stats_storage().is_empty());

            let consumer: AccountId = account("Consumer", 0, 0);

            TestUtils::assert_try_free_call_works(consumer.clone(), Granted(Succeeded));

            TestUtils::assert_stats_equal(
                consumer.clone(),
                vec![(3 /*315 / 100*/, 1)],
            );

            ///////

            TestUtils::set_block_number(330);

            TestUtils::assert_try_free_call_works(consumer.clone(), Granted(Errored));

            TestUtils::assert_stats_equal(
                consumer.clone(),
                vec![(3 /*330 / 100*/, 2)],
            );


            ////////

            TestUtils::set_block_number(780);

            TestUtils::assert_try_free_call_works(consumer.clone(), Granted(Errored));

            TestUtils::assert_stats_equal(
                consumer.clone(),
                vec![(7 /*780 / 100*/, 1)],
            );
        });
}


#[test]
fn consumer_with_quota_and_have_previous_usages() {
    ExtBuilder::default()
        .windows_config(vec![ WindowConfig::new(50, QuotaToWindowRatio::new(1)) ])
        .quota_calculation(|_, _| Some(34))
        .build()
        .execute_with(|| {
            let consumer: AccountId = account("Consumer", 0, 0);

            TestUtils::set_block_number(10);

            let stats: ConsumerStatsVec<Test> = vec![ConsumerStats::<BlockNumber> {
                timeline_index: 0,
                used_calls: 34,
            }].try_into().unwrap();
            
            <WindowStatsByConsumer<Test>>::insert(consumer, stats);

            // The consumer is out of quota
            TestUtils::assert_try_free_call_works(consumer.clone(), Declined(OutOfQuota));

            TestUtils::assert_stats_equal(
                consumer.clone(),
                vec![(0, 34)],
            );

            ////////

            TestUtils::set_block_number(55);

            // We have entered a new window
            TestUtils::assert_try_free_call_works(consumer.clone(), Granted(Succeeded));

            TestUtils::assert_stats_equal(
                consumer.clone(),
                vec![(1, 1)],
            );

            ////////

            TestUtils::set_block_number(80);


            // We still have quota to spend
            TestUtils::assert_try_free_call_works(consumer.clone(), Granted(Succeeded));

            TestUtils::assert_stats_equal(
                consumer.clone(),
                vec![(1, 2)],
            );


            /////

            TestUtils::set_block_number(100);

            TestUtils::assert_try_free_call_works(consumer.clone(), Granted(Errored));

            TestUtils::assert_stats_equal(
                consumer.clone(),
                vec![(2, 1)],
            );

        });
}


#[test]
fn testing_scenario_1() {
    ExtBuilder::default()
        .quota_calculation(|_,_| Some(55))
        .windows_config(vec![
            WindowConfig::new(100, QuotaToWindowRatio::new(1)),
            WindowConfig::new(20, QuotaToWindowRatio::new(3)),
            WindowConfig::new(10, QuotaToWindowRatio::new(2)),
        ])
        .build()
        .execute_with(|| {
            let consumer: AccountId = account("Consumer", 0, 0);

            TestUtils::set_block_number(70);
            TestUtils::set_stats_for_consumer(
                consumer.clone(),
                vec![(0, 34), (3, 17), (7, 17)],
            );

            TestUtils::assert_try_free_call_works(consumer.clone(), Granted(Errored));

            TestUtils::assert_stats_equal(
                consumer.clone(),
                vec![(0, 35), (3, 18), (7, 18)],
            );

            ///////

            TestUtils::set_block_number(71);

            // 2nd window config allows only 18 calls, consumer must wait until the window have passed
            TestUtils::assert_try_free_call_works(consumer.clone(), Declined(OutOfQuota));

            // nothing should change since the call have failed
            TestUtils::assert_stats_equal(
                consumer.clone(),
                vec![(0, 35), (3, 18), (7, 18)],
            );

            //////

            TestUtils::set_block_number(79);

            // 2nd window config allows only 18 calls, consumer must wait until the window have passed
            TestUtils::assert_try_free_call_works(consumer.clone(), Declined(OutOfQuota));

            // nothing should change since the call have failed
            TestUtils::assert_stats_equal(
                consumer.clone(),
                vec![(0, 35), (3, 18), (7, 18)],
            );

            /////

            TestUtils::set_block_number(80);

            // we have entered a new 2nd/3rd windows, so the call should be granted
            TestUtils::assert_try_free_call_works(consumer.clone(), Granted(Succeeded));

            TestUtils::assert_stats_equal(
                consumer.clone(),
                vec![(0, 36), (4, 1), (8, 1)],
            );

            ///////

            TestUtils::set_block_number(80);

            // we have entered a new 2nd/3rd windows, so the call should be granted
            TestUtils::assert_try_free_call_works(consumer.clone(), Granted(Errored));

            TestUtils::assert_stats_equal(
                consumer.clone(),
                vec![(0, 37), (4, 2), (8, 2)],
            );

            ///////

            TestUtils::set_block_number(90);

            TestUtils::assert_try_free_call_works(consumer.clone(), Granted(Succeeded));

            TestUtils::assert_stats_equal(
                consumer.clone(),
                vec![(0, 38), (4, 3), (9, 1)],
            );

            ///////

            TestUtils::set_block_number(101);

            TestUtils::assert_try_free_call_works(consumer.clone(), Granted(Errored));

            TestUtils::assert_stats_equal(
                consumer.clone(),
                vec![(1, 1), (5, 1), (10, 1)],
            );
        });
}
