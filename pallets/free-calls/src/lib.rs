//! # Free Calls Pallet
//!
//! Pallet for allowing accounts to send free calls based on a set quota.
//! The quota can be distributed over multiple overlapping windows to limit abuse.
//!
//! Resources:
//! - https://cloud.google.com/architecture/rate-limiting-strategies-techniques
//! - https://www.figma.com/blog/an-alternative-approach-to-rate-limiting/
//! - https://www.codementor.io/@arpitbhayani/system-design-sliding-window-based-rate-limiter-157x7sburi
//! - https://blog.cloudflare.com/counting-things-a-lot-of-different-things/

#![cfg_attr(not(feature = "std"), no_std)]
// #![feature(const_panic)] not needed for the new rust version

use codec::{Decode, Encode};
use frame_support::ensure;
use frame_support::log::info;
use frame_support::traits::IsSubType;
use sp_runtime::{
    traits::{DispatchInfoOf, SignedExtension, Saturating},
    transaction_validity::{InvalidTransaction, TransactionValidity, TransactionValidityError, ValidTransaction},
};
use sp_std::fmt::Debug;

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod test_pallet;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
mod weights;

pub use weights::WeightInfo;
use frame_support::traits::Contains;
use sp_runtime::traits::Zero;

#[frame_support::pallet]
pub mod pallet {
    use sp_std::convert::TryInto;
    use sp_std::num::NonZeroU16;
    use frame_support::weights::{extract_actual_weight, GetDispatchInfo};
    use frame_support::{dispatch::DispatchResult, log, pallet_prelude::*};
    use frame_support::dispatch::PostDispatchInfo;
    use sp_std::default::Default;
    use frame_support::traits::{Contains, IsSubType};
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::{Dispatchable};
    use sp_runtime::traits::Zero;
    use sp_std::boxed::Box;
    use sp_std::cmp::max;
    use sp_std::vec::Vec;
    use pallet_locker_mirror::{LockedInfoByAccount, LockedInfoOf};
    use crate::WeightInfo;

    /// The ratio between the quota and a particular window.
    ///
    /// ## Example:
    /// if ratio is 20 and the quota is 100 then each window should have maximum of 5 calls.
    /// max number of calls per window = quota / ratio.
    pub type QuotaToWindowRatio = NonZeroU16;

    /// Type to keep track of how many calls is in quota or used in a particular window.
    pub type NumberOfCalls = u16;

    /// A `BoundedVec` that can hold a list of `ConsumerStats` objects bounded by the size of WindowConfigs.
    pub type ConsumerStatsVec<T> = BoundedVec<ConsumerStats<<T as frame_system::Config>::BlockNumber>, WindowsConfigSize<T>>;

