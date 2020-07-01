#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{decl_module, decl_storage};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::Vec;
use frame_system::{self as system};

use pallet_utils::WhoAndWhen;
use pallet_profiles::{Profile, ProfileUpdate, AfterProfileUpdated};

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct ProfileHistoryRecord<T: Trait> {
    pub edited: WhoAndWhen<T>,
    pub old_data: ProfileUpdate,
}

/// The pallet's configuration trait.
pub trait Trait: system::Trait
    + pallet_utils::Trait
    + pallet_profiles::Trait
{}

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as ProfileHistoryModule {
        pub EditHistory get(fn edit_history): map hasher(blake2_128_concat) T::AccountId => Vec<ProfileHistoryRecord<T>>;
    }
}

decl_module! {
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {}
}

impl<T: Trait> ProfileHistoryRecord<T> {
    fn new(updated_by: T::AccountId) -> Self {
        ProfileHistoryRecord {
            edited: WhoAndWhen::<T>::new(updated_by),
            old_data: ProfileUpdate::default()
        }
    }
}

impl<T: Trait> AfterProfileUpdated<T> for Module<T> {
    fn after_profile_updated(sender: T::AccountId, _profile: &Profile<T>, old_data: ProfileUpdate) {
        let mut new_history_record = ProfileHistoryRecord::<T>::new(sender.clone());
        new_history_record.old_data = old_data;

        <EditHistory<T>>::mutate(sender, |ids| ids.push(new_history_record));
    }
}
