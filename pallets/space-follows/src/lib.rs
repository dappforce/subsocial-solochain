#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, ensure,
    dispatch::DispatchResult,
    traits::Get
};
use sp_std::prelude::*;
use frame_system::{self as system, ensure_signed};

use df_traits::{
    SpaceFollowsProvider,
    moderation::IsAccountBlocked,
};
use pallet_profiles::{Module as Profiles, SocialAccountById};
use pallet_spaces::{BeforeSpaceCreated, Module as Spaces, Space, SpaceById};
use pallet_utils::{Error as UtilsError, SpaceId, remove_from_vec};

pub mod rpc;

/// The pallet's configuration trait.
pub trait Config: system::Config
    + pallet_utils::Config
    + pallet_spaces::Config
    + pallet_profiles::Config
{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Config>::Event>;

    type BeforeSpaceFollowed: BeforeSpaceFollowed<Self>;

    type BeforeSpaceUnfollowed: BeforeSpaceUnfollowed<Self>;
}

decl_error! {
    pub enum Error for Module<T: Config> {
        /// Social account was not found by id.
        SocialAccountNotFound,
        /// Account is already a space follower.
        AlreadySpaceFollower,
        /// Account is not a space follower.
        NotSpaceFollower,
        /// Not allowed to follow a hidden space.
        CannotFollowHiddenSpace,
    }
}

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Config> as SpaceFollowsModule {
        pub SpaceFollowers get(fn space_followers):
            map hasher(twox_64_concat) SpaceId => Vec<T::AccountId>;

        pub SpaceFollowedByAccount get(fn space_followed_by_account):
            map hasher(blake2_128_concat) (T::AccountId, SpaceId) => bool;

        pub SpacesFollowedByAccount get(fn spaces_followed_by_account):
            map hasher(blake2_128_concat) T::AccountId => Vec<SpaceId>;
    }
}

decl_event!(
    pub enum Event<T> where
        <T as system::Config>::AccountId,
    {
        SpaceFollowed(/* follower */ AccountId, /* following */ SpaceId),
        SpaceUnfollowed(/* follower */ AccountId, /* unfollowing */ SpaceId),
    }
);

// The pallet's dispatchable functions.
decl_module! {
  pub struct Module<T: Config> for enum Call where origin: T::Origin {
    // Initializing errors
    type Error = Error<T>;

    // Initializing events
    fn deposit_event() = default;

    #[weight = 10_000 + T::DbWeight::get().reads_writes(5, 5)]
    pub fn follow_space(origin, space_id: SpaceId) -> DispatchResult {
      let follower = ensure_signed(origin)?;

      ensure!(!Self::space_followed_by_account((follower.clone(), space_id)), Error::<T>::AlreadySpaceFollower);

      let space = &mut Spaces::require_space(space_id)?;
      ensure!(!space.hidden, Error::<T>::CannotFollowHiddenSpace);

      ensure!(T::IsAccountBlocked::is_allowed_account(follower.clone(), space.id), UtilsError::<T>::AccountIsBlocked);

      Self::add_space_follower(follower, space)?;
      <SpaceById<T>>::insert(space_id, space);

      Ok(())
    }

    #[weight = 10_000 + T::DbWeight::get().reads_writes(5, 5)]
    pub fn unfollow_space(origin, space_id: SpaceId) -> DispatchResult {
      let follower = ensure_signed(origin)?;

      ensure!(Self::space_followed_by_account((follower.clone(), space_id)), Error::<T>::NotSpaceFollower);

      Self::unfollow_space_by_account(follower, space_id)
    }
  }
}

impl<T: Config> Module<T> {
    fn add_space_follower(follower: T::AccountId, space: &mut Space<T>) -> DispatchResult {
        space.inc_followers();

        let mut social_account = Profiles::get_or_new_social_account(follower.clone());
        social_account.inc_following_spaces();

        T::BeforeSpaceFollowed::before_space_followed(
            follower.clone(), social_account.reputation, space)?;

        let space_id = space.id;
        <SpaceFollowers<T>>::mutate(space_id, |followers| followers.push(follower.clone()));
        <SpaceFollowedByAccount<T>>::insert((follower.clone(), space_id), true);
        <SpacesFollowedByAccount<T>>::mutate(follower.clone(), |space_ids| space_ids.push(space_id));
        <SocialAccountById<T>>::insert(follower.clone(), social_account);

        Self::deposit_event(RawEvent::SpaceFollowed(follower, space_id));

        Ok(())
    }

    pub fn unfollow_space_by_account(follower: T::AccountId, space_id: SpaceId) -> DispatchResult {
        let space = &mut Spaces::require_space(space_id)?;
        space.dec_followers();

        let mut social_account = Profiles::social_account_by_id(follower.clone()).ok_or(Error::<T>::SocialAccountNotFound)?;
        social_account.dec_following_spaces();

        T::BeforeSpaceUnfollowed::before_space_unfollowed(follower.clone(), space)?;

        <SpacesFollowedByAccount<T>>::mutate(follower.clone(), |space_ids| remove_from_vec(space_ids, space_id));
        <SpaceFollowers<T>>::mutate(space_id, |account_ids| remove_from_vec(account_ids, follower.clone()));
        <SpaceFollowedByAccount<T>>::remove((follower.clone(), space_id));
        <SocialAccountById<T>>::insert(follower.clone(), social_account);
        <SpaceById<T>>::insert(space_id, space);

        Self::deposit_event(RawEvent::SpaceUnfollowed(follower, space_id));
        Ok(())
    }
}

impl<T: Config> SpaceFollowsProvider for Module<T> {
    type AccountId = T::AccountId;

    fn is_space_follower(account: Self::AccountId, space_id: SpaceId) -> bool {
        Module::<T>::space_followed_by_account((account, space_id))
    }
}

impl<T: Config> BeforeSpaceCreated<T> for Module<T> {
    fn before_space_created(creator: T::AccountId, space: &mut Space<T>) -> DispatchResult {
        // Make a space creator the first follower of this space:
        Module::<T>::add_space_follower(creator, space)
    }
}

/// Handler that will be called right before the space is followed.
pub trait BeforeSpaceFollowed<T: Config> {
    fn before_space_followed(follower: T::AccountId, follower_reputation: u32, space: &mut Space<T>) -> DispatchResult;
}

impl<T: Config> BeforeSpaceFollowed<T> for () {
    fn before_space_followed(_follower: T::AccountId, _follower_reputation: u32, _space: &mut Space<T>) -> DispatchResult {
        Ok(())
    }
}

/// Handler that will be called right before the space is unfollowed.
pub trait BeforeSpaceUnfollowed<T: Config> {
    fn before_space_unfollowed(follower: T::AccountId, space: &mut Space<T>) -> DispatchResult;
}

impl<T: Config> BeforeSpaceUnfollowed<T> for () {
    fn before_space_unfollowed(_follower: T::AccountId, _space: &mut Space<T>) -> DispatchResult {
        Ok(())
    }
}
