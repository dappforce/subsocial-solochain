use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_std::collections::btree_map::BTreeMap;
use sp_std::iter::FromIterator;
use sp_std::{vec, prelude::*};

use pallet_space_follows::Module as SpaceFollows;
use pallet_spaces::Module as Spaces;
use pallet_utils::{from_bool_to_option, PostId, rpc::{FlatContent, FlatWhoAndWhen}, SpaceId};

use crate::{Module, Post, PostExtension, Trait};

#[derive(Eq, PartialEq, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct FlatPost<AccountId, BlockNumber> {
    pub id: PostId,
    #[cfg_attr(feature = "std", serde(flatten))]
    pub who_and_when: FlatWhoAndWhen<AccountId, BlockNumber>,

    pub owner: AccountId,

    pub space_id: Option<SpaceId>,

    #[cfg_attr(feature = "std", serde(flatten))]
    pub content: FlatContent,

    pub is_hidden: Option<bool>,

    #[cfg_attr(features = "std", serde(flatten))]
    pub extension: PostExtension,

    pub is_regular_post: Option<bool>,
    pub is_shared_post: Option<bool>,
    pub is_comment: Option<bool>,

    pub root_post_id: Option<PostId>,
    pub parent_post_id: Option<PostId>,
    pub shared_post_id: Option<PostId>,

    pub visible_replies_count: u16,

    pub shares_count: u16,
    pub upvotes_count: u16,
    pub downvotes_count: u16,

    pub score: i32,
}

impl<T: Trait> From<Post<T>> for FlatPost<T::AccountId, T::BlockNumber> {
    fn from(from: Post<T>) -> Self {
        let Post {
            id, created, updated, owner,
            extension, space_id, content, hidden, replies_count,
            hidden_replies_count, shares_count, upvotes_count, downvotes_count, score
        } = from.clone();

        let comment_ext = from.get_comment_ext().ok();

        Self {
            id,
            who_and_when: (created, updated).into(),
            owner,
            space_id,
            content: content.into(),
            is_hidden: Some(hidden).filter(|value| *value == true),
            extension,
            is_regular_post: from_bool_to_option(from.is_regular_post()),
            is_shared_post: from_bool_to_option(from.is_sharing_post()),
            is_comment: from_bool_to_option(from.is_comment()),
            root_post_id: comment_ext.clone().map(|comment_ext| comment_ext.root_post_id),
            parent_post_id: comment_ext.and_then(|comment_ext| comment_ext.parent_id),
            shared_post_id: from.get_shared_post_id().ok(),
            visible_replies_count: replies_count.saturating_sub(hidden_replies_count),
            shares_count,
            upvotes_count,
            downvotes_count,
            score,
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn get_posts_by_ids(post_ids: Vec<PostId>) -> Vec<FlatPost<T::AccountId, T::BlockNumber>> {
        let mut posts = Vec::new();
        for post_id in post_ids.iter() {
            if let Some(post) = Self::require_post(*post_id).ok() {
                posts.push(post.into());
            }
        }
        posts
    }

    fn get_posts_slice_by_space_id<F: FnMut(&Post<T>) -> bool>(
        space_id: SpaceId,
        offset: u64,
        limit: u16,
        compare_fn: F,
    ) -> Vec<FlatPost<T::AccountId, T::BlockNumber>> {
        let post_ids: Vec<PostId> = Self::post_ids_by_space_id(space_id);

        Self::get_posts_slice(post_ids, offset, limit, compare_fn)
    }

    fn get_posts_slice<F: FnMut(&Post<T>) -> bool>(
        post_ids: Vec<PostId>,
        offset: u64,
        limit: u16,
        mut compare_fn: F,
    ) -> Vec<FlatPost<T::AccountId, T::BlockNumber>> {
        let mut posts = Vec::new();

        let mut start_from = offset;
        let mut iterate_until = offset;
        let last_post_id = post_ids.len().saturating_sub(1) as u64;

        'outer: loop {
            iterate_until = iterate_until.saturating_add(limit.into());

            if start_from > last_post_id { break; }
            if iterate_until > last_post_id {
                iterate_until = last_post_id;
            }

            for post_id in start_from..=iterate_until {
                if let Some(post) = Self::require_post(post_id).ok() {
                    if compare_fn(&post) {
                        posts.push(post.into());
                        if posts.len() >= limit as usize { break 'outer; }
                    }
                }
            }
            start_from = iterate_until;
        }

        posts
    }

    pub fn get_public_posts_by_space(
        space_id: SpaceId,
        offset: u64,
        limit: u16,
    ) -> Vec<FlatPost<T::AccountId, T::BlockNumber>> {
        let public_space = Spaces::<T>::require_space(space_id).ok().filter(|space| space.is_public());
        if public_space.is_some() {
            return Self::get_posts_slice_by_space_id(space_id, offset, limit, |post| post.is_public());
        }

        vec![]
    }

    pub fn get_unlisted_posts_by_space(
        space_id: SpaceId,
        offset: u64,
        limit: u16,
    ) -> Vec<FlatPost<T::AccountId, T::BlockNumber>> {
        let unlisted_space = Spaces::<T>::require_space(space_id).ok().filter(|space| !space.is_public());
        if unlisted_space.is_some() {
            return Self::get_posts_slice_by_space_id(space_id, offset, limit, |post| !post.is_public());
        }

        vec![]
    }

    pub fn get_reply_ids_by_post_id(post_id: PostId) -> Vec<PostId> {
        Self::try_get_post_replies_ids(post_id)
    }

    // TODO: replace with get_comment_tree
    //  - Additionally check comments depth
    /*pub fn get_post_replies(post_id: PostId) -> Vec<FlatPost<T::AccountId, T::BlockNumber>> {
        let replies_ids = Self::try_get_post_replies_ids(post_id);
        Self::get_posts_by_ids(replies_ids)
    }*/

    pub fn get_comment_ids_tree(post_id: PostId) -> BTreeMap<PostId, Vec<PostId>> {
        BTreeMap::from_iter(Self::get_post_reply_ids_tree(post_id))
    }

    fn get_post_ids_by_space<F: FnMut(&Post<T>) -> bool>(space_id: SpaceId, mut filter: F) -> Vec<PostId> {
        Self::post_ids_by_space_id(space_id)
            .iter()
            .filter_map(|id| Self::post_by_id(id))
            .filter(|post| filter(post))
            .map(|post| post.id)
            .collect()
    }

    pub fn get_public_post_ids_by_space(space_id: SpaceId) -> Vec<PostId> {
        let public_space = Spaces::<T>::require_space(space_id).ok().filter(|space| space.is_public());
        if public_space.is_some() {
            return Self::get_post_ids_by_space(space_id, |post| post.is_public());
        }

        vec![]
    }

    pub fn get_unlisted_post_ids_by_space(space_id: SpaceId) -> Vec<PostId> {
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
            .flat_map(|space_id| Self::post_ids_by_space_id(space_id))
            .collect();

        post_ids.sort_by(|a, b| b.cmp(a));

        Self::get_posts_slice(post_ids, offset, limit, |post| post.is_public() && !post.is_comment())
    }
}
