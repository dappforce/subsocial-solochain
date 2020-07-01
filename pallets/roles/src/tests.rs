use crate::mock::*;
use crate::*;

use frame_support::{
    assert_ok, assert_noop,
};
use pallet_utils::{Error as UtilsError};


#[test]
fn create_role_should_work() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_create_default_role()); // RoleId 1

        // Check whether Role is stored correctly
        assert!(Roles::role_by_id(ROLE1).is_some());

        // Check whether data in Role structure is correct
        let role = Roles::role_by_id(ROLE1).unwrap();
        assert_eq!(Roles::next_role_id(), ROLE2);

        assert!(role.updated.is_none());
        assert_eq!(role.space_id, SPACE1);
        assert_eq!(role.disabled, false);
        assert_eq!(role.content, self::default_role_content_ipfs());
        assert_eq!(
            role.permissions,
            BTreeSet::from_iter(self::permission_set_default().into_iter())
        );
    });
}

#[test]
fn create_role_should_work_with_a_few_roles() {
    ExtBuilder::build_with_a_few_roles_granted_to_account2().execute_with(|| {
        assert_ok!(
            _create_role(
                Some(Origin::signed(ACCOUNT2)),
                None, // On SpaceId 1
                None, // Without time_to_live
                None, // With default content
                Some(self::permission_set_updated())
            )
        ); // RoleId 3

        // Check whether Role is stored correctly
        assert!(Roles::role_by_id(ROLE3).is_some());

        // Check whether data in Role structure is correct
        let role = Roles::role_by_id(ROLE3).unwrap();
        assert_eq!(Roles::next_role_id(), ROLE4);

        assert!(role.updated.is_none());
        assert_eq!(role.space_id, SPACE1);
        assert_eq!(role.disabled, false);
        assert_eq!(role.content, self::default_role_content_ipfs());
        assert_eq!(
            role.permissions,
            BTreeSet::from_iter(self::permission_set_updated().into_iter())
        );
    });
}

#[test]
fn create_role_should_fail_with_space_not_found() {
    ExtBuilder::build().execute_with(|| {
        assert_noop!(
            _create_role(
                None, // From ACCOUNT1
                Some(SPACE2),
                None, // Without time_to_live
                None, // With default content
                None // With default permission set
            ), "SpaceNotFound"
        );
    });
}

#[test]
fn create_role_should_fail_with_no_permission() {
    ExtBuilder::build().execute_with(|| {
        assert_noop!(
            _create_role(
                Some(Origin::signed(ACCOUNT2)),
                None, // On SpaceId 1
                None, // Without time_to_live
                None, // With default content
                None // With default permission set
            ), Error::<Test>::NoPermissionToManageRoles
        );
    });
}

#[test]
fn create_role_should_fail_with_no_permissions_provided() {
    ExtBuilder::build().execute_with(|| {
        assert_noop!(
            _create_role(
                None, // From ACCOUNT1
                None, // On SpaceId 1
                None, // Without time_to_live
                None, // With default permission set
                Some(self::permission_set_empty())
            ),
            Error::<Test>::NoPermissionsProvided
        );
    });
}

#[test]
fn create_role_should_fail_with_ipfs_is_incorrect() {
    ExtBuilder::build().execute_with(|| {
        assert_noop!(_create_role(
            None, // From ACCOUNT1
            None, // On SpaceId 1
            None, // Without time_to_live
            Some(self::invalid_role_content_ipfs()),
            None // With default permissions set
        ), UtilsError::<Test>::InvalidIpfsCid);
    });
}

#[test]
fn create_role_should_fail_with_a_few_roles_no_permission() {
    ExtBuilder::build_with_a_few_roles_granted_to_account2().execute_with(|| {
        assert_ok!(_delete_role(None, Some(ROLE2)));
        assert_noop!(
            _create_role(
                Some(Origin::signed(ACCOUNT2)),
                None, // On SpaceId 1
                None, // Without time_to_live
                None, // With default content
                Some(self::permission_set_random())
            ), Error::<Test>::NoPermissionToManageRoles
        );
    });
}

#[test]
fn update_role_should_work() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_create_default_role()); // RoleId 1
        assert_ok!(_update_default_role());

        // Check whether Role is stored correctly
        assert!(Roles::role_by_id(ROLE1).is_some());

        // Check whether data in Role structure is correct
        let role = Roles::role_by_id(ROLE1).unwrap();

        assert!(role.updated.is_some());
        assert_eq!(role.space_id, SPACE1);
        assert_eq!(role.disabled, true);
        assert_eq!(role.content, self::updated_role_content_ipfs());
        assert_eq!(
            role.permissions,
            BTreeSet::from_iter(self::permission_set_updated().into_iter())
        );
    });
}

