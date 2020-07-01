#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, ensure,
    dispatch::{DispatchError, DispatchResult},
    traits::Get
};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;
use frame_system::{self as system, ensure_signed};

use df_traits::{SpaceForRoles, SpaceForRolesProvider};
use df_traits::{PermissionChecker, SpaceFollowsProvider};
use pallet_permissions::{SpacePermission, SpacePermissions, SpacePermissionsContext};
use pallet_utils::{is_valid_handle_char, Module as Utils, SpaceId, WhoAndWhen, Content};

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Space<T: Trait> {
    pub id: SpaceId,
    pub created: WhoAndWhen<T>,
    pub updated: Option<WhoAndWhen<T>>,
    pub hidden: bool,

    // Can be updated by the owner:
    pub owner: T::AccountId,
    pub handle: Option<Vec<u8>>,
    pub content: Content,
    pub parent_id: Option<SpaceId>,

    pub posts_count: u16,
    pub followers_count: u32,

    pub score: i32,

    /// Allows to override the default permissions for this space.
    pub permissions: Option<SpacePermissions>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
#[allow(clippy::option_option)]
pub struct SpaceUpdate {
    pub handle: Option<Option<Vec<u8>>>,
    pub content: Option<Content>,
    pub hidden: Option<bool>,
    pub parent_id: Option<Option<SpaceId>>,
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

    type AfterSpaceUpdated: AfterSpaceUpdated<Self>;
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
    /// User has no permission to create subspaces on this space
    NoPermissionToCreateSubspaces,
  }
}

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as SpacesModule {

        // TODO reserve space id 0 (zero) for 'Abyss'.
        pub NextSpaceId get(fn next_space_id): SpaceId = 1;

        pub SpaceById get(fn space_by_id): map hasher(twox_64_concat) SpaceId => Option<Space<T>>;

        pub SpaceIdByHandle get(fn space_id_by_handle):
            map hasher(blake2_128_concat) Vec<u8> => Option<SpaceId>;

        pub SpaceIdsByOwner get(fn space_ids_by_owner):
            map hasher(twox_64_concat) T::AccountId => Vec<SpaceId>;
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

    // Initializing errors
    type Error = Error<T>;

    // Initializing events
    fn deposit_event() = default;

    #[weight = 100_000 + T::DbWeight::get().reads_writes(4, 4)]
    pub fn create_space(
      origin,
      parent_id_opt: Option<SpaceId>,
      handle_opt: Option<Vec<u8>>,
      content: Content
    ) -> DispatchResult {
      let owner = ensure_signed(origin)?;

      Utils::<T>::is_valid_content(content.clone())?;

      let mut handle_in_lowercase: Vec<u8> = Vec::new();
      if let Some(original_handle) = handle_opt.clone() {
        handle_in_lowercase = Self::lowercase_and_validate_a_handle(original_handle)?;
      }

      // TODO: add tests for this case
      if let Some(parent_id) = parent_id_opt {
        let space = Self::require_space(parent_id)?;

        Self::ensure_account_has_space_permission(
          owner.clone(),
          &space,
          SpacePermission::CreateSubspaces,
          Error::<T>::NoPermissionToCreateSubspaces.into()
        )?;
      }

      let space_id = Self::next_space_id();
      let new_space = &mut Space::new(space_id, owner.clone(), content, handle_opt, parent_id_opt);

      T::BeforeSpaceCreated::before_space_created(owner.clone(), new_space)?;

      <SpaceById<T>>::insert(space_id, new_space);
      <SpaceIdsByOwner<T>>::mutate(owner.clone(), |ids| ids.push(space_id));
      NextSpaceId::mutate(|n| { *n += 1; });

      if !handle_in_lowercase.is_empty() {
        SpaceIdByHandle::insert(handle_in_lowercase, space_id);
      }

      Self::deposit_event(RawEvent::SpaceCreated(owner, space_id));
      Ok(())
    }

    #[weight = 100_000 + T::DbWeight::get().reads_writes(2, 3)]
    pub fn update_space(origin, space_id: SpaceId, update: SpaceUpdate) -> DispatchResult {
      let owner = ensure_signed(origin)?;

      let has_updates =
        update.handle.is_some() ||
        update.content.is_some() ||
        update.hidden.is_some() ||
        update.parent_id.is_some();

      ensure!(has_updates, Error::<T>::NoUpdatesForSpace);

      let mut space = Self::require_space(space_id)?;

      Self::ensure_account_has_space_permission(
        owner.clone(),
        &space,
        SpacePermission::UpdateSpace,
        Error::<T>::NoPermissionToUpdateSpace.into()
      )?;

      let mut fields_updated = 0;
      let mut old_data = SpaceUpdate::default();

      // TODO: add tests for this case
      if let Some(parent_id_opt) = update.parent_id {
        if parent_id_opt != space.parent_id {

          if let Some(parent_id) = parent_id_opt {
            let parent_space = Self::require_space(parent_id)?;

            Self::ensure_account_has_space_permission(
              owner.clone(),
              &parent_space,
              SpacePermission::CreateSubspaces,
              Error::<T>::NoPermissionToCreateSubspaces.into()
            )?;
          }

          old_data.parent_id = Some(space.parent_id);
          space.parent_id = parent_id_opt;
          fields_updated += 1;
        }
      }

      if let Some(content) = update.content {
        if content != space.content {
          Utils::<T>::is_valid_content(content.clone())?;

          old_data.content = Some(space.content);
          space.content = content;
          fields_updated += 1;
        }
      }

      if let Some(hidden) = update.hidden {
        if hidden != space.hidden {
          old_data.hidden = Some(space.hidden);
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
          old_data.handle = Some(space.handle);
          space.handle = handle_opt;
          fields_updated += 1;
        }
      }

      // Update this space only if at least one field should be updated:
      if fields_updated > 0 {
        space.updated = Some(WhoAndWhen::<T>::new(owner.clone()));

        <SpaceById<T>>::insert(space_id, space.clone());
        T::AfterSpaceUpdated::after_space_updated(owner.clone(), &space, old_data);

        Self::deposit_event(RawEvent::SpaceUpdated(owner, space_id));
      }
      Ok(())
    }
  }
}

