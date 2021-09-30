use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_std::collections::{btree_map::BTreeMap, btree_set::BTreeSet};
use sp_std::{vec, prelude::*};

use pallet_space_follows::Module as SpaceFollows;
use pallet_spaces::Module as Spaces;
use pallet_utils::{bool_to_option, PostId, rpc::{FlatContent, FlatWhoAndWhen, ShouldSkip}, SpaceId};

use crate::{Module, Post, PostExtension, FIRST_POST_ID, Config};
pub type RepliesByPostId<AccountId, BlockNumber> = BTreeMap<PostId, Vec<FlatPost<AccountId, BlockNumber>>>;

#[derive(Eq, PartialEq, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct FlatPostExtension {
    #[cfg_attr(feature = "std", serde(skip_serializing_if = "ShouldSkip::should_skip"))]
    pub is_regular_post: Option<bool>,
    #[cfg_attr(feature = "std", serde(skip_serializing_if = "ShouldSkip::should_skip"))]
    pub is_shared_post: Option<bool>,
    #[cfg_attr(feature = "std", serde(skip_serializing_if = "ShouldSkip::should_skip"))]
    pub is_comment: Option<bool>,

    #[cfg_attr(feature = "std", serde(skip_serializing_if = "ShouldSkip::should_skip"))]
    pub root_post_id: Option<PostId>,
    #[cfg_attr(feature = "std", serde(skip_serializing_if = "ShouldSkip::should_skip"))]
    pub parent_post_id: Option<PostId>,
    #[cfg_attr(feature = "std", serde(skip_serializing_if = "ShouldSkip::should_skip"))]
    pub shared_post_id: Option<PostId>,
}

impl From<PostExtension> for FlatPostExtension {
    fn from(from: PostExtension) -> Self {
        let mut flat_ext = Self::default();

        match from {
            PostExtension::RegularPost => {
                flat_ext.is_regular_post = Some(true);
            }
            PostExtension::Comment(comment_ext) => {
                flat_ext.is_comment = Some(true);
                flat_ext.root_post_id = Some(comment_ext.root_post_id);
                flat_ext.parent_post_id = comment_ext.parent_id;
            }
            PostExtension::SharedPost(shared_post_id) => {
                flat_ext.is_shared_post = Some(true);
                flat_ext.shared_post_id = Some(shared_post_id);
            }
        }

        flat_ext
    }
}

#[derive(Eq, PartialEq, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct FlatPost<AccountId, BlockNumber> {
    pub id: PostId,

    #[cfg_attr(feature = "std", serde(flatten))]
    pub who_and_when: FlatWhoAndWhen<AccountId, BlockNumber>,

    pub owner: AccountId,

    #[cfg_attr(feature = "std", serde(skip_serializing_if = "ShouldSkip::should_skip"))]
    pub space_id: Option<SpaceId>,

    #[cfg_attr(feature = "std", serde(flatten))]
    pub content: FlatContent,

    #[cfg_attr(feature = "std", serde(skip_serializing_if = "ShouldSkip::should_skip"))]
    pub is_hidden: Option<bool>,

    #[cfg_attr(feature = "std", serde(flatten))]
    pub extension: FlatPostExtension,

    pub replies_count: u16,
    pub hidden_replies_count: u16,
    pub visible_replies_count: u16,

    pub shares_count: u16,
    pub upvotes_count: u16,
    pub downvotes_count: u16,

    pub score: i32,
}

#[derive(Encode, Decode, Ord, PartialOrd, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum FlatPostKind {
    RegularPost,
    Comment,
    SharedPost
}

impl<T: Config> From<Post<T>> for FlatPostKind {
    fn from(from: Post<T>) -> Self {
        match from.extension {
            PostExtension::RegularPost => { Self::RegularPost }
            PostExtension::Comment(_) => { Self::Comment }
            PostExtension::SharedPost(_) => { Self::SharedPost }
        }
    }
}

impl<T: Config> From<Post<T>> for FlatPost<T::AccountId, T::BlockNumber> {
    fn from(from: Post<T>) -> Self {
        let Post {
            id, created, updated, owner,
            extension, space_id, content, hidden, replies_count,
            hidden_replies_count, shares_count, upvotes_count, downvotes_count, score
        } = from;

        Self {
            id,
            who_and_when: (created, updated).into(),
            owner,
            space_id,
            content: content.into(),
            is_hidden: bool_to_option(hidden),
            extension: extension.into(),
            replies_count,
            hidden_replies_count,
            visible_replies_count: replies_count.saturating_sub(hidden_replies_count),
            shares_count,
            upvotes_count,
            downvotes_count,
            score,
        }
    }
}

impl<T: Config> Module<T> {
    fn get_posts_by_ids_with_filter<F: FnMut(&Post<T>) -> bool>(
        all_post_ids: Vec<PostId>,
        offset: u64,
        limit: u16,
        mut filter: F,
    ) -> Vec<FlatPost<T::AccountId, T::BlockNumber>> {
        let mut posts = Vec::new();

        let (_, posts_ids) = all_post_ids.split_at(offset as usize);

        for post_id in posts_ids.iter() {
            if let Ok(post) = Self::require_post(*post_id) {
                if filter(&post) {
                    posts.push(post.into());
                }
            }

            if posts.len() >= limit as usize { break; }
        }

        posts
    }

    pub fn get_posts_by_ids (
        post_ids: Vec<PostId>,
        offset: u64,
        limit: u16,
    ) -> Vec<FlatPost<T::AccountId, T::BlockNumber>> {
        Self::get_posts_by_ids_with_filter(post_ids, offset, limit, |_| true)
    }