    /// Keeps track of the executed number of calls per window per consumer.
    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug)]
    pub struct ConsumerStats<BlockNumber> {
        // TODO: find a better name? maybe `stats_index`
        /// The index of this window in the timeline.
        pub timeline_index: BlockNumber,

        /// The number of calls executed during this window.
        pub used_calls: NumberOfCalls,
    }

    impl<BlockNumber> ConsumerStats<BlockNumber> {
        fn new(window_index: BlockNumber) -> Self {
            ConsumerStats {
                timeline_index: window_index,
                used_calls: 0,
            }
        }
    }

    /// Configuration of a rate limiting window in terms of length and ratio to quota.
    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug)]
    pub struct WindowConfig<BlockNumber> {
        /// The length of the window in number of blocks it will last.
        pub period: BlockNumber,

        /// The ratio between the quota and this window.
        pub quota_ratio: QuotaToWindowRatio,
    }

    impl<BlockNumber> WindowConfig<BlockNumber> {
        //TODO: try to also force period to be non zero.
        pub const fn new(period: BlockNumber, quota_ratio: Option<QuotaToWindowRatio>) -> Self {
            WindowConfig {
                period,
                quota_ratio: match quota_ratio {
                    Some(non_zero) => non_zero,
                    None => panic!("quota_ratio must be non zero"),
                },
            }
        }
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_locker_mirror::Config {
        /// The overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The call type from the runtime which has all the calls available in your runtime.
        type Call: Parameter
            + Dispatchable<Origin = Self::Origin, PostInfo = PostDispatchInfo>
            + GetDispatchInfo
            + From<frame_system::Call<Self>>
            + IsSubType<Call<Self>>
            + IsType<<Self as frame_system::Config>::Call>;

        /// The configurations that will be used to limit the usage of the allocated quota to these
        /// different configs.
        #[pallet::constant]
        type WindowsConfig: Get<Vec<WindowConfig<Self::BlockNumber>>>;

        /// Filter on which calls are permitted to be free.
        type CallFilter: Contains<<Self as Config>::Call>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;

        /// A calculation strategy to convert locked tokens info to a quota.
        type QuotaCalculationStrategy: QuotaCalculationStrategy<Self>;
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

    /// Keeps track of each windows usage for each consumer.
    #[pallet::storage]
    #[pallet::getter(fn window_stats_by_consumer)]
    pub(super) type WindowStatsByConsumer<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        ConsumerStatsVec<T>,
        ValueQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Free call was executed. [who, result]
        FreeCallResult(T::AccountId, DispatchResult),
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {

        /// Try to execute a call using the free allocated quota. This call may not execute because
        /// one of the following reasons:
        ///  * Caller has no free quota set.
        ///  * The caller has used all the allowed quota for at least one window config.
        ///
        /// Pre-validation:
        /// This call is pre validated using `FreeCallsPrevalidation` signed extension and will only
        /// be valid if the consumer can have a free call.
        #[pallet::weight({
            let boxed_call_info = call.get_dispatch_info();
            let boxed_call_weight = boxed_call_info.weight;
            let self_weight = <T as Config>::WeightInfo::try_free_call();

            let total_weight = self_weight.saturating_add(boxed_call_weight);
            (
                total_weight,
                boxed_call_info.class,
                Pays::No,
            )
        })]
        pub fn try_free_call(
            origin: OriginFor<T>,
            call: Box<<T as Config>::Call>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin.clone())?;

            let mut actual_weight = <T as Config>::WeightInfo::try_free_call();

            if T::CallFilter::contains(&call) &&
                Self::can_make_free_call(&sender, ShouldUpdateConsumerStats::YES) {

                let info = call.get_dispatch_info();

                // Dispatch the call
                let result = call.dispatch(origin);

                // Add the current weight for the boxed call
                actual_weight = actual_weight.saturating_add(extract_actual_weight(&result, &info));

                // Deposit an event with the result
                Self::deposit_event(Event::FreeCallResult(
                    sender,
                    result.map(|_| ()).map_err(|e| e.error),
                ));
            }

            Ok(PostDispatchInfo {
                actual_weight: Some(actual_weight),
                pays_fee: Pays::No,
            })
        }
    }

    pub enum ShouldUpdateConsumerStats {
        YES,
        NO,
    }

    impl<T: Config> Pallet<T> {
        /// Determine if `consumer` can have a free call and optionally update its window usage.
        ///
        /// Window usage for the caller `consumer` will only update if there is a quota and all of the
        /// previous window usages doesn't exceed the defined windows config.
        pub fn can_make_free_call(consumer: &T::AccountId, should_update_consumer_stats: ShouldUpdateConsumerStats) -> bool {
            let current_block = <frame_system::Pallet<T>>::block_number();

            let windows_config = T::WindowsConfig::get();

            if windows_config.is_empty() {
                return false;
            }

            let locked_info = <LockedInfoByAccount<T>>::get(consumer.clone());
            let quota = match T::QuotaCalculationStrategy::calculate(current_block, locked_info) {
                Some(quota) if quota > 0 => quota,
                _ => return false,
            };

            let old_stats: ConsumerStatsVec<T> = Self::window_stats_by_consumer(consumer.clone());
            let mut new_stats: ConsumerStatsVec<T> = Default::default();

            for (config_index, config) in windows_config.into_iter().enumerate() {
                let new_window_stats = Self::check_window(
                    current_block,
                    quota,
                    config,
                    old_stats.get(config_index),
                );

                match new_window_stats {
                    None => {
                        return false;
                    },
                    Some(window_stats) => {
                        if matches!(new_stats.try_push(window_stats), Err(_)) {
                            return false;
                        }
                    }
                };
            }

            if let ShouldUpdateConsumerStats::YES = should_update_consumer_stats {
                log::info!("{:?} updating window stats", consumer);
                <WindowStatsByConsumer<T>>::insert(
                    consumer.clone(),
                    new_stats,
                );
            }

            return true;
        }

        /// Checks if a window can allow one more call given its config and the last stored stats for
        /// the consumer.
        ///
        /// If the window can allow one more call, the new stats object is returned, otherwise `None`
        /// is returned.
        fn check_window(
            current_block: T::BlockNumber,
            quota: NumberOfCalls,
            config: WindowConfig<T::BlockNumber>,
            window_stats: Option<&ConsumerStats<T::BlockNumber>>,
        ) -> Option<ConsumerStats<T::BlockNumber>> {

            if config.period.is_zero() {
                return None;
            }

            let timeline_index = current_block / config.period;

            let reset_stats = || ConsumerStats::new(timeline_index);

            let mut stats = window_stats
                .map(|r| r.clone())
                .unwrap_or_else(reset_stats);

            if stats.timeline_index < timeline_index {
                stats = reset_stats();
            }

            let can_be_called = stats.used_calls < max(1, quota / config.quota_ratio);

            can_be_called.then(|| {
                stats.used_calls = stats.used_calls.saturating_add(1);
                stats
            })
        }
    }


    pub trait QuotaCalculationStrategy<T: Config> {
        fn calculate(current_block: T::BlockNumber, locked_info: Option<LockedInfoOf<T>>) -> Option<NumberOfCalls>;
    }
}

