use frame_support::{pallet_prelude::*};
use frame_system::pallet_prelude::*;
use frame_support::BoundedVec;
use crate::config::WindowsConfigSize;
use crate::quota::NumberOfCalls;
use scale_info::TypeInfo;

/// A `BoundedVec` that can hold a list of `ConsumerStats` objects bounded by the size of WindowConfigs.
pub type ConsumerStatsVec<T> = BoundedVec<ConsumerStats<<T as frame_system::Config>::BlockNumber>, WindowsConfigSize<T>>;

/// Keeps track of the executed number of calls per window per consumer (account).
#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct ConsumerStats<BlockNumber> {
    /// The index of this window in the timeline.
    pub timeline_index: BlockNumber,

    /// The number of calls executed during this window.
    pub used_calls: NumberOfCalls,
}

impl<BlockNumber> ConsumerStats<BlockNumber> {
    pub fn new(timeline_index: BlockNumber) -> Self {
        ConsumerStats {
            timeline_index,
            used_calls: 0,
        }
    }
}