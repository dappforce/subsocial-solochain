//! # Rate Limiter Module
//!
//! Module for rate limiting of free transactions on Subsocial network.
//! We use the technique of multiple fixed windows of different timespan to tracker the usage
//! of resources (transactions) per each account that asked for free transaction.
//!
//! Resources:
//! - https://cloud.google.com/architecture/rate-limiting-strategies-techniques
//! - https://www.figma.com/blog/an-alternative-approach-to-rate-limiting/
//! - https://www.codementor.io/@arpitbhayani/system-design-sliding-window-based-rate-limiter-157x7sburi
//! - https://blog.cloudflare.com/counting-things-a-lot-of-different-things/

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
	decl_module, decl_storage, decl_event, decl_error, ensure, Parameter,
	weights::{Pays, GetDispatchInfo, DispatchClass},
	traits::{Filter, Get, IsSubType},
};
use frame_system::{self as system, ensure_signed};
use sp_runtime::{
	RuntimeDebug, DispatchResult,
	traits::{Dispatchable, DispatchInfoOf, SignedExtension},
	transaction_validity::{
		TransactionValidity, ValidTransaction, InvalidTransaction, TransactionValidityError,
	},
};
use sp_std::{prelude::*, fmt::Debug};
use df_traits::{OnFreeTransaction, TrustHandler};

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

/// The type of a rate-limiting window.
/// It should be sufficient to have three types of windows, e.g. 5 minutes, 1 hour and 1 day.
/// We assume that the system may not need more than 256 types of rate limting windows.
pub type WindowType = u8;

// TODO Think: Maybe it could be a generic type?
/// One permit is one transaction.
pub type PermitUnit = u16;

// TODO maybe rename to TimeWindow WindowConfig SlidingWindow or RateLimitingWindow
#[derive(Encode, Decode, Clone, Eq, PartialEq, PartialOrd, RuntimeDebug)]
pub struct RateConfig<BlockNumber> {
	/// Duration of a period in the number of blocks.
	pub period: BlockNumber,

	// TODO or 'max permits' or 'permits per second' or 'period_limit' or 'quota(s)'
	/// The number of permissions available per account during one period.
	pub max_permits: PermitUnit,
}

// TODO rename to UsageStats or UsageTracker or QuotaTracker or PermitTracker
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct ConsumerStats<BlockNumber> {
	/// A block number of the last call made by this account.
	pub last_window: BlockNumber,

	/// A number of permits consumed by a given user in the current period.
	pub consumed_permits: PermitUnit,
}

impl<BlockNumber> ConsumerStats<BlockNumber> {
	fn new(last_window: BlockNumber) -> Self {
		ConsumerStats {
			last_window,
			consumed_permits: 0,
		}
	}
}

/// The pallet's configuration trait.
pub trait Config: system::Config {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Config>::Event>;

	/// The call type from the runtime which has all the calls available in your runtime.
	type Call: Parameter + GetDispatchInfo + Dispatchable<Origin = Self::Origin>;

	type CallFilter: Filter<<Self as Config>::Call>;

	// TODO Rename to RateLimitingWindows or SlidingWindows?
	type RateConfigs: Get<Vec<RateConfig<Self::BlockNumber>>>;

	type TrustHandler: TrustHandler<Self::AccountId>;
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as frame_system::Config>::AccountId,
	{
		FreeCallResult(AccountId, DispatchResult),
	}
);

decl_error! {
	pub enum Error for Module<T: Config> {}
}

decl_storage! {
	trait Store for Module<T: Config> as RateLimiterModule {

		// TODO rename to 'UsageByAccount' or 'UsageTrackers'?
		pub StatsByAccount get(fn stats_by_account):
			double_map
				hasher(blake2_128_concat) T::AccountId,
				hasher(twox_64_concat) WindowType
			=> Option<ConsumerStats<T::BlockNumber>>;
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {

		// Sorted vector of rate-limiting rate limiting windows.
		const RateConfigs: Vec<RateConfig<T::BlockNumber>> = {
			let mut v = T::RateConfigs::get();

			// It is important to have the windows sorted by a period duration in ascending order.
			// Because if a user has no free call in a smaller window,
			// then it does not make sense to check the other larger windows.
			v.sort_by_key(|x| x.period);
			v
		};

		// Initializing errors
		type Error = Error<T>;

		// Initializing events
		fn deposit_event() = default;

		// Extrinsics

		// TODO implement drop of the whole double map of stats
		// if a last window of the largest period is < the current window of this period.
		// maybe this will be helpful?
		// https://substrate.dev/rustdocs/v3.0.0/frame_support/storage/migration/fn.put_storage_value.html

		#[weight = {
			let dispatch_info = call.get_dispatch_info();
			(
				// TODO: use benchmarking for setting a weight
				dispatch_info.weight.saturating_add(T::DbWeight::get().reads_writes(2, 1)),
				DispatchClass::Normal,
				Pays::No,
			)
		}]
		fn try_free_call(origin, call: Box<<T as Config>::Call>) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;

			if Self::can_account_make_free_call_and_update_stats(&sender) {

				// Dispatch the call
				let result = call.dispatch(origin);

				// Deposit an event with the result
				Self::deposit_event(
					RawEvent::FreeCallResult(
						sender,
						result.map(|_| ()).map_err(|e| e.error),
					)
				);
			}

			Ok(())
		}
	}
}

