#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use scale_info::TypeInfo;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, ensure,
    dispatch::DispatchResult,
    traits::Get
};
use frame_system::{self as system, ensure_signed};

#[cfg(feature = "std")]
use serde::Deserialize;
use sp_runtime::{RuntimeDebug, DispatchError};
use sp_std::prelude::*;

use df_traits::moderation::IsAccountBlocked;
use pallet_permissions::SpacePermission;
use pallet_posts::{Module as Posts, PostById};
use pallet_spaces::Module as Spaces;
use pallet_utils::{Error as UtilsError, remove_from_vec, WhoAndWhen, PostId};

pub mod rpc;

pub type ReactionId = u64;

#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Deserialize))]
#[cfg_attr(feature = "std", serde(untagged))]
pub enum ReactionKind {
    Upvote,
    Downvote,
}

impl Default for ReactionKind {
    fn default() -> Self {
        ReactionKind::Upvote
    }
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct Reaction<T: Config> {

    /// Unique sequential identifier of a reaction. Examples of reaction ids: `1`, `2`, `3`,
    /// and so on.
    pub id: ReactionId,

    pub created: WhoAndWhen<T>,
    pub updated: Option<WhoAndWhen<T>>,
    pub kind: ReactionKind,
}

/// The pallet's configuration trait.
pub trait Config: system::Config
    + pallet_utils::Config
    + pallet_posts::Config
    + pallet_spaces::Config
{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Config>::Event>;
}

pub const FIRST_REACTION_ID: u64 = 1;

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Config> as ReactionsModule {

        /// The next reaction id.
        pub NextReactionId get(fn next_reaction_id): ReactionId = FIRST_REACTION_ID;

        pub ReactionById get(fn reaction_by_id):
            map hasher(twox_64_concat) ReactionId => Option<Reaction<T>>;

        pub ReactionIdsByPostId get(fn reaction_ids_by_post_id):
            map hasher(twox_64_concat) PostId => Vec<ReactionId>;

        pub PostReactionIdByAccount get(fn post_reaction_id_by_account):
            map hasher(twox_64_concat) (T::AccountId, PostId) => ReactionId;
    }
}

decl_event!(
    pub enum Event<T> where
        <T as system::Config>::AccountId,
    {
        PostReactionCreated(AccountId, PostId, ReactionId, ReactionKind),
        PostReactionUpdated(AccountId, PostId, ReactionId, ReactionKind),
        PostReactionDeleted(AccountId, PostId, ReactionId, ReactionKind),
    }
);

decl_error! {
    pub enum Error for Module<T: Config> {
        /// Reaction was not found by id.
        ReactionNotFound,
        /// Account has already reacted to this post/comment.
        AccountAlreadyReacted,
        /// There is no reaction by account on this post/comment.
        ReactionByAccountNotFound,
        /// Only reaction owner can update their reaction.
        NotReactionOwner,
        /// New reaction kind is the same as old one on this post/comment.
        SameReaction,

        /// Not allowed to react on a post/comment in a hidden space.
        CannotReactWhenSpaceHidden,
        /// Not allowed to react on a post/comment if a root post is hidden.
        CannotReactWhenPostHidden,

        /// User has no permission to upvote posts/comments in this space.
        NoPermissionToUpvote,
        /// User has no permission to downvote posts/comments in this space.
        NoPermissionToDownvote,
    }
}

