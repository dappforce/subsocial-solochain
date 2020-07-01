#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult}, ensure, traits::Get,
};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;
use frame_system::{self as system, ensure_signed};

use pallet_permissions::SpacePermission;
use pallet_spaces::{Module as Spaces, SpaceById};
use pallet_utils::{Module as Utils, SpaceId, vec_remove_on, WhoAndWhen, Content};

pub mod functions;

pub type PostId = u64;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Post<T: Trait> {
    pub id: PostId,
    pub created: WhoAndWhen<T>,
    pub updated: Option<WhoAndWhen<T>>,
    pub hidden: bool,

    pub space_id: Option<SpaceId>,
    pub extension: PostExtension,

    pub content: Content,

    pub direct_replies_count: u16,
    pub total_replies_count: u32,

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
pub enum PostExtension {
    RegularPost,
    Comment(CommentExt),
    SharedPost(PostId),
}

#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, RuntimeDebug)]
// TODO rename: CommentExt -> Comment
pub struct CommentExt {
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
    + pallet_spaces::Trait
{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    /// Max comments depth
    type MaxCommentDepth: Get<u32>;

    type PostScores: PostScores<Self>;

    type AfterPostUpdated: AfterPostUpdated<Self>;
}

pub trait PostScores<T: Trait> {
    fn score_post_on_new_share(account: T::AccountId, original_post: &mut Post<T>) -> DispatchResult;
    fn score_root_post_on_new_comment(account: T::AccountId, root_post: &mut Post<T>) -> DispatchResult;
}

impl<T: Trait> PostScores<T> for () {
    fn score_post_on_new_share(_account: T::AccountId, _original_post: &mut Post<T>) -> DispatchResult {
        Ok(())
    }
    fn score_root_post_on_new_comment(_account: T::AccountId, _root_post: &mut Post<T>) -> DispatchResult {
        Ok(())
    }
}

#[impl_trait_for_tuples::impl_for_tuples(10)]
pub trait AfterPostUpdated<T: Trait> {
    fn after_post_updated(account: T::AccountId, space: &Post<T>, old_data: PostUpdate);
}

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as PostsModule {
        pub NextPostId get(fn next_post_id): PostId = 1;
        pub PostById get(fn post_by_id): map hasher(twox_64_concat) PostId => Option<Post<T>>;
        pub ReplyIdsByPostId get(fn reply_ids_by_post_id): map hasher(twox_64_concat) PostId => Vec<PostId>;
        pub PostIdsBySpaceId get(fn post_ids_by_space_id): map hasher(twox_64_concat) SpaceId => Vec<PostId>;

        // TODO rename 'Shared...' to 'Sharing...'
        pub SharedPostIdsByOriginalPostId get(fn shared_post_ids_by_original_post_id): map hasher(twox_64_concat) PostId => Vec<PostId>;
    }
}

decl_event!(
    pub enum Event<T> where
        <T as system::Trait>::AccountId,
    {
        PostCreated(AccountId, PostId),
        PostUpdated(AccountId, PostId),
        PostDeleted(AccountId, PostId),
        PostShared(AccountId, PostId),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {

        // Post related errors:

        /// Post was not found by id.
        PostNotFound,
        /// Nothing to update in post.
        NoUpdatesForPost,
        /// Root post should have a space id.
        PostHasNoSpaceId,
        /// Not allowed to create a post/comment when a scope (space or root post) is hidden.
        CannotCreateInHiddenScope,

        // Sharing related errors:

        /// Original post not found when sharing.
        OriginalPostNotFound,
        /// Cannot share a post that shares another post.
        CannotShareSharingPost,

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
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {

    const MaxCommentDepth: u32 = T::MaxCommentDepth::get();

    // Initializing errors
    type Error = Error<T>;

    // Initializing events
    fn deposit_event() = default;

    #[weight = 100_000]
    pub fn create_post(origin, space_id_opt: Option<SpaceId>, extension: PostExtension, content: Content) -> DispatchResult {
      let creator = ensure_signed(origin)?;

      Utils::<T>::is_valid_content(content.clone())?;

      let new_post_id = Self::next_post_id();
      let new_post: Post<T> = Post::new(new_post_id, creator.clone(), space_id_opt, extension, content);

      // Get space from either from space_id_opt or extension if a Comment provided
      let mut space = new_post.get_space()?;
      ensure!(!space.hidden, Error::<T>::CannotCreateInHiddenScope);

      let root_post = &mut new_post.get_root_post()?;
      ensure!(!root_post.hidden, Error::<T>::CannotCreateInHiddenScope);

      // Check permissions
      match extension {
        PostExtension::RegularPost | PostExtension::SharedPost(_) => {
          Spaces::ensure_account_has_space_permission(
            creator.clone(),
            &space,
            SpacePermission::CreatePosts,
            Error::<T>::NoPermissionToCreatePosts.into()
          )?;
        },
        PostExtension::Comment(_) => {
          Spaces::ensure_account_has_space_permission(
            creator.clone(),
            &space,
            SpacePermission::CreateComments,
            Error::<T>::NoPermissionToCreateComments.into()
          )?;
        }
      }

      match extension {
        PostExtension::RegularPost => {
          space.inc_posts();
        },

        PostExtension::SharedPost(post_id) => {
          let original_post = &mut Self::post_by_id(post_id).ok_or(Error::<T>::OriginalPostNotFound)?;
          ensure!(!original_post.is_sharing_post(), Error::<T>::CannotShareSharingPost);

          // Check if it's allowed to share a post from the space of original post.
          Spaces::ensure_account_has_space_permission(
            creator.clone(),
            &original_post.get_space()?,
            SpacePermission::Share,
            Error::<T>::NoPermissionToShare.into()
          )?;

          space.inc_posts();
          Self::share_post(creator.clone(), original_post, new_post_id)?;
        },

        PostExtension::Comment(comment_ext) => {
          root_post.inc_total_replies();

          if let Some(parent_id) = comment_ext.parent_id {
            let mut parent_comment = Self::post_by_id(parent_id).ok_or(Error::<T>::UnknownParentComment)?;
            ensure!(parent_comment.is_comment(), Error::<T>::NotACommentByParentId);
            parent_comment.inc_direct_replies();

            let mut ancestors = Self::get_post_ancestors(parent_id);
            ancestors[0] = parent_comment;
            ensure!(ancestors.len() < T::MaxCommentDepth::get() as usize, Error::<T>::MaxCommentDepthReached);
            for mut post in ancestors {
              post.inc_total_replies();
              <PostById<T>>::insert(post.id, post.clone());
            }

            ReplyIdsByPostId::mutate(parent_id, |ids| ids.push(new_post_id));
          } else {
            root_post.inc_direct_replies();
            ReplyIdsByPostId::mutate(comment_ext.root_post_id, |ids| ids.push(new_post_id));
          }

          T::PostScores::score_root_post_on_new_comment(creator.clone(), root_post)?;

          PostById::insert(comment_ext.root_post_id, root_post);
        }
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

    #[weight = 100_000]
    pub fn update_post(origin, post_id: PostId, update: PostUpdate) -> DispatchResult {
      let editor = ensure_signed(origin)?;

      let has_updates =
        update.space_id.is_some() ||
        update.content.is_some() ||
        update.hidden.is_some();

      ensure!(has_updates, Error::<T>::NoUpdatesForPost);

      let mut post = Self::require_post(post_id)?;

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
        &post.get_space()?,
        permission_to_check,
        permission_error
      )?;

      let mut fields_updated = 0;
      let mut old_data = PostUpdate::default();

      if let Some(content) = update.content {
        if content != post.content {
          Utils::<T>::is_valid_content(content.clone())?;
          old_data.content = Some(post.content);
          post.content = content;
          fields_updated += 1;
        }
      }

      if let Some(hidden) = update.hidden {
        if hidden != post.hidden {
          old_data.hidden = Some(post.hidden);
          post.hidden = hidden;
          fields_updated += 1;
        }
      }

      // Move this post to another space:
      if let Some(space_id) = update.space_id {
        ensure!(post.is_root_post(), Error::<T>::CannotUpdateSpaceIdOnComment);

        if let Some(post_space_id) = post.space_id {
          if space_id != post_space_id {
            Spaces::<T>::ensure_space_exists(space_id)?;
            // TODO check that the current user has CreatePosts permission in new space_id.

            // Remove post_id from its old space:
            PostIdsBySpaceId::mutate(post_space_id, |post_ids| vec_remove_on(post_ids, post_id));

            // Add post_id to its new space:
            PostIdsBySpaceId::mutate(space_id, |ids| ids.push(post_id));
            old_data.space_id = post.space_id;
            post.space_id = Some(space_id);
            fields_updated += 1;
          }
        }
      }

      // Update this post only if at least one field should be updated:
      if fields_updated > 0 {
        post.updated = Some(WhoAndWhen::<T>::new(editor.clone()));

        <PostById<T>>::insert(post.id, post.clone());
        T::AfterPostUpdated::after_post_updated(editor.clone(), &post, old_data);

        Self::deposit_event(RawEvent::PostUpdated(editor, post_id));
      }
      Ok(())
    }
  }
}
