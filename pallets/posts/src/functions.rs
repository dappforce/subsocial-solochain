use frame_support::dispatch::DispatchResult;

use pallet_spaces::Space;
use pallet_utils::SpaceId;

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
            created: WhoAndWhen::<T>::new(created_by),
            updated: None,
            hidden: false,
            space_id: space_id_opt,
            extension,
            content,
            direct_replies_count: 0,
            total_replies_count: 0,
            shares_count: 0,
            upvotes_count: 0,
            downvotes_count: 0,
            score: 0
        }
    }

    pub fn is_owner(&self, account: &T::AccountId) -> bool {
        self.created.account == *account
    }

    pub fn is_root_post(&self) -> bool {
        !self.is_comment()
    }

    pub fn is_comment(&self) -> bool {
        match self.extension {
            PostExtension::Comment(_) => true,
            _ => false,
        }
    }

    pub fn is_sharing_post(&self) -> bool {
        match self.extension {
            PostExtension::SharedPost(_) => true,
            _ => false,
        }
    }

    pub fn get_comment_ext(&self) -> Result<CommentExt, DispatchError> {
        match self.extension {
            PostExtension::Comment(comment_ext) => Ok(comment_ext),
            _ => Err(Error::<T>::NotComment.into())
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

    pub fn get_space(&self) -> Result<Space<T>, DispatchError> {
        let root_post = self.get_root_post()?;
        let space_id = root_post.space_id.ok_or(Error::<T>::PostHasNoSpaceId)?;
        Spaces::require_space(space_id)
    }

    // TODO use macros to generate inc/dec fns for Space, Post.

    pub fn inc_direct_replies(&mut self) {
        self.direct_replies_count = self.direct_replies_count.saturating_add(1);
    }

    pub fn dec_direct_replies(&mut self) {
        self.direct_replies_count = self.direct_replies_count.saturating_sub(1);
    }

    pub fn inc_total_replies(&mut self) {
        self.total_replies_count = self.total_replies_count.saturating_add(1);
    }

    pub fn dec_total_replies(&mut self) {
        self.total_replies_count = self.total_replies_count.saturating_sub(1);
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

    /// Check that there is a `Post` with such `post_id` in the storage
    /// or return`PostNotFound` error.
    pub fn ensure_post_exists(post_id: PostId) -> DispatchResult {
        ensure!(<PostById<T>>::exists(post_id), Error::<T>::PostNotFound);
        Ok(())
    }

    /// Get `Post` by id from the storage or return `PostNotFound` error.
    pub fn require_post(post_id: SpaceId) -> Result<Post<T>, DispatchError> {
        Ok(Self::post_by_id(post_id).ok_or(Error::<T>::PostNotFound)?)
    }

    pub fn share_post(
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

    // TODO refactor to a tail recursion
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
}
