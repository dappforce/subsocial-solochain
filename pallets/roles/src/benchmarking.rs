//! Roles pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use sp_std::vec;
use frame_system::RawOrigin;
use frame_benchmarking::{benchmarks, account, whitelisted_caller};
use sp_runtime::traits::Bounded;
use pallet_utils::{Config as UtilsConfig, BalanceOf, Content, SpaceId};
use pallet_spaces::Module as SpacesModule;
use frame_support::{
    dispatch::DispatchError, traits::Currency,
};

const SPACE: SpaceId = 1001;
const ROLE: RoleId = 1;
const SEED: u32 = 0;

fn space_content_ipfs() -> Content {
    Content::IPFS(b"bafyreib3mgbou4xln42qqcgj6qlt3cif35x4ribisxgq7unhpun525l54e".to_vec())
}

fn space_handle() -> Option<Vec<u8>> {
    Some(b"Space_Handle".to_vec())
}

fn default_role_content_ipfs() -> Content {
    Content::IPFS(b"QmRAQB6YaCyidP37UdDnjFY5vQuiBrcqdyoW1CuDgwxkD4".to_vec())
}

fn permission_set_default() -> Vec<SpacePermission> {
    vec![SpacePermission::ManageRoles]
}

fn updated_role_content_ipfs() -> Content {
    Content::IPFS(b"QmZENA8YaCyidP37UdDnjFY5vQuiBrcqdyoW1CuDaazhR8".to_vec())
}

fn permission_set_updated() -> Vec<SpacePermission> {
    vec![SpacePermission::ManageRoles, SpacePermission::CreatePosts]
}

fn add_origin_with_space_and_balance<T: Config>() -> Result<RawOrigin<T::AccountId>, DispatchError> {
    let caller: T::AccountId = whitelisted_caller();
    let origin = RawOrigin::Signed(caller.clone());

    <T as UtilsConfig>::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

    SpacesModule::<T>::create_space(origin.clone().into(), None, space_handle(), space_content_ipfs(), None)?;

    Ok(origin)
}

fn add_origin_with_space_balance_and_role<T: Config>() -> Result<RawOrigin<T::AccountId>, DispatchError> {
    let origin = add_origin_with_space_and_balance::<T>()?;

    Module::<T>::create_role(origin.clone().into(), SPACE, Some(100u32.into()), default_role_content_ipfs(), permission_set_default())?;

    Ok(origin)
}

benchmarks! {
    create_role {
        let origin = add_origin_with_space_and_balance::<T>()?;
    }: _(origin, SPACE, Some(100u32.into()), default_role_content_ipfs(), permission_set_default())
    verify {
        assert!(RoleById::<T>::get(ROLE).is_some());

        let role = RoleById::<T>::get(ROLE).unwrap();

        assert!(role.updated.is_none());
        assert_eq!(role.space_id, SPACE);
        assert_eq!(role.disabled, false);
        assert_eq!(role.content, self::default_role_content_ipfs());
        assert_eq!(
            role.permissions,
            self::permission_set_default().into_iter().collect()
        );
    }

    update_role {
        let origin = add_origin_with_space_balance_and_role::<T>()?;

        let role_update: RoleUpdate = RoleUpdate {
            disabled: Some(true),
            content: Some(updated_role_content_ipfs()),
            permissions: Some(
                self::permission_set_updated().into_iter().collect()
            ),
        };
    }: _(origin, ROLE, role_update)
    verify {
        assert!(RoleById::<T>::get(ROLE).is_some());

        // Check whether data in Role structure is correct
        let role = RoleById::<T>::get(ROLE).unwrap();

        assert!(role.updated.is_some());
        assert_eq!(role.space_id, SPACE);
        assert_eq!(role.disabled, true);
        assert_eq!(role.content, updated_role_content_ipfs());
        assert_eq!(
            role.permissions,
            self::permission_set_updated().into_iter().collect()
        );
    }

    delete_role {
        let origin = add_origin_with_space_balance_and_role::<T>()?;
        let user: T::AccountId = account("user", 0, SEED);

        Module::<T>::grant_role(origin.clone().into(), ROLE, vec![User::Account(user.clone())])?;
    }: _(origin, ROLE)
    verify {
        assert!(RoleById::<T>::get(ROLE).is_none());
        assert!(UsersByRoleId::<T>::get(ROLE).is_empty());
        assert!(RoleIdsBySpaceId::get(SPACE).is_empty());
        assert!(RoleIdsByUserInSpace::<T>::get(User::Account(user), SPACE).is_empty());
    }

    grant_role {
        let origin = add_origin_with_space_balance_and_role::<T>()?;
        let user: T::AccountId = account("user", 0, SEED);
    }: _(origin, ROLE, vec![User::Account(user.clone())])
    verify {
        assert_eq!(UsersByRoleId::<T>::get(ROLE), vec![User::Account(user.clone())]);
        assert_eq!(RoleIdsByUserInSpace::<T>::get(User::Account(user.clone()), SPACE), vec![ROLE]);
    }

    revoke_role {
        let user: T::AccountId = account("user", 0, SEED);
        let origin = add_origin_with_space_balance_and_role::<T>()?;

        Module::<T>::grant_role(origin.clone().into(), ROLE, vec![User::Account(user.clone())])?;
    }: _(origin, ROLE, vec![User::Account(user.clone())])
    verify {
        assert!(UsersByRoleId::<T>::get(ROLE).is_empty());
        assert!(RoleIdsByUserInSpace::<T>::get(User::Account(user), SPACE).is_empty());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::{Test, ExtBuilder};
    use frame_support::assert_ok;

    #[test]
    fn test_benchmarks() {
        ExtBuilder::build_without_space().execute_with(|| {
            assert_ok!(test_benchmark_create_role::<Test>());
            assert_ok!(test_benchmark_update_role::<Test>());
            assert_ok!(test_benchmark_delete_role::<Test>());
            assert_ok!(test_benchmark_grant_role::<Test>());
            assert_ok!(test_benchmark_revoke_role::<Test>());
        });
    }
}
