//! # Roles Module
//!
//! This module allow you to create dynalic roles with an associated set of permissions
//! and grant them to users (accounts or space ids) within a given space.
//!
//! For example if you want to create a space that enables editors in a similar way to Medium,
//! you would create a role "Editor" with permissions such as `CreatePosts`, `UpdateAnyPost`,
//! and `HideAnyComment`. Then you would grant this role to the specific accounts you would like
//! to make editors.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    ensure,
    traits::Get,
    dispatch::DispatchResult
};
use sp_runtime::RuntimeDebug;
use sp_std::{collections::btree_set::BTreeSet, prelude::*};
use frame_system::{self as system, ensure_signed};

use df_traits::{
    PermissionChecker, SpaceFollowsProvider, SpaceForRolesProvider,
    moderation::{IsAccountBlocked, IsContentBlocked},
};
use pallet_permissions::{Module as Permissions, SpacePermission, SpacePermissionSet};
use pallet_utils::{Module as Utils, Error as UtilsError, SpaceId, User, WhoAndWhen, Content};

pub mod functions;
pub mod rpc;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub type RoleId = u64;

/// Information about a role's permissions, its' containing space, and its' content.
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Role<T: Config> {
    pub created: WhoAndWhen<T>,
    pub updated: Option<WhoAndWhen<T>>,

    /// Unique sequential identifier of a role. Examples of role ids: `1`, `2`, `3`, and so on.
    pub id: RoleId,

    /// An id of a space that contains this role.
    pub space_id: SpaceId,

    /// If `true` then the permissions associated with a given role will have no affect.
    /// This is useful if you would like to temporarily disable permissions from a given role,
    /// without removing the role from its' owners
    pub disabled: bool,

    /// An optional block number at which this role will expire. If `expires_at` is `Some`
    /// and the current block is greater or equal to its value, the permissions associated
    /// with a given role will have no affect.
    pub expires_at: Option<T::BlockNumber>,

    /// Content can optionally contain additional information associated with a role,
    /// such as a name, description, and image for a role. This may be useful for end users.
    pub content: Content,

    /// A set of permisions granted to owners of a particular role which are valid
    /// only within the space containing this role
    pub permissions: SpacePermissionSet,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct RoleUpdate {
    pub disabled: Option<bool>,
    pub content: Option<Content>,
    pub permissions: Option<SpacePermissionSet>,
}

/// The pallet's configuration trait.
pub trait Config: system::Config
    + pallet_permissions::Config
    + pallet_utils::Config
{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Config>::Event>;

    /// When deleting a role via `delete_role()` dispatch, this parameter is checked.
    /// If the number of users that own a given role is greater or equal to this number,
    /// then `TooManyUsersToDeleteRole` error will be returned and the dispatch will fail.
    type MaxUsersToProcessPerDeleteRole: Get<u16>;

    type Spaces: SpaceForRolesProvider<AccountId=Self::AccountId>;

    type SpaceFollows: SpaceFollowsProvider<AccountId=Self::AccountId>;

    type IsAccountBlocked: IsAccountBlocked<Self::AccountId>;

    type IsContentBlocked: IsContentBlocked;
}

decl_event!(
    pub enum Event<T> where
        <T as system::Config>::AccountId
    {
        RoleCreated(AccountId, SpaceId, RoleId),
        RoleUpdated(AccountId, RoleId),
        RoleDeleted(AccountId, RoleId),
        RoleGranted(AccountId, RoleId, Vec<User<AccountId>>),
        RoleRevoked(AccountId, RoleId, Vec<User<AccountId>>),
    }
);

decl_error! {
    pub enum Error for Module<T: Config> {
        /// Role was not found by id.
        RoleNotFound,

        /// `NextRoleId` exceeds its maximum value.
        RoleIdOverflow,

        /// Account does not have permission to manage roles in this space.
        NoPermissionToManageRoles,

        /// Nothing to update in role.
        NoUpdatesProvided,

        /// No permissions provided when trying to create a new role.
        /// A role must have at least one permission.
        NoPermissionsProvided,

        /// No users provided when trying to grant a role.
        /// A role must be granted/revoked to/from at least one user.
        NoUsersProvided,

        /// Canot remove a role from this many users in a single transaction.
        /// See `MaxUsersToProcessPerDeleteRole` parameter of this trait.
        TooManyUsersToDeleteRole,

        /// Cannot disable a role that is already disabled.
        RoleAlreadyDisabled,

        /// Cannot enable a role that is already enabled.
        RoleAlreadyEnabled,
    }
}

pub const FIRST_ROLE_ID: u64 = 1;

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Config> as PermissionsModule {

        /// The next role id.
        pub NextRoleId get(fn next_role_id): RoleId = FIRST_ROLE_ID;

        /// Get the details of a role by its' id.
        pub RoleById get(fn role_by_id):
            map hasher(twox_64_concat) RoleId => Option<Role<T>>;

        /// Get a list of all users (account or space ids) that a given role has been granted to.
        pub UsersByRoleId get(fn users_by_role_id):
            map hasher(twox_64_concat) RoleId => Vec<User<T::AccountId>>;

        /// Get a list of all role ids available in a given space.
        pub RoleIdsBySpaceId get(fn role_ids_by_space_id):
            map hasher(twox_64_concat) SpaceId => Vec<RoleId>;

        /// Get a list of all role ids owned by a given user (account or space id)
        /// within a given space.
        pub RoleIdsByUserInSpace get(fn role_ids_by_user_in_space): double_map
            hasher(blake2_128_concat) User<T::AccountId>,
            hasher(twox_64_concat) SpaceId
            => Vec<RoleId>;
    }
}

