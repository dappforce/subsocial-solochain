use std::borrow::Borrow;
use std::cell::RefCell;
use frame_benchmarking::account;
use frame_support::{assert_err, assert_ok};
use pallet_locker_mirror::{BalanceOf, LockedInfoByAccount, LockedInfoOf};
use crate::mock::*;
use rand::Rng;
use subsocial_primitives::Block;
use crate::{ConsumerStats, pallet as free_calls, Pallet, QuotaToWindowRatio, ShouldUpdateConsumerStats, WindowConfig, WindowType};
use crate::WindowStatsByConsumer;

fn assert_no_new_events() {
    assert!(TestUtils::system_events().is_empty());
}

fn assert_storage_have_no_change(old_storage: Vec<(AccountId, WindowType, ConsumerStats<BlockNumber>)>) {
    assert!(compare_ignore_order(&old_storage, &TestUtils::capture_stats_storage()))
}

fn random_locked_info() -> LockedInfoOf<Test> {
    let mut rng = rand::thread_rng();
    LockedInfoOf::<Test> {
        locked_amount: rng.gen_range(0..BalanceOf::<Test>::max_value()).into(),
        unlocks_at: rng.gen_range(0..<Test as frame_system::Config>::BlockNumber::max_value()).into(),
        lock_period: rng.gen_range(0..<Test as frame_system::Config>::BlockNumber::max_value()).into(),
    }
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
            assert_no_new_events();

            assert_eq!(get_captured_locked_tokens(), None);
            assert_eq!(get_captured_current_block(), Some(11));


            ///// try again but

            let locked_info = random_locked_info();
            <LockedInfoByAccount<Test>>::insert(consumer.clone(), locked_info.clone());

            TestUtils::set_block_number(55);

            let can_have_free_call = <Pallet<Test>>::can_make_free_call(
                &consumer,
                ShouldUpdateConsumerStats::YES,
            );

            assert_eq!(can_have_free_call, true);
            assert_no_new_events();

            assert_eq!(get_captured_locked_tokens(), Some(locked_info.clone()));
            assert_eq!(get_captured_current_block(), Some(55));


            //// change locked info and try again

            let new_locked_info = random_locked_info();
            <LockedInfoByAccount<Test>>::insert(consumer.clone(), new_locked_info.clone());

            let can_have_free_call = <Pallet<Test>>::can_make_free_call(
                &consumer,
                ShouldUpdateConsumerStats::YES,
            );

            assert_eq!(can_have_free_call, false, "Block number is still 55 and quota is 1");
            assert_no_new_events();

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
            assert_no_new_events();
            assert_storage_have_no_change(storage);
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
            assert_no_new_events();
            assert_storage_have_no_change(storage);
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
            assert_no_new_events();
            assert_storage_have_no_change(storage);
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
            assert_no_new_events();
            assert_storage_have_no_change(storage);
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
            assert_no_new_events();
            assert_storage_have_no_change(storage);
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
            assert_storage_have_no_change(storage);


            assert_no_new_events();
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

            let mut consumer_stats: Vec<_> = <WindowStatsByConsumer<Test>>::iter_prefix(consumer).collect();
            assert_eq!(consumer_stats.len(), 1, "We only have one window");

            let consumer_stats = consumer_stats.pop().unwrap();
            let expected_usage = ConsumerStats::<BlockNumber> {
                timeline_index: 3, // 315 / 100,
                used_calls: 1,
            };
            assert_eq!(consumer_stats, (0, expected_usage));

            ///////

            TestUtils::set_block_number(330);

            let can_have_free_call = <Pallet<Test>>::can_make_free_call(
                &consumer,
                ShouldUpdateConsumerStats::YES,
            );
            assert_eq!(can_have_free_call, true);

            let mut consumer_stats: Vec<_> = <WindowStatsByConsumer<Test>>::iter_prefix(consumer).collect();
            assert_eq!(consumer_stats.len(), 1, "We only have one window");

            let consumer_stats = consumer_stats.pop().unwrap();
            let expected_usage = ConsumerStats::<BlockNumber> {
                timeline_index: 3, // 330 / 100,
                used_calls: 2,
            };
            assert_eq!(consumer_stats, (0, expected_usage));


            ////////

            TestUtils::set_block_number(780);

            let can_have_free_call = <Pallet<Test>>::can_make_free_call(
                &consumer,
                ShouldUpdateConsumerStats::YES,
            );
            assert_eq!(can_have_free_call, true);

            let mut consumer_stats: Vec<_> = <WindowStatsByConsumer<Test>>::iter_prefix(consumer).collect();
            assert_eq!(consumer_stats.len(), 1, "We only have one window");

            let consumer_stats = consumer_stats.pop().unwrap();
            let expected_usage = ConsumerStats::<BlockNumber> {
                timeline_index: 7, // 780 / 100,
                used_calls: 1,
            };
            assert_eq!(consumer_stats, (0, expected_usage));
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

            <WindowStatsByConsumer<Test>>::insert(consumer, 0, ConsumerStats::<BlockNumber> {
                timeline_index: 0,
                used_calls: 34,
            });

            let can_have_free_call = <Pallet<Test>>::can_make_free_call(
                &consumer,
                ShouldUpdateConsumerStats::YES,
            );
            assert_eq!(can_have_free_call, false, "The consumer is out of quota");

            let mut consumer_stats: Vec<_> = <WindowStatsByConsumer<Test>>::iter_prefix(consumer).collect();
            assert_eq!(consumer_stats.len(), 1, "We only have one window");
            assert_eq!(consumer_stats.pop().unwrap(), (0, ConsumerStats::<BlockNumber> {
                timeline_index: 0,
                used_calls: 34,
            }));

            ////////

            TestUtils::set_block_number(55);

            let can_have_free_call = <Pallet<Test>>::can_make_free_call(
                &consumer,
                ShouldUpdateConsumerStats::YES,
            );
            assert_eq!(can_have_free_call, true, "We have entered a new window");

            let mut consumer_stats: Vec<_> = <WindowStatsByConsumer<Test>>::iter_prefix(consumer).collect();
            assert_eq!(consumer_stats.len(), 1, "We only have one window");
            assert_eq!(consumer_stats.pop().unwrap(), (0, ConsumerStats::<BlockNumber> {
                timeline_index: 1,
                used_calls: 1,
            }));

            ////////

            TestUtils::set_block_number(80);


            let can_have_free_call = <Pallet<Test>>::can_make_free_call(
                &consumer,
                ShouldUpdateConsumerStats::YES,
            );
            assert_eq!(can_have_free_call, true, "We still have quota to spend");

            let mut consumer_stats: Vec<_> = <WindowStatsByConsumer<Test>>::iter_prefix(consumer).collect();
            assert_eq!(consumer_stats.len(), 1, "We only have one window");
            assert_eq!(consumer_stats.pop().unwrap(), (0, ConsumerStats::<BlockNumber> {
                timeline_index: 1,
                used_calls: 2,
            }));


            /////

            TestUtils::set_block_number(100);


            let can_have_free_call = <Pallet<Test>>::can_make_free_call(
                &consumer,
                ShouldUpdateConsumerStats::YES,
            );
            assert_eq!(can_have_free_call, true);

            let mut consumer_stats: Vec<_> = <WindowStatsByConsumer<Test>>::iter_prefix(consumer).collect();
            assert_eq!(consumer_stats.len(), 1, "We only have one window");
            assert_eq!(consumer_stats.pop().unwrap(), (0, ConsumerStats::<BlockNumber> {
                timeline_index: 2,
                used_calls: 1,
            }));

        });
}
