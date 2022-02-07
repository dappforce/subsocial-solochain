use std::borrow::Borrow;
use std::cell::RefCell;
use std::convert::TryInto;
use frame_benchmarking::account;
use frame_support::{assert_err, assert_ok, BoundedVec};
use frame_system::EventRecord;
use pallet_locker_mirror::{BalanceOf, LockedInfoByAccount, LockedInfoOf};
use crate::mock::*;
use rand::Rng;
use sp_core::crypto::UncheckedInto;
use sp_runtime::testing::H256;
use subsocial_primitives::Block;
use crate::{ConsumerStats, ConsumerStatsVec, NumberOfCalls, pallet as free_calls, Pallet, QuotaToWindowRatio, ShouldUpdateConsumerStats, WindowConfig};
use crate::WindowStatsByConsumer;

pub struct TestUtils;
impl TestUtils {
    pub fn set_block_number(n: BlockNumber) {
        <frame_system::Pallet<Test>>::set_block_number(n)
    }

    pub fn system_events() -> Vec<EventRecord<Event, H256>> {
        <frame_system::Pallet<Test>>::events()
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
            unlocks_at: rng.gen_range(0..<Test as frame_system::Config>::BlockNumber::max_value()).into(),
            lock_period: rng.gen_range(0..<Test as frame_system::Config>::BlockNumber::max_value()).into(),
        }
    }

    pub fn assert_storage_have_no_change(old_storage: Vec<(AccountId, Vec<ConsumerStats<BlockNumber>>)>) {
        assert!(TestUtils::compare_ignore_order(&old_storage, &TestUtils::capture_stats_storage()))
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
}

////////////////// Begin Testing ///////////////////////

#[test]
fn dummy() {
    // just make sure everything is okay
    ExtBuilder::default()
        .build().execute_with(|| {
        assert_eq!(1 + 1, 2);

        // events are empty at the start
        assert!(TestUtils::system_events().is_empty());
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

            let can_have_free_call = <Pallet<Test>>::can_make_free_call(
                &consumer,
                ShouldUpdateConsumerStats::YES,
            );

            assert_eq!(can_have_free_call, false);
            TestUtils::assert_no_new_events();

            assert_eq!(get_captured_locked_tokens(), None);
            assert_eq!(get_captured_current_block(), Some(11));


            ///// try again but

            let locked_info = TestUtils::random_locked_info();
            <LockedInfoByAccount<Test>>::insert(consumer.clone(), locked_info.clone());

            TestUtils::set_block_number(55);

            let can_have_free_call = <Pallet<Test>>::can_make_free_call(
                &consumer,
                ShouldUpdateConsumerStats::YES,
            );

            assert_eq!(can_have_free_call, true);
            TestUtils::assert_no_new_events();

            assert_eq!(get_captured_locked_tokens(), Some(locked_info.clone()));
            assert_eq!(get_captured_current_block(), Some(55));


            //// change locked info and try again

            let new_locked_info = TestUtils::random_locked_info();
            <LockedInfoByAccount<Test>>::insert(consumer.clone(), new_locked_info.clone());

            let can_have_free_call = <Pallet<Test>>::can_make_free_call(
                &consumer,
                ShouldUpdateConsumerStats::YES,
            );

            assert_eq!(can_have_free_call, false, "Block number is still 55 and quota is 1");
            TestUtils::assert_no_new_events();

            assert_eq!(get_captured_locked_tokens(), Some(new_locked_info));
            assert_ne!(get_captured_locked_tokens(), Some(locked_info));
            assert_eq!(get_captured_current_block(), Some(55));
        });
}