impl<T: Trait> Space<T> {
    pub fn new(
        id: SpaceId,
        created_by: T::AccountId,
        content: Content,
        handle: Option<Vec<u8>>,
        parent_id: Option<SpaceId>,
    ) -> Self {
        Space {
            id,
            created: WhoAndWhen::<T>::new(created_by.clone()),
            updated: None,
            hidden: false,
            owner: created_by,
            handle,
            content,
            parent_id,
            posts_count: 0,
            followers_count: 0,
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

    pub fn inc_posts(&mut self) {
        self.posts_count = self.posts_count.saturating_add(1);
    }

    pub fn dec_posts(&mut self) {
        self.posts_count = self.posts_count.saturating_sub(1);
    }

    pub fn inc_followers(&mut self) {
        self.followers_count = self.followers_count.saturating_add(1);
    }

    pub fn dec_followers(&mut self) {
        self.followers_count = self.followers_count.saturating_sub(1);
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

impl Default for SpaceUpdate {
    fn default() -> Self {
        SpaceUpdate {
            handle: None,
            content: None,
            hidden: None,
            parent_id: None
        }
    }
}

impl<T: Trait> Module<T> {

    /// Check that there is a `Space` with such `space_id` in the storage
    /// or return`SpaceNotFound` error.
    pub fn ensure_space_exists(space_id: SpaceId) -> DispatchResult {
        ensure!(<SpaceById<T>>::contains_key(space_id), Error::<T>::SpaceNotFound);
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

#[impl_trait_for_tuples::impl_for_tuples(10)]
pub trait AfterSpaceUpdated<T: Trait> {
    fn after_space_updated(sender: T::AccountId, space: &Space<T>, old_data: SpaceUpdate);
}
