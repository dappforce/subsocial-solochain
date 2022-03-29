//! All related code to free-calls module

use frame_support::traits::Contains;
use sp_std::cmp::min;
use static_assertions::const_assert;
use pallet_free_calls::max_quota_percentage;
use pallet_free_calls::config::WindowConfig;
use pallet_free_calls::quota::{MAX_QUOTA_DECIMALS, NumberOfCalls};
use pallet_locker_mirror::LockedInfoOf;
use crate::BlockNumber;
use super::constants::time::*;
use super::constants::currency;
use super::{Runtime, Call};

// TODO: try to find a better way to calculate it based on the circulating supply
pub const FREE_CALLS_PER_SUB: u16 = 10;

pub const FREE_CALLS_WINDOWS_CONFIG: [WindowConfig<BlockNumber>; 3] = [
    WindowConfig::new(1 * DAYS, max_quota_percentage!(100)),
    WindowConfig::new(1 * HOURS, max_quota_percentage!(30)),
    WindowConfig::new(5 * MINUTES, max_quota_percentage!(10)),
];


// Assert at compile time that the free-calls configs are in the optimal shape.
const_assert!(check_free_calls_config(&FREE_CALLS_WINDOWS_CONFIG));
#[allow(dead_code)] // the code is not acutely dead.
const fn check_free_calls_config(configs: &'static [WindowConfig<BlockNumber>]) -> bool {
    // cannot have empty configs
    if configs.is_empty() {
        return false;
    }

    let mut prev_config = &configs[0];
    // first config cannot have anything but 100% as the fraction
    if prev_config.fraction_of_max_quota.get() != MAX_QUOTA_DECIMALS {
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

/// A calculation strategy for free calls quota
pub struct FreeCallsCalculationStrategy;
impl Default for FreeCallsCalculationStrategy { fn default() -> Self { Self } }
impl pallet_free_calls::MaxQuotaCalculationStrategy<Runtime> for FreeCallsCalculationStrategy {
    fn calculate(
        current_block: <Runtime as frame_system::Config>::BlockNumber,
        locked_info: Option<LockedInfoOf<Runtime>>
    ) -> Option<NumberOfCalls> {
        fn get_utilization_percent(lock_period: BlockNumber) -> u16 {
            if lock_period < 1 * WEEKS {
                return 15;
            }
            if lock_period < 1 * MONTHS {
                let num_of_weeks = min(3, lock_period / (1 * WEEKS)) as u16;
                return (num_of_weeks * 5) + 25;
            }

            let num_of_months = min(12, lock_period / (1 * MONTHS)) as u16;
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

        let num_of_tokens = locked_amount.saturating_div(currency::DOLLARS) as NumberOfCalls;

        let num_of_free_calls = num_of_tokens
            .saturating_mul(FREE_CALLS_PER_SUB)
            .saturating_mul(utilization_percent)
            .saturating_div(100);

        Some(num_of_free_calls)
    }
}