#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, ensure,
    dispatch::DispatchResult,
    traits::Get
};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;
use frame_system::{self as system, ensure_signed};

use pallet_utils::{Module as Utils, WhoAndWhen, Content};

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct SocialAccount<T: Trait> {
    pub followers_count: u32,
    pub following_accounts_count: u16,
    pub following_spaces_count: u16,
    pub reputation: u32,
    pub profile: Option<Profile<T>>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Profile<T: Trait> {
    pub created: WhoAndWhen<T>,
    pub updated: Option<WhoAndWhen<T>>,
    pub content: Content
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct ProfileUpdate {
    pub content: Option<Content>,
}

/// The pallet's configuration trait.
pub trait Trait: system::Trait
    + pallet_utils::Trait
{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    type AfterProfileUpdated: AfterProfileUpdated<Self>;
}

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as ProfilesModule {
        pub SocialAccountById get(fn social_account_by_id):
            map hasher(blake2_128_concat) T::AccountId => Option<SocialAccount<T>>;
    }
}

decl_event!(
    pub enum Event<T> where
        <T as system::Trait>::AccountId,
    {
        ProfileCreated(AccountId),
        ProfileUpdated(AccountId),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
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
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {

    // Initializing errors
    type Error = Error<T>;

    // Initializing events
    fn deposit_event() = default;

    #[weight = 100_000 + T::DbWeight::get().reads_writes(1, 2)]
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

    #[weight = 100_000 + T::DbWeight::get().reads_writes(1, 2)]
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

impl <T: Trait> SocialAccount<T> {
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

impl<T: Trait> SocialAccount<T> {
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

impl<T: Trait> Module<T> {
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
pub trait AfterProfileUpdated<T: Trait> {
    fn after_profile_updated(account: T::AccountId, post: &Profile<T>, old_data: ProfileUpdate);
}
