#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Serialize, Deserialize};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, fail,
    dispatch::{DispatchError, DispatchResult}, ensure, traits::Get,
};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;
use frame_system::{self as system, ensure_signed};

use df_traits::moderation::{IsAccountBlocked, IsContentBlocked, IsPostBlocked};
use pallet_permissions::SpacePermission;
use pallet_spaces::{Module as Spaces, Space, SpaceById};
use pallet_utils::{
    Module as Utils, Error as UtilsError,
    SpaceId, WhoAndWhen, Content, PostId
};

pub mod functions;

pub mod rpc;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Post<T: Config> {
    pub id: PostId,
    pub created: WhoAndWhen<T>,
    pub updated: Option<WhoAndWhen<T>>,

    pub owner: T::AccountId,

    pub extension: PostExtension,

    pub space_id: Option<SpaceId>,
    pub content: Content,
    pub hidden: bool,

    pub replies_count: u16,
    pub hidden_replies_count: u16,

    pub shares_count: u16,
    pub upvotes_count: u16,
    pub downvotes_count: u16,

    pub score: i32,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct PostUpdate {
    pub space_id: Option<SpaceId>,
    pub content: Option<Content>,
    pub hidden: Option<bool>,
}

#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(tag = "kind", content = "contentId"))]
pub enum PostExtension {
    RegularPost,
    Comment(Comment),
    SharedPost(PostId),
}

#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Comment {
    pub parent_id: Option<PostId>,
    pub root_post_id: PostId,
}

impl Default for PostExtension {
    fn default() -> Self {
        PostExtension::RegularPost
    }
}

/// The pallet's configuration trait.
pub trait Trait: system::Trait
    + pallet_utils::Trait
    + pallet_space_follows::Trait
    + pallet_spaces::Trait
{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Config>::Event>;

    /// Max comments depth
    type MaxCommentDepth: Get<u32>;

    type PostScores: PostScores<Self>;

    type AfterPostUpdated: AfterPostUpdated<Self>;

    type IsPostBlocked: IsPostBlocked<PostId=PostId>;
}

pub trait PostScores<T: Config> {
    fn score_post_on_new_share(account: T::AccountId, original_post: &mut Post<T>) -> DispatchResult;
    fn score_root_post_on_new_comment(account: T::AccountId, root_post: &mut Post<T>) -> DispatchResult;
}

impl<T: Config> PostScores<T> for () {
    fn score_post_on_new_share(_account: T::AccountId, _original_post: &mut Post<T>) -> DispatchResult {
        Ok(())
    }
    fn score_root_post_on_new_comment(_account: T::AccountId, _root_post: &mut Post<T>) -> DispatchResult {
        Ok(())
    }
}

#[impl_trait_for_tuples::impl_for_tuples(10)]
pub trait AfterPostUpdated<T: Config> {
    fn after_post_updated(account: T::AccountId, post: &Post<T>, old_data: PostUpdate);
}

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Config> as PostsModule {
        pub NextPostId get(fn next_post_id): PostId = 1;

        pub PostById get(fn post_by_id): map hasher(twox_64_concat) PostId => Option<Post<T>>;

        pub ReplyIdsByPostId get(fn reply_ids_by_post_id):
            map hasher(twox_64_concat) PostId => Vec<PostId>;

        pub PostIdsBySpaceId get(fn post_ids_by_space_id):
            map hasher(twox_64_concat) SpaceId => Vec<PostId>;

        // TODO rename 'Shared...' to 'Sharing...'
        pub SharedPostIdsByOriginalPostId get(fn shared_post_ids_by_original_post_id):
            map hasher(twox_64_concat) PostId => Vec<PostId>;
    }
}

decl_event!(
    pub enum Event<T> where
        <T as system::Config>::AccountId,
    {
        PostCreated(AccountId, PostId),
        PostUpdated(AccountId, PostId),
        PostDeleted(AccountId, PostId),
        PostShared(AccountId, PostId),
        PostMoved(AccountId, PostId),
    }
);

