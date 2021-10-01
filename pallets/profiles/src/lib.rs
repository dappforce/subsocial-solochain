#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, ensure,
    dispatch::DispatchResult,
    weights::Weight
};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;
use frame_system::{self as system, ensure_signed};

use pallet_utils::{Module as Utils, WhoAndWhen, Content};

pub mod rpc;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct SocialAccount<T: Config> {
    pub followers_count: u32,
    pub following_accounts_count: u16,
    pub following_spaces_count: u16,
    pub reputation: u32,
    pub profile: Option<Profile<T>>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Profile<T: Config> {
    pub created: WhoAndWhen<T>,
    pub updated: Option<WhoAndWhen<T>>,
    pub content: Content
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct ProfileUpdate {
    pub content: Option<Content>,
}
pub trait WeightInfo {
    fn create_profile() -> Weight;
    fn update_profile() -> Weight;
}

/// The pallet's configuration trait.
pub trait Config: system::Config
    + pallet_utils::Config
{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Config>::Event>;

    type AfterProfileUpdated: AfterProfileUpdated<Self>;

    type WeightInfo: WeightInfo;
}

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Config> as ProfilesModule {
        pub SocialAccountById get(fn social_account_by_id):
            map hasher(blake2_128_concat) T::AccountId => Option<SocialAccount<T>>;
    }
}

decl_event!(
    pub enum Event<T> where
        <T as system::Config>::AccountId,
    {
        ProfileCreated(AccountId),
        ProfileUpdated(AccountId),
    }
);

decl_error! {
    pub enum Error for Module<T: Config> {
        /// Social account was not found by id.
        SocialAccountNotFound,
        /// Profile is already created for this account.
        ProfileAlreadyCreated,
        /// Nothing to update in a profile.
        NoUpdatesForProfile,
        /// Account has no profile yet.
        AccountHasNoProfile,
    }
}

decl_module! {
  pub struct Module<T: Config> for enum Call where origin: T::Origin {

    // Initializing errors
    type Error = Error<T>;

    // Initializing events
    fn deposit_event() = default;

    #[weight = <T as Config>::WeightInfo::create_profile()]
    pub fn create_profile(origin, content: Content) -> DispatchResult {
      let owner = ensure_signed(origin)?;

      Utils::<T>::is_valid_content(content.clone())?;

      let mut social_account = Self::get_or_new_social_account(owner.clone());
      ensure!(social_account.profile.is_none(), Error::<T>::ProfileAlreadyCreated);

      social_account.profile = Some(
        Profile {
          created: WhoAndWhen::<T>::new(owner.clone()),
          updated: None,
          content
        }
      );
      <SocialAccountById<T>>::insert(owner.clone(), social_account);

      Self::deposit_event(RawEvent::ProfileCreated(owner));
      Ok(())
    }

    #[weight = <T as Config>::WeightInfo::update_profile()]
    pub fn update_profile(origin, update: ProfileUpdate) -> DispatchResult {
      let owner = ensure_signed(origin)?;

      let has_updates = update.content.is_some();

      ensure!(has_updates, Error::<T>::NoUpdatesForProfile);

      let mut social_account = Self::social_account_by_id(owner.clone()).ok_or(Error::<T>::SocialAccountNotFound)?;
      let mut profile = social_account.profile.ok_or(Error::<T>::AccountHasNoProfile)?;
      let mut is_update_applied = false;
      let mut old_data = ProfileUpdate::default();

      if let Some(content) = update.content {
        if content != profile.content {
          Utils::<T>::is_valid_content(content.clone())?;
          old_data.content = Some(profile.content);
          profile.content = content;
          is_update_applied = true;
        }
      }

      if is_update_applied {
        profile.updated = Some(WhoAndWhen::<T>::new(owner.clone()));
        social_account.profile = Some(profile.clone());

        <SocialAccountById<T>>::insert(owner.clone(), social_account);
        T::AfterProfileUpdated::after_profile_updated(owner.clone(), &profile, old_data);

        Self::deposit_event(RawEvent::ProfileUpdated(owner));
      }
      Ok(())
    }
  }
}

impl <T: Config> SocialAccount<T> {
    pub fn inc_followers(&mut self) {
        self.followers_count = self.followers_count.saturating_add(1);
    }

    pub fn dec_followers(&mut self) {
        self.followers_count = self.followers_count.saturating_sub(1);
    }

    pub fn inc_following_accounts(&mut self) {
        self.following_accounts_count = self.following_accounts_count.saturating_add(1);
    }

    pub fn dec_following_accounts(&mut self) {
        self.following_accounts_count = self.following_accounts_count.saturating_sub(1);
    }

    pub fn inc_following_spaces(&mut self) {
        self.following_spaces_count = self.following_spaces_count.saturating_add(1);
    }

    pub fn dec_following_spaces(&mut self) {
        self.following_spaces_count = self.following_spaces_count.saturating_sub(1);
    }
}

impl<T: Config> SocialAccount<T> {
    #[allow(clippy::comparison_chain)]
    pub fn change_reputation(&mut self, diff: i16) {
        if diff > 0 {
            self.reputation = self.reputation.saturating_add(diff.abs() as u32);
        } else if diff < 0 {
            self.reputation = self.reputation.saturating_sub(diff.abs() as u32);
        }
    }
}

impl Default for ProfileUpdate {
    fn default() -> Self {
        ProfileUpdate {
            content: None
        }
    }
}

impl<T: Config> Module<T> {
    pub fn get_or_new_social_account(account: T::AccountId) -> SocialAccount<T> {
        Self::social_account_by_id(account).unwrap_or(
            SocialAccount {
                followers_count: 0,
                following_accounts_count: 0,
                following_spaces_count: 0,
                reputation: 1,
                profile: None,
            }
        )
    }
}

#[impl_trait_for_tuples::impl_for_tuples(10)]
pub trait AfterProfileUpdated<T: Config> {
    fn after_profile_updated(account: T::AccountId, post: &Profile<T>, old_data: ProfileUpdate);
}
