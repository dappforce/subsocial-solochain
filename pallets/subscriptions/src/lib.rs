//! # Subscription Module
//!
//! The Subscription module allows supporters of any space that represents a creator or
//! a nonprofit organization to contribute financially (with native tokens) a daily, weekly,
//! monthly, quarterly or yearly basis. This module allows content creators to monetize
//! their content via recurring payments from their supporters.
//!
//! This pallet provides a way for creators to create a list of subscription plans (aka levels, tiers)
//! and specify a different price and time period per each plan. There are several pre-built
//! subscription periods: `Daily`, `Weekly`, `Monthly`, `Quarterly` and `Yearly`.
//!
//! This pallet uses Substrate's Schedule pallet to schedule recurring transfers from supporters'
//! (patrons') wallets to creators' wallets.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};
use sp_std::prelude::*;
use sp_runtime::RuntimeDebug;

use frame_support::{
	decl_module, decl_storage, decl_event, decl_error, ensure,
	dispatch::{Dispatchable, DispatchResult},
	traits::{
		Get, Currency, ExistenceRequirement,
		schedule::Named as ScheduleNamed, LockIdentifier,
	}
};
use frame_system::{self as system, ensure_signed, ensure_root};

use pallet_spaces::Module as Spaces;
use pallet_utils::{Module as Utils, SpaceId, Content, WhoAndWhen, remove_from_vec};

/*#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;*/

pub mod functions;

const SUBSCRIPTIONS_ID: LockIdentifier = *b"subscrip";

