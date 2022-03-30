//! All related code to free-calls module

use frame_support::log::{debug, info};
use sp_std::convert::TryInto;
use frame_support::traits::Contains;
use sp_std::cmp::min;
use sp_std::if_std;
use static_assertions::const_assert;
use pallet_free_calls::max_quota_percentage;
use pallet_free_calls::config::WindowConfig;
use pallet_free_calls::quota::{QUOTA_PRECISION, NumberOfCalls};
use pallet_locker_mirror::LockedInfoOf;
use crate::BlockNumber;
use super::constants::time::*;
use super::constants::currency;
use super::{Runtime, Call};

// TODO: try to find a better way to calculate it based on the circulating supply
pub const FREE_CALLS_PER_SUB: u16 = 10;

pub const FREE_CALLS_WINDOWS_CONFIGS: [WindowConfig<BlockNumber>; 3] = [
    WindowConfig::new(1 * DAYS, max_quota_percentage!(100)),
    WindowConfig::new(1 * HOURS, max_quota_percentage!(30)),
    WindowConfig::new(5 * MINUTES, max_quota_percentage!(10)),
];


// Assert at compile time that the free-calls configs are in the optimal shape.
const_assert!(check_free_calls_config(&FREE_CALLS_WINDOWS_CONFIGS));
#[allow(dead_code)] // the code is not acutely dead.
const fn check_free_calls_config(configs: &'static [WindowConfig<BlockNumber>]) -> bool {
    // cannot have empty configs
    if configs.is_empty() {
        return false;
    }

    let mut prev_config = &configs[0];
    // A fraction of the first config should be equal to the quota precision.
    if prev_config.fraction_of_max_quota.get() != QUOTA_PRECISION {
        return false;
    }

    let mut i = 1;

    while i < configs.len() {
        let current_config = &configs[i];

        // current period shouldn't be greater than or equal to the previous period
        if current_config.period >= prev_config.period {
            return false;
        }

        // current ratio shouldn't be larger than or equal to the previous ratio
        if current_config.fraction_of_max_quota.get() >= prev_config.fraction_of_max_quota.get() {
            return false;
        }

        prev_config = current_config;
        i = i + 1;
    }

    return true;
}

/// Filter the calls that can be used as free calls.
// TODO: add more calls to this filter. or maybe allow all calls???
pub struct FreeCallsFilter;
impl Default for FreeCallsFilter { fn default() -> Self { Self } }
impl Contains<Call> for FreeCallsFilter {
    fn contains(c: &Call) -> bool {
        match *c {
            Call::Posts(..) => true,
            Call::Profiles(..) => true,
            Call::ProfileFollows(..) => true,
            Call::Roles(..) => true,
            Call::Spaces(..) => true,
            Call::SpaceFollows(..) => true,
            Call::Reactions(..) => true,
            Call::SpaceOwnership(..) => true,
            // Call::Moderation(..) => true,
            Call::System(..) => cfg!(feature = "runtime-benchmarks"),
            _ => false,
        }
    }
}

/// A calculation strategy for free calls quota.
///
/// The calculation depends on the amount of token the user has locked and the time since lock. Each
/// token locked will grant the user [FREE_CALLS_PER_SUB] to be used as free calls, but the full ammount
/// will be not be fully accessible until 12 month. Before 12 month only a percentage of the free calls
/// will be granted.
///
/// ```text
/// +-------------+------+---------+
/// |    Time     | Days | Allowed |
/// +-------------+------+---------+
/// | Just Locked |    0 | 15%     |
/// | 1 week      |    7 | 30%     |
/// | 2 week      |   14 | 35%     |
/// | 3 week      |   21 | 40%     |
/// | 1 month     |   30 | 45%     |
/// | 2 month     |   60 | 50%     |
/// | 3 month     |   90 | 55%     |
/// | 4 month     |  120 | 60%     |
/// | 5 month     |  150 | 65%     |
/// | 6 month     |  180 | 70%     |
/// | 7 month     |  210 | 75%     |
/// | 8 month     |  240 | 80%     |
/// | 9 month     |  270 | 85%     |
/// | 10 month    |  300 | 90%     |
/// | 11 month    |  330 | 95%     |
/// | 12 month    |  360 | 100%    |
/// +-------------+------+---------+
/// ```
pub struct FreeCallsCalculationStrategy;
impl Default for FreeCallsCalculationStrategy { fn default() -> Self { Self } }
impl pallet_free_calls::MaxQuotaCalculationStrategy<Runtime> for FreeCallsCalculationStrategy {
    fn calculate(
        current_block: <Runtime as frame_system::Config>::BlockNumber,
        locked_info: Option<LockedInfoOf<Runtime>>
    ) -> Option<NumberOfCalls> {
        fn get_utilization_percent(lock_period: BlockNumber) -> u64 {
            if lock_period < 1 * WEEKS {
                return 15;
            }
            if lock_period < 1 * MONTHS {
                let num_of_weeks = min(3, lock_period / (1 * WEEKS)) as u64;
                return (num_of_weeks * 5) + 25;
            }

            let num_of_months = min(12, lock_period / (1 * MONTHS)) as u64;
            return (num_of_months * 5) + 40;
        }

        let LockedInfoOf::<Runtime>{
            locked_at,
            locked_amount,
            expires_at,
        } = match locked_info {
            Some(locked_info) => locked_info,
            None => return None,
        };

        if locked_at >= current_block {
            return None;
        }

        if matches!(expires_at, Some(expires_at) if current_block >= expires_at) {
            return None;
        }

        let lock_period = current_block - locked_at;

        let utilization_percent = get_utilization_percent(lock_period);

        let num_of_tokens = locked_amount.saturating_div(currency::DOLLARS) as u64;

        let num_of_free_calls = num_of_tokens
            .saturating_mul(FREE_CALLS_PER_SUB.into())
            .saturating_mul(utilization_percent)
            .saturating_div(100);

        Some(num_of_free_calls.try_into().unwrap_or(NumberOfCalls::MAX))
    }
}