/// Validate `try_free_call` calls prior to execution. Needed to avoid a DoS attack since they are
/// otherwise free to be included into blockchain.
#[derive(Encode, Decode, Clone, Eq, PartialEq)]
pub struct FreeCallsPrevalidation<T: Config + Send + Sync>(sp_std::marker::PhantomData<T>)
    where
        <T as frame_system::Config>::Call: IsSubType<Call<T>>;

impl<T: Config + Send + Sync> Debug for FreeCallsPrevalidation<T>
    where
        <T as frame_system::Config>::Call: IsSubType<Call<T>>,
{
    #[cfg(feature = "std")]
    fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        write!(f, "FreeCallsPrevalidation")
    }

    #[cfg(not(feature = "std"))]
    fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        Ok(())
    }
}

impl<T: Config + Send + Sync> FreeCallsPrevalidation<T>
    where
        <T as frame_system::Config>::Call: IsSubType<Call<T>>,
{
    /// Create new `SignedExtension` to check runtime version.
    pub fn new() -> Self {
        Self(sp_std::marker::PhantomData)
    }
}

#[repr(u8)]
enum FreeCallsValidityError {
    /// The caller is out of quota.
    OutOfQuota = 0,

    /// The call cannot be free.
    CallCannotBeFree = 1,
}

impl From<FreeCallsValidityError> for u8 {
    fn from(err: FreeCallsValidityError) -> Self {
        err as u8
    }
}

impl<T: Config + Send + Sync> SignedExtension for FreeCallsPrevalidation<T>
    where
        <T as frame_system::Config>::Call: IsSubType<Call<T>>,
{
    const IDENTIFIER: &'static str = "FreeCallsPrevalidation";

    type AccountId = T::AccountId;
    type Call = <T as frame_system::Config>::Call;
    type AdditionalSigned = ();
    type Pre = ();


    fn additional_signed(&self) -> Result<Self::AdditionalSigned, TransactionValidityError> {
        Ok(())
    }

    fn validate(
        &self,
        who: &Self::AccountId,
        call: &Self::Call,
        _info: &DispatchInfoOf<Self::Call>,
        _len: usize,
    ) -> TransactionValidity {
        if let Some(local_call) = call.is_sub_type() {
            if let Call::try_free_call(boxed_call) = local_call {
                ensure!(T::CallFilter::contains(boxed_call), InvalidTransaction::Custom(FreeCallsValidityError::CallCannotBeFree.into()));
                ensure!(Pallet::<T>::can_make_free_call(who, ShouldUpdateConsumerStats::NO), InvalidTransaction::Custom(FreeCallsValidityError::OutOfQuota.into()));
            }
        }
        Ok(ValidTransaction::default())
    }
}
