#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{decl_module, decl_storage};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::Vec;
use frame_system::{self as system};

use pallet_utils::WhoAndWhen;
use pallet_profiles::{Profile, ProfileUpdate, AfterProfileUpdated};

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct ProfileHistoryRecord<T: Config> {
    pub edited: WhoAndWhen<T>,
    pub old_data: ProfileUpdate,
}

/// The pallet's configuration trait.
pub trait Config: system::Config
    + pallet_utils::Config
    + pallet_profiles::Config
{}

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Config> as ProfileHistoryModule {
        pub EditHistory get(fn edit_history):
            map hasher(blake2_128_concat) T::AccountId => Vec<ProfileHistoryRecord<T>>;
    }
}

decl_module! {
  pub struct Module<T: Config> for enum Call where origin: T::Origin {}
}

impl<T: Config> ProfileHistoryRecord<T> {
    fn new(updated_by: T::AccountId, old_data: ProfileUpdate) -> Self {
        ProfileHistoryRecord {
            edited: WhoAndWhen::<T>::new(updated_by),
            old_data
        }
    }
}

impl<T: Config> AfterProfileUpdated<T> for Module<T> {
    fn after_profile_updated(sender: T::AccountId, _profile: &Profile<T>, old_data: ProfileUpdate) {
        <EditHistory<T>>::mutate(sender.clone(), |ids|
            ids.push(ProfileHistoryRecord::<T>::new(sender, old_data)));
    }
}
