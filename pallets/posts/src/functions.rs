use frame_support::dispatch::DispatchResult;

use pallet_utils::{SpaceId, remove_from_vec};

use super::*;

impl<T: Trait> Post<T> {

    pub fn new(
        id: PostId,
        created_by: T::AccountId,
        space_id_opt: Option<SpaceId>,
        extension: PostExtension,
        content: Content
    ) -> Self {
        Post {
            id,
            created: WhoAndWhen::<T>::new(created_by.clone()),
            updated: None,
            owner: created_by,
            extension,
            space_id: space_id_opt,
            content,
            hidden: false,
            replies_count: 0,
            hidden_replies_count: 0,
            shares_count: 0,
            upvotes_count: 0,
            downvotes_count: 0,
            score: 0
        }
    }

    pub fn ensure_owner(&self, account: &T::AccountId) -> DispatchResult {
        ensure!(self.is_owner(account), Error::<T>::NotAPostOwner);
        Ok(())
    }

    pub fn is_owner(&self, account: &T::AccountId) -> bool {
        self.owner == *account
    }

    pub fn is_root_post(&self) -> bool {
        !self.is_comment()
    }

    pub fn is_regular_post(&self) -> bool {
        matches!(self.extension, PostExtension::RegularPost)
    }

    pub fn is_comment(&self) -> bool {
        matches!(self.extension, PostExtension::Comment(_))
    }

    pub fn is_sharing_post(&self) -> bool {
        matches!(self.extension, PostExtension::SharedPost(_))
    }

    pub fn get_comment_ext(&self) -> Result<Comment, DispatchError> {
        match self.extension {
            PostExtension::Comment(comment_ext) => Ok(comment_ext),
            _ => Err(Error::<T>::NotComment.into())
        }
    }

    pub fn get_shared_post_id(&self) -> Result<PostId, DispatchError> {
        match self.extension {
            PostExtension::SharedPost(post_id) => Ok(post_id),
            _ => Err(Error::<T>::NotASharingPost.into())
        }
    }

    pub fn get_root_post(&self) -> Result<Post<T>, DispatchError> {
        match self.extension {
            PostExtension::RegularPost | PostExtension::SharedPost(_) =>
                Ok(self.clone()),
            PostExtension::Comment(comment) =>
                Module::require_post(comment.root_post_id),
        }
    }

    pub fn get_space_id(&self) -> Result<SpaceId, DispatchError> {
        Self::try_get_space_id(self).ok_or_else(|| Error::<T>::PostHasNoSpaceId.into())
    }

    pub fn try_get_space_id(&self) -> Option<SpaceId> {
        if let Ok(root_post) = self.get_root_post() {
            return root_post.space_id;
        }

        None
    }

    pub fn get_space(&self) -> Result<Space<T>, DispatchError> {
        let root_post = self.get_root_post()?;
        let space_id = root_post.space_id.ok_or(Error::<T>::PostHasNoSpaceId)?;
        Spaces::require_space(space_id)
    }

    pub fn try_get_space(&self) -> Option<Space<T>> {
        if let Ok(root_post) = self.get_root_post() {
            return root_post.space_id.and_then(|space_id| Spaces::require_space(space_id).ok());
        }

        None
    }

    // TODO use macros to generate inc/dec fns for Space, Post.

    pub fn inc_replies(&mut self) {
        self.replies_count = self.replies_count.saturating_add(1);
    }

    pub fn dec_replies(&mut self) {
        self.replies_count = self.replies_count.saturating_sub(1);
    }

    pub fn inc_hidden_replies(&mut self) {
        self.hidden_replies_count = self.hidden_replies_count.saturating_add(1);
    }

    pub fn dec_hidden_replies(&mut self) {
        self.hidden_replies_count = self.hidden_replies_count.saturating_sub(1);
    }

    pub fn inc_shares(&mut self) {
        self.shares_count = self.shares_count.saturating_add(1);
    }

    pub fn dec_shares(&mut self) {
        self.shares_count = self.shares_count.saturating_sub(1);
    }

    pub fn inc_upvotes(&mut self) {
        self.upvotes_count = self.upvotes_count.saturating_add(1);
    }

    pub fn dec_upvotes(&mut self) {
        self.upvotes_count = self.upvotes_count.saturating_sub(1);
    }

    pub fn inc_downvotes(&mut self) {
        self.downvotes_count = self.downvotes_count.saturating_add(1);
    }