impl<T: Config> Module<T> {
	fn update_account_stats(
		who: &T::AccountId,
		window_type: WindowType,
		stats: &mut ConsumerStats<T::BlockNumber>,
	) {
		stats.consumed_permits = stats.consumed_permits.saturating_add(1);
		StatsByAccount::<T>::insert(who, window_type, stats);
	}

	/// This function can update stats of a corresponding window,
	/// if account is eligible to have a free call withing a given window.
	fn can_account_make_free_call<F>(sender: &T::AccountId, update_stats: F) -> bool
	where
		F: FnOnce(&T::AccountId, WindowType, &mut ConsumerStats<T::BlockNumber>) + Copy,
	{
		if !T::TrustHandler::is_trusted_account(sender) {
			return false
		}

		let current_block = frame_system::Module::<T>::block_number();
		let windows = T::RateConfigs::get();
		let mut has_free_calls = false;

		for (i, window) in windows.into_iter().enumerate() {
			let window_type = i as WindowType;

			// Calculate the current window
			let current_window = current_block / window.period;

			let reset_stats = || ConsumerStats::new(current_window);

			// Get stats for this type of window
			let mut stats =
				StatsByAccount::<T>::get(&sender, window_type).unwrap_or_else(reset_stats);

			// If this is a new window for the user, reset their consumed permits.
			if stats.last_window < current_window {
				stats = reset_stats();
			}

			// Check that the user has an available free call
			has_free_calls = stats.consumed_permits < window.max_permits;

			if !has_free_calls {
				break
			}

			update_stats(&sender, window_type, &mut stats);
		}

		has_free_calls
	}

	pub fn check_account_can_make_free_call(sender: &T::AccountId) -> bool {
		Self::can_account_make_free_call(sender, |_, _, _| ())
	}

	pub fn can_account_make_free_call_and_update_stats(sender: &T::AccountId) -> bool {
		Self::can_account_make_free_call(sender, Self::update_account_stats)
	}
}

/// Validate `try_free_call` calls prior to execution. Needed to avoid a DoS attack since they are
/// otherwise free to place on chain.
#[derive(Encode, Decode, Clone, Eq, PartialEq)]
pub struct PrevalidateFreeCall<T: Config + Send + Sync>(sp_std::marker::PhantomData<T>)
where
	<T as frame_system::Config>::Call: IsSubType<Call<T>>;

impl<T: Config + Send + Sync> Debug for PrevalidateFreeCall<T>
where
	<T as frame_system::Config>::Call: IsSubType<Call<T>>,
{
	#[cfg(feature = "std")]
	fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		write!(f, "PrevalidateFreeCall")
	}

	#[cfg(not(feature = "std"))]
	fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		Ok(())
	}
}

impl<T: Config + Send + Sync> PrevalidateFreeCall<T>
where
	<T as frame_system::Config>::Call: IsSubType<Call<T>>,
{
	/// Create new `SignedExtension` to check runtime version.
	pub fn new() -> Self {
		Self(sp_std::marker::PhantomData)
	}
}

#[repr(u8)]
enum ValidityError {
	DisallowedCall = 0,
	UserNotPermitted = 1,
}

impl From<ValidityError> for u8 {
	fn from(err: ValidityError) -> Self {
		err as u8
	}
}

impl<T: Config + Send + Sync> SignedExtension for PrevalidateFreeCall<T>
where
	<T as frame_system::Config>::Call: IsSubType<Call<T>>,
{
	const IDENTIFIER: &'static str = "PrevalidateFreeCall";
	type AccountId = T::AccountId;
	type Call = <T as frame_system::Config>::Call;
	type AdditionalSigned = ();
	type Pre = ();

	fn additional_signed(&self) -> Result<Self::AdditionalSigned, TransactionValidityError> {
		Ok(())
	}

	/// <weight>
	/// The weight of this logic is included in the `attest` dispatchable.
	/// </weight>
	fn validate(
		&self,
		who: &Self::AccountId,
		call: &Self::Call,
		_info: &DispatchInfoOf<Self::Call>,
		_len: usize,
	) -> TransactionValidity {
		if let Some(local_call) = call.is_sub_type() {
			if let Call::try_free_call(boxed_call) = local_call {
				ensure!(
					T::TrustHandler::is_trusted_account(who),
					InvalidTransaction::Custom(ValidityError::UserNotPermitted.into())
				);
				ensure!(
					T::CallFilter::filter(boxed_call),
					InvalidTransaction::Custom(ValidityError::DisallowedCall.into())
				);
			}
		}
		Ok(ValidTransaction::default())
	}
}

impl<T: Config> OnFreeTransaction<T::AccountId> for Module<T> {
	fn can_account_make_free_call(sender: &T::AccountId) -> bool {
		Self::check_account_can_make_free_call(sender)
	}
}