#[test]
fn update_role_should_work_with_empty_perms_provided_no_changes() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_create_default_role()); // RoleId 1
        assert_ok!(
            _update_role(
                None, // From ACCOUNT1
                None, // On RoleId 1
                Some(
                    self::role_update(
                        Some(true),
                        None,
                        Some(BTreeSet::from_iter(self::permission_set_empty().into_iter()))
                    )
                )
            )
        );

        // Check whether data in Role structure is correct
        let role = Roles::role_by_id(ROLE1).unwrap();

        assert!(role.updated.is_some());
        assert_eq!(role.space_id, SPACE1);
        assert_eq!(role.disabled, true);
        assert_eq!(role.content, self::default_role_content_ipfs());
        assert_eq!(
            role.permissions,
            BTreeSet::from_iter(self::permission_set_default().into_iter())
        );
    });
}

#[test]
fn update_role_should_work_with_same_perms_provided_no_update() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_create_default_role()); // RoleId 1
        assert_ok!(
            _update_role(
                None, // From ACCOUNT1
                None, // On RoleId 1
                Some(
                    self::role_update(
                        None, // No changes for disabled
                        None, // No content changes
                        Some(
                            BTreeSet::from_iter(self::permission_set_default().into_iter())
                        ) // The same permissions_set (no changes should apply)
                    )
                )
            )
        );

        // Check whether data in Role structure is correct
        let role = Roles::role_by_id(ROLE1).unwrap();

        assert!(role.updated.is_none());
        assert_eq!(
            role.permissions,
            BTreeSet::from_iter(self::permission_set_default().into_iter())
        );
    });
}

#[test]
fn update_role_should_work_with_a_few_roles() {
    ExtBuilder::build_with_a_few_roles_granted_to_account2().execute_with(|| {
        assert_ok!(
            _update_role(
                Some(Origin::signed(ACCOUNT2)),
                Some(ROLE1),
                Some(self::role_update(
                    None,
                    None,
                    Some(BTreeSet::from_iter(self::permission_set_updated().into_iter()))
                ))
            )
        );

        // Check whether Role is stored correctly
        assert!(Roles::role_by_id(ROLE1).is_some());

        // Check whether data in Role structure is correct
        let role = Roles::role_by_id(ROLE1).unwrap();

        assert!(role.updated.is_some());
        assert_eq!(role.space_id, SPACE1);
        assert_eq!(role.disabled, false);
        assert_eq!(role.content, self::default_role_content_ipfs());
        assert_eq!(
            role.permissions,
            BTreeSet::from_iter(self::permission_set_updated().into_iter())
        );
    });
}

#[test]
fn update_role_should_work_not_updated_all_the_same() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_create_default_role()); // RoleId 1
        assert_ok!(
            _update_role(
                None, // From ACCOUNT1
                None, // On RoleId 1
                Some(
                    self::role_update(
                        Some(false),
                        Some(self::default_role_content_ipfs()),
                        Some(BTreeSet::from_iter(self::permission_set_default().into_iter()))
                    )
                )
            )
        );

        // Check whether Role is stored correctly
        assert!(Roles::role_by_id(ROLE1).is_some());

        // Check whether data in Role structure is correct
        let role = Roles::role_by_id(ROLE1).unwrap();

        assert!(role.updated.is_none());
        assert_eq!(role.space_id, SPACE1);
        assert_eq!(role.disabled, false);
        assert_eq!(role.content, self::default_role_content_ipfs());
        assert_eq!(
            role.permissions,
            BTreeSet::from_iter(self::permission_set_default().into_iter())
        );
    });
}

#[test]
fn update_role_should_fail_with_role_not_found() {
    ExtBuilder::build().execute_with(|| {
        assert_noop!(_update_default_role(), Error::<Test>::RoleNotFound);
    });
}

#[test]
fn update_role_should_fail_with_no_permission() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_create_default_role()); // RoleId 1
        assert_noop!(
            _update_role(
                Some(Origin::signed(ACCOUNT2)),
                None, // On RoleId 1
                None // With RoleUpdate that updates every mutable (updatable) field
            ), Error::<Test>::NoPermissionToManageRoles
        );
    });
}

#[test]
fn update_role_should_fail_with_no_role_updates() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_create_default_role()); // RoleId 1
        assert_noop!(_update_role(
            None, // From ACCOUNT1
            None, // On RoleId 1
            Some(self::role_update(None, None, None))
        ), Error::<Test>::NoUpdatesProvided);
    });
}

