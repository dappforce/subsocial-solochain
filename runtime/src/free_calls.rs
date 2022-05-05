//! All related code to free-calls module

use super::Call;
use crate::BlockNumber;
use frame_support::traits::Contains;
use pallet_free_calls::config::{
    check_free_calls_config, hash_windows_configs, ConfigHash, WindowConfig,
};
use pallet_free_calls::max_quota_percentage;
use pallet_free_calls::quota::QUOTA_PRECISION;
use static_assertions::const_assert;
use subsocial_primitives::time::*;

pub const FREE_CALLS_WINDOWS_CONFIGS: [WindowConfig<BlockNumber>; 3] = [
    WindowConfig::new(1 * DAYS, max_quota_percentage!(100)),
    WindowConfig::new(1 * HOURS, max_quota_percentage!(30)),
    WindowConfig::new(5 * MINUTES, max_quota_percentage!(10)),
];

pub const FREE_CALLS_CONFIG_HASH: ConfigHash = hash_windows_configs(&FREE_CALLS_WINDOWS_CONFIGS);

// Assert at compile time that the free-calls configs are in the optimal shape.
const_assert!(check_free_calls_config(&FREE_CALLS_WINDOWS_CONFIGS));

/// Filter the calls that can be used as free calls.
// TODO: add more calls to this filter. or maybe allow all calls???
pub struct FreeCallsFilter;
impl Default for FreeCallsFilter {
    fn default() -> Self {
        Self
    }
}
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