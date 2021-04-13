use sp_std::prelude::*;
use sp_std::collections::{ btree_set::BTreeSet };

use crate::{Module, Config, RoleId, Role};
use pallet_utils::{SpaceId, User};
use pallet_permissions::{SpacePermission};
use sp_std::iter::FromIterator;

impl<T: Config> Module<T> {
    pub fn get_space_permissions_by_user(account: T::AccountId, space_id: SpaceId) -> Vec<SpacePermission> {
        let role_ids: Vec<RoleId> = Self::role_ids_by_user_in_space((User::Account(account), space_id));

        BTreeSet::from_iter(
            role_ids
                .iter()
                .filter_map(|id| Self::role_by_id(id))
                .flat_map(|role: Role<T>| role.permissions.into_iter())
                .into_iter()
        )
            .iter()
            .cloned()
            .collect()
    }
}