pub type SubscriptionPlanId = u64;
pub type SubscriptionId = u64;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum SubscriptionPeriod<BlockNumber> {
	Daily,
	Weekly,
	Monthly,
	Quarterly,
	Yearly,
	Custom(BlockNumber), // Currently not supported
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct SubscriptionPlan<T: Config> {
	pub id: SubscriptionPlanId,
	pub created: WhoAndWhen<T>,
	pub updated: Option<WhoAndWhen<T>>,

	pub is_active: bool,

	pub content: Content,
	pub space_id: SpaceId, // Describes what space is this plan related to

	pub wallet: Option<T::AccountId>,
	pub price: BalanceOf<T>,
	pub period: SubscriptionPeriod<T::BlockNumber>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Subscription<T: Config> {
	pub id: SubscriptionId,
	pub created: WhoAndWhen<T>,
	pub updated: Option<WhoAndWhen<T>>,

	pub is_active: bool,

	pub wallet: Option<T::AccountId>,
	pub plan_id: SubscriptionPlanId,
}

type BalanceOf<T> = <<T as pallet_utils::Config>::Currency as Currency<<T as system::Config>::AccountId>>::Balance;

/// The pallet's configuration trait.
pub trait Config:
	system::Config
	+ pallet_utils::Config
	+ pallet_spaces::Config
	+ pallet_sudo::Config
{
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Config>::Event>;

	type Subscription: Dispatchable<Origin=<Self as system::Config>::Origin> + From<Call<Self>>;

	type Scheduler: ScheduleNamed<Self::BlockNumber, Self::Subscription, Self::Origin>;

	type DailyPeriodInBlocks: Get<Self::BlockNumber>;

	type MonthlyPeriodInBlocks: Get<Self::BlockNumber>;

	type WeeklyPeriodInBlocks: Get<Self::BlockNumber>;

	type QuarterlyPeriodInBlocks: Get<Self::BlockNumber>;

	type YearlyPeriodInBlocks: Get<Self::BlockNumber>;
}

pub const FIRST_SUBSCRIPTION_PLAN_ID: u64 = 1;

decl_storage! {
	trait Store for Module<T: Config> as SubscriptionsModule {

		// Plans:

		/// The next subscription plan id.
		pub NextPlanId get(fn next_plan_id): SubscriptionPlanId = FIRST_SUBSCRIPTION_PLAN_ID;

		pub PlanById get(fn plan_by_id):
			map hasher(twox_64_concat) SubscriptionPlanId => Option<SubscriptionPlan<T>>;

		pub PlanIdsBySpace get(fn plan_ids_by_space):
			map hasher(twox_64_concat) SpaceId => Vec<SubscriptionPlanId>;

		// Subscriptions:

		pub NextSubscriptionId get(fn next_subscription_id): SubscriptionId = 1;

		pub SubscriptionById get(fn subscription_by_id):
			map hasher(twox_64_concat) SubscriptionId => Option<Subscription<T>>;

		pub SubscriptionIdsByPatron get(fn subscription_ids_by_patron):
			map hasher(blake2_128_concat) T::AccountId => Vec<SubscriptionId>;

		pub SubscriptionIdsBySpace get(fn subscription_ids_by_space):
			map hasher(twox_64_concat) SpaceId => Vec<SubscriptionId>;

		// Wallets

		/// A recipient's wallet that receives transfers sent from their subscribers.
		pub RecipientWallet get(fn recipient_wallet):
			map hasher(twox_64_concat) SpaceId => Option<T::AccountId>;

		/// A subscriber's wallet that is used to pay for their active subscriptions.
		pub SubscriberWallet get(fn subscriber_wallet):
			map hasher(twox_64_concat) T::AccountId => Option<T::AccountId>;
	}
}

// The pallet's events
decl_event!(
	pub enum Event<T> where
		AccountId = <T as system::Config>::AccountId
	{
		SubscriptionPlanCreated(AccountId, SubscriptionPlanId),
		// todo: complete event list for this pallet once dispatches are implemented
	}
);

decl_error! {
	pub enum Error for Module<T: Config> {
		AlreadySubscribed,
		CannotScheduleReccurentPayment,
		NoPermissionToUpdateSubscriptionPlan,
		NotSubscriber,
		NothingToUpdate,
		PlanIsNotActive,
		PriceLowerExistencialDeposit,
		RecipientNotFound,
		RecurringPaymentMissing,
		SubscriptionIsNotActive,
		SubscriptionNotFound,
		SubscriptionPlanNotFound,
	}
}

decl_module! {
	/// The module declaration.
	pub struct Module<T: Config> for enum Call where origin: <T as system::Config>::Origin {

		const DailyPeriodInBlocks: T::BlockNumber = T::DailyPeriodInBlocks::get();
		const WeeklyPeriodInBlocks: T::BlockNumber = T::WeeklyPeriodInBlocks::get();
		const MonthlyPeriodInBlocks: T::BlockNumber = T::MonthlyPeriodInBlocks::get();
		const QuarterlyPeriodInBlocks: T::BlockNumber = T::QuarterlyPeriodInBlocks::get();
		const YearlyPeriodInBlocks: T::BlockNumber = T::YearlyPeriodInBlocks::get();

		// Initializing errors
		type Error = Error<T>;

		// Initializing events
		fn deposit_event() = default;

		/// Create a new subscription plan for a specified space.
		/// It's possible to specify a price and time period (in blocks) for the plan.
		/// Content could be an IPFS CID that points to an off-chain data such as
		/// plan's title, description and cover image.
		#[weight = T::DbWeight::get().reads_writes(3, 3) + 25_000]
		pub fn create_plan(
			origin,
			space_id: SpaceId,
			custom_wallet: Option<T::AccountId>,
			price: BalanceOf<T>,
			period: SubscriptionPeriod<T::BlockNumber>,
			content: Content
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			Utils::<T>::is_valid_content(content.clone())?;

			ensure!(
				price >= <T as pallet_utils::Config>::Currency::minimum_balance(),
				Error::<T>::PriceLowerExistencialDeposit
			);

			let space = Spaces::<T>::require_space(space_id)?;
			Self::ensure_subscriptions_manager(sender.clone(), &space)?;

			let plan_id = Self::next_plan_id();
			let subscription_plan = SubscriptionPlan::<T>::new(
				plan_id,
				sender,
				space_id,
				custom_wallet,
				price,
				period,
				content
			);

			PlanById::<T>::insert(plan_id, subscription_plan);
			PlanIdsBySpace::mutate(space_id, |ids| ids.push(plan_id));
			NextPlanId::mutate(|x| { *x += 1 });

			Ok(())
		}

		/// Update some details (a wallet) on a specific subscription plan.
		#[weight = T::DbWeight::get().reads_writes(2, 1) + 10_000]
		pub fn update_plan(origin, plan_id: SubscriptionPlanId, new_wallet: Option<T::AccountId>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let mut plan = Self::require_plan(plan_id)?;

			let space = Spaces::<T>::require_space(plan.space_id)?;
			Self::ensure_subscriptions_manager(sender.clone(), &space)?;

			ensure!(new_wallet != plan.wallet, Error::<T>::NothingToUpdate);
			plan.wallet = new_wallet;
			plan.updated = Some(WhoAndWhen::<T>::new(sender));
			PlanById::<T>::insert(plan_id, plan);

			Ok(())
		}

		/// Delete a subscription plan by its id.
		#[weight = 10_000]
		pub fn delete_plan(origin, plan_id: SubscriptionPlanId) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let mut plan = Self::require_plan(plan_id)?;
			ensure!(plan.is_active, Error::<T>::PlanIsNotActive);

			let space = Spaces::<T>::require_space(plan.space_id)?;
			Self::ensure_subscriptions_manager(sender, &space)?;

			let space_subscriptions = SubscriptionIdsBySpace::take(plan.space_id);
			let plan_subscriptions = space_subscriptions.iter()
				.filter(|id| Self::filter_subscriptions_by_plan(**id, plan_id));

			for id in plan_subscriptions {
				if let Ok(mut subscription) = Self::require_subscription(*id) {
					Self::cancel_recurring_subscription_payment(*id);
					subscription.is_active = false;
					SubscriptionById::<T>::insert(id, subscription);
				}
			}

			plan.is_active = false;
			PlanById::<T>::insert(plan_id, plan.clone());
			PlanIdsBySpace::mutate(plan.space_id, |ids| remove_from_vec(ids, plan_id));

			Ok(())
		}

		/// Specify a default wallet to which subscribers will pay in case a subscription plan
		/// does not specify its own wallet.
		#[weight = T::DbWeight::get().reads_writes(1, 1) + 10_000]
		pub fn set_recipient_wallet(origin, space_id: SpaceId, wallet: T::AccountId) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let space = Spaces::<T>::require_space(space_id)?;
			space.ensure_space_owner(sender)?;

			RecipientWallet::<T>::insert(space.id, wallet);
			Ok(())
		}

		#[weight = T::DbWeight::get().reads_writes(1, 1) + 10_000]
		pub fn remove_recipient_wallet(origin, space_id: SpaceId) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let space = Spaces::<T>::require_space(space_id)?;
			space.ensure_space_owner(sender)?;

			RecipientWallet::<T>::remove(space.id);
			Ok(())
		}

		/// Subscribe to a selected subscription plan and optionally specify a wallet
		/// that will be used for recurring payments fro this subscription.
		#[weight = T::DbWeight::get().reads_writes(5, 1) + 50_000]
		pub fn subscribe(
			origin,
			plan_id: SubscriptionPlanId,
			custom_wallet: Option<T::AccountId>
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let plan = Self::require_plan(plan_id)?;
			ensure!(plan.is_active, Error::<T>::PlanIsNotActive);

			let subscriptions = Self::subscription_ids_by_patron(&sender);
			let is_already_subscribed = subscriptions.iter().any(|subscription_id| {
				if let Ok(subscription) = Self::require_subscription(*subscription_id) {
					return subscription.plan_id == plan_id;
				}
				false
			});
			ensure!(is_already_subscribed, Error::<T>::AlreadySubscribed);

			let subscription_id = Self::next_subscription_id();
			let subscription = Subscription::<T>::new(
				subscription_id,
				sender.clone(),
				custom_wallet,
				plan_id
			);

			Self::schedule_recurring_subscription_payment(subscription_id, plan.period.clone())?;

			let recipient = plan.try_get_recipient();
			ensure!(recipient.is_some(), Error::<T>::RecipientNotFound);

			// todo: maybe implement function `transfer_or_reserve`?
			<T as pallet_utils::Config>::Currency::transfer(
				&sender,
				&recipient.unwrap(),
				plan.price,
				ExistenceRequirement::KeepAlive
			).map_err(|err| {
				Self::cancel_recurring_subscription_payment(subscription_id);
				err
			})?;

			SubscriptionById::<T>::insert(subscription_id, subscription);
			SubscriptionIdsByPatron::<T>::mutate(sender, |ids| ids.push(subscription_id));
			SubscriptionIdsBySpace::mutate(plan.space_id, |ids| ids.push(subscription_id));

			Ok(())
		}

		#[weight = T::DbWeight::get().reads_writes(1, 1) + 10_000]
		pub fn update_subscription(
			origin,
			subscription_id: SubscriptionId,
			new_wallet: Option<T::AccountId>
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let mut subscription = Self::require_subscription(subscription_id)?;
			subscription.ensure_subscriber(&sender)?;

			ensure!(new_wallet != subscription.wallet, Error::<T>::NothingToUpdate);

			subscription.wallet = new_wallet;
			subscription.updated = Some(WhoAndWhen::<T>::new(sender));
			SubscriptionById::<T>::insert(subscription_id, subscription);

			Ok(())
		}

		/// Unsubscribe from one of your current subscriptions by its id.
		#[weight = T::DbWeight::get().reads_writes(4, 3) + 25_000]
		pub fn unsubscribe(origin, subscription_id: SubscriptionId) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let mut subscription = Self::require_subscription(subscription_id)?;
			subscription.ensure_subscriber(&sender)?;

			ensure!(subscription.is_active, Error::<T>::SubscriptionIsNotActive);

			// todo: add scheduled task to make subscription inactive at the end
			Self::do_unsubscribe(sender, &mut subscription)?;

			Ok(())
		}

		/// Specify a default wallet that will be used to pay for subscriptions of this `origin`.
		#[weight = T::DbWeight::get().reads_writes(0, 1) + 10_000]
		pub fn set_subscriber_wallet(
			origin,
			wallet: T::AccountId
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			SubscriberWallet::<T>::insert(sender, wallet);

			Ok(())
		}

		/// Remove a default wallet that was used to pay for subscriptions of this `origin`.
		/// If an account has no default subscription wallet, then the payments will be made
		/// from its account id.
		#[weight = T::DbWeight::get().reads_writes(0, 1) + 10_000]
		pub fn remove_subscriber_wallet(origin) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			SubscriberWallet::<T>::remove(sender);

			Ok(())
		}

		#[weight = T::DbWeight::get().reads_writes(4, 1) + 25_000]
		pub fn process_subscription_payment(origin, subscription_id: SubscriptionId) -> DispatchResult {
			ensure_root(origin)?;

			// todo: remove recurring payment if something in this block goes wrong
			let mut subscription = Self::require_subscription(subscription_id)?;
			let plan = Self::require_plan(subscription.plan_id)?;
			let recipient = plan.try_get_recipient();
			ensure!(recipient.is_some(), Error::<T>::RecipientNotFound);

			subscription.is_active = <T as pallet_utils::Config>::Currency::transfer(
				&subscription.created.account,
				&recipient.unwrap(),
				plan.price,
				ExistenceRequirement::KeepAlive
			).is_err();

			SubscriptionById::<T>::insert(subscription_id, subscription);

			Ok(())
		}
	}
}
