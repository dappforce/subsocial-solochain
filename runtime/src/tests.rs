/////// Quota Calculation Strategy Tests

use std::collections::HashMap;
use sp_std::map;
use pallet_free_calls::NumberOfCalls;
use pallet_locker_mirror::{self, LockedInfo, LockedInfoOf};
use subsocial_primitives::{Balance, BlockNumber};
use crate::FreeCallsCalculationStrategy;
use crate::Runtime;
use crate::constants::currency::*;
use crate::constants::time::*;
use pallet_free_calls::QuotaCalculationStrategy;

#[test]
fn test_free_calls_quota_calculation_strategy() {

    let current_block = 1000 * MONTHS;

    let before_current_block = current_block - 1;

    let after_current_block = current_block + 1;

    let locked_info = |amount: Balance, lock_period: BlockNumber| -> LockedInfoOf<Runtime> {
        let locked_at = current_block - lock_period;
        LockedInfoOf::<Runtime> {
            locked_at,
            locked_amount: amount.into(),
            expires_at: None,
        }
    };

    let add_exp = |locked_info: LockedInfoOf<Runtime>, expires_at: BlockNumber| -> LockedInfoOf<Runtime> {
        LockedInfoOf::<Runtime> {
            locked_at: locked_info.locked_at,
            locked_amount: locked_info.locked_amount,
            expires_at: Some(expires_at),
        }
    };

    let test_scenarios: Vec<(LockedInfoOf<Runtime>, Option<NumberOfCalls>)> = map!(
        locked_info(1 * CENTS, 10) => Some(0)
    );

    // no locked_info will returns none
    assert_eq!(
        FreeCallsCalculationStrategy::calculate(current_block, None),
        None,
    );
    assert_eq!(
        FreeCallsCalculationStrategy::calculate(before_current_block, None),
        None,
    );
    assert_eq!(
        FreeCallsCalculationStrategy::calculate(after_current_block, None),
        None,
    );

    for (locked_info, expected_result) in test_scenarios.into_iter() {
        let x = locked_info.clone();
        assert_eq!(
            FreeCallsCalculationStrategy::calculate(current_block, Some(x)),
            expected_result,
        );

        // test expiration
        let x = add_exp(locked_info.clone(), current_block);
        assert_eq!(
            FreeCallsCalculationStrategy::calculate(current_block, Some(x)),
            None,
        );

        let x = add_exp(locked_info.clone(), before_current_block);
        assert_eq!(
            FreeCallsCalculationStrategy::calculate(current_block, Some(x)),
            None,
        );

        let x = add_exp(locked_info.clone(), after_current_block);
        assert_eq!(
            FreeCallsCalculationStrategy::calculate(current_block, Some(x)),
            expected_result,
        );
    }
}

////////////////////////////////////////