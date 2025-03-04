//! # Spaces Module
//!
//! Spaces are the primary components of Subsocial. This module allows you to create a Space
//! and customize it by updating its' owner(s), content, unique handle, and permissions.
//!
//! To understand how Spaces fit into the Subsocial ecosystem, you can think of how
//! folders and files work in a file system. Spaces are similar to folders, that can contain Posts,
//! in this sense. The permissions of the Space and Posts can be customized so that a Space
//! could be as simple as a personal blog (think of a page on Facebook) or as complex as community
//! (think of a subreddit) governed DAO.
//!
//! Spaces can be compared to existing entities on web 2.0 platforms such as:
//!
//! - Blogs on Blogger,
//! - Publications on Medium,
//! - Groups or pages on Facebook,
//! - Accounts on Twitter and Instagram,
//! - Channels on YouTube,
//! - Servers on Discord,
//! - Forums on Discourse.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use scale_info::TypeInfo;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, ensure,
    dispatch::{DispatchError, DispatchResult, DispatchResultWithPostInfo},
    traits::{Get, Currency, ExistenceRequirement, ReservableCurrency},
    weights::Pays,
};
use sp_runtime::{RuntimeDebug, traits::Zero};
use sp_std::prelude::*;
use frame_system::{self as system, ensure_signed, ensure_root};

use df_traits::{
    SpaceForRoles, SpaceForRolesProvider, PermissionChecker, SpaceFollowsProvider,
    moderation::{IsAccountBlocked, IsContentBlocked},
};
use pallet_permissions::{Module as Permissions, SpacePermission, SpacePermissions, SpacePermissionsContext};
use pallet_utils::{Module as Utils, Error as UtilsError, SpaceId, WhoAndWhen, Content};

pub mod rpc;
pub mod migrations;

/// Information about a space's owner, its' content, visibility and custom permissions.
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct Space<T: Config> {

    /// Unique sequential identifier of a space. Examples of space ids: `1`, `2`, `3`, and so on.
    pub id: SpaceId,

    pub created: WhoAndWhen<T>,
    pub updated: Option<WhoAndWhen<T>>,

    /// The current owner of a given space.
    pub owner: T::AccountId,

    // The next fields can be updated by the owner:

    pub parent_id: Option<SpaceId>,

    /// Unique alpha-numeric identifier that can be used in a space's URL.
    /// Handle can only contain numbers, letter and underscore: `0`-`9`, `a`-`z`, `_`.
    pub handle: Option<Vec<u8>>,

    pub content: Content,

    /// Hidden field is used to recommend to end clients (web and mobile apps) that a particular
    /// space and its' posts should not be shown.
    pub hidden: bool,

    /// The total number of posts in a given space.
    pub posts_count: u32,

    /// The number of hidden posts in a given space.
    pub hidden_posts_count: u32,

    /// The number of account following a given space.
    pub followers_count: u32,

    pub score: i32,

    /// This allows you to override Subsocial's default permissions by enabling or disabling role
    /// permissions.
    pub permissions: Option<SpacePermissions>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, Default, RuntimeDebug, TypeInfo)]