decl_error! {
    pub enum Error for Module<T: Config> {

        // Post related errors:

        /// Post was not found by id.
        PostNotFound,
        /// Nothing to update in post.
        NoUpdatesForPost,
        /// Root post should have a space id.
        PostHasNoSpaceId,
        /// Not allowed to create a post/comment when a scope (space or root post) is hidden.
        CannotCreateInHiddenScope,
        /// Post has no any replies
        NoRepliesOnPost,
        /// Trying to move post to the same space.
        MoveToTheSameSpace,

        // Sharing related errors:

        /// Original post not found when sharing.
        OriginalPostNotFound,
        /// Cannot share a post that shares another post.
        CannotShareSharingPost,
        /// Post extension is not a sharing post.
        NotASharingPost,

        // Comment related errors:

        /// Unknown parent comment id.
        UnknownParentComment,
        /// Post by parent_id is not of Comment extension.
        NotACommentByParentId,
        /// Cannot update space id on comment.
        CannotUpdateSpaceIdOnComment,
        /// Max comment depth reached.
        MaxCommentDepthReached,
        /// Only comment author can update his comment.
        NotACommentAuthor,
        /// Post extension is not a comment.
        NotComment,

        // Permissions related errors:

        /// User has no permission to create root posts in this space.
        NoPermissionToCreatePosts,
        /// User has no permission to create comments (aka replies) in this space.
        NoPermissionToCreateComments,
        /// User has no permission to share posts/comments from this space to another space.
        NoPermissionToShare,
        /// User is not a post author and has no permission to update posts in this space.
        NoPermissionToUpdateAnyPost,
        /// A post owner is not allowed to update their own posts in this space.
        NoPermissionToUpdateOwnPosts,
        /// A comment owner is not allowed to update their own comments in this space.
        NoPermissionToUpdateOwnComments,
    }
}

