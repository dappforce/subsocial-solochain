use frame_support::{pallet_prelude::*};
use frame_system::pallet_prelude::*;
use frame_support::BoundedVec;
use crate::config::{ConfigHash, WindowsConfigSize};
use crate::quota::NumberOfCalls;
use scale_info::TypeInfo;
use crate::Config;

/// A collection of windows_stats along with a hash of the config used to validate the windows.
///
/// The [config_hash] can be used to detect if the the config did change in a Runtime upgrade.
#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct ConsumerStats<T: Config> {
    pub windows_stats: WindowStatsVec<T>,
    pub config_hash: ConfigHash,
}

impl<T: Config> ConsumerStats<T> {
    pub fn new(windows_stats: WindowStatsVec<T>, config_hash: ConfigHash) -> Self {
        Self {
            windows_stats,
            config_hash,
        }
    }

    /// Try to push a new window_stats into the collection.
    pub fn try_push_window_stats(&mut self, window_stats: WindowStats<T::BlockNumber>) -> Result<(), ()> {
        self.windows_stats.try_push(window_stats)
    }

    pub fn get_window_stats(&self, index: usize) -> Option<&WindowStats<T::BlockNumber>> {
        self.windows_stats.get(index)
    }
}

/// A `BoundedVec` that can hold a list of `WindowStats` objects bounded by the size of WindowConfigs.
pub type WindowStatsVec<T> = BoundedVec<WindowStats<<T as frame_system::Config>::BlockNumber>, WindowsConfigSize<T>>;

/// Keeps track of the executed number of calls per window per consumer (account).
#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct WindowStats<BlockNumber> {
    /// The index of this window in the timeline.
    pub timeline_index: BlockNumber,

    /// The number of calls executed during this window.
    pub used_calls: NumberOfCalls,
}

impl<BlockNumber> WindowStats<BlockNumber> {
    pub fn new(timeline_index: BlockNumber) -> Self {
        WindowStats {
            timeline_index,
            used_calls: 0,
        }
    }
}