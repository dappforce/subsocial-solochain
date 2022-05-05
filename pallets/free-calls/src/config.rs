use sp_std::marker::PhantomData;
use sp_std::vec::Vec;
use crate::Config;
use crate::quota::{FractionOfMaxQuota, QUOTA_PRECISION};
use scale_info::TypeInfo;
use frame_support::{pallet_prelude::*};
use frame_system::pallet_prelude::*;
use subsocial_primitives as primitives;
use sp_std::convert::TryInto;

pub type ConfigHash = u64;

/// Configuration of a multiple rate limiting windows.
#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct RateLimiterConfig<BlockNumber> {
    pub windows_configs: Vec<WindowConfig<BlockNumber>>,

    /// A unique number that identifies this exact set of [windows_configs].
    ///
    /// When changing window config, make sure to also change this number to invalidate old stats
    /// recorded for consumers.
    pub hash: ConfigHash,
}

impl<BlockNumber> RateLimiterConfig<BlockNumber> {
    pub const fn new(windows_configs: Vec<WindowConfig<BlockNumber>>, hash: ConfigHash) -> Self {
        Self {
            windows_configs,
            hash,
        }
    }
}


/// Configuration of a rate limiting window in terms of window length and allocated quota.
#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, Copy)]
pub struct WindowConfig<BlockNumber> {
    /// The length of the window in number of blocks it will last.
    pub period: BlockNumber,

    /// The fraction of max quota allocated for this window.
    pub fraction_of_max_quota: FractionOfMaxQuota,
}

impl<BlockNumber> WindowConfig<BlockNumber> {
    //TODO: try to also force period to be non zero.
    pub const fn new(period: BlockNumber, fraction_of_max_quota: FractionOfMaxQuota) -> Self {
        WindowConfig {
            period,
            fraction_of_max_quota,
        }
    }
}

/// Retrieves the size of `T::WindowsConfig` to be used for `BoundedVec` declaration.
pub struct WindowsConfigSize<T: Config>(PhantomData<T>);

impl<T: Config> Default for WindowsConfigSize<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T: Config> Get<u32> for WindowsConfigSize<T> {
    fn get() -> u32 {
        T::RateLimiterConfig::get().windows_configs.len().try_into().unwrap()
    }
}


/// Validate the windows configurations.
pub const fn check_free_calls_config(configs: &'static [WindowConfig<primitives::BlockNumber>]) -> bool {
    // cannot have empty configs
    if configs.is_empty() {
        return false;
    }

    let mut prev_config = &configs[0];
    // A fraction of the first config should be equal to the quota precision.
    if prev_config.fraction_of_max_quota != QUOTA_PRECISION {
        return false;
    }

    let mut i = 1;

    while i < configs.len() {
        let current_config = &configs[i];

        if current_config.period >= prev_config.period {
            return false;
        }

        if current_config.fraction_of_max_quota >= prev_config.fraction_of_max_quota {
            return false;
        }

        prev_config = current_config;
        i = i + 1;
    }

    return true;
}

/// Create a hash for a collection of windows config.
/// We call this function at compile time.
pub const fn hash_windows_configs(configs: &'static [WindowConfig<primitives::BlockNumber>]) -> ConfigHash {
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
pub const fn hash_window_config(config: &'static WindowConfig<primitives::BlockNumber>) -> ConfigHash {
    let mut hash = 7u64;
    hash = 31 * hash + config.period as u64;
    hash = 31 * hash + config.fraction_of_max_quota as u64;
    hash
}

#[cfg(test)]
mod window_hashing_tests {
    use subsocial_primitives::BlockNumber;
    use subsocial_primitives::time::*;
    use crate::config::{hash_window_config, hash_windows_configs, WindowConfig};
    use crate::max_quota_percentage;

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