#[test]
fn denied_if_configs_are_empty() {
    ExtBuilder::default()
        .windows_config(vec![])
        .build()
        .execute_with(|| {
            let storage = TestUtils::capture_stats_storage();

            let consumer: AccountId = account("Consumer", 0, 0);

            let can_have_free_call = <Pallet<Test>>::can_make_free_call(
                &consumer,
                ShouldUpdateConsumerStats::YES,
            );

            assert_eq!(can_have_free_call, false);
            TestUtils::assert_no_new_events();
            TestUtils::assert_storage_have_no_change(storage);
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

            let can_have_free_call = <Pallet<Test>>::can_make_free_call(
                &consumer,
                ShouldUpdateConsumerStats::YES,
            );

            assert_eq!(can_have_free_call, false);
            TestUtils::assert_no_new_events();
            TestUtils::assert_storage_have_no_change(storage);
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

            let can_have_free_call = <Pallet<Test>>::can_make_free_call(
                &consumer,
                ShouldUpdateConsumerStats::YES,
            );

            assert_eq!(can_have_free_call, false);
            TestUtils::assert_no_new_events();
            TestUtils::assert_storage_have_no_change(storage);
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

            let can_have_free_call = <Pallet<Test>>::can_make_free_call(
                &consumer,
                ShouldUpdateConsumerStats::YES,
            );

            assert_eq!(can_have_free_call, false);
            TestUtils::assert_no_new_events();
            TestUtils::assert_storage_have_no_change(storage);
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

            let can_have_free_call = <Pallet<Test>>::can_make_free_call(
                &consumer,
                ShouldUpdateConsumerStats::YES,
            );

            assert_eq!(can_have_free_call, false);
            TestUtils::assert_no_new_events();
            TestUtils::assert_storage_have_no_change(storage);
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
                let can_have_free_call = <Pallet<Test>>::can_make_free_call(
                    &consumer,
                    ShouldUpdateConsumerStats::YES,
                );
                assert_eq!(can_have_free_call, true);
            }

            let storage = TestUtils::capture_stats_storage();

            // consumer is now out of quota and trying to get free calls until
            // block number 19 will fail
            for i in 5..20 {
                TestUtils::set_block_number(i);
                let can_have_free_call = <Pallet<Test>>::can_make_free_call(
                    &consumer,
                    ShouldUpdateConsumerStats::YES,
                );
                assert_eq!(can_have_free_call, false);
            }
            TestUtils::assert_storage_have_no_change(storage);


            TestUtils::assert_no_new_events();
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

            let can_have_free_call = <Pallet<Test>>::can_make_free_call(
                &consumer,
                ShouldUpdateConsumerStats::YES,
            );

            assert_eq!(can_have_free_call, true);


            TestUtils::assert_stats_equal(
                consumer.clone(),
                vec![(3 /*315 / 100*/, 1)],
            );

            ///////

            TestUtils::set_block_number(330);

            let can_have_free_call = <Pallet<Test>>::can_make_free_call(
                &consumer,
                ShouldUpdateConsumerStats::YES,
            );
            assert_eq!(can_have_free_call, true);

            TestUtils::assert_stats_equal(
                consumer.clone(),
                vec![(3 /*330 / 100*/, 2)],
            );


            ////////

            TestUtils::set_block_number(780);

            let can_have_free_call = <Pallet<Test>>::can_make_free_call(
                &consumer,
                ShouldUpdateConsumerStats::YES,
            );
            assert_eq!(can_have_free_call, true);

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

            let can_have_free_call = <Pallet<Test>>::can_make_free_call(
                &consumer,
                ShouldUpdateConsumerStats::YES,
            );
            assert_eq!(can_have_free_call, false, "The consumer is out of quota");

            TestUtils::assert_stats_equal(
                consumer.clone(),
                vec![(0, 34)],
            );

            ////////

            TestUtils::set_block_number(55);

            let can_have_free_call = <Pallet<Test>>::can_make_free_call(
                &consumer,
                ShouldUpdateConsumerStats::YES,
            );
            assert_eq!(can_have_free_call, true, "We have entered a new window");

            TestUtils::assert_stats_equal(
                consumer.clone(),
                vec![(1, 1)],
            );

            ////////

            TestUtils::set_block_number(80);


            let can_have_free_call = <Pallet<Test>>::can_make_free_call(
                &consumer,
                ShouldUpdateConsumerStats::YES,
            );
            assert_eq!(can_have_free_call, true, "We still have quota to spend");

            TestUtils::assert_stats_equal(
                consumer.clone(),
                vec![(1, 2)],
            );


            /////

            TestUtils::set_block_number(100);


            let can_have_free_call = <Pallet<Test>>::can_make_free_call(
                &consumer,
                ShouldUpdateConsumerStats::YES,
            );
            assert_eq!(can_have_free_call, true);

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

            let can_have_free_call = <Pallet<Test>>::can_make_free_call(
                &consumer,
                ShouldUpdateConsumerStats::YES,
            );
            assert_eq!(can_have_free_call, true);

            TestUtils::assert_stats_equal(
                consumer.clone(),
                vec![(0, 35), (3, 18), (7, 18)],
            );

            ///////

            TestUtils::set_block_number(71);

            let can_have_free_call = <Pallet<Test>>::can_make_free_call(
                &consumer,
                ShouldUpdateConsumerStats::YES,
            );
            assert_eq!(can_have_free_call, false, "2nd window config allows only 18 calls, consumer must wait until the window have passed");

            // nothing should change since the call have failed
            TestUtils::assert_stats_equal(
                consumer.clone(),
                vec![(0, 35), (3, 18), (7, 18)],
            );

            //////

            TestUtils::set_block_number(79);

            let can_have_free_call = <Pallet<Test>>::can_make_free_call(
                &consumer,
                ShouldUpdateConsumerStats::YES,
            );
            assert_eq!(can_have_free_call, false, "2nd window config allows only 18 calls, consumer must wait until the window have passed");

            // nothing should change since the call have failed
            TestUtils::assert_stats_equal(
                consumer.clone(),
                vec![(0, 35), (3, 18), (7, 18)],
            );

            /////

            TestUtils::set_block_number(80);

            let can_have_free_call = <Pallet<Test>>::can_make_free_call(
                &consumer,
                ShouldUpdateConsumerStats::YES,
            );
            assert_eq!(can_have_free_call, true, "we have entered a new 2nd/3rd windows, so the call should be granted");

            TestUtils::assert_stats_equal(
                consumer.clone(),
                vec![(0, 36), (4, 1), (8, 1)],
            );

            ///////

            TestUtils::set_block_number(80);

            let can_have_free_call = <Pallet<Test>>::can_make_free_call(
                &consumer,
                ShouldUpdateConsumerStats::YES,
            );
            assert_eq!(can_have_free_call, true, "we have entered a new 2nd/3rd windows, so the call should be granted");

            TestUtils::assert_stats_equal(
                consumer.clone(),
                vec![(0, 37), (4, 2), (8, 2)],
            );

            ///////

            TestUtils::set_block_number(90);

            let can_have_free_call = <Pallet<Test>>::can_make_free_call(
                &consumer,
                ShouldUpdateConsumerStats::YES,
            );
            assert_eq!(can_have_free_call, true);

            TestUtils::assert_stats_equal(
                consumer.clone(),
                vec![(0, 38), (4, 3), (9, 1)],
            );

            ///////

            TestUtils::set_block_number(101);

            let can_have_free_call = <Pallet<Test>>::can_make_free_call(
                &consumer,
                ShouldUpdateConsumerStats::YES,
            );
            assert_eq!(can_have_free_call, true);

            TestUtils::assert_stats_equal(
                consumer.clone(),
                vec![(1, 1), (5, 1), (10, 1)],
            );
        });
}
