#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};
use sp_runtime::RuntimeDebug;
use frame_support::{
    decl_module, decl_storage, decl_error, decl_event,
    traits::Get,
    dispatch::{DispatchResult, DispatchError},
};
use frame_system::{self as system, ensure_signed};
use sp_std::prelude::*;

use pallet_permissions::SpacePermission;
use pallet_spaces::{Module as Spaces};
use pallet_utils::{Module as Utils, WhoAndWhen, SpaceId, Content};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

type BadgeId = u64;
type SpaceAwardId = u64;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Badge<T: Trait> {
    created: WhoAndWhen<T>,
    updated: Option<WhoAndWhen<T>>,
    content: Content,
    space_id: SpaceId,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct SpaceAward<T: Trait> {
    created: WhoAndWhen<T>,
    badge_id: BadgeId,
    recipient: SpaceId,
    expires_at: Option<T::BlockNumber>,
    accepted: bool
}

pub trait Trait: system::Trait
    + pallet_utils::Trait
    + pallet_spaces::Trait
{
    // todo: Add Event
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_event!(
	pub enum Event<T> where
		AccountId = <T as system::Trait>::AccountId
	{
		BadgeCreated(AccountId, BadgeId),
		BadgeUpdated(AccountId, BadgeId),
		BadgeDeleted(AccountId, BadgeId),
		BadgeAwarded(AccountId, SpaceAwardId, BadgeId, SpaceId),
		SpaceAwardAccepted(AccountId, SpaceAwardId, SpaceId),
		SpaceAwardDeleted(AccountId, SpaceAwardId),
	}
);

decl_storage! {
	trait Store for Module<T: Trait> as TemplateModule {
        NextBadgeId get(fn next_badge_id): BadgeId = 1;
        NextSpaceAwardId get(fn next_space_award_id): SpaceAwardId = 1;

        pub BadgeById get(fn badge_by_id):
            map hasher(twox_64_concat) BadgeId => Option<Badge<T>>;

        pub SpaceAwardById get(fn space_award_by_id):
            map hasher(twox_64_concat) SpaceAwardId => Option<SpaceAward<T>>;

        pub SpaceAwardsBySpaceId get(fn space_awards_by_space_id):
            map hasher(twox_64_concat) SpaceId => Vec<SpaceAwardId>;

        pub SpaceAwardIdByExpirationBlock get(fn space_award_id_by_expiration_block):
            map hasher(blake2_128_concat) T::BlockNumber => Vec<SpaceAwardId>;

        pub SpaceAwardsByBadgeId get(fn space_awards_by_badge_id):
         map hasher(blake2_128_concat) BadgeId => Vec<SpaceAwardId>;
	}
}

decl_error! {
	pub enum Error for Module<T: Trait> {
        BadgeNotFound,
        NoPermissionToManageAwards,
        NoPermissionToManageBadges,
        SpaceAwardNotFound
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

         #[weight = T::DbWeight::get().reads_writes(2, 2) + 10_000]
         pub fn create_badge(origin, space_id: SpaceId, content: Content) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Utils::<T>::is_valid_content(content.clone())?;

            Self::ensure_badge_manager(who.clone(), space_id)?;

            let badge_id = Self::next_badge_id();
            let new_badge = Badge {
                created: WhoAndWhen::<T>::new(who.clone()),
                content,
                space_id,
                updated: None
            };

            <BadgeById<T>>::insert(badge_id, new_badge);
            NextBadgeId::mutate(|x| *x += 1);

            Self::deposit_event(RawEvent::BadgeCreated(who, badge_id));
            Ok(())
         }

        #[weight = T::DbWeight::get().reads_writes(2, 1) + 10_000]
        pub fn update_badge(origin, badge_id: BadgeId, content: Content) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Utils::<T>::is_valid_content(content.clone())?;

            let mut updated_badge = Self::require_badge(badge_id)?;
            Self::ensure_badge_manager(who.clone(), updated_badge.space_id)?;

            updated_badge.content = content;
            updated_badge.updated = Some(WhoAndWhen::<T>::new(who.clone()));

            <BadgeById<T>>::insert(badge_id, updated_badge);

            Self::deposit_event(RawEvent::BadgeUpdated(who, badge_id));
            Ok(())
        }

        #[weight = T::DbWeight::get().reads_writes(2, 1) + 10_000]
        pub fn delete_badge(origin, badge_id: BadgeId) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let badge = Self::badge_by_id(badge_id).ok_or(Error::<T>::BadgeNotFound)?;
            Self::ensure_badge_manager(who.clone(), badge.space_id)?;

            let space_award_id = SpaceAwardsByBadgeId::take(badge_id);
             for id in space_award_id.iter() {
                <SpaceAwardById<T>>::remove(id);
            }

            <BadgeById<T>>::remove(badge_id);

            Self::deposit_event(RawEvent::BadgeDeleted(who, badge_id));
            Ok(())
        }

        #[weight = T::DbWeight::get().reads_writes(6, 4) + 10_000]
        pub fn award_badge(
            origin,
            badge_id: BadgeId,
            recipient: SpaceId,
            expire_after_opt: Option<T::BlockNumber>
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let badge = Self::require_badge(badge_id)?;
            Self::ensure_award_manager(who.clone(), badge.space_id)?;

            let space_award_id = Self::next_space_award_id();
            let expires_at_opt = expire_after_opt.map(|x| x + <system::Module<T>>::block_number());

            let new_space_award = SpaceAward {
                badge_id,
                created: WhoAndWhen::<T>::new(who.clone()),
                recipient,
                expires_at: expires_at_opt,
                accepted: false
            };

            if let Some(expires_at) = expires_at_opt {
                <SpaceAwardIdByExpirationBlock<T>>::mutate(expires_at, |ids| ids.push(space_award_id));
            }
            SpaceAwardsBySpaceId::mutate(recipient, |ids| ids.push(space_award_id));
            SpaceAwardsByBadgeId::mutate(badge_id, |ids| ids.push(space_award_id));

            <SpaceAwardById<T>>::insert(space_award_id, new_space_award);
            NextSpaceAwardId::mutate(|x| *x += 1);

            Self::deposit_event(RawEvent::BadgeAwarded(who, space_award_id, badge_id, recipient));
            Ok(())
        }

        #[weight = T::DbWeight::get().reads_writes(2, 1) + 10_000]
        pub fn accept_award(origin, award_id: SpaceAwardId) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let mut space_award = Self::space_award_by_id(award_id).ok_or(Error::<T>::SpaceAwardNotFound)?;
            Self::ensure_award_manager(who.clone(), space_award.recipient)?;

            space_award.accepted = true;
            <SpaceAwardById<T>>::insert(award_id, space_award.clone());

            Self::deposit_event(RawEvent::SpaceAwardAccepted(who, award_id, space_award.recipient));
            Ok(())
        }

        #[weight = T::DbWeight::get().reads_writes(1, 1) + 10_000]
        pub fn delete_badge_award(origin, space_award_id: SpaceAwardId) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let space_award = Self::space_award_by_id(space_award_id).ok_or(Error::<T>::SpaceAwardNotFound)?;
            let badge = Self::badge_by_id(space_award.badge_id).ok_or(Error::<T>::BadgeNotFound)?;
            Self::ensure_award_manager(who.clone(), badge.space_id)?;

            <SpaceAwardById<T>>::remove(space_award_id);

            Self::deposit_event(RawEvent::SpaceAwardDeleted(who, space_award_id));
            Ok(())
        }

        // todo(i): move to offchain worker
         fn on_finalize(n: T::BlockNumber) {
            let badge_id = <SpaceAwardIdByExpirationBlock<T>>::take(n);
            for id in badge_id.iter() {
                <SpaceAwardById<T>>::remove(id);
            }
        }
	}
}

impl<T: Trait> Module<T> {
    pub fn require_badge(badge_id: BadgeId) -> Result<Badge<T>, DispatchError> {
        Ok(Self::badge_by_id(badge_id).ok_or(Error::<T>::BadgeNotFound)?)
    }

    fn ensure_badge_manager(who: T::AccountId, space_id: SpaceId) -> DispatchResult {
        let space = Spaces::<T>::require_space(space_id)?;
        Spaces::<T>::ensure_account_has_space_permission(
            who,
            &space,
            SpacePermission::ManageBadges,
            Error::<T>::NoPermissionToManageBadges.into()
        )
    }

    fn ensure_award_manager(who: T::AccountId, space_id: SpaceId) -> DispatchResult {
        let space = Spaces::<T>::require_space(space_id)?;
        Spaces::<T>::ensure_account_has_space_permission(
            who,
            &space,
            SpacePermission::ManageAwards,
            Error::<T>::NoPermissionToManageAwards.into()
        )
    }
}
