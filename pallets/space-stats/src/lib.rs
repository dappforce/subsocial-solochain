#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::string_lit_as_bytes)]

use codec::{Decode, Encode};
use frame_support::{decl_module, decl_storage};
use sp_runtime::RuntimeDebug;

use pallet_spaces::{Space};
use pallet_space_follows::{BeforeSpaceFollowed, BeforeSpaceUnfollowed};
use pallet_posts::{Post, AfterPostCreated};

use pallet_utils::SpaceId;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct SpaceStats {
    pub posts_count: u16,
    pub followers_count: u32
}

/// The pallet's configuration trait.
pub trait Trait: system::Trait
    + pallet_utils::Trait
    + pallet_space_follows::Trait
    + pallet_posts::Trait
{}

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as TemplateModule {
        pub SpaceStatsBySpaceId get(fn space_stats_by_space_id): map SpaceId => Option<SpaceStats>;
    }
}

// The pallet's dispatchable functions.
decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {}
}

impl<T: Trait> Module<T> {
    pub fn get_or_new_stats(space_id: SpaceId) -> SpaceStats {
        if let Some(stats) = Module::<T>::space_stats_by_space_id(space_id) {
            return stats;
        }
        SpaceStats::default()
    }

    pub fn update_followers_count(space_id: SpaceId, increment: bool) {
        let mut space_stats = Module::<T>::get_or_new_stats(space_id);

        if increment {
            space_stats.inc_followers();
        } else {
            space_stats.dec_followers();
        }

        SpaceStatsBySpaceId::insert(space_id, space_stats);
    }
}

impl Default for SpaceStats {
    fn default() -> Self {
        SpaceStats {
            followers_count: 0,
            posts_count: 0
        }
    }
}

impl SpaceStats {
    pub fn inc_followers(&mut self) {
        self.followers_count = self.followers_count.saturating_add(1);
    }

    pub fn dec_followers(&mut self) {
        self.followers_count = self.followers_count.saturating_sub(1);
    }

    pub fn inc_posts(&mut self) {
        self.posts_count = self.posts_count.saturating_add(1);
    }

    pub fn dec_posts(&mut self) {
        self.posts_count = self.posts_count.saturating_sub(1);
    }
}

impl<T: Trait> BeforeSpaceFollowed<T> for Module<T> {
    fn before_space_followed(_follower: T::AccountId, _follower_reputation: u32, space: &mut Space<T>) {
        Module::<T>::update_followers_count(space.id, true);
    }
}

impl<T: Trait> BeforeSpaceUnfollowed<T> for Module<T> {
    fn before_space_unfollowed(_follower: T::AccountId, space: &mut Space<T>) {
        Module::<T>::update_followers_count(space.id, false);
    }
}

impl<T: Trait> AfterPostCreated<T> for Module<T> {
    fn after_post_created(post: &Post<T>, space: &mut Space<T>) {
        if !post.is_comment() {
            let mut space_stats = Module::<T>::get_or_new_stats(space.id);

            space_stats.inc_posts();

            SpaceStatsBySpaceId::insert(space.id, space_stats);
        }
    }
}
