/////// Quota Calculation Strategy Tests

use std::collections::HashMap;
use sp_std::map;
use pallet_free_calls::NumberOfCalls;
use pallet_locker_mirror::{self, LockedInfoOf};
use subsocial_primitives::{Balance, BlockNumber};
use crate::FreeCallsCalculationStrategy;
use crate::Runtime;
use crate::constants::currency::*;
use crate::constants::time::*;
use pallet_free_calls::QuotaCalculationStrategy;

#[test]
fn test_free_calls_quota_calculation_strategy() {

    let unlocks_at = 101010;

    let before_unlocks_at = 101010 - 10;
    let after_unlocks_at = 101010 + 10;

    let locked_info = |amount: Balance, lock_period: BlockNumber| -> LockedInfoOf<Runtime> {
        LockedInfoOf::<Runtime> {
            locked_amount: amount.into(),
            lock_period,
            unlocks_at,
        }
    };

    type strategy = FreeCallsCalculationStrategy;

    let test_scenarios: Vec<(LockedInfoOf<Runtime>, Option<NumberOfCalls>)> = map!(
        // less than a token will grant no free calls
        locked_info(1 * CENTS, 1 * WEEKS) => None,
        locked_info(10 * CENTS, 1 * DAYS) => None,
        locked_info(50 * CENTS, 1 * HOURS) => None,

        // less than 1 week will grant no free calls
        locked_info(1 * DOLLARS, 1 * DAYS) => None,
        locked_info(100 * DOLLARS, 2 * DAYS) => None,
        locked_info(100 * DOLLARS, 4 * HOURS) => None,
        locked_info(100_000 * DOLLARS, 6 * DAYS) => None,

        // test multipliers
        locked_info(1 * DOLLARS, 1 * WEEKS) => Some(6),
        locked_info(1 * DOLLARS, 2 * WEEKS) => Some(7),
        locked_info(1 * DOLLARS, 3 * WEEKS) => Some(8),
        locked_info(1 * DOLLARS, 1 * MONTHS) => Some(9),
        locked_info(1 * DOLLARS, 2 * MONTHS) => Some(10),
        locked_info(1 * DOLLARS, 3 * MONTHS) => Some(11),
        locked_info(1 * DOLLARS, 4 * MONTHS) => Some(12),
        locked_info(1 * DOLLARS, 5 * MONTHS) => Some(13),
        locked_info(1 * DOLLARS, 6 * MONTHS) => Some(14),
        locked_info(1 * DOLLARS, 7 * MONTHS) => Some(15),
        locked_info(1 * DOLLARS, 8 * MONTHS) => Some(16),
        locked_info(1 * DOLLARS, 9 * MONTHS) => Some(17),
        locked_info(1 * DOLLARS, 10 * MONTHS) => Some(18),
        locked_info(1 * DOLLARS, 11 * MONTHS) => Some(19),
        locked_info(1 * DOLLARS, 12 * MONTHS) => Some(20),


        // test more than 12 months
        locked_info(1 * DOLLARS, 13 * MONTHS) => Some(20),
        locked_info(10 * DOLLARS, 15 * MONTHS) => Some(10 * 20),
        locked_info(35 * DOLLARS, 500 * MONTHS) => Some(35 * 20),

        // 4 weeks (28) days will be treated as 3 weeks
        locked_info(2 * DOLLARS, 4 * WEEKS) => Some(2 * 8),


        // extra tests to be sure :)
        locked_info(2 * DOLLARS, 10 * MONTHS) => Some(2 * 18),
        locked_info(100 * DOLLARS, 10 * MONTHS) => Some(100 * 18),
        locked_info(10_000 * DOLLARS, 10 * MONTHS) => Some(NumberOfCalls::MAX), // prevent overflow

        locked_info(32 * DOLLARS, 2 * MONTHS) => Some(32 * 10),
        locked_info(56 * DOLLARS, 2 * MONTHS) => Some(56 * 10),
        locked_info(1_000_000 * DOLLARS, 2 * MONTHS) => Some(NumberOfCalls::MAX) // prevent overflow
    );

    // no locked_info will returns none
    assert_eq!(strategy::calculate(unlocks_at, None), None);
    assert_eq!(strategy::calculate(after_unlocks_at, None), None);
    assert_eq!(strategy::calculate(before_unlocks_at, None), None);

    for (locked_info, expected_result) in test_scenarios.into_iter() {
        assert_eq!(strategy::calculate(unlocks_at, Some(locked_info.clone())), None);
        assert_eq!(strategy::calculate(after_unlocks_at, Some(locked_info.clone())), None);
        assert_eq!(strategy::calculate(before_unlocks_at, Some(locked_info.clone())), expected_result);
    }

}

////////////////////////////////////////