decl_module! {
  pub struct Module<T: Config> for enum Call where origin: T::Origin {

    // Initializing errors
    type Error = Error<T>;

    // Initializing events
    fn deposit_event() = default;

    #[weight = 10_000 + T::DbWeight::get().reads_writes(6, 5)]
    pub fn create_post_reaction(origin, post_id: PostId, kind: ReactionKind) -> DispatchResult {
      let owner = ensure_signed(origin)?;

      let post = &mut Posts::require_post(post_id)?;
      ensure!(
        !<PostReactionIdByAccount<T>>::contains_key((owner.clone(), post_id)),
        Error::<T>::AccountAlreadyReacted
      );

      let space = post.get_space()?;
      ensure!(!space.hidden, Error::<T>::CannotReactWhenSpaceHidden);
      ensure!(Posts::<T>::is_root_post_visible(post_id)?, Error::<T>::CannotReactWhenPostHidden);

      ensure!(T::IsAccountBlocked::is_allowed_account(owner.clone(), space.id), UtilsError::<T>::AccountIsBlocked);

      match kind {
        ReactionKind::Upvote => {
          Spaces::ensure_account_has_space_permission(
            owner.clone(),
            &post.get_space()?,
            SpacePermission::Upvote,
            Error::<T>::NoPermissionToUpvote.into()
          )?;
          post.inc_upvotes();
        },
        ReactionKind::Downvote => {
          Spaces::ensure_account_has_space_permission(
            owner.clone(),
            &post.get_space()?,
            SpacePermission::Downvote,
            Error::<T>::NoPermissionToDownvote.into()
          )?;
          post.inc_downvotes();
        }
      }

      <PostById<T>>::insert(post_id, post.clone());
      let reaction_id = Self::insert_new_reaction(owner.clone(), kind);
      ReactionIdsByPostId::mutate(post.id, |ids| ids.push(reaction_id));
      <PostReactionIdByAccount<T>>::insert((owner.clone(), post_id), reaction_id);

      Self::deposit_event(RawEvent::PostReactionCreated(owner, post_id, reaction_id, kind));
      Ok(())
    }

    #[weight = 10_000 + T::DbWeight::get().reads_writes(3, 2)]
    pub fn update_post_reaction(origin, post_id: PostId, reaction_id: ReactionId, new_kind: ReactionKind) -> DispatchResult {
      let owner = ensure_signed(origin)?;

      ensure!(
        <PostReactionIdByAccount<T>>::contains_key((owner.clone(), post_id)),
        Error::<T>::ReactionByAccountNotFound
      );

      let mut reaction = Self::require_reaction(reaction_id)?;
      let post = &mut Posts::require_post(post_id)?;

      ensure!(owner == reaction.created.account, Error::<T>::NotReactionOwner);
      ensure!(reaction.kind != new_kind, Error::<T>::SameReaction);

      if let Some(space_id) = post.try_get_space_id() {
        ensure!(T::IsAccountBlocked::is_allowed_account(owner.clone(), space_id), UtilsError::<T>::AccountIsBlocked);
      }

      reaction.kind = new_kind;
      reaction.updated = Some(WhoAndWhen::<T>::new(owner.clone()));

      match new_kind {
        ReactionKind::Upvote => {
          post.inc_upvotes();
          post.dec_downvotes();
        },
        ReactionKind::Downvote => {
          post.inc_downvotes();
          post.dec_upvotes();
        },
      }

      <ReactionById<T>>::insert(reaction_id, reaction);
      <PostById<T>>::insert(post_id, post);

      Self::deposit_event(RawEvent::PostReactionUpdated(owner, post_id, reaction_id, new_kind));
      Ok(())
    }

    #[weight = 10_000 + T::DbWeight::get().reads_writes(4, 4)]
    pub fn delete_post_reaction(origin, post_id: PostId, reaction_id: ReactionId) -> DispatchResult {
      let owner = ensure_signed(origin)?;

      ensure!(
        <PostReactionIdByAccount<T>>::contains_key((owner.clone(), post_id)),
        Error::<T>::ReactionByAccountNotFound
      );

      // TODO extract Self::require_reaction(reaction_id)?;
      let reaction = Self::require_reaction(reaction_id)?;
      let post = &mut Posts::require_post(post_id)?;

      ensure!(owner == reaction.created.account, Error::<T>::NotReactionOwner);
      if let Some(space_id) = post.try_get_space_id() {
        ensure!(T::IsAccountBlocked::is_allowed_account(owner.clone(), space_id), UtilsError::<T>::AccountIsBlocked);
      }

      match reaction.kind {
        ReactionKind::Upvote => post.dec_upvotes(),
        ReactionKind::Downvote => post.dec_downvotes(),
      }

      <PostById<T>>::insert(post_id, post.clone());
      <ReactionById<T>>::remove(reaction_id);
      ReactionIdsByPostId::mutate(post.id, |ids| remove_from_vec(ids, reaction_id));
      <PostReactionIdByAccount<T>>::remove((owner.clone(), post_id));

      Self::deposit_event(RawEvent::PostReactionDeleted(owner, post_id, reaction_id, reaction.kind));
      Ok(())
    }
  }
}

impl<T: Config> Module<T> {

    pub fn insert_new_reaction(account: T::AccountId, kind: ReactionKind) -> ReactionId {
        let id = Self::next_reaction_id();
        let reaction: Reaction<T> = Reaction {
            id,
            created: WhoAndWhen::<T>::new(account),
            updated: None,
            kind,
        };

        <ReactionById<T>>::insert(id, reaction);
        NextReactionId::mutate(|n| { *n += 1; });

        id
    }

    /// Get `Reaction` by id from the storage or return `ReactionNotFound` error.
    pub fn require_reaction(reaction_id: ReactionId) -> Result<Reaction<T>, DispatchError> {
        Ok(Self::reaction_by_id(reaction_id).ok_or(Error::<T>::ReactionNotFound)?)
    }
}
