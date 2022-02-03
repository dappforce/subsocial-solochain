use frame_benchmarking::account;
use frame_support::{assert_err, assert_ok};
use crate::mock::*;
use crate::{ConsumerStats, pallet as free_calls, Pallet, QuotaToWindowRatio, ShouldUpdateConsumerStats, WindowConfig, WindowType};
use crate::WindowStatsByConsumer;

fn assert_no_new_events() {
    assert!(TestUtils::system_events().is_empty());
}

fn assert_storage_have_no_change(old_storage: Vec<(AccountId, WindowType, ConsumerStats<BlockNumber>)>) {
    assert!(compare_ignore_order(&old_storage, &TestUtils::get_stats_storage()))
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
fn denied_if_configs_are_empty() {
    ExtBuilder::default()
        .windows_config(vec![])
        .build()
        .execute_with(|| {
            let storage = TestUtils::get_stats_storage();

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
            let storage = TestUtils::get_stats_storage();

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
            let storage = TestUtils::get_stats_storage();

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
            let storage = TestUtils::get_stats_storage();

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
            let storage = TestUtils::get_stats_storage();

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