decl_module! {
  pub struct Module<T: Config> for enum Call where origin: T::Origin {

    const MaxCommentDepth: u32 = T::MaxCommentDepth::get();

    // Initializing errors
    type Error = Error<T>;

    // Initializing events
    fn deposit_event() = default;

    #[weight = 100_000 + T::DbWeight::get().reads_writes(8, 8)]
    pub fn create_post(
      origin,
      space_id_opt: Option<SpaceId>,
      extension: PostExtension,
      content: Content
    ) -> DispatchResult {
      let creator = ensure_signed(origin)?;

      Utils::<T>::is_valid_content(content.clone())?;

      let new_post_id = Self::next_post_id();
      let new_post: Post<T> = Post::new(new_post_id, creator.clone(), space_id_opt, extension, content.clone());

      // Get space from either space_id_opt or Comment if a comment provided
      let space = &mut new_post.get_space()?;
      ensure!(!space.hidden, Error::<T>::CannotCreateInHiddenScope);

      ensure!(!T::IsAccountBlocked::is_account_blocked(creator.clone(), space.id), UtilsError::<T>::AccountIsBlocked);
      ensure!(!T::IsContentBlocked::is_content_blocked(content, space.id), UtilsError::<T>::ContentIsBlocked);

      let root_post = &mut new_post.get_root_post()?;
      ensure!(!root_post.hidden, Error::<T>::CannotCreateInHiddenScope);

      // Check whether account has permission to create Post (by extension)
      let mut permission_to_check = SpacePermission::CreatePosts;
      let mut error_on_permission_failed = Error::<T>::NoPermissionToCreatePosts;

      if let PostExtension::Comment(_) = extension {
        permission_to_check = SpacePermission::CreateComments;
        error_on_permission_failed = Error::<T>::NoPermissionToCreateComments;
      }

      Spaces::ensure_account_has_space_permission(
        creator.clone(),
        &space,
        permission_to_check,
        error_on_permission_failed.into()
      )?;

      match extension {
        PostExtension::RegularPost => space.inc_posts(),
        PostExtension::SharedPost(post_id) => Self::create_sharing_post(&creator, new_post_id, post_id, space)?,
        PostExtension::Comment(comment_ext) => Self::create_comment(&creator, new_post_id, comment_ext, root_post)?,
      }

      if new_post.is_root_post() {
        SpaceById::insert(space.id, space.clone());
        PostIdsBySpaceId::mutate(space.id, |ids| ids.push(new_post_id));
      }

      PostById::insert(new_post_id, new_post);
      NextPostId::mutate(|n| { *n += 1; });

      Self::deposit_event(RawEvent::PostCreated(creator, new_post_id));
      Ok(())
    }

    #[weight = 100_000 + T::DbWeight::get().reads_writes(5, 3)]
    pub fn update_post(origin, post_id: PostId, update: PostUpdate) -> DispatchResult {
      let editor = ensure_signed(origin)?;

      let has_updates =
        update.content.is_some() ||
        update.hidden.is_some();

      ensure!(has_updates, Error::<T>::NoUpdatesForPost);

      let mut post = Self::require_post(post_id)?;
      let mut space_opt = post.try_get_space();

      if let Some(space) = &space_opt {
        ensure!(!T::IsAccountBlocked::is_account_blocked(editor.clone(), space.id), UtilsError::<T>::AccountIsBlocked);

        let is_owner = post.is_owner(&editor);
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
        } else { // not a comment
          if is_owner {
            permission_to_check = SpacePermission::UpdateOwnPosts;
            permission_error = Error::<T>::NoPermissionToUpdateOwnPosts.into();
          } else {
            permission_to_check = SpacePermission::UpdateAnyPost;
            permission_error = Error::<T>::NoPermissionToUpdateAnyPost.into();
          }
        }

        Spaces::ensure_account_has_space_permission(
          editor.clone(),
          space,
          permission_to_check,
          permission_error
        )?;
      }

      let mut is_update_applied = false;
      let mut old_data = PostUpdate::default();

      if let Some(content) = update.content {
        if content != post.content {
          Utils::<T>::is_valid_content(content.clone())?;

          if let Some(space) = &space_opt {
              ensure!(!T::IsContentBlocked::is_content_blocked(content.clone(), space.id), UtilsError::<T>::ContentIsBlocked);
          }

          old_data.content = Some(post.content.clone());
          post.content = content;
          is_update_applied = true;
        }
      }

      if let Some(hidden) = update.hidden {
        if hidden != post.hidden {
          space_opt = space_opt.map(|mut space| {
            if hidden {
                space.inc_hidden_posts();
            } else {
                space.dec_hidden_posts();
            }

            space
          });

          if let PostExtension::Comment(comment_ext) = post.extension {
            Self::update_counters_on_comment_hidden_change(&comment_ext, hidden)?;
          }

          old_data.hidden = Some(post.hidden);
          post.hidden = hidden;
          is_update_applied = true;
        }
      }

      // Update this post only if at least one field should be updated:
      if is_update_applied {
        post.updated = Some(WhoAndWhen::<T>::new(editor.clone()));

        if let Some(space) = space_opt {
            <SpaceById<T>>::insert(space.id, space);
        }

        <PostById<T>>::insert(post.id, post.clone());
        T::AfterPostUpdated::after_post_updated(editor.clone(), &post, old_data);

        Self::deposit_event(RawEvent::PostUpdated(editor, post_id));
      }
      Ok(())
    }

    #[weight = T::DbWeight::get().reads(1) + 50_000]
    pub fn move_post(origin, post_id: PostId, new_space_id: Option<SpaceId>) -> DispatchResult {
      let who = ensure_signed(origin)?;

      let post = &mut Self::require_post(post_id)?;

      ensure!(new_space_id != post.space_id, Error::<T>::MoveToTheSameSpace);

      let old_space_id = post.space_id;

      if let Some(space_id) = new_space_id {
        Self::move_post_to_space(who.clone(), post, space_id)?;
      } else {
        Self::delete_post_from_space(post_id)?;
      }

      let historical_data = PostUpdate {
        space_id: old_space_id,
        content: None,
        hidden: None,
      };

      T::AfterPostUpdated::after_post_updated(who.clone(), &post, historical_data);

      Self::deposit_event(RawEvent::PostMoved(who, post_id));
      Ok(())
    }
  }
}