#[test]
fn update_role_should_fail_with_ipfs_is_incorrect() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_create_default_role()); // RoleId 1
        assert_noop!(_update_role(
            None, // From ACCOUNT1
            None, // On RoleId 1
            Some(self::role_update(None, Some(self::invalid_role_content_ipfs()), None))
        ), UtilsError::<Test>::InvalidIpfsCid);
    });
}

#[test]
fn update_role_should_fail_with_a_few_roles_no_permission() {
    ExtBuilder::build_with_a_few_roles_granted_to_account2().execute_with(|| {
        assert_ok!(_delete_role(None, Some(ROLE2)));
        assert_noop!(
            _update_role(
                Some(Origin::signed(ACCOUNT2)),
                None, // On RoleId 1
                Some(self::role_update(
                    None,
                    None,
                    Some(BTreeSet::from_iter(self::permission_set_default().into_iter()))
                ))
            ), Error::<Test>::NoPermissionToManageRoles
        );
    });
}

#[test]
fn grant_role_should_work() {
    ExtBuilder::build().execute_with(|| {
        let user = User::Account(ACCOUNT2);

        assert_ok!(_create_default_role()); // RoleId 1
        assert_ok!(_grant_default_role()); // Grant RoleId 1 to ACCOUNT2

        // Change whether data was stored correctly
        assert_eq!(Roles::users_by_role_id(ROLE1), vec![user.clone()]);
        assert_eq!(Roles::role_ids_by_user_in_space((user, SPACE1)), vec![ROLE1]);
    });
}

#[test]
fn grant_role_should_work_with_a_few_roles() {
    ExtBuilder::build_with_a_few_roles_granted_to_account2().execute_with(|| {
        let user = User::Account(ACCOUNT3);
        assert_ok!(
            _grant_role(
                Some(Origin::signed(ACCOUNT2)),
                None, // RoleId 1
                Some(vec![User::Account(ACCOUNT3)])
            )
        );

        // Check whether data is stored correctly
        assert_eq!(Roles::users_by_role_id(ROLE1), vec![User::Account(ACCOUNT2), User::Account(ACCOUNT3)]);
        assert_eq!(Roles::role_ids_by_user_in_space((user, SPACE1)), vec![ROLE1]);
    });
}

#[test]
fn grant_role_should_fail_with_role_not_found() {
    ExtBuilder::build().execute_with(|| {
        assert_noop!(_grant_default_role(), Error::<Test>::RoleNotFound);
    });
}

#[test]
fn grant_role_should_fail_with_no_permission() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_create_default_role()); // RoleId 1
        assert_noop!(
            _grant_role(
                Some(Origin::signed(ACCOUNT2)),
                None, // RoleId 1
                Some(vec![User::Account(ACCOUNT3)])
            ), Error::<Test>::NoPermissionToManageRoles
        );
    });
}

#[test]
fn grant_role_should_fail_with_no_users_provided() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_create_default_role()); // RoleId 1
        assert_noop!(
            _grant_role(
                None, // From ACCOUNT1
                None, // RoleId 1
                Some(vec![])
            ), Error::<Test>::NoUsersProvided
        );
    });
}

#[test]
fn grant_role_should_fail_with_a_few_roles_no_permission() {
    ExtBuilder::build_with_a_few_roles_granted_to_account2().execute_with(|| {
        assert_ok!(_delete_role(None, Some(ROLE2)));
        assert_noop! (
            _grant_role(
                Some(Origin::signed(ACCOUNT2)),
                None, // RoleId 1
                Some(vec![User::Account(ACCOUNT3)])
            ), Error::<Test>::NoPermissionToManageRoles
        );
    });
}

#[test]
fn revoke_role_should_work() {
    ExtBuilder::build().execute_with(|| {
        let user = User::Account(ACCOUNT2);

        assert_ok!(_create_default_role()); // RoleId 1
        assert_ok!(_grant_default_role()); // Grant RoleId 1 to ACCOUNT2
        assert_ok!(_revoke_default_role()); // Revoke RoleId 1 from ACCOUNT2

        // Change whether data was stored correctly
        assert!(Roles::users_by_role_id(ROLE1).is_empty());
        assert!(Roles::role_ids_by_user_in_space((user, SPACE1)).is_empty());
    });
}

#[test]
fn revoke_role_should_work_with_a_few_roles() {
    ExtBuilder::build_with_a_few_roles_granted_to_account2().execute_with(|| {
        let user = User::Account(ACCOUNT3);
        assert_ok!(
            _revoke_role(
                Some(Origin::signed(ACCOUNT2)),
                None, // RoleId 1
                Some(vec![User::Account(ACCOUNT2)])
            )
        );

        // Check whether data is stored correctly
        assert!(Roles::users_by_role_id(ROLE1).is_empty());
        assert!(Roles::role_ids_by_user_in_space((user, SPACE1)).is_empty());
    });
}

