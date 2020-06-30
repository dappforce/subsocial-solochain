#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
  decl_module,
  traits::Get
};
use sp_runtime::RuntimeDebug;
use sp_std::{
  collections::btree_set::BTreeSet,
  prelude::*
};
use frame_system::{self as system};

use pallet_utils::SpaceId;

#[derive(Encode, Decode, Ord, PartialOrd, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum SpacePermission {
  /// Create, update, delete, grant and revoke roles in this space.
  ManageRoles,

  /// Act on behalf of this space within this space.
  RepresentSpaceInternally,
  /// Act on behalf of this space outside of this space.
  RepresentSpaceExternally,

  /// Update this space.
  UpdateSpace,

  /// Block accounts and other outer spaces within this space.
  BlockUsers,
  /// Report accounts and other outer spaces within this space.
  ReportUsers,

  // Related to subspaces in this space:
  CreateSubspaces,
  UpdateOwnSubspaces,
  DeleteOwnSubspaces,
  HideOwnSubspaces,

  UpdateAnySubspace,
  DeleteAnySubspace,
  HideAnySubspace,

  BlockSubspaces,
  ReportSubspaces,

  // Related to posts in this space:
  CreatePosts,
  UpdateOwnPosts,
  DeleteOwnPosts,
  HideOwnPosts,

  UpdateAnyPost,
  DeleteAnyPost,
  HideAnyPost,

  BlockPosts,
  ReportPosts,

  // Related to comments in this space:
  CreateComments,
  UpdateOwnComments,
  DeleteOwnComments,
  HideOwnComments,

  // NOTE: It was made on purpose that it's not possible to update or delete not own comments.
  // Instead it's possible to allow to report, hide and block comments.
  HideAnyComment,
  BlockComments,
  ReportComments,

  /// Upvote any post or comment in this space.
  Upvote,
  /// Downvote any post or comment in this space.
  Downvote,
  /// Share any post or comment from this space to another outer space.
  Share,

  /// Override permissions per subspace in this space.
  OverrideSubspacePermissions,
  /// Override permissions per post in this space.
  OverridePostPermissions,
}

pub type SpacePermissionSet = BTreeSet<SpacePermission>;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct SpacePermissions {
  pub none: Option<SpacePermissionSet>,
  pub everyone: Option<SpacePermissionSet>,
  pub follower: Option<SpacePermissionSet>,
  pub space_owner: Option<SpacePermissionSet>,
}

impl Default for SpacePermissions {
  fn default() -> SpacePermissions {
    SpacePermissions {
      none: None,
      everyone: None,
      follower: None,
      space_owner: None,
    }
  }
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct SpacePermissionsContext {
  pub space_id: SpaceId,
  pub is_space_owner: bool,
  pub is_space_follower: bool,
  pub space_perms: Option<SpacePermissions>
}

/// The pallet's configuration trait.
pub trait Trait: system::Trait {
  type DefaultSpacePermissions: Get<SpacePermissions>;
}

decl_module! {
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {
    const DefaultSpacePermissions: SpacePermissions = T::DefaultSpacePermissions::get();
  }
}

impl SpacePermission {
  fn is_present_in_role(&self, perms_opt: Option<SpacePermissionSet>) -> bool {
    if let Some(perms) = perms_opt {
      if perms.contains(self) {
        return true
      }
    }
    false
  }
}

impl<T: Trait> Module<T> {

  fn get_overrides_or_defaults(
    overrides: Option<SpacePermissionSet>,
    defaults: Option<SpacePermissionSet>
  ) -> Option<SpacePermissionSet> {

    if overrides.is_some() {
      overrides
    } else {
      defaults
    }
  }

  fn resolve_space_perms(
    space_perms: Option<SpacePermissions>,
  ) -> SpacePermissions {

    let defaults = T::DefaultSpacePermissions::get();
    let overrides = space_perms.unwrap_or_default();

    SpacePermissions {
      none: Self::get_overrides_or_defaults(overrides.none, defaults.none),
      everyone: Self::get_overrides_or_defaults(overrides.everyone, defaults.everyone),
      follower: Self::get_overrides_or_defaults(overrides.follower, defaults.follower),
      space_owner: Self::get_overrides_or_defaults(overrides.space_owner, defaults.space_owner)
    }
  }

  pub fn has_user_a_space_permission(
    ctx: SpacePermissionsContext,
    permission: SpacePermission,
  ) -> Option<bool> {

    let perms_by_role = Self::resolve_space_perms(ctx.space_perms);

    // Check if this permission is forbidden:
    if permission.is_present_in_role(perms_by_role.none) {
      return Some(false)
    }

    let is_space_owner = ctx.is_space_owner;
    let is_follower = is_space_owner || ctx.is_space_follower;

    if
      permission.is_present_in_role(perms_by_role.everyone) ||
      is_follower && permission.is_present_in_role(perms_by_role.follower) ||
      is_space_owner && permission.is_present_in_role(perms_by_role.space_owner)
    {
      return Some(true)
    }

    None
  }
}