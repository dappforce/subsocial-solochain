#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::dispatch::{DispatchError, DispatchResult};

use pallet_permissions::{
  SpacePermission,
  SpacePermissions,
  SpacePermissionsContext
};
use pallet_utils::{SpaceId, User};

pub mod moderation;

/// Minimal set of fields from Space struct that are required by roles pallet.
pub struct SpaceForRoles<AccountId> {
  pub owner: AccountId,
  pub permissions: Option<SpacePermissions>,
}

pub trait SpaceForRolesProvider {
  type AccountId;

  fn get_space(id: SpaceId) -> Result<SpaceForRoles<Self::AccountId>, DispatchError>;
}

pub trait SpacesProvider {
  fn ensure_space_exists(space_id: SpaceId) -> DispatchResult;
}

pub trait SpaceFollowsProvider {
  type AccountId;

  fn is_space_follower(account: Self::AccountId, space_id: SpaceId) -> bool;
}

pub trait PermissionChecker {
  type AccountId;

  fn ensure_user_has_space_permission(
    user: User<Self::AccountId>,
    ctx: SpacePermissionsContext,
    permission: SpacePermission,
    error: DispatchError,
  ) -> DispatchResult;

  fn ensure_account_has_space_permission(
    account: Self::AccountId,
    ctx: SpacePermissionsContext,
    permission: SpacePermission,
    error: DispatchError,
  ) -> DispatchResult {

    Self::ensure_user_has_space_permission(
      User::Account(account),
      ctx,
      permission,
      error
    )
  }
}
