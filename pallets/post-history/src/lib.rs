#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use scale_info::TypeInfo;
use frame_support::{decl_module, decl_storage};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::Vec;
use frame_system::{self as system};

use pallet_posts::{Post, PostUpdate, AfterPostUpdated};
use pallet_utils::{WhoAndWhen, PostId};

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct PostHistoryRecord<T: Config> {
    pub edited: WhoAndWhen<T>,
    pub old_data: PostUpdate,
}

/// The pallet's configuration trait.
pub trait Config: system::Config
    + pallet_utils::Config
    + pallet_posts::Config
{}

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Config> as PostHistoryModule {
        pub EditHistory get(fn edit_history):
            map hasher(twox_64_concat) PostId => Vec<PostHistoryRecord<T>>;
    }
}

decl_module! {
  pub struct Module<T: Config> for enum Call where origin: T::Origin {}
}

impl<T: Config> PostHistoryRecord<T> {
    fn new(updated_by: T::AccountId, old_data: PostUpdate) -> Self {
        PostHistoryRecord {
            edited: WhoAndWhen::<T>::new(updated_by),
            old_data
        }
    }
}

impl<T: Config> AfterPostUpdated<T> for Module<T> {
    fn after_post_updated(sender: T::AccountId, post: &Post<T>, old_data: PostUpdate) {
        <EditHistory<T>>::mutate(post.id, |ids|
            ids.push(PostHistoryRecord::<T>::new(sender, old_data)));
    }
}
