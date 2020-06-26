#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::string_lit_as_bytes)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::DispatchResult, ensure, traits::Get,
};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;
use system::ensure_signed;

use pallet_utils::{is_valid_handle_char, Module as Utils, WhoAndWhen, Content};

// mod tests;

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

    pub username: Vec<u8>,
    pub content: Content
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct ProfileUpdate {
    pub username: Option<Vec<u8>>,
    pub content: Option<Content>,
}

/// The pallet's configuration trait.
pub trait Trait: system::Trait
    + pallet_utils::Trait
{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    /// Minimal length of profile username
    type MinUsernameLen: Get<u32>;

    /// Maximal length of profile username
    type MaxUsernameLen: Get<u32>;

    type AfterProfileUpdated: AfterProfileUpdated<Self>;
}

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as ProfilesModule {
        pub SocialAccountById get(fn social_account_by_id): map T::AccountId => Option<SocialAccount<T>>;
        pub AccountByProfileUsername get(fn account_by_profile_username): map Vec<u8> => Option<T::AccountId>;
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
        /// Username is taken.
        UsernameIsTaken,
        /// Username is too short.
        UsernameIsTooShort,
        /// Username is too long.
        UsernameIsTooLong,
        /// Username contains invalid chars.
        UsernameContainsInvalidChars,
    }
}

decl_module! {
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {

    // TODO replace with MaxHandleLen
    /// Minimal length of profile username
    const MinUsernameLen: u32 = T::MinUsernameLen::get();

    // TODO replace with MinHandleLen
    /// Maximal length of profile username
    const MaxUsernameLen: u32 = T::MaxUsernameLen::get();

    // Initializing events
    fn deposit_event() = default;

    pub fn create_profile(origin, username: Vec<u8>, content: Content) {
      let owner = ensure_signed(origin)?;

      let mut social_account = Self::get_or_new_social_account(owner.clone());
      ensure!(social_account.profile.is_none(), Error::<T>::ProfileAlreadyCreated);
      Self::is_username_valid(username.clone())?;
      Utils::<T>::is_valid_content(content.clone())?;

      social_account.profile = Some(
        Profile {
          created: WhoAndWhen::<T>::new(owner.clone()),
          updated: None,
          username: username.clone(),
          content
        }
      );
      <AccountByProfileUsername<T>>::insert(username, owner.clone());
      <SocialAccountById<T>>::insert(owner.clone(), social_account);

      Self::deposit_event(RawEvent::ProfileCreated(owner));
    }

    pub fn update_profile(origin, update: ProfileUpdate) {
      let owner = ensure_signed(origin)?;

      let has_updates =
        update.username.is_some() ||
        update.content.is_some();

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

      if let Some(username) = update.username {
        if username != profile.username {
          Self::is_username_valid(username.clone())?;
          <AccountByProfileUsername<T>>::remove(profile.username.clone());
          <AccountByProfileUsername<T>>::insert(username.clone(), owner.clone());
          old_data.username = Some(profile.username);
          profile.username = username;
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
            username: None,
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

    // TODO replace with code from ::lowercase_and_validate_a_handle()
    pub fn is_username_valid(username: Vec<u8>) -> DispatchResult {
        ensure!(Self::account_by_profile_username(username.clone()).is_none(), Error::<T>::UsernameIsTaken);
        ensure!(username.len() >= T::MinUsernameLen::get() as usize, Error::<T>::UsernameIsTooShort);
        ensure!(username.len() <= T::MaxUsernameLen::get() as usize, Error::<T>::UsernameIsTooLong);
        ensure!(username.iter().all(|&x| is_valid_handle_char(x)), Error::<T>::UsernameContainsInvalidChars);
        Ok(())
    }
}

#[impl_trait_for_tuples::impl_for_tuples(10)]
pub trait AfterProfileUpdated<T: Trait> {
    fn after_profile_updated(account: T::AccountId, post: &Profile<T>, old_data: ProfileUpdate);
}
