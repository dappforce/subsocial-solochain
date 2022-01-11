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

pub use pallet::*;
//
// #[cfg(test)]
// mod mock;
//
// #[cfg(test)]
// mod test_pallet;
//
// #[cfg(test)]
// mod tests;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{dispatch::DispatchResult, log, pallet_prelude::*};
    use frame_support::weights::GetDispatchInfo;
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::Dispatchable;
    use sp_std::boxed::Box;
    use sp_std::vec::Vec;
    use sp_std::cmp::max;


    // TODO: find a better name
    /// The ratio between the quota and a particular window.
    ///
    /// ## Example:
    /// if ratio is 20 and the quota is 100 then each window should have maximum of 5 calls.
    /// max number of calls per window = quota / ratio.
    pub type QuotaToWindowRatio = u16;

    /// Type to keep track of how many calls is in quota or used in a particular window.
    pub type NumberOfCalls = u16;

    /// Defines the type that will be used to describe window size and window index.
    /// 3~4 windows should be sufficient (1 block, 3 mins, 1 hour, 1 day).
    pub type WindowConfigsSize = u8;

    /// Keeps track of the executed number of calls per window per account.
    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug)]
    pub struct WindowStats<BlockNumber> {
        /// The index of this window in the timeline.
        pub index: BlockNumber,

        /// The number of calls executed during this window.
        pub num_of_calls: NumberOfCalls,
    }

    impl<BlockNumber> WindowStats<BlockNumber> {
        fn new(window_index: BlockNumber) -> Self {
            WindowStats {
                index: window_index,
                num_of_calls: 0,
            }
        }
    }

    /// Configuration of window.
    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug)]
    pub struct WindowConfig<BlockNumber> {
        /// The span of the window.
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
        type Call: Parameter + GetDispatchInfo + Dispatchable<Origin=Self::Origin>;

        /// The configurations that will be used to limit the usage of the allocated quota to these
        /// different configs.
        #[pallet::constant]
        type WindowsConfig: Get<Vec<WindowConfig<Self::BlockNumber>>>;

        /// The origin which can change the allocated quota for accounts.
        type ManagerOrigin: EnsureOrigin<Self::Origin>;
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
        WindowConfigsSize,
        WindowStats<T::BlockNumber>,
    >;


    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// free call was executed. [who, result]
        FreeCallResult(T::AccountId, DispatchResult),
    }

    /// Try to execute a call using the free allocated quota. This call may not execute because one of
    /// the following reasons:
    ///  * Caller have no free quota set.
    ///  * The caller have used all the allowed intersects for one or all of the current windows.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        // TODO: fix weight
        #[pallet::weight(10_000)]
        pub fn try_free_call(origin: OriginFor<T>, call: Box<<T as Config>::Call>) -> DispatchResult {
            let sender = ensure_signed(origin.clone())?;

            if Self::can_make_free_call_and_update_stats(&sender) {

                // Dispatch the call
                let result = call.dispatch(origin);

                // Deposit an event with the result
                Self::deposit_event(
                    Event::FreeCallResult(
                        sender,
                        result.map(|_| ()).map_err(|e| e.error),
                    )
                );
            }

            Ok(())
        }


        /// Set an account's quota. This will fail if the caller doesn't match `T::ManagerOrigin`.
        #[pallet::weight(10_000)]
        pub fn change_account_quota(origin: OriginFor<T>, account: T::AccountId, quota: NumberOfCalls) -> DispatchResult {
            let _ = T::ManagerOrigin::ensure_origin(origin);

            <QuotaByAccount<T>>::insert(account, quota);

            Ok(())
        }
    }

    struct Window<T: Config> {
        account: T::AccountId,
        config_index: WindowConfigsSize,
        config: WindowConfig<T::BlockNumber>,
        window_index: T::BlockNumber,
        stats: WindowStats<T::BlockNumber>,
        can_be_called: bool,
    }

    impl<T: Config> Window<T> {
        fn build(
            account: T::AccountId,
            quota: NumberOfCalls,
            current_block: T::BlockNumber,
            config_index: WindowConfigsSize,
            config: WindowConfig<T::BlockNumber>,
            window_stats: Option<WindowStats<T::BlockNumber>>,
        ) -> Self {
            let config_index = config_index as WindowConfigsSize;

            let window_index = current_block / config.period;

            let reset_stats = || { WindowStats::new(window_index) };

            let mut stats = window_stats.unwrap_or_else(reset_stats);

            if stats.index < window_index {
                stats = reset_stats();
            }

            let can_be_called = stats.num_of_calls < max(1, quota / config.quota_ratio);

            Window {
                account: account.clone(),
                config_index,
                config,
                window_index,
                stats,
                can_be_called,
            }
        }

        fn increment_window_stats(&mut self) {
            self.stats.num_of_calls = self.stats.num_of_calls.saturating_add(1);
            <WindowStatsByAccount<T>>::insert(self.account.clone(), self.config_index, self.stats.clone());
        }
    }

    impl<T: Config> Pallet<T> {
        fn can_make_free_call_and_update_stats(account: &T::AccountId) -> bool {
            let current_block = <frame_system::Pallet<T>>::block_number();
            let windows_config = T::WindowsConfig::get();
            let quota = Self::quota_by_account(account);

            let quota = match quota {
                Some(quota) => quota,
                None => return false,
            };

            let mut windows: Vec<Window<T>> = Vec::new();
            let mut can_call = false;

            for (config_index, config) in windows_config
                .into_iter()
                .enumerate() {
                let config_index = config_index as WindowConfigsSize;
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
                for window in &mut windows {
                    window.increment_window_stats();
                }
            } else {
                log::info!("{:?} don't have free calls", account);
            }

            can_call
        }
    }
}
