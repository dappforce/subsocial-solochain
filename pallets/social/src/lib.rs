#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::string_lit_as_bytes)]

pub mod functions;
pub mod roles;

#[cfg(test)]
mod tests;

use sp_std::prelude::*;
use codec::{Encode, Decode};
use frame_support::{
  decl_module, decl_storage, decl_event, decl_error, ensure,
  traits::Get,
  dispatch::DispatchError
};
use sp_runtime::RuntimeDebug;
use system::ensure_signed;

use pallet_utils::{Module as Utils, WhoAndWhen, SpaceId};
use pallet_permissions::{SpacePermission, SpacePermissions};
use df_traits::PermissionChecker;

pub type PostId = u64;
pub type ReactionId = u64;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Space<T: Trait> {
  pub id: SpaceId,
  pub created: WhoAndWhen<T>,
  pub updated: Option<WhoAndWhen<T>>,
  pub hidden: bool,

  // Can be updated by the owner:
  pub owner: T::AccountId,
  pub handle: Option<Vec<u8>>,
  pub ipfs_hash: Vec<u8>,

  pub posts_count: u16,
  pub followers_count: u32,

  pub edit_history: Vec<SpaceHistoryRecord<T>>,

  pub score: i32,

  /// Allows to override the default permissions for this space.
  pub permissions: Option<SpacePermissions>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
#[allow(clippy::option_option)]
pub struct SpaceUpdate {
  pub handle: Option<Option<Vec<u8>>>,
  pub ipfs_hash: Option<Vec<u8>>,
  pub hidden: Option<bool>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct SpaceHistoryRecord<T: Trait> {
  pub edited: WhoAndWhen<T>,
  pub old_data: SpaceUpdate,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Post<T: Trait> {
  pub id: PostId,
  pub created: WhoAndWhen<T>,
  pub updated: Option<WhoAndWhen<T>>,
  pub hidden: bool,

  pub space_id: Option<SpaceId>,
  pub extension: PostExtension,

  pub ipfs_hash: Vec<u8>,
  pub edit_history: Vec<PostHistoryRecord<T>>,

  pub direct_replies_count: u16,
  pub total_replies_count: u32,

  pub shares_count: u16,
  pub upvotes_count: u16,
  pub downvotes_count: u16,

  pub score: i32
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct PostUpdate {
  pub space_id: Option<SpaceId>,
  pub ipfs_hash: Option<Vec<u8>>,
  pub hidden: Option<bool>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct PostHistoryRecord<T: Trait> {
  pub edited: WhoAndWhen<T>,
  pub old_data: PostUpdate,
}

#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, RuntimeDebug)]
pub enum PostExtension {
  RegularPost,
  Comment(CommentExt),
  SharedPost(PostId),
}

#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, RuntimeDebug)]
pub struct CommentExt {
  parent_id: Option<PostId>,
  root_post_id: PostId,
}

impl Default for PostExtension {
  fn default() -> Self {
    PostExtension::RegularPost
  }
}

#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, RuntimeDebug)]
pub enum ReactionKind {
  Upvote,
  Downvote,
}

impl Default for ReactionKind {
  fn default() -> Self {
    ReactionKind::Upvote
  }
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Reaction<T: Trait> {
  pub id: ReactionId,
  pub created: WhoAndWhen<T>,
  pub updated: Option<WhoAndWhen<T>>,
  pub kind: ReactionKind,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct SocialAccount<T: Trait> {
  pub followers_count: u32,
  pub following_accounts_count: u16,
  pub following_spaces_count: u16,
  pub reputation: u32,
  pub profile: Option<Profile<T>>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Profile<T: Trait> {
  pub created: WhoAndWhen<T>,
  pub updated: Option<WhoAndWhen<T>>,

  pub username: Vec<u8>,
  pub ipfs_hash: Vec<u8>,

  pub edit_history: Vec<ProfileHistoryRecord<T>>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct ProfileUpdate {
  pub username: Option<Vec<u8>>,
  pub ipfs_hash: Option<Vec<u8>>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct ProfileHistoryRecord<T: Trait> {
  pub edited: WhoAndWhen<T>,
  pub old_data: ProfileUpdate,
}

#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, RuntimeDebug)]
pub enum ScoringAction {
  UpvotePost,
  DownvotePost,
  SharePost,
  CreateComment,
  UpvoteComment,
  DownvoteComment,
  ShareComment,
  FollowSpace,
  FollowAccount,
}

impl Default for ScoringAction {
  fn default() -> Self {
    ScoringAction::FollowAccount
  }
}

/// The pallet's configuration trait.
pub trait Trait: system::Trait + pallet_utils::Trait {
  /// The overarching event type.
  type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

  /// Minimal length of blog handle
  type MinHandleLen: Get<u32>;

  /// Maximal length of space handle
  type MaxHandleLen: Get<u32>;

  /// Minimal length of profile username
  type MinUsernameLen: Get<u32>;

  /// Maximal length of profile username
  type MaxUsernameLen: Get<u32>;

  /// Weights of the related social account actions
  type FollowSpaceActionWeight: Get<i16>;
  type FollowAccountActionWeight: Get<i16>;
  type UpvotePostActionWeight: Get<i16>;
  type DownvotePostActionWeight: Get<i16>;
  type SharePostActionWeight: Get<i16>;
  type CreateCommentActionWeight: Get<i16>;
  type UpvoteCommentActionWeight: Get<i16>;
  type DownvoteCommentActionWeight: Get<i16>;
  type ShareCommentActionWeight: Get<i16>;

  /// Max comments depth
  type MaxCommentDepth: Get<u32>;

  type Roles: PermissionChecker<AccountId = Self::AccountId>;
}

decl_error! {
  pub enum Error for Module<T: Trait> {
    /// Space was not found by id
    SpaceNotFound,
    /// Space handle is too short
    HandleIsTooShort,
    /// Space handle is too long
    HandleIsTooLong,
    /// Space handle is not unique
    HandleIsNotUnique,
    /// Space handle contains invalid characters
    HandleContainsInvalidChars,
    /// Nothing to update in space
    NoUpdatesInSpace,
    /// Only space owner can manage their space
    NotASpaceOwner,
    /// The current space owner cannot transfer ownership to himself
    CannotTranferToCurrentOwner,
    /// There is no transfer ownership by space that is provided
    NoPendingTransferOnSpace,
    /// The account is not allowed to apply transfer ownership
    NotAllowedToAcceptOwnershipTransfer,
    /// The account is not allowed to reject transfer ownership
    NotAllowedToRejectOwnershipTransfer,

    /// Post was not found by id
    PostNotFound,
    /// Nothing to update in post
    NoUpdatesInPost,
    /// Overflow caused adding post on space
    OverflowAddingPostOnSpace,
    /// Cannot create post not defining space_id
    SpaceIdIsUndefined,
    /// Not allowed to create post/comment when entity is hidden
    BannedToCreateWhenHidden,
    /// Not allowed to follow space when it's hidden
    BannedToFollowWhenHidden,
    /// Not allowed to create/update reaction to post/comment when entity is hidden
    BannedToChangeReactionWhenHidden,

    /// Unknown parent comment id
    UnknownParentComment,
    /// Post by parent_id is not of Comment extension
    NotACommentByParentId,
    /// New comment IPFS-hash is the same as old one
    CommentIPFSHashNotDiffer,
    /// Overflow adding comment on post
    OverflowAddingCommentOnPost,
    /// Cannot update space id on Comment
    CannotUpdateSpaceIdOnComment,
    /// Max comment depth reached
    MaxCommentDepthReached,
    /// Only comment author can update his comment
    NotACommentAuthor,

    /// Reaction was not found by id
    ReactionNotFound,
    /// Account has already reacted to this post/comment
    AccountAlreadyReacted,
    /// Account has not reacted to this post/comment yet
    AccountNotYetReacted,
    /// There is no post/comment reaction by account that could be deleted
    ReactionByAccountNotFound,
    /// Overflow caused upvoting post/comment
    OverflowUpvoting,
    /// Overflow caused downvoting post/comment
    OverflowDownvoting,
    /// Only reaction owner can update their reaction
    NotAReactionOwner,
    /// New reaction kind is the same as old one
    NewReactionKindNotDiffer,

    /// Account is already following this space
    AccountIsFollowingSpace,
    /// Account is not following this space
    AccountIsNotFollowingSpace,
    /// Account can not follow itself
    AccountCannotFollowItself,
    /// Account can not unfollow itself
    AccountCannotUnfollowItself,
    /// Account is already followed
    AccountIsAlreadyFollowed,
    /// Account is not followed by current follower
    AccountIsNotFollowed,
    /// Underflow unfollowing space
    UnderflowUnfollowingSpace,
    /// Overflow caused following space
    OverflowFollowingSpace,
    /// Overflow caused following account
    OverflowFollowingAccount,
    /// Underflow caused unfollowing account
    UnderflowUnfollowingAccount,

    /// Social account was not found by id
    SocialAccountNotFound,
    /// Follower social account was not found by id
    FollowerAccountNotFound,
    /// Social account that is being followed was not found by id
    FollowedAccountNotFound,

    /// Out of bounds updating space score
    OutOfBoundsUpdatingSpaceScore,
    /// Out of bounds reverting space score
    OutOfBoundsRevertingSpaceScore,
    /// Out of bounds updating post score
    OutOfBoundsUpdatingPostScore,
    /// Out of bounds reverting post score
    OutOfBoundsRevertingPostScore,
    /// Post extension is a comment
    PostIsAComment,
    /// Out of bounds updating comment score
    OutOfBoundsUpdatingCommentScore,
    /// Out of bounds reverting comment score
    OutOfBoundsRevertingCommentScore,
    /// Post extension is not a comment
    PostIsNotAComment,
    /// Out of bounds updating social account reputation
    OutOfBoundsUpdatingAccountReputation,
    /// Scored account reputation difference by account and action not found
    ReputationDiffNotFound,

    /// Original post not found when sharing
    OriginalPostNotFound,
    /// Overflow caused on total shares counter when sharing post/comment
    OverflowTotalShares,
    /// Overflow caused on shares by account counter when sharing post/comment
    OverflowPostShares,
    /// Cannot share post that is not a RegularPost
    CannotShareSharedPost,

    /// Profile for this account already exists
    ProfileAlreadyExists,
    /// Nothing to update in a profile
    NoUpdatesInProfile,
    /// Account has no profile yet
    ProfileDoesNotExist,
    /// Profile username is busy
    UsernameIsBusy,
    /// Username is too short
    UsernameIsTooShort,
    /// Username is too long
    UsernameIsTooLong,
    /// Username is not alphanumeric
    UsernameIsNotAlphanumeric,

    /// User has no permission to update this space
    NoPermissionToUpdateSpace,
    /// User has no permission to create posts in this space
    NoPermissionToCreatePosts,
    /// User has no permission to create comments in this space
    NoPermissionToCreateComments,
    /// User has no permission to share posts/comments from this space
    NoPermissionToShare,
    /// User is an author, but has no permission to update own posts in this space
    NoPermissionToUpdateOwnPosts,
    /// User is not an author and has no permission to update posts in this space
    NoPermissionToUpdateAnyPosts,
    /// User has no permission to update own comments in this space posts
    NoPermissionToUpdateOwnComments,
    /// User has no permission to upvote posts/comments in this space
    NoPermissionToUpvote,
    /// User has no permission to downvote posts/comments in this space
    NoPermissionToDownvote
  }
}

// This pallet's storage items.
decl_storage! {
  trait Store for Module<T: Trait> as TemplateModule {
    pub SpaceById get(fn space_by_id): map SpaceId => Option<Space<T>>;
    pub PostById get(fn post_by_id): map PostId => Option<Post<T>>;
    pub ReactionById get(fn reaction_by_id): map ReactionId => Option<Reaction<T>>;
    pub SocialAccountById get(fn social_account_by_id): map T::AccountId => Option<SocialAccount<T>>;

    pub SpaceIdsByOwner get(fn space_ids_by_owner): map T::AccountId => Vec<SpaceId>;
    pub PostIdsBySpaceId get(fn post_ids_by_space_id): map SpaceId => Vec<PostId>;

    pub ReplyIdsByPostId get(fn reply_ids_by_post_id): map PostId => Vec<PostId>;

    pub ReactionIdsByPostId get(fn reaction_ids_by_post_id): map PostId => Vec<ReactionId>;
    pub PostReactionIdByAccount get(fn post_reaction_id_by_account): map (T::AccountId, PostId) => ReactionId;

    pub SpaceIdByHandle get(fn space_id_by_handle): map Vec<u8> => Option<SpaceId>;

    pub SpacesFollowedByAccount get(fn spaces_followed_by_account): map T::AccountId => Vec<SpaceId>;
    pub SpaceFollowers get(fn space_followers): map SpaceId => Vec<T::AccountId>;
    pub SpaceFollowedByAccount get(fn space_followed_by_account): map (T::AccountId, SpaceId) => bool;

    pub AccountFollowedByAccount get(fn account_followed_by_account): map (T::AccountId, T::AccountId) => bool;
    pub AccountsFollowedByAccount get(fn accounts_followed_by_account): map T::AccountId => Vec<T::AccountId>;
    pub AccountFollowers get(fn account_followers): map T::AccountId => Vec<T::AccountId>;

    pub NextSpaceId get(fn next_space_id): SpaceId = 1;
    pub NextPostId get(fn next_post_id): PostId = 1;
    pub NextReactionId get(fn next_reaction_id): ReactionId = 1;

    pub AccountReputationDiffByAccount get(fn account_reputation_diff_by_account): map (T::AccountId, T::AccountId, ScoringAction) => Option<i16>; // TODO shorten name (?refactor)
    pub PostScoreByAccount get(fn post_score_by_account): map (T::AccountId, PostId, ScoringAction) => Option<i16>;

    pub PostSharesByAccount get(fn post_shares_by_account): map (T::AccountId, PostId) => u16;
    pub SharedPostIdsByOriginalPostId get(fn shared_post_ids_by_original_post_id): map PostId => Vec<PostId>;

    pub AccountByProfileUsername get(fn account_by_profile_username): map Vec<u8> => Option<T::AccountId>;

    pub PendingSpaceOwner get(fn pending_space_owner): map SpaceId => Option<T::AccountId>;
  }
}

// The pallet's dispatchable functions.
decl_module! {
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {
    /// Minimal length of space handle
    const MinHandleLen: u32 = T::MinHandleLen::get();

    /// Maximal length of space handle
    const MaxHandleLen: u32 = T::MaxHandleLen::get();

    /// Minimal length of profile username
    const MinUsernameLen: u32 = T::MinUsernameLen::get();

    /// Maximal length of profile username
    const MaxUsernameLen: u32 = T::MaxUsernameLen::get();

    /// Weights of the related social account actions
    const FollowSpaceActionWeight: i16 = T::FollowSpaceActionWeight::get();
    const FollowAccountActionWeight: i16 = T::FollowAccountActionWeight::get();
    const UpvotePostActionWeight: i16 = T::UpvotePostActionWeight::get();
    const DownvotePostActionWeight: i16 = T::DownvotePostActionWeight::get();
    const SharePostActionWeight: i16 = T::SharePostActionWeight::get();
    const CreateCommentActionWeight: i16 = T::CreateCommentActionWeight::get();
    const UpvoteCommentActionWeight: i16 = T::UpvoteCommentActionWeight::get();
    const DownvoteCommentActionWeight: i16 = T::DownvoteCommentActionWeight::get();
    const ShareCommentActionWeight: i16 = T::ShareCommentActionWeight::get();

    /// Max comments depth
    const MaxCommentDepth: u32 = T::MaxCommentDepth::get();

    // Initializing events
    // this is needed only if you are using events in your pallet
    fn deposit_event() = default;

    pub fn create_space(origin, handle_opt: Option<Vec<u8>>, ipfs_hash: Vec<u8>) {
      let owner = ensure_signed(origin)?;

      Utils::<T>::is_ipfs_hash_valid(ipfs_hash.clone())?;

      let mut handle: Vec<u8> = Vec::new();
      if let Some(original_handle) = handle_opt.clone() {
        handle = Self::lowercase_and_validate_a_handle(original_handle)?;
      }

      let space_id = Self::next_space_id();
      let new_space = &mut Space::create(space_id, owner.clone(), ipfs_hash, handle_opt);

      // Space creator automatically follows their space:
      Self::add_space_follower(owner.clone(), new_space)?;

      if !handle.is_empty() {
        SpaceIdByHandle::insert(handle, space_id);
      }

      <SpaceById<T>>::insert(space_id, new_space);
      <SpaceIdsByOwner<T>>::mutate(owner.clone(), |ids| ids.push(space_id));
      NextSpaceId::mutate(|n| { *n += 1; });
      Self::deposit_event(RawEvent::SpaceCreated(owner, space_id));
    }

    pub fn update_space(origin, space_id: SpaceId, update: SpaceUpdate) {
      let owner = ensure_signed(origin)?;

      let has_updates =
        update.handle.is_some() ||
        update.ipfs_hash.is_some() ||
        update.hidden.is_some();

      ensure!(has_updates, Error::<T>::NoUpdatesInSpace);

      let mut space = Self::space_by_id(space_id).ok_or(Error::<T>::SpaceNotFound)?;

      Self::ensure_account_has_space_permission(
        owner.clone(),
        &space,
        SpacePermission::UpdateSpace,
        Error::<T>::NoPermissionToUpdateSpace.into()
      )?;

      let mut fields_updated = 0;
      let mut new_history_record = SpaceHistoryRecord {
        edited: WhoAndWhen::<T>::new(owner.clone()),
        old_data: SpaceUpdate {handle: None, ipfs_hash: None, hidden: None}
      };

      if let Some(ipfs_hash) = update.ipfs_hash {
        if ipfs_hash != space.ipfs_hash {
          Utils::<T>::is_ipfs_hash_valid(ipfs_hash.clone())?;
          new_history_record.old_data.ipfs_hash = Some(space.ipfs_hash);
          space.ipfs_hash = ipfs_hash;
          fields_updated += 1;
        }
      }

      if let Some(hidden) = update.hidden {
        if hidden != space.hidden {
          new_history_record.old_data.hidden = Some(space.hidden);
          space.hidden = hidden;
          fields_updated += 1;
        }
      }

      if let Some(handle_opt) = update.handle {
        if handle_opt != space.handle {
          if let Some(mut handle) = handle_opt.clone() {
            handle = Self::lowercase_and_validate_a_handle(handle)?;
            SpaceIdByHandle::insert(handle, space_id);
          }
          if let Some(space_handle) = space.handle.clone() {
            SpaceIdByHandle::remove(space_handle);
          }
          new_history_record.old_data.handle = Some(space.handle);
          space.handle = handle_opt;
          fields_updated += 1;
        }
      }

      // Update this space only if at least one field should be updated:
      if fields_updated > 0 {
        space.updated = Some(WhoAndWhen::<T>::new(owner.clone()));
        space.edit_history.push(new_history_record);
        <SpaceById<T>>::insert(space_id, space);
        Self::deposit_event(RawEvent::SpaceUpdated(owner, space_id));
      }
    }

    pub fn transfer_space_ownership(origin, space_id: SpaceId, transfer_to: T::AccountId) {
      let who = ensure_signed(origin)?;

      let space = Self::space_by_id(space_id).ok_or(Error::<T>::SpaceNotFound)?;
      space.ensure_space_owner(who.clone())?;

      ensure!(who != transfer_to, Error::<T>::CannotTranferToCurrentOwner);
      Space::<T>::ensure_space_stored(space_id)?;

      <PendingSpaceOwner<T>>::insert(space_id, transfer_to.clone());
      Self::deposit_event(RawEvent::SpaceOwnershipTransferCreated(who, space_id, transfer_to));
    }

    pub fn accept_pending_ownership(origin, space_id: SpaceId) {
      let who = ensure_signed(origin)?;

      let mut space = Self::space_by_id(space_id).ok_or(Error::<T>::SpaceNotFound)?;
      let transfer_to = Self::pending_space_owner(space_id).ok_or(Error::<T>::NoPendingTransferOnSpace)?;
      ensure!(who == transfer_to, Error::<T>::NotAllowedToAcceptOwnershipTransfer);

      // Here we know that the origin is eligible to become a new owner of this space.
      <PendingSpaceOwner<T>>::remove(space_id);

      space.owner = who.clone();
      <SpaceById<T>>::insert(space_id, space);
      Self::deposit_event(RawEvent::SpaceOwnershipTransfered(who, space_id));
    }

    pub fn reject_pending_ownership(origin, space_id: SpaceId) {
      let who = ensure_signed(origin)?;

      let space = Self::space_by_id(space_id).ok_or(Error::<T>::SpaceNotFound)?;
      let transfer_to = Self::pending_space_owner(space_id).ok_or(Error::<T>::NoPendingTransferOnSpace)?;
      ensure!(who == transfer_to || who == space.owner, Error::<T>::NotAllowedToRejectOwnershipTransfer);

      <PendingSpaceOwner<T>>::remove(space_id);
      Self::deposit_event(RawEvent::SpaceOwnershipTransferRejected(who, space_id));
    }

    pub fn follow_space(origin, space_id: SpaceId) {
      let follower = ensure_signed(origin)?;

      let space = &mut (Self::space_by_id(space_id).ok_or(Error::<T>::SpaceNotFound)?);
      ensure!(!Self::space_followed_by_account((follower.clone(), space_id)), Error::<T>::AccountIsFollowingSpace);
      ensure!(!space.hidden, Error::<T>::BannedToFollowWhenHidden);

      Self::add_space_follower(follower, space)?;
      <SpaceById<T>>::insert(space_id, space);
    }

    pub fn unfollow_space(origin, space_id: SpaceId) {
      let follower = ensure_signed(origin)?;

      let space = &mut (Self::space_by_id(space_id).ok_or(Error::<T>::SpaceNotFound)?);
      ensure!(Self::space_followed_by_account((follower.clone(), space_id)), Error::<T>::AccountIsNotFollowingSpace);

      let mut social_account = Self::social_account_by_id(follower.clone()).ok_or(Error::<T>::SocialAccountNotFound)?;
      social_account.following_spaces_count = social_account.following_spaces_count
        .checked_sub(1)
        .ok_or(Error::<T>::UnderflowUnfollowingSpace)?;
      space.followers_count = space.followers_count.checked_sub(1).ok_or(Error::<T>::UnderflowUnfollowingSpace)?;

      if space.created.account != follower {
        let author = space.created.account.clone();
        if let Some(score_diff) = Self::account_reputation_diff_by_account((follower.clone(), author.clone(), ScoringAction::FollowSpace)) {
          space.score = space.score.checked_sub(score_diff as i32).ok_or(Error::<T>::OutOfBoundsUpdatingSpaceScore)?;
          Self::change_social_account_reputation(author, follower.clone(), -score_diff, ScoringAction::FollowSpace)?;
        }
      }

      <SpacesFollowedByAccount<T>>::mutate(follower.clone(), |space_ids| Self::vec_remove_on(space_ids, space_id));
      <SpaceFollowers<T>>::mutate(space_id, |account_ids| Self::vec_remove_on(account_ids, follower.clone()));
      <SpaceFollowedByAccount<T>>::remove((follower.clone(), space_id));
      <SocialAccountById<T>>::insert(follower.clone(), social_account);
      <SpaceById<T>>::insert(space_id, space);

      Self::deposit_event(RawEvent::SpaceUnfollowed(follower, space_id));
    }

    pub fn follow_account(origin, account: T::AccountId) {
      let follower = ensure_signed(origin)?;

      ensure!(follower != account, Error::<T>::AccountCannotFollowItself);
      ensure!(!<AccountFollowedByAccount<T>>::exists((follower.clone(), account.clone())), Error::<T>::AccountIsAlreadyFollowed);

      let mut follower_account = Self::get_or_new_social_account(follower.clone());
      let mut followed_account = Self::get_or_new_social_account(account.clone());

      follower_account.following_accounts_count = follower_account.following_accounts_count
        .checked_add(1).ok_or(Error::<T>::OverflowFollowingAccount)?;
      followed_account.followers_count = followed_account.followers_count
        .checked_add(1).ok_or(Error::<T>::OverflowFollowingAccount)?;

      Self::change_social_account_reputation(account.clone(), follower.clone(),
        Self::get_score_diff(follower_account.reputation, ScoringAction::FollowAccount),
        ScoringAction::FollowAccount
      )?;

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

      let mut follower_account = Self::social_account_by_id(follower.clone()).ok_or(Error::<T>::FollowerAccountNotFound)?;
      let mut followed_account = Self::social_account_by_id(account.clone()).ok_or(Error::<T>::FollowedAccountNotFound)?;

      ensure!(<AccountFollowedByAccount<T>>::exists((follower.clone(), account.clone())), Error::<T>::AccountIsNotFollowed);

      follower_account.following_accounts_count = follower_account.following_accounts_count
        .checked_sub(1).ok_or(Error::<T>::UnderflowUnfollowingAccount)?;
      followed_account.followers_count = followed_account.followers_count
        .checked_sub(1).ok_or(Error::<T>::UnderflowUnfollowingAccount)?;

      let reputation_diff = Self::account_reputation_diff_by_account(
        (follower.clone(), account.clone(), ScoringAction::FollowAccount)
      ).ok_or(Error::<T>::ReputationDiffNotFound)?;
      Self::change_social_account_reputation(account.clone(), follower.clone(),
        reputation_diff,
        ScoringAction::FollowAccount
      )?;

      <SocialAccountById<T>>::insert(follower.clone(), follower_account);
      <SocialAccountById<T>>::insert(account.clone(), followed_account);
      <AccountsFollowedByAccount<T>>::mutate(follower.clone(), |account_ids| Self::vec_remove_on(account_ids, account.clone()));
      <AccountFollowers<T>>::mutate(account.clone(), |account_ids| Self::vec_remove_on(account_ids, follower.clone()));
      <AccountFollowedByAccount<T>>::remove((follower.clone(), account.clone()));

      Self::deposit_event(RawEvent::AccountUnfollowed(follower, account));
    }

    pub fn create_profile(origin, username: Vec<u8>, ipfs_hash: Vec<u8>) {
      let owner = ensure_signed(origin)?;

      let mut social_account = Self::get_or_new_social_account(owner.clone());
      ensure!(social_account.profile.is_none(), Error::<T>::ProfileAlreadyExists);
      Self::is_username_valid(username.clone())?;
      Utils::<T>::is_ipfs_hash_valid(ipfs_hash.clone())?;

      social_account.profile = Some(
        Profile {
          created: WhoAndWhen::<T>::new(owner.clone()),
          updated: None,
          username: username.clone(),
          ipfs_hash,
          edit_history: Vec::new()
        }
      );
      <AccountByProfileUsername<T>>::insert(username, owner.clone());
      <SocialAccountById<T>>::insert(owner.clone(), social_account);

      Self::deposit_event(RawEvent::ProfileCreated(owner));
    }

    pub fn update_profile(origin, update: ProfileUpdate) {
      let owner = ensure_signed(origin)?;

      let has_updates =
        update.username.is_some() ||
        update.ipfs_hash.is_some();

      ensure!(has_updates, Error::<T>::NoUpdatesInProfile);

      let mut social_account = Self::social_account_by_id(owner.clone()).ok_or(Error::<T>::SocialAccountNotFound)?;
      let mut profile = social_account.profile.ok_or(Error::<T>::ProfileDoesNotExist)?;
      let mut is_update_applied = false;
      let mut new_history_record = ProfileHistoryRecord {
        edited: WhoAndWhen::<T>::new(owner.clone()),
        old_data: ProfileUpdate {username: None, ipfs_hash: None}
      };

      if let Some(ipfs_hash) = update.ipfs_hash {
        if ipfs_hash != profile.ipfs_hash {
          Utils::<T>::is_ipfs_hash_valid(ipfs_hash.clone())?;
          new_history_record.old_data.ipfs_hash = Some(profile.ipfs_hash);
          profile.ipfs_hash = ipfs_hash;
          is_update_applied = true;
        }
      }

      if let Some(username) = update.username {
        if username != profile.username {
          Self::is_username_valid(username.clone())?;
          <AccountByProfileUsername<T>>::remove(profile.username.clone());
          <AccountByProfileUsername<T>>::insert(username.clone(), owner.clone());
          new_history_record.old_data.username = Some(profile.username);
          profile.username = username;
          is_update_applied = true;
        }
      }

      if is_update_applied {
        profile.updated = Some(WhoAndWhen::<T>::new(owner.clone()));
        profile.edit_history.push(new_history_record);
        social_account.profile = Some(profile);
        <SocialAccountById<T>>::insert(owner.clone(), social_account);

        Self::deposit_event(RawEvent::ProfileUpdated(owner));
      }
    }

    pub fn create_post(origin, space_id_opt: Option<SpaceId>, extension: PostExtension, ipfs_hash: Vec<u8>) {
      let owner = ensure_signed(origin)?;

      Utils::<T>::is_ipfs_hash_valid(ipfs_hash.clone())?;

      let new_post_id = Self::next_post_id();
      let new_post: Post<T> = Post::create(new_post_id, owner.clone(), space_id_opt, extension, ipfs_hash);

      // Get space from either from space_id_opt or extension if a Comment provided
      let mut space = new_post.get_space()?;
      ensure!(!space.hidden, Error::<T>::BannedToCreateWhenHidden);

      let root_post = &mut (new_post.get_root_post()?);
      ensure!(!root_post.hidden, Error::<T>::BannedToCreateWhenHidden);

      // Check permissions
      match extension {
        PostExtension::RegularPost | PostExtension::SharedPost(_) => {
          Self::ensure_account_has_space_permission(
            owner.clone(),
            &space,
            SpacePermission::CreatePosts,
            Error::<T>::NoPermissionToCreatePosts.into()
          )?;
        },
        PostExtension::Comment(_) => {
          Self::ensure_account_has_space_permission(
            owner.clone(),
            &space,
            SpacePermission::CreateComments,
            Error::<T>::NoPermissionToCreateComments.into()
          )?;
        }
      }

      match extension {
        PostExtension::RegularPost => {
          space.increment_posts_count()?;
        },
        PostExtension::SharedPost(post_id) => {
          let post = &mut (Self::post_by_id(post_id).ok_or(Error::<T>::OriginalPostNotFound)?);
          ensure!(!post.is_shared_post(), Error::<T>::CannotShareSharedPost);

          // TODO: maybe check multiple permissions per function call?
          Self::ensure_account_has_space_permission(
            owner.clone(),
            &post.get_space()?,
            SpacePermission::Share,
            Error::<T>::NoPermissionToShare.into()
          )?;

          space.posts_count = space.posts_count.checked_add(1).ok_or(Error::<T>::OverflowAddingPostOnSpace)?;
          Self::share_post(owner.clone(), post, new_post_id)?;
        },
        PostExtension::Comment(comment_ext) => {
          root_post.total_replies_count = root_post.total_replies_count.checked_add(1).ok_or(Error::<T>::OverflowAddingCommentOnPost)?;

          if let Some(parent_id) = comment_ext.parent_id {
            let mut parent_comment = Self::post_by_id(parent_id).ok_or(Error::<T>::UnknownParentComment)?;
            ensure!(parent_comment.is_comment(), Error::<T>::NotACommentByParentId);
            parent_comment.direct_replies_count = parent_comment.direct_replies_count.checked_add(1).ok_or(Error::<T>::OverflowAddingCommentOnPost)?;

            let mut ancestors = Self::get_ancestors(parent_id);
            ancestors[0] = parent_comment;
            ensure!(ancestors.len() < T::MaxCommentDepth::get() as usize, Error::<T>::MaxCommentDepthReached);
            for mut post in ancestors {
              post.total_replies_count = post.total_replies_count.checked_add(1).ok_or(Error::<T>::OverflowAddingCommentOnPost)?;
              <PostById<T>>::insert(post.id, post.clone());
            }

            ReplyIdsByPostId::mutate(parent_id, |ids| ids.push(new_post_id));
          } else {
            root_post.direct_replies_count = root_post.direct_replies_count.checked_add(1)
              .ok_or(Error::<T>::OverflowAddingCommentOnPost)?;

            ReplyIdsByPostId::mutate(comment_ext.root_post_id, |ids| ids.push(new_post_id));
          }

          Self::change_post_score_by_extension(owner.clone(), root_post, ScoringAction::CreateComment)?;

          <PostById<T>>::insert(comment_ext.root_post_id, root_post);
        }
      }

      if !new_post.is_comment() {
        <SpaceById<T>>::insert(space.id, space.clone());
        PostIdsBySpaceId::mutate(space.id, |ids| ids.push(new_post_id));
      }

      <PostById<T>>::insert(new_post_id, new_post);
      NextPostId::mutate(|n| { *n += 1; });

      Self::deposit_event(RawEvent::PostCreated(owner, new_post_id));
    }

    pub fn update_post(origin, post_id: PostId, update: PostUpdate) {
      let owner = ensure_signed(origin)?;

      let has_updates =
        update.space_id.is_some() ||
        update.ipfs_hash.is_some() ||
        update.hidden.is_some();

      ensure!(has_updates, Error::<T>::NoUpdatesInPost);

      let mut post = Self::post_by_id(post_id).ok_or(Error::<T>::PostNotFound)?;

      let is_owner = owner == post.created.account;
      let is_comment = post.is_comment();

      let permission_to_check: SpacePermission;
      let permission_error: DispatchError;

      if is_comment {
        if is_owner {
          permission_to_check = SpacePermission::UpdateOwnComments;
          permission_error = Error::<T>::NoPermissionToUpdateOwnComments.into();
        } else {
          return Err(Error::<T>::NotACommentAuthor.into());
        }
      } else {
        if is_owner {
          permission_to_check = SpacePermission::UpdateOwnPosts;
          permission_error = Error::<T>::NoPermissionToUpdateOwnPosts.into();
        } else {
          permission_to_check = SpacePermission::UpdateAnyPosts;
          permission_error = Error::<T>::NoPermissionToUpdateAnyPosts.into();
        }
      }

      Self::ensure_account_has_space_permission(
        owner.clone(),
        &post.get_space()?,
        permission_to_check,
        permission_error
      )?;

      let mut fields_updated = 0;
      let mut new_history_record = PostHistoryRecord {
        edited: WhoAndWhen::<T>::new(owner.clone()),
        old_data: PostUpdate {space_id: None, ipfs_hash: None, hidden: None}
      };

      if let Some(ipfs_hash) = update.ipfs_hash {
        if ipfs_hash != post.ipfs_hash {
          Utils::<T>::is_ipfs_hash_valid(ipfs_hash.clone())?;
          new_history_record.old_data.ipfs_hash = Some(post.ipfs_hash);
          post.ipfs_hash = ipfs_hash;
          fields_updated += 1;
        } else if post.is_comment() {
          return Err(Error::<T>::CommentIPFSHashNotDiffer.into());
        }
      }

      if let Some(hidden) = update.hidden {
        if hidden != post.hidden {
          new_history_record.old_data.hidden = Some(post.hidden);
          post.hidden = hidden;
          fields_updated += 1;
        }
      }

      // Move this post to another space:
      if let Some(space_id) = update.space_id {
        ensure!(!post.is_comment(), Error::<T>::CannotUpdateSpaceIdOnComment);

        if let Some(post_space_id) = post.space_id {
          if space_id != post_space_id {
            Space::<T>::ensure_space_stored(space_id)?;

            // Remove post_id from its old space:
            PostIdsBySpaceId::mutate(post_space_id, |post_ids| Self::vec_remove_on(post_ids, post_id));

            // Add post_id to its new space:
            PostIdsBySpaceId::mutate(space_id, |ids| ids.push(post_id));
            new_history_record.old_data.space_id = post.space_id;
            post.space_id = Some(space_id);
            fields_updated += 1;
          }
        }
      }

      // Update this post only if at least one field should be updated:
      if fields_updated > 0 {
        post.updated = Some(WhoAndWhen::<T>::new(owner.clone()));
        post.edit_history.push(new_history_record);
        <PostById<T>>::insert(post_id, post);

        Self::deposit_event(RawEvent::PostUpdated(owner, post_id));
      }
    }

    pub fn create_post_reaction(origin, post_id: PostId, kind: ReactionKind) {
      let owner = ensure_signed(origin)?;

      let post = &mut (Self::post_by_id(post_id).ok_or(Error::<T>::PostNotFound)?);
      ensure!(
        !<PostReactionIdByAccount<T>>::exists((owner.clone(), post_id)),
        Error::<T>::AccountAlreadyReacted
      );

      let space = post.get_space()?;
      ensure!(!space.hidden && !Self::is_root_post_hidden(post_id)?, Error::<T>::BannedToChangeReactionWhenHidden);

      let reaction_id = Self::new_reaction(owner.clone(), kind);

      match kind {
        ReactionKind::Upvote => {
          Self::ensure_account_has_space_permission(
            owner.clone(),
            &post.get_space()?,
            SpacePermission::Upvote,
            Error::<T>::NoPermissionToUpvote.into()
          )?;
          post.upvotes_count = post.upvotes_count.checked_add(1).ok_or(Error::<T>::OverflowUpvoting)?;
        },
        ReactionKind::Downvote => {
          Self::ensure_account_has_space_permission(
            owner.clone(),
            &post.get_space()?,
            SpacePermission::Downvote,
            Error::<T>::NoPermissionToDownvote.into()
          )?;
          post.downvotes_count = post.downvotes_count.checked_add(1).ok_or(Error::<T>::OverflowDownvoting)?;
        }
      }
      let action = Self::scoring_action_by_post_extension(post.extension, kind, false);

      if post.created.account != owner {
        Self::change_post_score_by_extension(owner.clone(), post, action)?;
      }
      else {
        <PostById<T>>::insert(post_id, post.clone());
      }

      ReactionIdsByPostId::mutate(post_id, |ids| ids.push(reaction_id));
      <PostReactionIdByAccount<T>>::insert((owner.clone(), post_id), reaction_id);
      Self::deposit_event(RawEvent::PostReactionCreated(owner, post_id, reaction_id));
    }

    pub fn update_post_reaction(origin, post_id: PostId, reaction_id: ReactionId, new_kind: ReactionKind) {
      let owner = ensure_signed(origin)?;

      ensure!(
        <PostReactionIdByAccount<T>>::exists((owner.clone(), post_id)),
        Error::<T>::AccountNotYetReacted
      );

      let mut reaction = Self::reaction_by_id(reaction_id).ok_or(Error::<T>::ReactionNotFound)?;
      let post = &mut (Self::post_by_id(post_id).ok_or(Error::<T>::PostNotFound)?);

      let space = post.get_space()?;
      ensure!(!space.hidden && !post.hidden, Error::<T>::BannedToChangeReactionWhenHidden);

      ensure!(owner == reaction.created.account, Error::<T>::NotAReactionOwner);
      ensure!(reaction.kind != new_kind, Error::<T>::NewReactionKindNotDiffer);

      reaction.kind = new_kind;
      reaction.updated = Some(WhoAndWhen::<T>::new(owner.clone()));

      match new_kind {
        ReactionKind::Upvote => {
          post.upvotes_count += 1;
          post.downvotes_count -= 1;
        },
        ReactionKind::Downvote => {
          post.downvotes_count += 1;
          post.upvotes_count -= 1;
        },
      }
      let action = Self::scoring_action_by_post_extension(post.extension, new_kind, false);
      let action_to_cancel = Self::scoring_action_by_post_extension(post.extension, new_kind, true);

      Self::change_post_score_by_extension(owner.clone(), post, action_to_cancel)?;
      Self::change_post_score_by_extension(owner.clone(), post, action)?;

      <ReactionById<T>>::insert(reaction_id, reaction);
      <PostById<T>>::insert(post_id, post);

      Self::deposit_event(RawEvent::PostReactionUpdated(owner, post_id, reaction_id));
    }

    pub fn delete_post_reaction(origin, post_id: PostId, reaction_id: ReactionId) {
      let owner = ensure_signed(origin)?;

      ensure!(
        <PostReactionIdByAccount<T>>::exists((owner.clone(), post_id)),
        Error::<T>::ReactionByAccountNotFound
      );

      let reaction = Self::reaction_by_id(reaction_id).ok_or(Error::<T>::ReactionNotFound)?;
      let post = &mut (Self::post_by_id(post_id).ok_or(Error::<T>::PostNotFound)?);

      ensure!(owner == reaction.created.account, Error::<T>::NotAReactionOwner);

      match reaction.kind {
        ReactionKind::Upvote => post.upvotes_count -= 1,
        ReactionKind::Downvote => post.downvotes_count -= 1,
      }
      let action_to_cancel = Self::scoring_action_by_post_extension(post.extension, reaction.kind, false);

      Self::change_post_score_by_extension(owner.clone(), post, action_to_cancel)?;

      <PostById<T>>::insert(post_id, post);
      <ReactionById<T>>::remove(reaction_id);
      ReactionIdsByPostId::mutate(post_id, |ids| Self::vec_remove_on(ids, reaction_id));
      <PostReactionIdByAccount<T>>::remove((owner.clone(), post_id));

      Self::deposit_event(RawEvent::PostReactionDeleted(owner, post_id, reaction_id));
    }
  }
}

decl_event!(
  pub enum Event<T> where
    <T as system::Trait>::AccountId,
   {
    SpaceCreated(AccountId, SpaceId),
    SpaceUpdated(AccountId, SpaceId),
    SpaceDeleted(AccountId, SpaceId),

    SpaceOwnershipTransferCreated(/* current owner */ AccountId, SpaceId, /* new owner */ AccountId),
    SpaceOwnershipTransfered(AccountId, SpaceId),
    SpaceOwnershipTransferRejected(AccountId, SpaceId),

    SpaceFollowed(AccountId, SpaceId),
    SpaceUnfollowed(AccountId, SpaceId),

    AccountReputationChanged(AccountId, ScoringAction, u32),

    AccountFollowed(AccountId, AccountId),
    AccountUnfollowed(AccountId, AccountId),

    PostCreated(AccountId, PostId),
    PostUpdated(AccountId, PostId),
    PostDeleted(AccountId, PostId),
    PostShared(AccountId, PostId),

    PostReactionCreated(AccountId, PostId, ReactionId),
    PostReactionUpdated(AccountId, PostId, ReactionId),
    PostReactionDeleted(AccountId, PostId, ReactionId),

    ProfileCreated(AccountId),
    ProfileUpdated(AccountId),
  }
);