#[allow(clippy::option_option)]
pub struct SpaceUpdate {
    pub parent_id: Option<Option<SpaceId>>,
    pub handle: Option<Option<Vec<u8>>>,
    pub content: Option<Content>,
    pub hidden: Option<bool>,
    pub permissions: Option<Option<SpacePermissions>>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct SpacesSettings {
    pub handles_enabled: bool
}

impl Default for SpacesSettings {
    fn default() -> Self {
        Self {
            handles_enabled: true,
        }
    }
}

type BalanceOf<T> =
  <<T as Config>::Currency as Currency<<T as system::Config>::AccountId>>::Balance;

/// The pallet's configuration trait.
pub trait Config: system::Config
    + pallet_utils::Config
    + pallet_permissions::Config
{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Config>::Event>;

    type Currency: ReservableCurrency<Self::AccountId>;

    type Roles: PermissionChecker<AccountId=Self::AccountId>;

    type SpaceFollows: SpaceFollowsProvider<AccountId=Self::AccountId>;

    type BeforeSpaceCreated: BeforeSpaceCreated<Self>;

    type AfterSpaceUpdated: AfterSpaceUpdated<Self>;

    type IsAccountBlocked: IsAccountBlocked<Self::AccountId>;

    type IsContentBlocked: IsContentBlocked;

    type HandleDeposit: Get<BalanceOf<Self>>;
}

decl_error! {
  pub enum Error for Module<T: Config> {
    /// Space was not found by id.
    SpaceNotFound,
    /// Space handle is not unique.
    SpaceHandleIsNotUnique,
    /// Handles are disabled in `PalletSettings`.
    HandlesAreDisabled,
    /// Nothing to update in this space.
    NoUpdatesForSpace,
    /// Only space owners can manage this space.
    NotASpaceOwner,
    /// User has no permission to update this space.
    NoPermissionToUpdateSpace,
    /// User has no permission to create subspaces within this space.
    NoPermissionToCreateSubspaces,
    /// Space is at root level, no `parent_id` specified.
    SpaceIsAtRoot,
    /// New spaces' settings don't differ from the old ones.
    NoUpdatesForSpacesSettings,
  }
}

pub const FIRST_SPACE_ID: u64 = 1;
pub const RESERVED_SPACE_COUNT: u64 = 1000;

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Config> as SpacesModule {

        /// The next space id.
        pub NextSpaceId get(fn next_space_id): SpaceId = RESERVED_SPACE_COUNT + 1;

        /// Get the details of a space by its' id.
        pub SpaceById get(fn space_by_id) build(|config: &GenesisConfig<T>| {
          let mut spaces: Vec<(SpaceId, Space<T>)> = Vec::new();
          let endowed_account = config.endowed_account.clone();
          for id in FIRST_SPACE_ID..=RESERVED_SPACE_COUNT {
            spaces.push((id, Space::<T>::new(id, None, endowed_account.clone(), Content::None, None, None)));
          }
          spaces
        }):
            map hasher(twox_64_concat) SpaceId => Option<Space<T>>;

        /// Find a given space id by its' unique handle.
        /// If a handle is not registered, nothing will be returned (`None`).
        pub SpaceIdByHandle get(fn space_id_by_handle):
            map hasher(blake2_128_concat) Vec<u8> => Option<SpaceId>;

        /// Find the ids of all spaces owned, by a given account.
        pub SpaceIdsByOwner get(fn space_ids_by_owner):
            map hasher(twox_64_concat) T::AccountId => Vec<SpaceId>;

        pub PalletSettings get(fn settings): SpacesSettings;

        /// True if `SpaceIdByHandle` storage is already fixed.
        // TODO delete this storage and corresponding migration, after the migration executed and the storage value is `true`.
        pub SpaceIdByHandleStorageFixed: bool = false;
    }
    add_extra_genesis {
      config(endowed_account): T::AccountId;
      build(|_: &Self| {
        SpaceIdByHandleStorageFixed::put(true);
      })
    }
}

decl_event!(
    pub enum Event<T> where
        <T as system::Config>::AccountId,
    {
        SpaceCreated(AccountId, SpaceId),
        SpaceUpdated(AccountId, SpaceId),
        SpaceDeleted(AccountId, SpaceId),
    }
);