#[test]
fn revoke_role_should_fail_with_role_not_found() {
    ExtBuilder::build().execute_with(|| {
        assert_noop!(_revoke_default_role(), Error::<Test>::RoleNotFound);
    });
}

#[test]
fn revoke_role_should_fail_with_no_users_provided() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_create_default_role()); // RoleId 1
        assert_noop!(_revoke_role(None, None, Some(vec![])), Error::<Test>::NoUsersProvided);
    });
}

#[test]
fn revoke_role_should_fail_with_no_permission() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_create_default_role()); // RoleId 1
        assert_noop!(
            _revoke_role(
                Some(Origin::signed(ACCOUNT2)),
                None, // RoleId 1
                Some(vec![User::Account(ACCOUNT3)])
            ),
            Error::<Test>::NoPermissionToManageRoles
        );
    });
}

#[test]
fn revoke_role_should_fail_with_a_few_roles_no_permission() {
    ExtBuilder::build_with_a_few_roles_granted_to_account2().execute_with(|| {
        assert_ok!(_delete_role(None, Some(ROLE2)));
        assert_noop! (
            _revoke_role(
                Some(Origin::signed(ACCOUNT2)),
                None, // RoleId 1
                Some(vec![User::Account(ACCOUNT3)])
            ), Error::<Test>::NoPermissionToManageRoles
        );
    });
}

#[test]
fn delete_role_should_work() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_create_default_role()); // RoleId 1
        assert_ok!(_grant_default_role());
        assert_ok!(_delete_default_role());

        // Check whether storages are cleaned up
        assert!(Roles::role_by_id(ROLE1).is_none());
        assert!(Roles::users_by_role_id(ROLE1).is_empty());
        assert!(Roles::role_ids_by_space_id(SPACE1).is_empty());
        assert!(Roles::role_ids_by_user_in_space((User::Account(ACCOUNT2), SPACE1)).is_empty());
        assert_eq!(Roles::next_role_id(), ROLE2);
    });
}

#[test]
fn delete_role_should_work_with_a_few_roles() {
    ExtBuilder::build_with_a_few_roles_granted_to_account2().execute_with(|| {
        assert_ok!(
            _delete_role(
                Some(Origin::signed(ACCOUNT2)),
                None // RoleId 1
            )
        );

        // Check whether storages are cleaned up
        assert!(Roles::role_by_id(ROLE1).is_none());
        assert!(Roles::users_by_role_id(ROLE1).is_empty());
        assert_eq!(Roles::role_ids_by_space_id(SPACE1), vec![ROLE2]);
        assert_eq!(Roles::role_ids_by_user_in_space((User::Account(ACCOUNT2), SPACE1)), vec![ROLE2]);
        assert_eq!(Roles::next_role_id(), ROLE3);
    });
}

#[test]
fn delete_role_should_fail_with_role_not_found() {
    ExtBuilder::build().execute_with(|| {
        assert_noop!(_delete_default_role(), Error::<Test>::RoleNotFound);
    });
}

#[test]
fn delete_role_should_fail_with_no_permission() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_create_default_role()); // RoleId 1
        assert_noop!(
            _delete_role(
                Some(Origin::signed(ACCOUNT2)),
                None // RoleId 1
            ), Error::<Test>::NoPermissionToManageRoles
        );
    });
}

#[test]
fn delete_role_should_fail_with_too_many_users_for_delete_role() {
    ExtBuilder::build().execute_with(|| {
        let mut users: Vec<User<AccountId>> = Vec::new();
        for account in 2..23 {
            users.push(User::Account(account));
        }

        assert_ok!(_create_default_role()); // RoleId 1
        assert_ok!(_grant_role(None, None, Some(users))); // Grant RoleId 1 to ACCOUNT2-ACCOUNT20
        assert_noop!(_delete_default_role(), Error::<Test>::TooManyUsersToDelete);
    });
}

#[test]
fn delete_role_should_fail_with_a_few_roles_no_permission() {
    ExtBuilder::build_with_a_few_roles_granted_to_account2().execute_with(|| {
        assert_ok!(_delete_role(None, Some(ROLE2)));
        assert_noop! (
            _delete_role(
                Some(Origin::signed(ACCOUNT2)),
                None // RoleId 1
            ), Error::<Test>::NoPermissionToManageRoles
        );
    });
}
