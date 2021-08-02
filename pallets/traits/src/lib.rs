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

pub trait OnFreeTransaction<AccountId> {
  fn can_account_make_free_call(who: &AccountId) -> bool;
}

impl<AccountId> OnFreeTransaction<AccountId> for () {
  fn can_account_make_free_call(_who: &AccountId) -> bool {
    false
  }
}

pub trait TrustHandler<AccountId> {
  fn is_trusted_account(who: &AccountId) -> bool;

  fn is_email_confirmed(who: &AccountId) -> bool;

  fn is_phone_number_confirmed(who: &AccountId) -> bool;
}

impl<AccountId> TrustHandler<AccountId> for () {
  fn is_trusted_account(_who: &AccountId) -> bool {
    false
  }

  fn is_email_confirmed(_who: &AccountId) -> bool {
    false
  }

  fn is_phone_number_confirmed(_who: &AccountId) -> bool {
    false
  }
}