#[cfg(test)]
mod tests {
    use pallet_locker_mirror::LockedInfoOf;
    use pallet_free_calls::MaxQuotaCalculationStrategy;
    use crate::*;
    use rstest::rstest;
    use pallet_free_calls::quota::NumberOfCalls;

    #[rstest]
    // FREE_CALLS_PER_SUB = 10
    #[case(1 * CENTS, 10, Some(0))]

    #[case(1 * DOLLARS, 1 * DAYS, Some(1))]
    #[case(10 * DOLLARS, 1 * DAYS, Some(15))]
    #[case(100 * DOLLARS, 1 * DAYS, Some(150))]

    #[case(1 * DOLLARS, 1 * WEEKS, Some(3))]
    #[case(10 * DOLLARS, 1 * WEEKS, Some(30))]

    #[case(1 * DOLLARS, 2 * WEEKS, Some(3))]
    #[case(10 * DOLLARS, 2 * WEEKS, Some(35))]

    #[case(1 * DOLLARS, 3 * WEEKS, Some(4))]
    #[case(10 * DOLLARS, 3 * WEEKS, Some(40))]

    // 4 weeks (28) is treated as 3 weeks
    #[case(1 * DOLLARS, 4 * WEEKS, Some(4))]
    #[case(10 * DOLLARS, 4 * WEEKS, Some(40))]

    #[case(5 * DOLLARS, 1 * MONTHS, Some(22))]
    #[case(20 * DOLLARS, 1 * MONTHS, Some(90))]

    #[case(5 * DOLLARS, 2 * MONTHS, Some(25))]
    #[case(20 * DOLLARS, 2 * MONTHS, Some(100))]

    #[case(5 * DOLLARS, 3 * MONTHS, Some(27))]
    #[case(20 * DOLLARS, 3 * MONTHS, Some(110))]

    #[case(5 * DOLLARS, 4 * MONTHS, Some(30))]
    #[case(20 * DOLLARS, 4 * MONTHS, Some(120))]

    #[case(5 * DOLLARS, 5 * MONTHS, Some(32))]
    #[case(20 * DOLLARS, 5 * MONTHS, Some(130))]
    #[case(500 * DOLLARS, 5 * MONTHS, Some(3250))]

    // treated as 5 MONTHS
    #[case(500 * DOLLARS, 5 * MONTHS + 1 * WEEKS, Some(3250))]

    #[case(100 * DOLLARS, 6 * MONTHS, Some(700))]
    #[case(100 * DOLLARS, 7 * MONTHS, Some(750))]
    #[case(100 * DOLLARS, 8 * MONTHS, Some(800))]
    #[case(100 * DOLLARS, 9 * MONTHS, Some(850))]
    #[case(100 * DOLLARS, 10 * MONTHS, Some(900))]
    #[case(100 * DOLLARS, 11 * MONTHS, Some(950))]
    #[case(100 * DOLLARS, 12 * MONTHS, Some(1000))]

    #[case(100 * DOLLARS, 13 * MONTHS, Some(1000))]
    #[case(100 * DOLLARS, 100 * MONTHS, Some(1000))]
    #[trace]
    fn quota_calculation_strategy_tests(
        #[case] amount: Balance,
        #[case] locked_period: BlockNumber,
        #[case] expected_quota: Option<NumberOfCalls>,
    ) {
        let current_block = 1000 * MONTHS;
        let before_current_block = current_block - 1;
        let after_current_block = current_block + 1;


        let locked_at = current_block - locked_period;
        let locked_info = LockedInfoOf::<Runtime> {
            locked_at,
            locked_amount: amount.into(),
            expires_at: None,
        };

        let locked_info_not_yet_expired = {
            let mut locked_info = locked_info.clone();
            locked_info.expires_at = Some(after_current_block);
            locked_info
        };

        let locked_info_expired = {
            let mut locked_info = locked_info.clone();
            locked_info.expires_at = Some(before_current_block);
            locked_info
        };

        let locked_info_just_expired = {
            let mut locked_info = locked_info.clone();
            locked_info.expires_at = Some(current_block);
            locked_info
        };

        ///////////////////////////////////////

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

        assert_eq!(
            FreeCallsCalculationStrategy::calculate(current_block, Some(locked_info)),
            expected_quota,
        );

        // test expiration
        assert_eq!(
            FreeCallsCalculationStrategy::calculate(current_block, Some(locked_info_just_expired)),
            None,
        );
        assert_eq!(
            FreeCallsCalculationStrategy::calculate(current_block, Some(locked_info_expired)),
            None,
        );
        assert_eq!(
            FreeCallsCalculationStrategy::calculate(current_block, Some(locked_info_not_yet_expired)),
            expected_quota,
        );

    }
}