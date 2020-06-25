#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::string_lit_as_bytes)]

use codec::{Decode, Encode};
use frame_support::{
    decl_module, decl_storage
};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::Vec;

use pallet_utils::{SpaceId, WhoAndWhen};
use pallet_spaces::{Space, SpaceUpdate, AfterSpaceUpdated};

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct SpaceHistoryRecord<T: Trait> {
    pub edited: WhoAndWhen<T>,
    pub old_data: SpaceUpdate,
}

/// The pallet's configuration trait.
pub trait Trait: system::Trait
    + pallet_spaces::Trait
    + pallet_utils::Trait
{}

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as SpaceHistoryModule {
        pub SpaceHistoryBySpaceId get(fn space_history_by_space_id): map SpaceId => Vec<SpaceHistoryRecord<T>>;
    }
}

// The pallet's dispatchable functions.
decl_module! {
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {}
}

impl<T: Trait> SpaceHistoryRecord<T> {
    fn new(updated_by: T::AccountId) -> Self {
        SpaceHistoryRecord {
            edited: WhoAndWhen::<T>::new(updated_by),
            old_data: SpaceUpdate::default()
        }
    }
}

impl<T: Trait> AfterSpaceUpdated<T> for Module<T> {
    fn after_space_updated(sender: T::AccountId, space: &Space<T>, old_data: SpaceUpdate) {
        let mut new_history_record = SpaceHistoryRecord::<T>::new(sender);
        new_history_record.old_data = old_data;

        <SpaceHistoryBySpaceId<T>>::mutate(space.id, |ids| ids.push(new_history_record));
    }
}
