#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::string_lit_as_bytes)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult}, ensure, traits::Get,
};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;
use system::ensure_signed;

use df_traits::{SpaceForRoles, SpaceForRolesProvider};
use df_traits::{PermissionChecker, SpaceFollowsProvider};
use pallet_permissions::{SpacePermission, SpacePermissions, SpacePermissionsContext};
use pallet_utils::{is_valid_handle_char, Module as Utils, SpaceId, WhoAndWhen};

// #[cfg(tests)]
// mod tests;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Space<T: Trait> {
    pub id: SpaceId,
    pub created: WhoAndWhen<T>,
    pub updated: Option<WhoAndWhen<T>>,
    pub hidden: bool,

    // Can be updated by the owner:
    pub owner: T::AccountId,
    pub handle: Option<Vec<u8>>,
    pub ipfs_hash: Vec<u8>,

    pub edit_history: Vec<SpaceHistoryRecord<T>>,

    pub score: i32,

    /// Allows to override the default permissions for this space.
    pub permissions: Option<SpacePermissions>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
#[allow(clippy::option_option)]
pub struct SpaceUpdate {
    pub handle: Option<Option<Vec<u8>>>,
    pub ipfs_hash: Option<Vec<u8>>,
    pub hidden: Option<bool>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct SpaceHistoryRecord<T: Trait> {
    pub edited: WhoAndWhen<T>,
    pub old_data: SpaceUpdate,
}

/// The pallet's configuration trait.
pub trait Trait: system::Trait
    + pallet_utils::Trait
{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    /// Minimal length of blog handle
    type MinHandleLen: Get<u32>;

    /// Maximal length of space handle
    type MaxHandleLen: Get<u32>;

    type Roles: PermissionChecker<AccountId=Self::AccountId>;

    type SpaceFollows: SpaceFollowsProvider<AccountId=Self::AccountId>;

    type BeforeSpaceCreated: BeforeSpaceCreated<Self>;
}

decl_error! {
  pub enum Error for Module<T: Trait> {
    /// Space was not found by id.
    SpaceNotFound,
    /// Space handle is too short.
    HandleIsTooShort,
    /// Space handle is too long.
    HandleIsTooLong,
    /// Space handle is not unique.
    HandleIsNotUnique,
    /// Space handle contains invalid characters.
    HandleContainsInvalidChars,
    /// Nothing to update in space.
    NoUpdatesForSpace,
    /// Only space owner can manage their space.
    NotASpaceOwner,
    /// User has no permission to update this space.
    NoPermissionToUpdateSpace,
  }
}

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as TemplateModule {

        // TODO reserve space id 0 (zero) for 'Abyss'.

        pub NextSpaceId get(fn next_space_id): SpaceId = 1;
        pub SpaceById get(fn space_by_id): map SpaceId => Option<Space<T>>;
        pub SpaceIdByHandle get(fn space_id_by_handle): map Vec<u8> => Option<SpaceId>;
        pub SpaceIdsByOwner get(fn space_ids_by_owner): map T::AccountId => Vec<SpaceId>;
    }
}

decl_event!(
    pub enum Event<T> where
        <T as system::Trait>::AccountId,
    {
        SpaceCreated(AccountId, SpaceId),
        SpaceUpdated(AccountId, SpaceId),
        SpaceDeleted(AccountId, SpaceId),
    }
);

