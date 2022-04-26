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
use frame_support::traits::IsSubType;
use sp_runtime::traits::DispatchInfoOf;
use sp_runtime::traits::SignedExtension;
use sp_runtime::transaction_validity::InvalidTransaction;
use sp_runtime::transaction_validity::TransactionValidity;
use sp_runtime::transaction_validity::TransactionValidityError;
use sp_runtime::transaction_validity::ValidTransaction;
use sp_std::fmt::Debug;

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod test_pallet;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
mod weights;
pub mod quota;
pub mod config;
pub mod quota_strategy;
pub mod stats;

pub use weights::WeightInfo;
use frame_support::traits::Contains;
use scale_info::TypeInfo;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::weights::{extract_actual_weight, GetDispatchInfo};
    use frame_support::{dispatch::DispatchResult, log, pallet_prelude::*};
    use frame_support::dispatch::PostDispatchInfo;
    use frame_support::metadata::StorageEntryModifier::Default;
    use frame_support::traits::{Contains, IsSubType};
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::{Dispatchable};
    use sp_runtime::traits::Zero;
    use sp_std::boxed::Box;
    use sp_std::cmp::max;
    use sp_std::vec::Vec;
    use pallet_locker_mirror::{BalanceOf, LockedInfo, LockedInfoByAccount, LockedInfoOf};
    use pallet_utils::bool_to_option;
    use scale_info::TypeInfo;
    use crate::config::{RateLimiterConfig, WindowConfig, WindowsConfigSize};
    use crate::quota::{calculate_quota, FractionOfMaxQuota, NumberOfCalls};
    use crate::quota_strategy::MaxQuotaCalculationStrategy;
    use crate::stats::{ConsumerStats, WindowStats, WindowStatsVec};
    use crate::WeightInfo;

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
        type RateLimiterConfig: Get<RateLimiterConfig<Self::BlockNumber>>;

        /// Filter on which calls are permitted to be free.
        type CallFilter: Contains<<Self as Config>::Call>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;

        /// A calculation strategy to convert locked tokens info to a max quota per largest window.
        type MaxQuotaCalculationStrategy: MaxQuotaCalculationStrategy<Self::AccountId, Self::BlockNumber, BalanceOf<Self>>;

        /// Maximum number of accounts that can be added as eligible at a time.
        //TODO: remove this after we integrate locking tokens
        #[pallet::constant]
        type AccountsSetLimit: Get<u32>;

        /// Amount of free quota granted to eligible accounts.
        //TODO: remove this after we integrate locking tokens
        #[pallet::constant]
        type FreeQuotaPerEligibleAccount: Get<NumberOfCalls>;
    }

    /// Keeps track of each windows usage for each consumer.
    #[pallet::storage]
    #[pallet::getter(fn stats_by_consumer)]
    pub(super) type StatsByConsumer<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        ConsumerStats<T>,
        OptionQuery,
    >;

    /// Keeps track of all eligible accounts for free calls
    //TODO: remove this after we integrate locking tokens
    #[pallet::storage]
    pub(super) type EligibleAccounts<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        bool,
        ValueQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Free call was executed.
        FreeCallResult { who: T::AccountId, result: DispatchResult },

        /// List of eligible accounts added.
        //TODO: remove this after we integrate locking tokens
        EligibleAccountsAdded { added_accounts: u16 },
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
            let consumer = ensure_signed(origin.clone())?;

            let mut actual_weight = <T as Config>::WeightInfo::try_free_call();

            let current_block = <frame_system::Pallet<T>>::block_number();

            let maybe_new_stats = bool_to_option(T::CallFilter::contains(&call))
                .and_then(|_| Self::can_make_free_call(&consumer, current_block));

            if let Some(new_stats) = maybe_new_stats {

                Self::update_consumer_stats(consumer.clone(), new_stats);

                let info = call.get_dispatch_info();

                // Dispatch the call
                let result = call.dispatch(origin);

                // Add the current weight for the boxed call
                actual_weight = actual_weight.saturating_add(extract_actual_weight(&result, &info));

                // Deposit an event with the result
                Self::deposit_event(Event::FreeCallResult {
                    who: consumer,
                    result: result.map(|_| ()).map_err(|e| e.error)
                });
            }

            Ok(PostDispatchInfo {
                actual_weight: Some(actual_weight),
                pays_fee: Pays::No,
            })
        }

        #[pallet::weight(10_000)]
        pub fn add_eligible_accounts(
            origin: OriginFor<T>,
            eligible_accounts: BoundedVec<T::AccountId, T::AccountsSetLimit>,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            let accounts_len = eligible_accounts.len();

            for eligible_account in eligible_accounts {
                <EligibleAccounts<T>>::insert(&eligible_account, true);
            }

            Self::deposit_event(Event::EligibleAccountsAdded { added_accounts: accounts_len as u16 });
            Ok(Pays::No.into())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Determine if the `consumer` can have a free call.
        ///
        /// If the consumer can have a free call the new stats that should be applied will be returned,
        /// otherwise `None` is returned.
        pub fn can_make_free_call(
            consumer: &T::AccountId,
            current_block: T::BlockNumber,
        ) -> Option<ConsumerStats<T>> {
            let RateLimiterConfig::<T::BlockNumber> {
                windows_configs,
                hash: config_hash,
            } = T::RateLimiterConfig::get();

            if windows_configs.is_empty() {
                return None;
            }

            let max_quota = match Self::get_max_quota(consumer.clone(), current_block) {
                0 => return None,
                max_quota => max_quota,
            };

            let get_default_stats = || ConsumerStats::new(
                WindowStatsVec::default(),
                config_hash,
            );

            let old_stats: ConsumerStats<T> = Self::stats_by_consumer(consumer.clone())
                .filter(|stats| stats.config_hash == config_hash) // filter out stats with a different config hash
                .unwrap_or_else(get_default_stats);

            let mut new_stats: ConsumerStats<T> = get_default_stats();

            for (config_index, config) in windows_configs.into_iter().enumerate() {
                let window_stats = Self::is_call_allowed_in_window(
                    current_block,
                    max_quota,
                    config,
                    old_stats.get_window_stats(config_index),
                )?;

                new_stats.try_push_window_stats(window_stats).ok()?;
            }

            return Some(new_stats);
        }

        /// Checks if a window can allow one more call given its config and the last stored stats for
        /// the consumer.
        ///
        /// If the window can allow one more call, the new stats object is returned, otherwise `None`
        /// is returned.
        fn is_call_allowed_in_window(
            current_block: T::BlockNumber,
            max_quota: NumberOfCalls,
            config: WindowConfig<T::BlockNumber>,
            window_stats: Option<&WindowStats<T::BlockNumber>>,
        ) -> Option<WindowStats<T::BlockNumber>> {

            if config.period.is_zero() {
                return None;
            }

            let current_timeline_index = current_block / config.period;

            let reset_stats = || WindowStats::new(current_timeline_index);

            let mut stats = window_stats
                .map(|r| r.clone())
                .unwrap_or_else(reset_stats);

            // We need to reset stats if we moved to a new window.
            if stats.timeline_index < current_timeline_index {
                stats = reset_stats();
            }

            let can_be_called = stats.used_calls < calculate_quota(max_quota, config.fraction_of_max_quota);

            can_be_called.then(|| {
                stats.used_calls = stats.used_calls.saturating_add(1);
                stats
            })
        }

        pub fn update_consumer_stats(consumer: T::AccountId, new_stats: ConsumerStats<T>) {
            log::info!("{:?} updating consumer stats", consumer);
            <StatsByConsumer<T>>::insert(
                consumer,
                new_stats,
            );
        }

        pub fn get_max_quota(
            consumer: T::AccountId,
            current_block: T::BlockNumber,
        ) -> NumberOfCalls {
            let locked_info = <LockedInfoByAccount<T>>::get(consumer.clone());
            return match T::MaxQuotaCalculationStrategy::calculate(consumer.clone(), current_block, locked_info) {
                Some(max_quota) => max_quota,
                _ => Zero::zero(),
            }
        }
    }
}

/// Validate `try_free_call` calls prior to execution. Needed to avoid a DoS attack since they are
/// otherwise free to be included into blockchain.
#[derive(Encode, Decode, Clone, Eq, PartialEq, TypeInfo)]
#[scale_info(skip_type_params(T))]
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
pub enum FreeCallsValidityError {
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
            if let Call::try_free_call { call: boxed_call } = local_call {
                let current_block = <frame_system::Pallet<T>>::block_number();

                ensure!(
                    T::CallFilter::contains(boxed_call),
                    InvalidTransaction::Custom(FreeCallsValidityError::CallCannotBeFree.into()),
                );
                ensure!(
                    Pallet::<T>::can_make_free_call(who, current_block).is_some(),
                    InvalidTransaction::Custom(FreeCallsValidityError::OutOfQuota.into()),
                );
            }
        }
        Ok(ValidTransaction::default())
    }
}
