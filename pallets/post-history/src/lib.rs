#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::string_lit_as_bytes)]

use codec::{Decode, Encode};
use frame_support::{decl_module, decl_storage};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::Vec;

use pallet_posts::{PostId, Post, PostUpdate, AfterPostUpdated};
use pallet_utils::WhoAndWhen;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct PostHistoryRecord<T: Trait> {
    pub edited: WhoAndWhen<T>,
    pub old_data: PostUpdate,
}

/// The pallet's configuration trait.
pub trait Trait: system::Trait
    + pallet_utils::Trait
    + pallet_posts::Trait
{}

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as PostHistoryModule {
        pub PostHistoryByPostId get(fn post_history_by_post_id): map PostId => Vec<PostHistoryRecord<T>>;
    }
}

decl_module! {
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {}
}

impl<T: Trait> PostHistoryRecord<T> {
    fn new(updated_by: T::AccountId) -> Self {
        PostHistoryRecord {
            edited: WhoAndWhen::<T>::new(updated_by),
            old_data: PostUpdate::default()
        }
    }
}

impl<T: Trait> AfterPostUpdated<T> for Module<T> {
    fn after_post_updated(sender: T::AccountId, post: &Post<T>, old_data: PostUpdate) {
        let mut new_history_record = PostHistoryRecord::<T>::new(sender);
        new_history_record.old_data = old_data;

        <PostHistoryByPostId<T>>::mutate(post.id, |ids| ids.push(new_history_record));
    }
}
