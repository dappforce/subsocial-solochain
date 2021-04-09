use frame_support::dispatch::DispatchResult;

use pallet_utils::{SpaceId, vec_remove_on};

use super::*;

impl<T: Config> Post<T> {

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
        !self.hidden && !self.content.is_none()
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

impl<T: Config> Module<T> {

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

    pub fn try_get_post_replies_ids(post_id: PostId) -> Vec<PostId> {
        let mut replies: Vec<PostId> = Vec::new();

        if let Some(post) = Self::post_by_id(post_id) {
            replies.push(post.id);
            for reply_id in Self::reply_ids_by_post_id(post_id).iter() {
                replies.extend(Self::try_get_post_replies_ids(*reply_id).iter().cloned());
            }
        }

        replies
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

    pub fn get_post_reply_ids_tree(post_id: PostId) -> Vec<(PostId, Vec<PostId>)> {
        let mut replies_tuple: Vec<(PostId, Vec<PostId>)> = Vec::new();
        let post_replies: Vec<PostId> = Self::reply_ids_by_post_id(post_id);

        if post_replies.first().is_some() {
            replies_tuple.push((post_id, post_replies.clone()));
            for reply_id in post_replies.iter() {
                replies_tuple.extend(Self::get_post_reply_ids_tree(*reply_id));
            }
        }

        replies_tuple
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
        ReplyIdsByPostId::mutate(commented_post_id, |ids| ids.push(new_post_id));

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
            if post.hidden {
                f(&mut space.hidden_posts_count);
                f(&mut space.posts_count);
            } else {
                f(&mut space.posts_count);
            }
        }).map(|_| ())
    }

    pub(crate) fn move_post_to_space(
        editor: T::AccountId,
        post: &mut Post<T>,
        new_space_id: SpaceId
    ) -> DispatchResult {
        let old_space_id = post.get_space_id()?;
        let new_space = Spaces::<T>::require_space(new_space_id)?;

        ensure!(
            !T::IsAccountBlocked::is_account_blocked(editor.clone(), new_space_id),
            UtilsError::<T>::AccountIsBlocked
        );

        Spaces::ensure_account_has_space_permission(
            editor,
            &new_space,
            SpacePermission::CreatePosts,
            Error::<T>::NoPermissionToCreatePosts.into()
        )?;

        ensure!(!T::IsPostBlocked::is_post_blocked(post.id, new_space_id), UtilsError::<T>::PostIsBlocked);
        ensure!(
            !T::IsContentBlocked::is_content_blocked(post.content.clone(), new_space_id),
            UtilsError::<T>::ContentIsBlocked
        );

        match post.extension {
            PostExtension::RegularPost | PostExtension::SharedPost(_) => {

                // Substitute posts count from the old space
                Self::mutate_posts_count_on_space(
                    old_space_id,
                    post,
                    |counter| *counter = counter.saturating_sub(1)
                )?;
                // Add posts count to the new space
                Self::mutate_posts_count_on_space(
                    new_space_id,
                    post,
                    |counter| *counter = counter.saturating_add(1)
                )?;

                // Substitute score from the old space
                Spaces::<T>::mutate_space_by_id(
                    old_space_id,
                    |space| space.score = space.score.saturating_sub(post.score)
                )?;
                // Add score to the new space
                Spaces::<T>::mutate_space_by_id(
                    new_space_id,
                    |space| space.score = space.score.saturating_add(post.score)
                )?;

                post.space_id = Some(new_space_id);

                PostIdsBySpaceId::mutate(old_space_id, |post_ids| vec_remove_on(post_ids, post.id));
                PostIdsBySpaceId::mutate(new_space_id, |ids| ids.push(post.id));
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

            // Choose desired counter change whether comment was hidden or not
            let mut update_replies_change: fn(&mut Post<T>) = Post::dec_replies;
            if post.hidden {
                update_replies_change = Post::dec_hidden_replies;
            }

            update_replies_change(root_post);
            PostById::<T>::insert(root_post.id, root_post.clone());
            Self::for_each_post_ancestor(parent_id, |p| update_replies_change(p))?;

            // Subtract CreateComment score weight on root post and its space
            T::PostScores::score_root_post_on_new_comment(post.created.account.clone(), root_post)?;
            let replies = Self::get_post_replies(post_id)?;
            for reply in replies.iter() {
                T::PostScores::score_root_post_on_new_comment(reply.created.account.clone(), root_post)?;
            }
        } else {
            let space_id = post.get_space_id()?;
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

            PostIdsBySpaceId::mutate(space_id, |post_ids| vec_remove_on(post_ids, post_id));
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
