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

use codec::{Decode, Encode};
use frame_support::ensure;
use frame_support::traits::IsSubType;
use sp_runtime::{
    traits::{DispatchInfoOf, SignedExtension, Saturating},
    transaction_validity::{InvalidTransaction, TransactionValidity, TransactionValidityError, ValidTransaction},
};
use sp_std::fmt::Debug;

pub use pallet::*;

#[cfg(test)]
mod mock;

// #[cfg(test)]
// mod test_pallet;
//
// #[cfg(test)]
// mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
mod weights;

pub use weights::WeightInfo;
use frame_support::traits::Contains;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::weights::GetDispatchInfo;
    use frame_support::{dispatch::DispatchResult, log, pallet_prelude::*};
    use frame_support::dispatch::PostDispatchInfo;
    use frame_support::traits::{Contains, IsSubType};
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::Dispatchable;
    use sp_runtime::traits::Zero;
    use sp_std::boxed::Box;
    use sp_std::cmp::max;
    use sp_std::vec::Vec;
    use crate::WeightInfo;

    // TODO: find a better name
    // TODO: disallow users to enter 0
    // ideas for name: Fraction, Shares, ....
    /// The ratio between the quota and a particular window.
    ///
    /// ## Example:
    /// if ratio is 20 and the quota is 100 then each window should have maximum of 5 calls.
    /// max number of calls per window = quota / ratio.
    pub type QuotaToWindowRatio = u16;

    /// Type to keep track of how many calls is in quota or used in a particular window.
    pub type NumberOfCalls = u16;

    /// Defines the type that will be used to describe window size and config index.
    /// 3~4 windows should be sufficient (1 block, 3 mins, 1 hour, 1 day).
    pub type WindowConfigsSize = u8;

    /// Keeps track of the executed number of calls per window per account.
    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug)]
    pub struct ConsumerStats<BlockNumber> {
        // TODO: find a better name?
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

    /// Configuration of window.
    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug)]
    pub struct WindowConfig<BlockNumber> {
        /// The span of the window in number of blocks it will last.
        pub period: BlockNumber,

        /// The ratio between the quota and a this window.
        pub quota_ratio: QuotaToWindowRatio,
    }

    impl<BlockNumber> WindowConfig<BlockNumber> {
        pub const fn new(period: BlockNumber, quota_ratio: QuotaToWindowRatio) -> Self {
            WindowConfig {
                period,
                quota_ratio,
            }
        }
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
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
        const WINDOWS_CONFIG: &'static [WindowConfig<Self::BlockNumber>];

        /// The origin which can change the allocated quota for accounts.
        type ManagerOrigin: EnsureOrigin<Self::Origin>;

        /// Filter on which calls are permitted to be free.
        type CallFilter: Contains<<Self as Config>::Call>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
    }

    #[pallet::extra_constants]
    impl<T: Config> Pallet<T> {
        /// The configurations that will be used to limit the usage of the allocated quota to these
        /// different configs.
        fn windows_config() -> &'static [WindowConfig<T::BlockNumber>] { T::WINDOWS_CONFIG }
    }

    /// Keeps tracks of the allocated quota to each account.
    #[pallet::storage]
    #[pallet::getter(fn quota_by_account)]
    pub(super) type QuotaByAccount<T: Config> = StorageMap<
        _,
        Twox64Concat,
        T::AccountId,
        NumberOfCalls,
        OptionQuery,
    >;

    /// Keeps track of each windows usage for each account.
    #[pallet::storage]
    #[pallet::getter(fn window_stats_by_account)]
    pub(super) type WindowStatsByAccount<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Twox64Concat,
        // Index of the window in the list of window configurations.
        WindowConfigsSize,
        ConsumerStats<T::BlockNumber>,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// free call was executed. [who, result]
        FreeCallResult(T::AccountId, DispatchResult),

        /// quota have been changed for an account. [who, allocated_quota]
        AccountQuotaChanged(T::AccountId, NumberOfCalls),
    }

    /// Try to execute a call using the free allocated quota. This call may not execute because one of
    /// the following reasons:
    ///  * Caller have no free quota set.
    ///  * The caller have used all the allowed intersects for one or all of the current windows.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        // TODO: fix weight
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
                Self::can_make_free_call(&sender, ShouldUpdateAccountStats::YES) {
                // Add the current weight for the boxed call
                actual_weight = actual_weight.saturating_add(call.get_dispatch_info().weight);

                // Dispatch the call
                let result = call.dispatch(origin);

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

        // TODO: remove me and migrate to a mirroring pallet for
        /// Set an account's quota. This will fail if the caller doesn't match `T::ManagerOrigin`.
        #[pallet::weight(10_000)]
        pub fn change_account_quota(
            origin: OriginFor<T>,
            account: T::AccountId,
            quota: NumberOfCalls,
        ) -> DispatchResult {
            let _ = T::ManagerOrigin::ensure_origin(origin);


            // TODO: create clear_account_quota extrinsic
            // if quota == 0 {
            //     <QuotaByAccount<T>>::remo(account.clone(), quota);
            // } else {
            <QuotaByAccount<T>>::insert(account.clone(), quota);
            // }
            Self::deposit_event(Event::AccountQuotaChanged(account, quota));

            Ok(())
        }
    }

    struct Window<T: Config> {
        account: T::AccountId,
        config_index: WindowConfigsSize,
        config: &'static WindowConfig<T::BlockNumber>,
        timeline_index: T::BlockNumber,
        stats: ConsumerStats<T::BlockNumber>,
        can_be_called: bool,
    }

    impl<T: Config> Window<T> {
        // TODO: refactor this into more lightweight version??
        fn build(
            account: T::AccountId,
            quota: NumberOfCalls,
            current_block: T::BlockNumber,
            config_index: WindowConfigsSize,
            config: &'static WindowConfig<T::BlockNumber>,
            window_stats: Option<ConsumerStats<T::BlockNumber>>,
        ) -> Self {
            let timeline_index = current_block / config.period;

            let reset_stats = || ConsumerStats::new(timeline_index);

            let mut stats = window_stats.unwrap_or_else(reset_stats);

            if stats.timeline_index < timeline_index {
                stats = reset_stats();
            }

            let can_be_called = stats.used_calls < max(1, quota / config.quota_ratio);

            Window {
                account: account.clone(),
                config_index,
                config,
                timeline_index,
                stats,
                can_be_called,
            }
        }

        fn increment_window_stats(&mut self) {
            self.stats.used_calls = self.stats.used_calls.saturating_add(1);
            <WindowStatsByAccount<T>>::insert(
                self.account.clone(),
                self.config_index,
                self.stats.clone(),
            );
        }
    }

    pub enum ShouldUpdateAccountStats {
        YES,
        NO,
    }

    impl<T: Config> Pallet<T> {
        /// Determine if `account` can have a free call and optionally update user window usage.
        ///
        /// Window usage for the caller `account` will only update if there is quota and all of the
        /// previous window usages doesn't exceed the defined windows config.
        pub fn can_make_free_call(account: &T::AccountId, should_update_account_stats: ShouldUpdateAccountStats) -> bool {
            let current_block = <frame_system::Pallet<T>>::block_number();

            let windows_config = T::WINDOWS_CONFIG;

            if windows_config.is_empty() {
                return false;
            }

            let quota = Self::quota_by_account(account);

            let quota = match quota {
                Some(quota) if quota > 0 => quota,
                _ => return false,
            };

            let mut windows: Vec<Window<T>> = Vec::new();
            let mut can_call = false;

            // TODO: sort configs to allow this to fail fast
            // TODO: using period and ratio
            for (config_index, config) in windows_config.into_iter().enumerate() {
                let config_index = config_index as WindowConfigsSize;

                if config.period.is_zero() || config.quota_ratio.is_zero() {
                    can_call = false;
                    break;
                }

                let window = Window::build(
                    account.clone(),
                    quota,
                    current_block,
                    config_index,
                    config,
                    Self::window_stats_by_account(account.clone(), config_index),
                );

                can_call = window.can_be_called;
                if !can_call {
                    break;
                }

                windows.push(window);
            }

            if can_call {
                log::info!("{:?} can have this free call", account);
                if let ShouldUpdateAccountStats::YES = should_update_account_stats {
                    log::info!("{:?} updating window stats", account);
                    for window in &mut windows {
                        window.increment_window_stats();
                    }
                }
            } else {
                log::info!("{:?} don't have free calls", account);
            }

            can_call
        }
    }
}


/// Validate `try_free_call` calls prior to execution. Needed to avoid a DoS attack since they are
/// otherwise free to place on chain.
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
    DisallowedCall = 1,
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
                ensure!(T::CallFilter::contains(boxed_call), InvalidTransaction::Custom(FreeCallsValidityError::DisallowedCall.into()));
                ensure!(Pallet::<T>::can_make_free_call(who, ShouldUpdateAccountStats::NO), InvalidTransaction::Custom(FreeCallsValidityError::OutOfQuota.into()));
            }
        }
        Ok(ValidTransaction::default())
    }
}
