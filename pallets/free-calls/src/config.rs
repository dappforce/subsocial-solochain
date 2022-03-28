use sp_std::marker::PhantomData;
use crate::Config;
use crate::quota::FractionOfMaxQuota;
use scale_info::TypeInfo;
use frame_support::{pallet_prelude::*};
use frame_system::pallet_prelude::*;
use sp_std::convert::TryInto;

/// Configuration of a rate limiting window in terms of window length and allocated quota.
#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
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
        T::WindowsConfig::get().len().try_into().unwrap()
    }
}