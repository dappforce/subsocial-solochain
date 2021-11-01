use crate::*;

use sp_runtime::traits::Saturating;
use pallet_sudo::Module as Sudo;
use frame_support::{
    dispatch::DispatchError,
    traits::schedule::DispatchTime,
};

use pallet_permissions::SpacePermission;
use pallet_spaces::Space;

impl<T: Config> Module<T> {
    pub fn require_plan(plan_id: SubscriptionPlanId) -> Result<SubscriptionPlan<T>, DispatchError> {
        Ok(Self::plan_by_id(plan_id).ok_or(Error::<T>::SubscriptionPlanNotFound)?)
    }

    pub fn require_subscription(subscription_id: SubscriptionId) -> Result<Subscription<T>, DispatchError> {
        Ok(Self::subscription_by_id(subscription_id).ok_or(Error::<T>::SubscriptionNotFound)?)
    }

    pub fn ensure_subscriptions_manager(account: T::AccountId, space: &Space<T>) -> DispatchResult {
        Spaces::<T>::ensure_account_has_space_permission(
            account,
            space,
            SpacePermission::UpdateSpaceSettings,
            Error::<T>::NoPermissionToUpdateSubscriptionPlan.into()
        )
    }

    pub fn get_period_in_blocks(period: SubscriptionPeriod<T::BlockNumber>) -> T::BlockNumber {
        match period {
            SubscriptionPeriod::Daily => T::DailyPeriodInBlocks::get(),
            SubscriptionPeriod::Weekly => T::WeeklyPeriodInBlocks::get(),
            SubscriptionPeriod::Monthly => T::MonthlyPeriodInBlocks::get(),
            SubscriptionPeriod::Quarterly => T::QuarterlyPeriodInBlocks::get(),
            SubscriptionPeriod::Yearly => T::YearlyPeriodInBlocks::get(),
            SubscriptionPeriod::Custom(block_number) => block_number,
        }
    }

    pub(crate) fn schedule_recurring_subscription_payment(
        subscription_id: SubscriptionId,
        period: SubscriptionPeriod<T::BlockNumber>
    ) -> DispatchResult {
        let period_in_blocks = Self::get_period_in_blocks(period);
        let task_name = (SUBSCRIPTIONS_ID, subscription_id).encode();
        let when = <system::Module<T>>::block_number().saturating_add(period_in_blocks);

        T::Scheduler::schedule_named(
            task_name,
            DispatchTime::At(when),
            Some((period_in_blocks, 12)),
            1,
            frame_system::RawOrigin::Signed(Sudo::<T>::key()).into(),
            Call::process_subscription_payment(subscription_id).into()
        ).map_err(|_| Error::<T>::CannotScheduleReccurentPayment)?;
        Ok(())
    }

    pub(crate) fn cancel_recurring_subscription_payment(subscription_id: SubscriptionId) {
        let _ = T::Scheduler::cancel_named((SUBSCRIPTIONS_ID, subscription_id).encode())
            .map_err(|_| Error::<T>::RecurringPaymentMissing);
        // todo: emmit event with status
    }

    pub(crate) fn do_unsubscribe(who: T::AccountId, subscription: &mut Subscription<T>) -> DispatchResult {
        let space_id = Self::require_plan(subscription.plan_id)?.space_id;
        let subscription_id = subscription.id;

        Self::cancel_recurring_subscription_payment(subscription_id);
        subscription.is_active = false;

        SubscriptionById::<T>::insert(subscription_id, subscription);
        SubscriptionIdsByPatron::<T>::mutate(who, |ids| remove_from_vec(ids, subscription_id));
        SubscriptionIdsBySpace::mutate(space_id, |ids| remove_from_vec(ids, subscription_id));

        Ok(())
    }

    pub(crate) fn filter_subscriptions_by_plan(
        subscription_id: SubscriptionId,
        plan_id: SubscriptionPlanId
    ) -> bool {
        if let Ok(subscription) = Self::require_subscription(subscription_id) {
            return subscription.plan_id == plan_id;
        }
        false
    }
}

impl<T: Config> SubscriptionPlan<T> {
    pub fn new(
        id: SubscriptionPlanId,
        created_by: T::AccountId,
        space_id: SpaceId,
        wallet: Option<T::AccountId>,
        price: BalanceOf<T>,
        period: SubscriptionPeriod<T::BlockNumber>,
        content: Content
    ) -> Self {
        Self {
            id,
            created: WhoAndWhen::<T>::new(created_by),
            updated: None,
            is_active: true,
            content,
            space_id,
            wallet,
            price,
            period,
        }
    }

    pub fn try_get_recipient(&self) -> Option<T::AccountId> {
        self.wallet.clone()
            .or_else(|| Module::<T>::recipient_wallet(self.space_id))
            .or_else(|| {
                Spaces::<T>::require_space(self.space_id).map(|space| space.owner).ok()
            })
    }
}

impl<T: Config> Subscription<T> {
    pub fn new(
        id: SubscriptionId,
        created_by: T::AccountId,
        wallet: Option<T::AccountId>,
        plan_id: SubscriptionPlanId
    ) -> Self {
        Self {
            id,
            created: WhoAndWhen::<T>::new(created_by),
            updated: None,
            is_active: true,
            wallet,
            plan_id,
        }
    }

    pub fn ensure_subscriber(&self, who: &T::AccountId) -> DispatchResult {
        ensure!(&self.created.account == who, Error::<T>::NotSubscriber);
        Ok(())
    }
}