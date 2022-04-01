//! All related code to free-calls module

use frame_support::log::{debug, info};
use sp_std::convert::TryInto;
use frame_support::traits::Contains;
use sp_std::cmp::min;
use sp_std::if_std;
use static_assertions::const_assert;
use pallet_free_calls::max_quota_percentage;
use pallet_free_calls::config::{ConfigHash, RateLimiterConfig, WindowConfig};
use pallet_free_calls::quota::{QUOTA_PRECISION, NumberOfCalls};
use pallet_locker_mirror::LockedInfoOf;
use crate::{Block, BlockNumber};
use subsocial_primitives::time::*;
use subsocial_primitives::currency;
use super::{Runtime, Call};


pub const FREE_CALLS_WINDOWS_CONFIGS: [WindowConfig<BlockNumber>; 3] = [
    WindowConfig::new(1 * DAYS, max_quota_percentage!(100)),
    WindowConfig::new(1 * HOURS, max_quota_percentage!(30)),
    WindowConfig::new(5 * MINUTES, max_quota_percentage!(10)),
];

pub const FREE_CALLS_CONFIG_HASH: ConfigHash = hash_windows_configs(&FREE_CALLS_WINDOWS_CONFIGS);

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

        if current_config.period >= prev_config.period {
            return false;
        }

        if current_config.fraction_of_max_quota.get() >= prev_config.fraction_of_max_quota.get() {
            return false;
        }

        prev_config = current_config;
        i = i + 1;
    }

    return true;
}

/// Create a hash for a collection of windows config.
const fn hash_windows_configs(configs: &'static [WindowConfig<BlockNumber>]) -> ConfigHash {
    let mut hash = 7u64;
    let mut i = 0;
    while i < configs.len() {
        let current_config = &configs[i];
        hash = 31 * hash + hash_window_config(current_config);
        i = i + 1;
    }
    hash
}

/// Create a hash for one window config.
const fn hash_window_config(config: &'static WindowConfig<BlockNumber>) -> ConfigHash {
    let mut hash = 7u64;
    hash = 31 * hash + config.period as u64;
    hash = 31 * hash + config.fraction_of_max_quota.get() as u64;
    hash
}

#[cfg(test)]
mod window_hashing_tests {
    use pallet_free_calls::config::WindowConfig;
    use pallet_free_calls::max_quota_percentage;
    use subsocial_primitives::BlockNumber;
    use subsocial_primitives::time::*;
    use crate::free_calls::{hash_window_config, hash_windows_configs};

    #[test]
    fn hash_window_config_should_work_as_expected() {
        static C1: WindowConfig<BlockNumber> = WindowConfig::new(1 * DAYS, max_quota_percentage!(100));
        static C2: WindowConfig<BlockNumber> = WindowConfig::new(1 * DAYS, max_quota_percentage!(100));

        assert_eq!(
            hash_window_config(&C1),
            hash_window_config(&C2),
            "Should return the same hash for the same window config",
        );
        //////

        static C3: WindowConfig<BlockNumber> = WindowConfig::new(1 * DAYS, max_quota_percentage!(100));
        static C4: WindowConfig<BlockNumber> = WindowConfig::new(1 * DAYS, max_quota_percentage!(99));
        static C5: WindowConfig<BlockNumber> = WindowConfig::new(2 * DAYS, max_quota_percentage!(99));

        assert_ne!(
            hash_window_config(&C3),
            hash_window_config(&C4),
            "Hash should be different for different window configs",
        );
        assert_ne!(
            hash_window_config(&C4),
            hash_window_config(&C5),
            "Hash should be different for different window configs",
        );
    }

    #[test]
    fn hash_windows_configs_should_work_as_expected() {
        static C1: WindowConfig<BlockNumber> = WindowConfig::new(1 * DAYS, max_quota_percentage!(100));
        static C2: WindowConfig<BlockNumber> = WindowConfig::new(3 * DAYS, max_quota_percentage!(50));
        static C3: WindowConfig<BlockNumber> = WindowConfig::new(3 * MONTHS, max_quota_percentage!(10));
        static C4: WindowConfig<BlockNumber> = WindowConfig::new(3 * MONTHS, max_quota_percentage!(11));
        static C5: WindowConfig<BlockNumber> = WindowConfig::new(1 * HOURS, max_quota_percentage!(1));
        static C6: WindowConfig<BlockNumber> = WindowConfig::new(9 * MINUTES, max_quota_percentage!(68));
        static C7: WindowConfig<BlockNumber> = WindowConfig::new(1 * MINUTES, max_quota_percentage!(12));

        static A1: [WindowConfig<BlockNumber>; 0] = [];
        static A2: [WindowConfig<BlockNumber>; 0] = [];
        assert_eq!(
            hash_windows_configs(&A1),
            hash_windows_configs(&A2),
        );

        static A3: [WindowConfig<BlockNumber>; 1] = [C1];
        static A4: [WindowConfig<BlockNumber>; 1] = [C1];
        assert_eq!(
            hash_windows_configs(&A3),
            hash_windows_configs(&A4),
        );

        static A5: [WindowConfig<BlockNumber>; 1] = [C2];
        static A6: [WindowConfig<BlockNumber>; 1] = [C1];
        assert_ne!(
            hash_windows_configs(&A5),
            hash_windows_configs(&A6),
        );

        static A7: [WindowConfig<BlockNumber>; 2] = [C2, C1];
        static A8: [WindowConfig<BlockNumber>; 2] = [C1, C2];
        assert_ne!(
            hash_windows_configs(&A7),
            hash_windows_configs(&A8),
        );

        static A9: [WindowConfig<BlockNumber>; 2] = [C1, C2];
        static A10: [WindowConfig<BlockNumber>; 2] = [C1, C2];
        assert_eq!(
            hash_windows_configs(&A9),
            hash_windows_configs(&A10),
        );

        static A11: [WindowConfig<BlockNumber>; 4] = [C4, C5, C6, C1];
        static A12: [WindowConfig<BlockNumber>; 4] = [C4, C5, C6, C1];
        assert_eq!(
            hash_windows_configs(&A11),
            hash_windows_configs(&A12),
        );

        static A13: [WindowConfig<BlockNumber>; 4] = [C4, C5, C6, C1];
        static A14: [WindowConfig<BlockNumber>; 4] = [C4, C5, C1, C1];
        assert_ne!(
            hash_windows_configs(&A13),
            hash_windows_configs(&A14),
        );

        static A15: [WindowConfig<BlockNumber>; 4] = [C4, C5, C6, C1];
        static A16: [WindowConfig<BlockNumber>; 4] = [C4, C5, C1, C6];
        assert_ne!(
            hash_windows_configs(&A15),
            hash_windows_configs(&A16),
        );

        static A17: [WindowConfig<BlockNumber>; 4] = [C1, C5, C6, C1];
        static A18: [WindowConfig<BlockNumber>; 4] = [C4, C5, C6, C4];
        assert_ne!(
            hash_windows_configs(&A17),
            hash_windows_configs(&A18),
        );

        static A19: [WindowConfig<BlockNumber>; 4] = [C4, C5, C6, C1];
        static A20: [WindowConfig<BlockNumber>; 3] = [C4, C5, C6];
        assert_ne!(
            hash_windows_configs(&A19),
            hash_windows_configs(&A20),
        );
    }
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