// The pallet's dispatchable functions.
decl_module! {
  pub struct Module<T: Config> for enum Call where origin: T::Origin {

    const MaxUsersToProcessPerDeleteRole: u16 = T::MaxUsersToProcessPerDeleteRole::get();

    // Initializing errors
    type Error = Error<T>;

    // Initializing events
    fn deposit_event() = default;

    /// Create a new role, with a list of permissions, within a given space.
    ///
    /// `content` can optionally contain additional information associated with a role,
    /// such as a name, description, and image for a role. This may be useful for end users.
    ///
    /// Only the space owner or a user with `ManageRoles` permission can call this dispatch.
    #[weight = 10_000 + T::DbWeight::get().reads_writes(2, 3)]
    pub fn create_role(
      origin,
      space_id: SpaceId,
      time_to_live: Option<T::BlockNumber>,
      content: Content,
      permissions: Vec<SpacePermission>
    ) -> DispatchResult {
      let who = ensure_signed(origin)?;

      ensure!(!permissions.is_empty(), Error::<T>::NoPermissionsProvided);

      Utils::<T>::is_valid_content(content.clone())?;
      ensure!(T::IsContentBlocked::is_allowed_content(content.clone(), space_id), UtilsError::<T>::ContentIsBlocked);

      Self::ensure_role_manager(who.clone(), space_id)?;

      let permissions_set = permissions.into_iter().collect();
      let new_role = Role::<T>::new(who.clone(), space_id, time_to_live, content, permissions_set)?;

      // TODO review strange code:
      let next_role_id = new_role.id.checked_add(1).ok_or(Error::<T>::RoleIdOverflow)?;
      NextRoleId::put(next_role_id);

      <RoleById<T>>::insert(new_role.id, new_role.clone());
      RoleIdsBySpaceId::mutate(space_id, |role_ids| { role_ids.push(new_role.id) });

      Self::deposit_event(RawEvent::RoleCreated(who, space_id, new_role.id));
      Ok(())
    }

    /// Update an existing role by a given id.
    /// Only the space owner or a user with `ManageRoles` permission can call this dispatch.
    #[weight = 10_000 + T::DbWeight::get().reads_writes(2, 1)]
    pub fn update_role(origin, role_id: RoleId, update: RoleUpdate) -> DispatchResult {
      let who = ensure_signed(origin)?;

      let has_updates =
        update.disabled.is_some() ||
        update.content.is_some() ||
        update.permissions.is_some();

      ensure!(has_updates, Error::<T>::NoUpdatesProvided);

      let mut role = Self::require_role(role_id)?;

      Self::ensure_role_manager(who.clone(), role.space_id)?;

      let mut is_update_applied = false;

      if let Some(disabled) = update.disabled {
        if disabled != role.disabled {
          role.set_disabled(disabled)?;
          is_update_applied = true;
        }
      }

      if let Some(content) = update.content {
        if content != role.content {
          Utils::<T>::is_valid_content(content.clone())?;
          ensure!(T::IsContentBlocked::is_allowed_content(content.clone(), role.space_id), UtilsError::<T>::ContentIsBlocked);

          role.content = content;
          is_update_applied = true;
        }
      }

      if let Some(permissions) = update.permissions {
        if !permissions.is_empty() {
          let permissions_diff: Vec<_> = permissions.symmetric_difference(&role.permissions).cloned().collect();

          if !permissions_diff.is_empty() {
            role.permissions = permissions;
            is_update_applied = true;
          }
        }
      }

      if is_update_applied {
        role.updated = Some(WhoAndWhen::<T>::new(who.clone()));

        <RoleById<T>>::insert(role_id, role);
        Self::deposit_event(RawEvent::RoleUpdated(who, role_id));
      }
      Ok(())
    }

    /// Delete a given role and clean all associated storage items.
    /// Only the space owner or a user with `ManageRoles` permission can call this dispatch.
    #[weight = 1_000_000 + T::DbWeight::get().reads_writes(6, 5)]
    pub fn delete_role(origin, role_id: RoleId) -> DispatchResult {
      let who = ensure_signed(origin)?;

      let role = Self::require_role(role_id)?;

      Self::ensure_role_manager(who.clone(), role.space_id)?;

      let users = Self::users_by_role_id(role_id);
      ensure!(
        users.len() <= T::MaxUsersToProcessPerDeleteRole::get() as usize,
        Error::<T>::TooManyUsersToDeleteRole
      );

      let role_idx_by_space_opt = Self::role_ids_by_space_id(role.space_id).iter()
        .position(|x| { *x == role_id });

      if let Some(role_idx) = role_idx_by_space_opt {
        RoleIdsBySpaceId::mutate(role.space_id, |n| { n.swap_remove(role_idx) });
      }

      role.revoke_from_users(users);

      <RoleById<T>>::remove(role_id);
      <UsersByRoleId<T>>::remove(role_id);

      Self::deposit_event(RawEvent::RoleDeleted(who, role_id));
      Ok(())
    }

    /// Grant a given role to a list of users.
    /// Only the space owner or a user with `ManageRoles` permission can call this dispatch.
    #[weight = 1_000_000 + T::DbWeight::get().reads_writes(4, 2)]
    pub fn grant_role(origin, role_id: RoleId, users: Vec<User<T::AccountId>>) -> DispatchResult {
      let who = ensure_signed(origin)?;

      ensure!(!users.is_empty(), Error::<T>::NoUsersProvided);
      let users_set: BTreeSet<User<T::AccountId>> = Utils::<T>::convert_users_vec_to_btree_set(users)?;

      let role = Self::require_role(role_id)?;

      Self::ensure_role_manager(who.clone(), role.space_id)?;

      for user in users_set.iter() {
        if !Self::users_by_role_id(role_id).contains(&user) {
          <UsersByRoleId<T>>::mutate(role_id, |users| { users.push(user.clone()); });
        }
        if !Self::role_ids_by_user_in_space(user.clone(), role.space_id).contains(&role_id) {
          <RoleIdsByUserInSpace<T>>::mutate(user.clone(), role.space_id, |roles| { roles.push(role_id); })
        }
      }

      Self::deposit_event(RawEvent::RoleGranted(who, role_id, users_set.iter().cloned().collect()));
      Ok(())
    }

    /// Revoke a given role from a list of users.
    /// Only the space owner or a user with `ManageRoles` permission can call this dispatch.
    #[weight = 1_000_000 + T::DbWeight::get().reads_writes(4, 2)]
    pub fn revoke_role(origin, role_id: RoleId, users: Vec<User<T::AccountId>>) -> DispatchResult {
      let who = ensure_signed(origin)?;

      ensure!(!users.is_empty(), Error::<T>::NoUsersProvided);

      let role = Self::require_role(role_id)?;

      Self::ensure_role_manager(who.clone(), role.space_id)?;

      role.revoke_from_users(users.clone());

      Self::deposit_event(RawEvent::RoleRevoked(who, role_id, users));
      Ok(())
    }
  }
}
