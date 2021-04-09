#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, ensure,
    dispatch::DispatchResult,
    traits::Get
};
use sp_std::prelude::*;
use frame_system::{self as system, ensure_signed};

use pallet_profiles::{Module as Profiles, SocialAccountById};
use pallet_utils::vec_remove_on;

pub mod rpc;

/// The pallet's configuration trait.
pub trait Config: system::Config
    + pallet_utils::Config
    + pallet_profiles::Config
{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Config>::Event>;

    type BeforeAccountFollowed: BeforeAccountFollowed<Self>;

    type BeforeAccountUnfollowed: BeforeAccountUnfollowed<Self>;
}

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Config> as ProfileFollowsModule {
        pub AccountFollowers get(fn account_followers):
            map hasher(blake2_128_concat) T::AccountId => Vec<T::AccountId>;

        pub AccountFollowedByAccount get(fn account_followed_by_account):
            map hasher(blake2_128_concat) (T::AccountId, T::AccountId) => bool;

        pub AccountsFollowedByAccount get(fn accounts_followed_by_account):
            map hasher(blake2_128_concat) T::AccountId => Vec<T::AccountId>;
    }
}

decl_event!(
    pub enum Event<T> where
        <T as system::Config>::AccountId,
    {
        AccountFollowed(/* follower */ AccountId, /* following */ AccountId),
        AccountUnfollowed(/* follower */ AccountId, /* unfollowing */ AccountId),
    }
);

decl_error! {
    pub enum Error for Module<T: Config> {
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
    }
}

decl_module! {
  pub struct Module<T: Config> for enum Call where origin: T::Origin {

    // Initializing errors
    type Error = Error<T>;

    // Initializing events
    fn deposit_event() = default;

    #[weight = 10_000 + T::DbWeight::get().reads_writes(4, 4)]
    pub fn follow_account(origin, account: T::AccountId) -> DispatchResult {
      let follower = ensure_signed(origin)?;

      ensure!(follower != account, Error::<T>::AccountCannotFollowItself);
      ensure!(!<AccountFollowedByAccount<T>>::contains_key((follower.clone(), account.clone())),
        Error::<T>::AlreadyAccountFollower);

      let mut follower_account = Profiles::get_or_new_social_account(follower.clone());
      let mut followed_account = Profiles::get_or_new_social_account(account.clone());

      follower_account.inc_following_accounts();
      followed_account.inc_followers();

      T::BeforeAccountFollowed::before_account_followed(
        follower.clone(), follower_account.reputation, account.clone())?;

      <SocialAccountById<T>>::insert(follower.clone(), follower_account);
      <SocialAccountById<T>>::insert(account.clone(), followed_account);
      <AccountsFollowedByAccount<T>>::mutate(follower.clone(), |ids| ids.push(account.clone()));
      <AccountFollowers<T>>::mutate(account.clone(), |ids| ids.push(follower.clone()));
      <AccountFollowedByAccount<T>>::insert((follower.clone(), account.clone()), true);

      Self::deposit_event(RawEvent::AccountFollowed(follower, account));
      Ok(())
    }

    #[weight = 10_000 + T::DbWeight::get().reads_writes(4, 4)]
    pub fn unfollow_account(origin, account: T::AccountId) -> DispatchResult {
      let follower = ensure_signed(origin)?;

      ensure!(follower != account, Error::<T>::AccountCannotUnfollowItself);
      ensure!(<AccountFollowedByAccount<T>>::contains_key((follower.clone(), account.clone())), Error::<T>::NotAccountFollower);

      let mut follower_account = Profiles::social_account_by_id(follower.clone()).ok_or(Error::<T>::FollowerAccountNotFound)?;
      let mut followed_account = Profiles::social_account_by_id(account.clone()).ok_or(Error::<T>::FollowedAccountNotFound)?;

      follower_account.dec_following_accounts();
      followed_account.dec_followers();

      T::BeforeAccountUnfollowed::before_account_unfollowed(follower.clone(), account.clone())?;

      <SocialAccountById<T>>::insert(follower.clone(), follower_account);
      <SocialAccountById<T>>::insert(account.clone(), followed_account);
      <AccountsFollowedByAccount<T>>::mutate(follower.clone(), |account_ids| vec_remove_on(account_ids, account.clone()));
      <AccountFollowers<T>>::mutate(account.clone(), |account_ids| vec_remove_on(account_ids, follower.clone()));
      <AccountFollowedByAccount<T>>::remove((follower.clone(), account.clone()));

      Self::deposit_event(RawEvent::AccountUnfollowed(follower, account));
      Ok(())
    }
  }
}

/// Handler that will be called right before the account is followed.
pub trait BeforeAccountFollowed<T: Config> {
    fn before_account_followed(follower: T::AccountId, follower_reputation: u32, following: T::AccountId) -> DispatchResult;
}

impl<T: Config> BeforeAccountFollowed<T> for () {
    fn before_account_followed(_follower: T::AccountId, _follower_reputation: u32, _following: T::AccountId) -> DispatchResult {
        Ok(())
    }
}

/// Handler that will be called right before the account is unfollowed.
pub trait BeforeAccountUnfollowed<T: Config> {
    fn before_account_unfollowed(follower: T::AccountId, following: T::AccountId) -> DispatchResult;
}

impl<T: Config> BeforeAccountUnfollowed<T> for () {
    fn before_account_unfollowed(_follower: T::AccountId, _following: T::AccountId) -> DispatchResult {
        Ok(())
    }
}