// The pallet's dispatchable functions.
decl_module! {
  pub struct Module<T: Config> for enum Call where origin: T::Origin {

    const HandleDeposit: BalanceOf<T> = T::HandleDeposit::get();

    // Initializing errors
    type Error = Error<T>;

    // Initializing events
    fn deposit_event() = default;

    fn on_runtime_upgrade() -> frame_support::weights::Weight {
      let mut final_weight = Zero::zero();

      if !SpaceIdByHandleStorageFixed::get() {
        final_weight = migrations::fix_corrupted_handles_storage::<T>();
      }

      final_weight
    }

    #[weight = 500_000 + T::DbWeight::get().reads_writes(5, 4)]
    pub fn create_space(
      origin,
      parent_id_opt: Option<SpaceId>,
      handle_opt: Option<Vec<u8>>,
      content: Content,
      permissions_opt: Option<SpacePermissions>
    ) -> DispatchResult {
      let owner = ensure_signed(origin)?;

      Utils::<T>::is_valid_content(content.clone())?;

      if handle_opt.is_some() {
        Self::ensure_handles_enabled()?;
      }

      // TODO: add tests for this case
      if let Some(parent_id) = parent_id_opt {
        let parent_space = Self::require_space(parent_id)?;

        ensure!(T::IsAccountBlocked::is_allowed_account(owner.clone(), parent_id), UtilsError::<T>::AccountIsBlocked);
        ensure!(T::IsContentBlocked::is_allowed_content(content.clone(), parent_id), UtilsError::<T>::ContentIsBlocked);

        Self::ensure_account_has_space_permission(
          owner.clone(),
          &parent_space,
          SpacePermission::CreateSubspaces,
          Error::<T>::NoPermissionToCreateSubspaces.into()
        )?;
      }

      let permissions = permissions_opt.map(|perms| {
        Permissions::<T>::override_permissions(perms)
      });

      let space_id = Self::next_space_id();
      let new_space = &mut Space::new(space_id, parent_id_opt, owner.clone(), content, handle_opt.clone(), permissions);

      if let Some(handle) = handle_opt {
        new_space.reserve_handle(handle)?;
      }

      // FIXME: What's about handle reservation if this fails?
      T::BeforeSpaceCreated::before_space_created(owner.clone(), new_space)?;

      <SpaceById<T>>::insert(space_id, new_space);
      <SpaceIdsByOwner<T>>::mutate(owner.clone(), |ids| ids.push(space_id));
      NextSpaceId::mutate(|n| { *n += 1; });

      Self::deposit_event(RawEvent::SpaceCreated(owner, space_id));
      Ok(())
    }

    #[weight = 500_000 + T::DbWeight::get().reads_writes(3, 3)]
    pub fn update_space(origin, space_id: SpaceId, update: SpaceUpdate) -> DispatchResult {
      let owner = ensure_signed(origin)?;

      let has_updates =
        update.parent_id.is_some() ||
        update.handle.is_some() ||
        update.content.is_some() ||
        update.hidden.is_some() ||
        update.permissions.is_some();

      ensure!(has_updates, Error::<T>::NoUpdatesForSpace);

      let mut space = Self::require_space(space_id)?;

      ensure!(T::IsAccountBlocked::is_allowed_account(owner.clone(), space.id), UtilsError::<T>::AccountIsBlocked);

      Self::ensure_account_has_space_permission(
        owner.clone(),
        &space,
        SpacePermission::UpdateSpace,
        Error::<T>::NoPermissionToUpdateSpace.into()
      )?;

      let mut is_update_applied = false;
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
          is_update_applied = true;
        }
      }

      if let Some(content) = update.content {
        if content != space.content {
          Utils::<T>::is_valid_content(content.clone())?;

          ensure!(T::IsContentBlocked::is_allowed_content(content.clone(), space.id), UtilsError::<T>::ContentIsBlocked);
          if let Some(parent_id) = space.parent_id {
            ensure!(T::IsContentBlocked::is_allowed_content(content.clone(), parent_id), UtilsError::<T>::ContentIsBlocked);
          }

          old_data.content = Some(space.content);
          space.content = content;
          is_update_applied = true;
        }
      }

      if let Some(hidden) = update.hidden {
        if hidden != space.hidden {
          old_data.hidden = Some(space.hidden);
          space.hidden = hidden;
          is_update_applied = true;
        }
      }

      if let Some(overrides_opt) = update.permissions {
        if space.permissions != overrides_opt {
          old_data.permissions = Some(space.permissions);

          if let Some(overrides) = overrides_opt.clone() {
            space.permissions = Some(Permissions::<T>::override_permissions(overrides));
          } else {
            space.permissions = overrides_opt;
          }

          is_update_applied = true;
        }
      }

      let is_handle_updated = Self::update_handle(&space, update.handle.clone())?;
      if is_handle_updated {
          old_data.handle = Some(space.handle);
          space.handle = update.handle.unwrap();
          is_update_applied = true
        }

      // Update this space only if at least one field should be updated:
      if is_update_applied {
        space.updated = Some(WhoAndWhen::<T>::new(owner.clone()));

        <SpaceById<T>>::insert(space_id, space.clone());
        T::AfterSpaceUpdated::after_space_updated(owner.clone(), &space, old_data);

        Self::deposit_event(RawEvent::SpaceUpdated(owner, space_id));
      }
      Ok(())
    }

    #[weight = 10_000 + T::DbWeight::get().reads_writes(1, 1)]
    pub fn update_settings(origin, new_settings: SpacesSettings) -> DispatchResult {
      ensure_root(origin)?;

      let space_settings = Self::settings();
      ensure!(space_settings != new_settings, Error::<T>::NoUpdatesForSpacesSettings);

      PalletSettings::mutate(|settings| *settings = new_settings);

      Ok(())
    }

    #[weight = 10_000 + T::DbWeight::get().reads_writes(2, 2)]
    pub fn force_unreserve_handle(origin, handle: Vec<u8>) -> DispatchResultWithPostInfo {
      ensure_root(origin)?;

      let lowercased_handle = handle.to_ascii_lowercase();

      if let Some(space_id) = Self::space_id_by_handle(&lowercased_handle) {
        if let Ok(mut space) = Self::require_space(space_id) {
          space.unreserve_handle(lowercased_handle)?;

          space.handle = None;
          SpaceById::<T>::insert(space_id, space);
        } else {
          SpaceIdByHandle::remove(&lowercased_handle);
        }
      }

      Ok(Pays::No.into())
    }
  }
}