    pub fn dec_downvotes(&mut self) {
        self.downvotes_count = self.downvotes_count.saturating_sub(1);
    }

    #[allow(clippy::comparison_chain)]
    pub fn change_score(&mut self, diff: i16) {
        if diff > 0 {
            self.score = self.score.saturating_add(diff.abs() as i32);
        } else if diff < 0 {
            self.score = self.score.saturating_sub(diff.abs() as i32);
        }
    }

    pub fn is_public(&self) -> bool {
        !self.hidden && self.content.is_some()
    }

    pub fn is_unlisted(&self) -> bool {
        !self.is_public()
    }

    pub fn change_hidden(&mut self, hidden: bool, space_opt: &mut Option<Space<T>>) -> DispatchResult {
        if hidden != self.hidden {
            *space_opt = space_opt.clone().map(|mut space| {
                if hidden {
                    space.inc_hidden_posts();
                } else {
                    space.dec_hidden_posts();
                }

                space
            });

            if let PostExtension::Comment(comment_ext) = self.extension {
                Module::<T>::update_counters_on_comment_hidden_change(&comment_ext, hidden)?;
            }

            self.hidden = hidden;
        }

        Ok(())
    }
}

impl Default for PostUpdate {
    fn default() -> Self {
        PostUpdate {
            space_id: None,
            content: None,
            hidden: None
        }
    }
}

impl<T: Trait> Module<T> {

