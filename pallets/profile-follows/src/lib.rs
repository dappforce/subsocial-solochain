#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::string_lit_as_bytes)]

use frame_support::{decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure};
use sp_std::prelude::*;
use system::ensure_signed;

use pallet_profiles::{Module as Profiles, SocialAccountById};
use pallet_utils::vec_remove_on;

// mod tests;

/// The pallet's configuration trait.
pub trait Trait: system::Trait
    + pallet_utils::Trait
    + pallet_profiles::Trait
{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    type OnBeforeAccountFollowed: OnBeforeAccountFollowed<Self>;

    type OnBeforeAccountUnfollowed: OnBeforeAccountUnfollowed<Self>;
}

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as TemplateModule {
        pub AccountFollowers get(fn account_followers): map T::AccountId => Vec<T::AccountId>;
        pub AccountFollowedByAccount get(fn account_followed_by_account): map (T::AccountId, T::AccountId) => bool;
        pub AccountsFollowedByAccount get(fn accounts_followed_by_account): map T::AccountId => Vec<T::AccountId>;
    }
}

decl_event!(
    pub enum Event<T> where
        <T as system::Trait>::AccountId,
    {
        AccountFollowed(/* follower */ AccountId, /* following */ AccountId),
        AccountUnfollowed(/* follower */ AccountId, /* unfollowing */ AccountId),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Follower social account was not found by id.
        FollowerAccountNotFound,
        /// Social account that is being followed was not found by id.
        FollowedAccountNotFound,

        /// Account can not follow itself.
        AccountCannotFollowItself,
        /// Account can not unfollow itself.
        AccountCannotUnfollowItself,
        
        /// Account (Alice) is already a follower of another account (Bob).
        AlreadyAccountFollower,
        /// Account (Alice) is not a follower of another account (Bob).
        NotAccountFollower,
        
        /// Overflow caused following account.
        FollowAccountOverflow,
        /// Underflow caused unfollowing account.
        UnfollowAccountUnderflow,
    }
}

decl_module! {
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {

    // Initializing events
    fn deposit_event() = default;

    pub fn follow_account(origin, account: T::AccountId) {
      let follower = ensure_signed(origin)?;

      ensure!(follower != account, Error::<T>::AccountCannotFollowItself);
      ensure!(!<AccountFollowedByAccount<T>>::exists((follower.clone(), account.clone())),
        Error::<T>::AlreadyAccountFollower);

      let mut follower_account = Profiles::get_or_new_social_account(follower.clone());
      let mut followed_account = Profiles::get_or_new_social_account(account.clone());

      follower_account.following_accounts_count = follower_account.following_accounts_count
        .checked_add(1).ok_or(Error::<T>::FollowAccountOverflow)?;
      followed_account.followers_count = followed_account.followers_count
        .checked_add(1).ok_or(Error::<T>::FollowAccountOverflow)?;

      T::OnBeforeAccountFollowed::on_before_account_followed(
        follower.clone(), follower_account.reputation, account.clone())?;

      <SocialAccountById<T>>::insert(follower.clone(), follower_account);
      <SocialAccountById<T>>::insert(account.clone(), followed_account);
      <AccountsFollowedByAccount<T>>::mutate(follower.clone(), |ids| ids.push(account.clone()));
      <AccountFollowers<T>>::mutate(account.clone(), |ids| ids.push(follower.clone()));
      <AccountFollowedByAccount<T>>::insert((follower.clone(), account.clone()), true);

      Self::deposit_event(RawEvent::AccountFollowed(follower, account));
    }

    pub fn unfollow_account(origin, account: T::AccountId) {
      let follower = ensure_signed(origin)?;

      ensure!(follower != account, Error::<T>::AccountCannotUnfollowItself);

      let mut follower_account = Profiles::social_account_by_id(follower.clone()).ok_or(Error::<T>::FollowerAccountNotFound)?;
      let mut followed_account = Profiles::social_account_by_id(account.clone()).ok_or(Error::<T>::FollowedAccountNotFound)?;

      ensure!(<AccountFollowedByAccount<T>>::exists((follower.clone(), account.clone())), Error::<T>::NotAccountFollower);

      follower_account.following_accounts_count = follower_account.following_accounts_count
        .checked_sub(1).ok_or(Error::<T>::UnfollowAccountUnderflow)?;
      followed_account.followers_count = followed_account.followers_count
        .checked_sub(1).ok_or(Error::<T>::UnfollowAccountUnderflow)?;

      T::OnBeforeAccountUnfollowed::on_before_account_unfollowed(follower.clone(), account.clone())?;

      <SocialAccountById<T>>::insert(follower.clone(), follower_account);
      <SocialAccountById<T>>::insert(account.clone(), followed_account);
      <AccountsFollowedByAccount<T>>::mutate(follower.clone(), |account_ids| vec_remove_on(account_ids, account.clone()));
      <AccountFollowers<T>>::mutate(account.clone(), |account_ids| vec_remove_on(account_ids, follower.clone()));
      <AccountFollowedByAccount<T>>::remove((follower.clone(), account.clone()));

      Self::deposit_event(RawEvent::AccountUnfollowed(follower, account));
    }
  }
}

/// Handler that will be called right before the account is followed.
pub trait OnBeforeAccountFollowed<T: Trait> {
    fn on_before_account_followed(follower: T::AccountId, follower_reputation: u32, following: T::AccountId) -> DispatchResult;
}

impl<T: Trait> OnBeforeAccountFollowed<T> for () {
    fn on_before_account_followed(_follower: T::AccountId, _follower_reputation: u32, _following: T::AccountId) -> DispatchResult {
        Ok(())
    }
}

/// Handler that will be called right before the account is unfollowed.
pub trait OnBeforeAccountUnfollowed<T: Trait> {
    fn on_before_account_unfollowed(follower: T::AccountId, following: T::AccountId) -> DispatchResult;
}

impl<T: Trait> OnBeforeAccountUnfollowed<T> for () {
    fn on_before_account_unfollowed(_follower: T::AccountId, _following: T::AccountId) -> DispatchResult {
        Ok(())
    }
}
