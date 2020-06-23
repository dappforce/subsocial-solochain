#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::string_lit_as_bytes)]

use codec::{Decode, Encode};
use frame_support::{decl_module, decl_storage};
use sp_runtime::{RuntimeDebug};

use pallet_spaces::{Space};
use pallet_posts::{PostId, Post, PostExtension, AfterPostCreated, AfterCommentCreated};
// use pallet_reactions::PostReactionScores;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct PostStats {
    pub direct_replies_count: u16,
    pub total_replies_count: u32,

    pub shares_count: u16,
    pub upvotes_count: u16,
    pub downvotes_count: u16
}

/// The pallet's configuration trait.
pub trait Trait: system::Trait
    + pallet_posts::Trait
    // + pallet_reactions::Trait
{}

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as TemplateModule {
        PostStatsByPostId get(fn post_stats_by_post_id): map PostId => Option<PostStats>;
    }
}

// The pallet's dispatchable functions.
decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {}
}

impl<T: Trait> Module<T> {
    pub fn get_or_new_stats(post_id: PostId) -> PostStats {
        Module::<T>::post_stats_by_post_id(post_id).unwrap_or_default()
    }
}

impl Default for PostStats {
    fn default() -> Self {
        PostStats {
            direct_replies_count: 0,
            total_replies_count: 0,
            shares_count: 0,
            upvotes_count: 0,
            downvotes_count: 0
        }
    }
}

//noinspection RsUnresolvedReference
impl PostStats {
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
/*
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
*/
}

impl<T: Trait> AfterPostCreated<T> for Module<T> {
    fn after_post_created(post: &Post<T>, _space: &mut Space<T>) {
        if let PostExtension::SharedPost(original_post_id) = post.extension {
            let mut stats = Self::get_or_new_stats(original_post_id);
            stats.inc_shares();
            PostStatsByPostId::insert(original_post_id, stats);
        }
    }
}

impl<T: Trait> AfterCommentCreated<T> for Module<T> {
    fn after_comment_created(post: &Post<T>, ancestors: &[Post<T>]) {
        if let PostExtension::Comment(comment_ext) = post.extension {
            let mut root_post_stats = Self::get_or_new_stats(comment_ext.root_post_id);
            root_post_stats.inc_total_replies();

            if let Some(parent_id) = comment_ext.parent_id {
                let mut parent_comment_stats = Self::get_or_new_stats(parent_id);
                parent_comment_stats.inc_direct_replies();
                PostStatsByPostId::insert(parent_id, parent_comment_stats);

                for post in ancestors {
                    PostStatsByPostId::mutate(post.id, |stats_opt: &mut Option<PostStats>| {
                        let mut new_stats = stats_opt.clone().unwrap_or_default();
                        new_stats.inc_total_replies();
                        *stats_opt = Some(new_stats);
                    });
                }
            } else {
                root_post_stats.inc_direct_replies();
            }
            PostStatsByPostId::insert(comment_ext.root_post_id, root_post_stats);
        }
    }
}