    pub fn ensure_account_can_update_post(
        editor: &T::AccountId,
        post: &Post<T>,
        space: &Space<T>
    ) -> DispatchResult {
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
        } else {
          // Not a comment

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
        )
    }

    /// Check that there is a `Post` with such `post_id` in the storage
    /// or return`PostNotFound` error.
    pub fn ensure_post_exists(post_id: PostId) -> DispatchResult {
        ensure!(<PostById<T>>::contains_key(post_id), Error::<T>::PostNotFound);
        Ok(())
    }

    /// Get `Post` by id from the storage or return `PostNotFound` error.
    pub fn require_post(post_id: SpaceId) -> Result<Post<T>, DispatchError> {
        Ok(Self::post_by_id(post_id).ok_or(Error::<T>::PostNotFound)?)
    }

    fn share_post(
        account: T::AccountId,
        original_post: &mut Post<T>,
        shared_post_id: PostId
    ) -> DispatchResult {
        original_post.inc_shares();

        T::PostScores::score_post_on_new_share(account.clone(), original_post)?;

        let original_post_id = original_post.id;
        PostById::insert(original_post_id, original_post.clone());
        SharedPostIdsByOriginalPostId::mutate(original_post_id, |ids| ids.push(shared_post_id));

        Self::deposit_event(RawEvent::PostShared(account, original_post_id));

        Ok(())
    }

    pub fn is_root_post_hidden(post_id: PostId) -> Result<bool, DispatchError> {
        let post = Self::require_post(post_id)?;
        let root_post = post.get_root_post()?;
        Ok(root_post.hidden)
    }

    pub fn is_root_post_visible(post_id: PostId) -> Result<bool, DispatchError> {
        Self::is_root_post_hidden(post_id).map(|v| !v)
    }

    pub fn mutate_post_by_id<F: FnOnce(&mut Post<T>)> (
        post_id: PostId,
        f: F
    ) -> Result<Post<T>, DispatchError> {
        <PostById<T>>::mutate(post_id, |post_opt| {
            if let Some(ref mut post) = post_opt.clone() {
                f(post);
                *post_opt = Some(post.clone());

                return Ok(post.clone());
            }

            Err(Error::<T>::PostNotFound.into())
        })
    }

    // TODO refactor to a tail recursion
    /// Get all post ancestors (parent_id) including this post
    pub fn get_post_ancestors(post_id: PostId) -> Vec<Post<T>> {
        let mut ancestors: Vec<Post<T>> = Vec::new();

        if let Some(post) = Self::post_by_id(post_id) {
            ancestors.push(post.clone());
            if let Some(parent_id) = post.get_comment_ext().ok().unwrap().parent_id {
                ancestors.extend(Self::get_post_ancestors(parent_id).iter().cloned());
            }
        }

        ancestors
    }

    /// Applies function to all post ancestors (parent_id) including this post
    pub fn for_each_post_ancestor<F: FnMut(&mut Post<T>) + Copy> (
        post_id: PostId,
        f: F
    ) -> DispatchResult {
        let post = Self::mutate_post_by_id(post_id, f)?;

        if let PostExtension::Comment(comment_ext) = post.extension {
            if let Some(parent_id) = comment_ext.parent_id {
                Self::for_each_post_ancestor(parent_id, f)?;
            }
        }

        Ok(())
    }

    pub fn try_get_post_replies(post_id: PostId) -> Vec<Post<T>> {
        let mut replies: Vec<Post<T>> = Vec::new();

        if let Some(post) = Self::post_by_id(post_id) {
            replies.push(post);
            for reply_id in Self::reply_ids_by_post_id(post_id).iter() {
                replies.extend(Self::try_get_post_replies(*reply_id).iter().cloned());
            }
        }

        replies
    }

    /// Recursively et all nested post replies (reply_ids_by_post_id)
    pub fn get_post_replies(post_id: PostId) -> Result<Vec<Post<T>>, DispatchError> {
        let reply_ids = Self::reply_ids_by_post_id(post_id);
        ensure!(!reply_ids.is_empty(), Error::<T>::NoRepliesOnPost);

        let mut replies: Vec<Post<T>> = Vec::new();
        for reply_id in reply_ids.iter() {
            replies.extend(Self::try_get_post_replies(*reply_id));
        }
        Ok(replies)
    }
    // TODO: maybe add for_each_reply?

    pub(crate) fn create_comment(
        creator: &T::AccountId,
        new_post_id: PostId,
        comment_ext: Comment,
        root_post: &mut Post<T>
    ) -> DispatchResult {
        let mut commented_post_id = root_post.id;

        if let Some(parent_id) = comment_ext.parent_id {
            let parent_comment = Self::post_by_id(parent_id).ok_or(Error::<T>::UnknownParentComment)?;
            ensure!(parent_comment.is_comment(), Error::<T>::NotACommentByParentId);

            let ancestors = Self::get_post_ancestors(parent_id);
            ensure!(ancestors.len() < T::MaxCommentDepth::get() as usize, Error::<T>::MaxCommentDepthReached);

            commented_post_id = parent_id;
        }

        root_post.inc_replies();
        T::PostScores::score_root_post_on_new_comment(creator.clone(), root_post)?;

        Self::for_each_post_ancestor(commented_post_id, |post| post.inc_replies())?;
        PostById::insert(root_post.id, root_post);
        ReplyIdsByPostId::mutate(commented_post_id, |reply_ids| reply_ids.push(new_post_id));

        Ok(())
    }

    pub(crate) fn create_sharing_post(
        creator: &T::AccountId,
        new_post_id: PostId,
        original_post_id: PostId,
        space: &mut Space<T>
    ) -> DispatchResult {
        let original_post = &mut Self::post_by_id(original_post_id)
            .ok_or(Error::<T>::OriginalPostNotFound)?;

        ensure!(!original_post.is_sharing_post(), Error::<T>::CannotShareSharingPost);

        // Check if it's allowed to share a post from the space of original post.
        Spaces::ensure_account_has_space_permission(
            creator.clone(),
            &original_post.get_space()?,
            SpacePermission::Share,
            Error::<T>::NoPermissionToShare.into()
        )?;

        space.inc_posts();

        Self::share_post(creator.clone(), original_post, new_post_id)
    }

    fn mutate_posts_count_on_space<F: FnMut(&mut u32) + Copy> (
        space_id: SpaceId,
        post: &Post<T>,
        mut f: F
    ) -> DispatchResult {
        Spaces::<T>::mutate_space_by_id(space_id, |space: &mut Space<T>| {
            f(&mut space.posts_count);
            if post.hidden {
                f(&mut space.hidden_posts_count);
            }
        }).map(|_| ())
    }

    pub(crate) fn move_post_to_space(
        editor: T::AccountId,
        post: &mut Post<T>,
        new_space_id: SpaceId
    ) -> DispatchResult {
        let old_space_id_opt = post.try_get_space_id();
        let new_space = Spaces::<T>::require_space(new_space_id)?;

        ensure!(
            T::IsAccountBlocked::is_allowed_account(editor.clone(), new_space_id),
            UtilsError::<T>::AccountIsBlocked
        );
        Spaces::ensure_account_has_space_permission(
            editor,
            &new_space,
            SpacePermission::CreatePosts,
            Error::<T>::NoPermissionToCreatePosts.into()
        )?;
        ensure!(
            T::IsPostBlocked::is_allowed_post(post.id, new_space_id),
            UtilsError::<T>::PostIsBlocked
        );
        ensure!(
            T::IsContentBlocked::is_allowed_content(post.content.clone(), new_space_id),
            UtilsError::<T>::ContentIsBlocked
        );

        match post.extension {
            PostExtension::RegularPost | PostExtension::SharedPost(_) => {

                if let Some(old_space_id) = old_space_id_opt {

                    // Decrease the number of posts on the old space
                    Self::mutate_posts_count_on_space(
                        old_space_id,
                        post,
                        |counter| *counter = counter.saturating_sub(1)
                    )?;

                    // Decrease a score on the old space
                    Spaces::<T>::mutate_space_by_id(
                        old_space_id,
                        |space| space.score = space.score.saturating_sub(post.score)
                    )?;

                    PostIdsBySpaceId::mutate(old_space_id, |post_ids| remove_from_vec(post_ids, post.id));
                }

                // Increase the number of posts on the new space
                Self::mutate_posts_count_on_space(
                    new_space_id,
                    post,
                    |counter| *counter = counter.saturating_add(1)
                )?;

                // Increase a score on the new space
                Spaces::<T>::mutate_space_by_id(
                    new_space_id,
                    |space| space.score = space.score.saturating_add(post.score)
                )?;

                PostIdsBySpaceId::mutate(new_space_id, |post_ids| post_ids.push(post.id));

                post.space_id = Some(new_space_id);
                PostById::<T>::insert(post.id, post);

                Ok(())
            },
            _ => fail!(Error::<T>::CannotUpdateSpaceIdOnComment),
        }
    }

    pub fn delete_post_from_space(post_id: PostId) -> DispatchResult {
        let mut post = Self::require_post(post_id)?;

        if let PostExtension::Comment(comment_ext) = post.extension {
            post.extension = PostExtension::RegularPost;

            let root_post = &mut Self::require_post(comment_ext.root_post_id)?;
            let parent_id = comment_ext.parent_id.unwrap_or(root_post.id);

            let dec_replies_count: fn(&mut Post<T>) = |p| {
                p.dec_replies();
                if p.hidden {
                    p.dec_hidden_replies();
                }
            };

            dec_replies_count(root_post);
            PostById::<T>::insert(root_post.id, root_post.clone());
            Self::for_each_post_ancestor(parent_id, dec_replies_count)?;

            // Subtract the weight of CreateComment from the root post and its space
            T::PostScores::score_root_post_on_new_comment(post.created.account.clone(), root_post)?;
            let replies = Self::get_post_replies(post_id)?;
            for reply in replies.iter() {
                T::PostScores::score_root_post_on_new_comment(reply.created.account.clone(), root_post)?;
            }
        } else {
            // If post is not a comment:

            let space_id = post.get_space_id()?;

            // Decrease the number of posts on the space
            Self::mutate_posts_count_on_space(
                space_id,
                &post,
                |counter| *counter = counter.saturating_sub(1)
            )?;

            Spaces::<T>::mutate_space_by_id(
                space_id,
                |space| space.score = space.score.saturating_sub(post.score)
            )?;

            post.space_id = None;
            PostIdsBySpaceId::mutate(space_id, |post_ids| remove_from_vec(post_ids, post_id));
        }

        PostById::<T>::insert(post.id, post);

        Ok(())
    }

    /// Rewrite ancestor counters when Post hidden status changes
    /// Warning: This will affect storage state!
    pub(crate) fn update_counters_on_comment_hidden_change(
        comment_ext: &Comment,
        becomes_hidden: bool
    ) -> DispatchResult {
        let root_post = &mut Self::require_post(comment_ext.root_post_id)?;
        let commented_post_id = comment_ext.parent_id.unwrap_or(root_post.id);

        let mut update_hidden_replies: fn(&mut Post<T>) = Post::inc_hidden_replies;
        if !becomes_hidden {
            update_hidden_replies = Post::dec_hidden_replies;
        }

        Self::for_each_post_ancestor(commented_post_id, |post| update_hidden_replies(post))?;

        update_hidden_replies(root_post);
        PostById::insert(root_post.id, root_post);

        Ok(())
    }
}