    pub fn get_public_posts_by_ids(
        post_ids: Vec<PostId>,
        offset: u64,
        limit: u16,
    ) -> Vec<FlatPost<T::AccountId, T::BlockNumber>> {
        Self::get_posts_by_ids_with_filter(post_ids, offset, limit, |post| post.is_public())
    }

    fn get_posts_slice_by_space_id<F: FnMut(&Post<T>) -> bool>(
        space_id: SpaceId,
        offset: u64,
        limit: u16,
        filter: F,
    ) -> Vec<FlatPost<T::AccountId, T::BlockNumber>> {
        let mut post_ids: Vec<PostId> = Self::post_ids_by_space_id(space_id);
        post_ids.reverse();

        Self::get_posts_by_ids_with_filter(post_ids, offset, limit, filter)
    }

    pub fn get_public_posts_by_space_id(
        space_id: SpaceId,
        offset: u64,
        limit: u16,
    ) -> Vec<FlatPost<T::AccountId, T::BlockNumber>> {
        if let Ok(space) = Spaces::<T>::require_space(space_id) {
            return Self::get_posts_slice_by_space_id(space.id, offset, limit, |post| post.is_public());
        }

        vec![]
    }

    pub fn get_unlisted_posts_by_space_id(
        space_id: SpaceId,
        offset: u64,
        limit: u16,
    ) -> Vec<FlatPost<T::AccountId, T::BlockNumber>> {
        if let Ok(space) = Spaces::<T>::require_space(space_id) {
            return Self::get_posts_slice_by_space_id(space.id, offset, limit, |post| post.is_unlisted());
        }

        vec![]
    }

    pub fn get_reply_ids_by_parent_id(parent_id: PostId) -> Vec<PostId> {
        Self::reply_ids_by_post_id(parent_id)
    }

    pub fn get_replies_by_parent_id(parent_id: PostId, offset: u64, limit: u16) -> Vec<FlatPost<T::AccountId, T::BlockNumber>> {
        let reply_ids = Self::get_reply_ids_by_parent_id(parent_id);
        Self::get_posts_by_ids(reply_ids, offset, limit)
    }

    pub fn get_reply_ids_by_parent_ids(parent_ids: Vec<PostId>) -> BTreeMap<PostId, Vec<PostId>> {
        let mut reply_ids_by_parent: BTreeMap<PostId, Vec<PostId>> = BTreeMap::new();

        for parent_id in parent_ids.iter() {
            let reply_ids = Self::get_reply_ids_by_parent_id(*parent_id);

            if !reply_ids.is_empty() {
                reply_ids_by_parent.insert(*parent_id, reply_ids);
            }
        }

        reply_ids_by_parent
    }

    pub fn get_replies_by_parent_ids(
        parent_ids: Vec<PostId>,
        offset: u64,
        limit: u16
    ) -> RepliesByPostId<T::AccountId, T::BlockNumber> {

       Self::get_reply_ids_by_parent_ids(parent_ids)
           .into_iter()
           .map(|(parent_id, reply_ids)|
               (parent_id, Self::get_posts_by_ids(reply_ids, offset, limit))
           )
           .collect()
    }

    pub fn get_public_posts(
        kind_filter: Vec<FlatPostKind>,
        start_id: u64,
        limit: u16,
    ) -> Vec<FlatPost<T::AccountId, T::BlockNumber>> {

        let no_filter = kind_filter.is_empty();
        let kind_filter_set: BTreeSet<_> = kind_filter.into_iter().collect();

        let mut posts = Vec::new();
        let mut post_id = start_id;

        while posts.len() < limit as usize && post_id >= FIRST_POST_ID {
            if let Ok(post) = Self::require_post(post_id) {
                let kind: FlatPostKind = post.clone().into();

                if post.is_public() && (no_filter || kind_filter_set.contains(&kind)) {
                    posts.push(post.into());
                }
            }
            post_id = post_id.saturating_sub(1);
        }

        posts
    }

    fn get_post_ids_by_space<F: FnMut(&Post<T>) -> bool>(space_id: SpaceId, mut filter: F) -> Vec<PostId> {
        Self::post_ids_by_space_id(space_id)
            .iter()
            .filter_map(Self::post_by_id)
            .filter(|post| filter(post))
            .map(|post| post.id)
            .collect()
    }

    pub fn get_public_post_ids_by_space_id(space_id: SpaceId) -> Vec<PostId> {
        let public_space = Spaces::<T>::require_space(space_id).ok().filter(|space| space.is_public());
        if public_space.is_some() {
            return Self::get_post_ids_by_space(space_id, |post| post.is_public());
        }

        vec![]
    }

    pub fn get_unlisted_post_ids_by_space_id(space_id: SpaceId) -> Vec<PostId> {
        let unlisted_space = Spaces::<T>::require_space(space_id).ok().filter(|space| !space.is_public());
        if unlisted_space.is_some() {
            return Self::get_post_ids_by_space(space_id, |post| !post.is_public());
        }

        vec![]
    }

    pub fn get_next_post_id() -> PostId {
        Self::next_post_id()
    }

    pub fn get_feed(account: T::AccountId, offset: u64, limit: u16) -> Vec<FlatPost<T::AccountId, T::BlockNumber>> {
        let mut post_ids: Vec<PostId> = SpaceFollows::<T>::spaces_followed_by_account(account)
            .iter()
            .flat_map(Self::post_ids_by_space_id)
            .collect();

        // Sort post ids in a descending order
        post_ids.sort_by(|a, b| b.cmp(a));

        Self::get_posts_by_ids_with_filter(post_ids, offset, limit, |post| post.is_public() && !post.is_comment())
    }
}