impl<T: Config> Space<T> {
    pub fn new(
        id: SpaceId,
        parent_id: Option<SpaceId>,
        created_by: T::AccountId,
        content: Content,
        handle: Option<Vec<u8>>,
        permissions: Option<SpacePermissions>,
    ) -> Self {
        Space {
            id,
            created: WhoAndWhen::<T>::new(created_by.clone()),
            updated: None,
            owner: created_by,
            parent_id,
            handle,
            content,
            hidden: false,
            posts_count: 0,
            hidden_posts_count: 0,
            followers_count: 0,
            score: 0,
            permissions,
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

    pub fn inc_hidden_posts(&mut self) {
        self.hidden_posts_count = self.hidden_posts_count.saturating_add(1);
    }

    pub fn dec_hidden_posts(&mut self) {
        self.hidden_posts_count = self.hidden_posts_count.saturating_sub(1);
    }

    pub fn inc_followers(&mut self) {
        self.followers_count = self.followers_count.saturating_add(1);
    }

    pub fn dec_followers(&mut self) {
        self.followers_count = self.followers_count.saturating_sub(1);
    }

    pub fn try_get_parent(&self) -> Result<SpaceId, DispatchError> {
        self.parent_id.ok_or_else(|| Error::<T>::SpaceIsAtRoot.into())
    }

    pub fn is_public(&self) -> bool {
        !self.hidden && self.content.is_some()
    }

    pub fn is_unlisted(&self) -> bool {
        !self.is_public()
    }

    pub fn reserve_handle(
      &self,
      handle: Vec<u8>
    ) -> DispatchResult {
      let handle_in_lowercase = Module::<T>::lowercase_and_ensure_unique_handle(handle)?;
      Module::<T>::reserve_handle_deposit(&self.owner)?;
      SpaceIdByHandle::insert(handle_in_lowercase, self.id);
      Ok(())
    }

    pub fn unreserve_handle(
      &self,
      handle: Vec<u8>
    ) -> DispatchResult {
      let handle_in_lowercase = Utils::<T>::lowercase_handle(handle);
      Module::<T>::unreserve_handle_deposit(&self.owner);
      SpaceIdByHandle::remove(handle_in_lowercase);
      Ok(())
    }
}

impl<T: Config> Module<T> {

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

    pub fn ensure_handles_enabled() -> DispatchResult {
        ensure!(Self::settings().handles_enabled, Error::<T>::HandlesAreDisabled);
        Ok(())
    }

    pub fn try_move_space_to_root(space_id: SpaceId) -> DispatchResult {
        let mut space = Self::require_space(space_id)?;
        space.parent_id = None;

        SpaceById::<T>::insert(space_id, space);
        Ok(())
    }

    pub fn mutate_space_by_id<F: FnOnce(&mut Space<T>)> (
        space_id: SpaceId,
        f: F
    ) -> Result<Space<T>, DispatchError> {
        <SpaceById<T>>::mutate(space_id, |space_opt| {
            if let Some(ref mut space) = space_opt.clone() {
                f(space);
                *space_opt = Some(space.clone());

                return Ok(space.clone());
            }

            Err(Error::<T>::SpaceNotFound.into())
        })
    }

    /// Lowercase a handle and ensure that it's unique, i.e. no space reserved this handle yet.
    fn lowercase_and_ensure_unique_handle(handle: Vec<u8>) -> Result<Vec<u8>, DispatchError> {
        let handle_in_lowercase = Utils::<T>::lowercase_and_validate_a_handle(handle)?;

        // Check if a handle is unique across all spaces' handles:
        ensure!(Self::space_id_by_handle(handle_in_lowercase.clone()).is_none(), Error::<T>::SpaceHandleIsNotUnique);

        Ok(handle_in_lowercase)
    }

    pub fn reserve_handle_deposit(space_owner: &T::AccountId) -> DispatchResult {
        <T as Config>::Currency::reserve(space_owner, T::HandleDeposit::get())
    }

    pub fn unreserve_handle_deposit(space_owner: &T::AccountId) -> BalanceOf<T> {
        <T as Config>::Currency::unreserve(space_owner, T::HandleDeposit::get())
    }

    /// This function will be performed only if a space has a handle.
    /// Unreserve a handle deposit from the current space owner,
    /// then transfer deposit amount to a new owner
    /// and reserve this amount from a new owner.
    pub fn maybe_transfer_handle_deposit_to_new_space_owner(space: &Space<T>, new_owner: &T::AccountId) -> DispatchResult {
        if space.handle.is_some() {
            let old_owner = &space.owner;
            Self::unreserve_handle_deposit(old_owner);
            <T as Config>::Currency::transfer(
                old_owner,
                new_owner,
                T::HandleDeposit::get(),
                ExistenceRequirement::KeepAlive
            )?;
            Self::reserve_handle_deposit(new_owner)?;
        }
        Ok(())
    }

    fn update_handle(
        space: &Space<T>,
        maybe_new_handle: Option<Option<Vec<u8>>>,
    ) -> Result<bool, DispatchError> {
        let mut is_handle_updated = false;
        if let Some(new_handle_opt) = maybe_new_handle {

            // We need to ensure that the space handles feature is enabled
            // before allowing to edit them
            Self::ensure_handles_enabled()?;

            if let Some(old_handle) = space.handle.clone() {
                // If the space has a handle

                if let Some(new_handle) = new_handle_opt {
                    if new_handle != old_handle {
                        // Change the current handle to a new one

                        // Validate data first
                        let old_handle_lc = Utils::<T>::lowercase_handle(old_handle);
                        let new_handle_lc = Self::lowercase_and_ensure_unique_handle(new_handle)?;

                        // Update storage once data is valid
                        SpaceIdByHandle::remove(old_handle_lc);
                        SpaceIdByHandle::insert(new_handle_lc, space.id);
                        is_handle_updated = true;
                    }
                } else {
                    // Unreserve the current handle
                    space.unreserve_handle(old_handle)?;
                    is_handle_updated = true;
                }
            } else if let Some(new_handle) = new_handle_opt {
                // Reserve a handle for the space that has no handle yet
                space.reserve_handle(new_handle)?;
                is_handle_updated = true;
            }
        }
        Ok(is_handle_updated)
    }
}

impl<T: Config> SpaceForRolesProvider for Module<T> {
    type AccountId = T::AccountId;

    fn get_space(id: SpaceId) -> Result<SpaceForRoles<Self::AccountId>, DispatchError> {
        let space = Module::<T>::require_space(id)?;

        Ok(SpaceForRoles {
            owner: space.owner,
            permissions: space.permissions,
        })
    }
}

pub trait BeforeSpaceCreated<T: Config> {
    fn before_space_created(follower: T::AccountId, space: &mut Space<T>) -> DispatchResult;
}

impl<T: Config> BeforeSpaceCreated<T> for () {
    fn before_space_created(_follower: T::AccountId, _space: &mut Space<T>) -> DispatchResult {
        Ok(())
    }
}

#[impl_trait_for_tuples::impl_for_tuples(10)]
pub trait AfterSpaceUpdated<T: Config> {
    fn after_space_updated(sender: T::AccountId, space: &Space<T>, old_data: SpaceUpdate);
}