// The pallet's dispatchable functions.
decl_module! {
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {

    /// Minimal length of space handle
    const MinHandleLen: u32 = T::MinHandleLen::get();

    /// Maximal length of space handle
    const MaxHandleLen: u32 = T::MaxHandleLen::get();

    // Initializing events
    fn deposit_event() = default;

    pub fn create_space(origin, handle_opt: Option<Vec<u8>>, ipfs_hash: Vec<u8>) {
      let owner = ensure_signed(origin)?;

      Utils::<T>::is_ipfs_hash_valid(ipfs_hash.clone())?;

      let mut handle_in_lowercase: Vec<u8> = Vec::new();
      if let Some(original_handle) = handle_opt.clone() {
        handle_in_lowercase = Self::lowercase_and_validate_a_handle(original_handle)?;
      }

      let space_id = Self::next_space_id();
      let new_space = &mut Space::new(space_id, owner.clone(), ipfs_hash, handle_opt);

      T::BeforeSpaceCreated::before_space_created(owner.clone(), new_space)?;

      <SpaceById<T>>::insert(space_id, new_space);
      <SpaceIdsByOwner<T>>::mutate(owner.clone(), |ids| ids.push(space_id));
      NextSpaceId::mutate(|n| { *n += 1; });

      if !handle_in_lowercase.is_empty() {
        SpaceIdByHandle::insert(handle_in_lowercase, space_id);
      }

      Self::deposit_event(RawEvent::SpaceCreated(owner, space_id));
    }

    pub fn update_space(origin, space_id: SpaceId, update: SpaceUpdate) {
      let owner = ensure_signed(origin)?;

      let has_updates =
        update.handle.is_some() ||
        update.ipfs_hash.is_some() ||
        update.hidden.is_some();

      ensure!(has_updates, Error::<T>::NoUpdatesForSpace);

      let mut space = Self::require_space(space_id)?;

      Self::ensure_account_has_space_permission(
        owner.clone(),
        &space,
        SpacePermission::UpdateSpace,
        Error::<T>::NoPermissionToUpdateSpace.into()
      )?;

      let mut fields_updated = 0;
      let mut new_history_record = SpaceHistoryRecord {
        edited: WhoAndWhen::<T>::new(owner.clone()),
        old_data: SpaceUpdate {
            handle: None,
            ipfs_hash: None,
            hidden: None
        }
      };

      if let Some(ipfs_hash) = update.ipfs_hash {
        if ipfs_hash != space.ipfs_hash {
          Utils::<T>::is_ipfs_hash_valid(ipfs_hash.clone())?;
          new_history_record.old_data.ipfs_hash = Some(space.ipfs_hash);
          space.ipfs_hash = ipfs_hash;
          fields_updated += 1;
        }
      }

      if let Some(hidden) = update.hidden {
        if hidden != space.hidden {
          new_history_record.old_data.hidden = Some(space.hidden);
          space.hidden = hidden;
          fields_updated += 1;
        }
      }

      if let Some(handle_opt) = update.handle {
        if handle_opt != space.handle {
          if let Some(new_handle) = handle_opt.clone() {
            let handle_in_lowercase = Self::lowercase_and_validate_a_handle(new_handle)?;
            SpaceIdByHandle::insert(handle_in_lowercase, space_id);
          }
          if let Some(old_handle) = space.handle.clone() {
            SpaceIdByHandle::remove(old_handle);
          }
          new_history_record.old_data.handle = Some(space.handle);
          space.handle = handle_opt;
          fields_updated += 1;
        }
      }

      // Update this space only if at least one field should be updated:
      if fields_updated > 0 {
        space.updated = Some(WhoAndWhen::<T>::new(owner.clone()));
        space.edit_history.push(new_history_record);
        <SpaceById<T>>::insert(space_id, space);
        Self::deposit_event(RawEvent::SpaceUpdated(owner, space_id));
      }
    }
  }
}

impl<T: Trait> Space<T> {
    pub fn new(
        id: SpaceId,
        created_by: T::AccountId,
        ipfs_hash: Vec<u8>,
        handle: Option<Vec<u8>>,
    ) -> Self {
        Space {
            id,
            created: WhoAndWhen::<T>::new(created_by.clone()),
            updated: None,
            hidden: false,
            owner: created_by,
            handle,
            ipfs_hash,
            edit_history: Vec::new(),
            score: 0,
            permissions: None,
        }
    }

