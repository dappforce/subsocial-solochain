#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{decl_module, decl_storage};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::Vec;
use frame_system::{self as system};

use pallet_utils::{SpaceId, WhoAndWhen};
use pallet_spaces::{Space, SpaceUpdate, AfterSpaceUpdated};

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct SpaceHistoryRecord<T: Config> {
    pub edited: WhoAndWhen<T>,
    pub old_data: SpaceUpdate,
}

/// The pallet's configuration trait.
pub trait Config: system::Config
    + pallet_spaces::Config
    + pallet_utils::Config
{}

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Config> as SpaceHistoryModule {
        pub EditHistory get(fn edit_history):
            map hasher(twox_64_concat) SpaceId => Vec<SpaceHistoryRecord<T>>;
    }
}

// The pallet's dispatchable functions.
decl_module! {
  pub struct Module<T: Config> for enum Call where origin: T::Origin {}
}

impl<T: Config> SpaceHistoryRecord<T> {
    fn new(updated_by: T::AccountId, old_data: SpaceUpdate) -> Self {
        SpaceHistoryRecord {
            edited: WhoAndWhen::<T>::new(updated_by),
            old_data
        }
    }
}

impl<T: Config> AfterSpaceUpdated<T> for Module<T> {
    fn after_space_updated(sender: T::AccountId, space: &Space<T>, old_data: SpaceUpdate) {
        <EditHistory<T>>::mutate(space.id, |ids|
            ids.push(SpaceHistoryRecord::<T>::new(sender, old_data)));
    }
}