    pub fn is_owner(&self, account: &T::AccountId) -> bool {
        self.owner == *account
    }

    pub fn is_follower(&self, account: &T::AccountId) -> bool {
        T::SpaceFollows::is_space_follower(account.clone(), self.id)
    }

    pub fn ensure_space_owner(&self, account: T::AccountId) -> DispatchResult {
        ensure!(self.is_owner(&account), Error::<T>::NotASpaceOwner);
        Ok(())
    }

    #[allow(clippy::comparison_chain)]
    pub fn change_score(&mut self, diff: i16) {
        if diff > 0 {
            self.score = self.score.saturating_add(diff.abs() as i32);
        } else if diff < 0 {
            self.score = self.score.saturating_sub(diff.abs() as i32);
        }
    }
}

impl<T: Trait> Module<T> {

    /// Check that there is a `Space` with such `space_id` in the storage
    /// or return`SpaceNotFound` error.
    pub fn ensure_space_exists(space_id: SpaceId) -> DispatchResult {
        ensure!(<SpaceById<T>>::exists(space_id), Error::<T>::SpaceNotFound);
        Ok(())
    }

    /// Get `Space` by id from the storage or return `SpaceNotFound` error.
    pub fn require_space(space_id: SpaceId) -> Result<Space<T>, DispatchError> {
        Ok(Self::space_by_id(space_id).ok_or(Error::<T>::SpaceNotFound)?)
    }

    /// Check if a handle length fits into min/max values.
    /// Lowercase a provided handle.
    /// Check if a handle consists of valid chars: 0-9, a-z, _.
    /// Check if a handle is unique across all spaces' handles (required one a storage read).
    pub fn lowercase_and_validate_a_handle(handle: Vec<u8>) -> Result<Vec<u8>, DispatchError> {
        // Check min and max lengths of a handle:
        ensure!(handle.len() >= T::MinHandleLen::get() as usize, Error::<T>::HandleIsTooShort);
        ensure!(handle.len() <= T::MaxHandleLen::get() as usize, Error::<T>::HandleIsTooLong);

        let handle_in_lowercase = handle.to_ascii_lowercase();

        // Check if a handle consists of valid chars: 0-9, a-z, _.
        ensure!(handle_in_lowercase.iter().all(|&x| is_valid_handle_char(x)), Error::<T>::HandleContainsInvalidChars);

        // Check if a handle is unique across all spaces' handles:
        ensure!(Self::space_id_by_handle(handle_in_lowercase.clone()).is_none(), Error::<T>::HandleIsNotUnique);

        Ok(handle_in_lowercase)
    }

    pub fn ensure_account_has_space_permission(
        account: T::AccountId,
        space: &Space<T>,
        permission: SpacePermission,
        error: DispatchError,
    ) -> DispatchResult {
        let is_owner = space.is_owner(&account);
        let is_follower = space.is_follower(&account);

        let ctx = SpacePermissionsContext {
            space_id: space.id,
            is_space_owner: is_owner,
            is_space_follower: is_follower,
            space_perms: space.permissions.clone(),
        };

        T::Roles::ensure_account_has_space_permission(
            account,
            ctx,
            permission,
            error,
        )
    }
}

impl<T: Trait> SpaceForRolesProvider for Module<T> {
    type AccountId = T::AccountId;

    fn get_space(id: SpaceId) -> Result<SpaceForRoles<Self::AccountId>, DispatchError> {
        let space = Module::<T>::require_space(id)?;

        Ok(SpaceForRoles {
            owner: space.owner,
            permissions: space.permissions,
        })
    }
}

pub trait BeforeSpaceCreated<T: Trait> {
    fn before_space_created(follower: T::AccountId, space: &mut Space<T>) -> DispatchResult;
}

impl<T: Trait> BeforeSpaceCreated<T> for () {
    fn before_space_created(_follower: T::AccountId, _space: &mut Space<T>) -> DispatchResult {
        Ok(())
    }
}
