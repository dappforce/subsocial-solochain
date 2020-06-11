use crate::*;
use frame_support::{
    assert_ok, assert_noop,
    impl_outer_origin, parameter_types,
    weights::Weight,
    dispatch::DispatchResult,
};
use sp_core::H256;
use sp_io::TestExternalities;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    testing::Header,
    Perbill,
    DispatchError
};

use pallet_permissions::{
    SpacePermission as SP,
    SpacePermissions,
};
use df_traits::{SpaceForRoles};

use pallet_utils::{Error as UtilsError};

impl_outer_origin! {
  pub enum Origin for Test {}
}

#[derive(Clone, Eq, PartialEq)]
pub struct Test;

parameter_types! {
  pub const BlockHashCount: u64 = 250;
  pub const MaximumBlockWeight: Weight = 1024;
  pub const MaximumBlockLength: u32 = 2 * 1024;
  pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
}

impl system::Trait for Test {
    type Origin = Origin;
    type Call = ();
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = ();
    type BlockHashCount = BlockHashCount;
    type MaximumBlockWeight = MaximumBlockWeight;
    type MaximumBlockLength = MaximumBlockLength;
    type AvailableBlockRatio = AvailableBlockRatio;
    type Version = ();
    type ModuleToIndex = ();
}

parameter_types! {
  pub const MinimumPeriod: u64 = 5;
}

impl pallet_timestamp::Trait for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
}

parameter_types! {
  pub const IpfsHashLen: u32 = 46;
}

impl pallet_utils::Trait for Test {
    type IpfsHashLen = IpfsHashLen;
}

parameter_types! {

  pub const DefaultSpacePermissions: SpacePermissions = SpacePermissions {

    // No permissions disabled by default
    none: None,

    everyone: Some(BTreeSet::from_iter(vec![
      SP::UpdateOwnSubspaces,
      SP::DeleteOwnSubspaces,

      SP::UpdateOwnPosts,
      SP::DeleteOwnPosts,

      SP::CreateComments,
      SP::UpdateOwnComments,
      SP::DeleteOwnComments,

      SP::Upvote,
      SP::Downvote,
      SP::Share
    ].into_iter())),

    // Followers can do everything that everyone else can.
    follower: None,

    space_owner: Some(BTreeSet::from_iter(vec![
      SP::ManageRoles,
      SP::RepresentSpaceInternally,
      SP::RepresentSpaceExternally,
      SP::OverridePostPermissions,

      SP::CreateSubspaces,
      SP::CreatePosts,

      SP::UpdateSpace,
      SP::UpdateAnySubspaces,
      SP::UpdateAnyPosts,

      SP::BlockSubspaces,
      SP::BlockPosts,
      SP::BlockComments,
      SP::BlockUsers
    ].into_iter()))
  };
}

impl pallet_permissions::Trait for Test {
    type DefaultSpacePermissions = DefaultSpacePermissions;
}

parameter_types! {
  pub const MaxUsersToProcessPerDeleteRole: u16 = 20;
}

impl Trait for Test {
    type Event = ();
    type MaxUsersToProcessPerDeleteRole = MaxUsersToProcessPerDeleteRole;
    type Spaces = Roles;
}

type System = system::Module<Test>;
type Roles = Module<Test>;

pub type AccountId = u64;
pub type BlockNumber = u64;

const ACCOUNT1: AccountId = 1;
const ACCOUNT2: AccountId = 2;
const _ACCOUNT3: AccountId = 3;

const ROLE1: RoleId = 1;
const ROLE2: RoleId = 2;

const SPACE1: SpaceId = 1;
const SPACE2: SpaceId = 2;

impl<T: Trait> SpaceForRolesProvider for Module<T> {
    type AccountId = u64;
    type SpaceId = u64;

    // This function should return an error every time Space doesn't exist by SpaceId
    // Currently, we have a list of valid space id's to check
    fn get_space(id: Self::SpaceId) -> Result<SpaceForRoles<Self::AccountId>, DispatchError> {
        if self::valid_space_ids().contains(&id) {
            return Ok(SpaceForRoles { owner: ACCOUNT1, permissions: None })
        }

        Err("SpaceNotFound".into())
    }

    // No need to use function within this pallet, returns true by default
    fn is_space_follower(_account: Self::AccountId, _space_id: Self::SpaceId) -> bool {
        true
    }
}

pub struct ExtBuilder;

impl ExtBuilder {
    pub fn build() -> TestExternalities {
        let storage = system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();
        let mut ext = TestExternalities::from(storage);
        ext.execute_with(|| System::set_block_number(1));
        ext
    }
}

fn default_role_ipfs_hash() -> Option<Vec<u8>> {
    Option::from(b"QmRAQB6YaCyidP37UdDnjFY5vQuiBrcqdyoW1CuDgwxkD4".to_vec())
}

fn updated_role_ipfs_hash() -> Option<Vec<u8>> {
    Option::from(b"QmZENA8YaCyidP37UdDnjFY5vQuiBrcqdyoW1CuDaazhR8".to_vec())
}

fn invalid_role_ipfs_hash() -> Option<Vec<u8>> {
    Option::from(b"QmRAQB6DaazhR8".to_vec())
}

/// Permissions Set that includes next permission: ManageRoles
fn permission_set_default() -> Vec<SpacePermission> {
    vec![SP::ManageRoles]
}

/// Permissions Set that includes next permissions: ManageRoles, CreatePosts
fn permission_set_updated() -> Vec<SpacePermission> {
    vec![SP::ManageRoles, SP::CreatePosts]
}

fn valid_space_ids() -> Vec<SpaceId> {
    vec![SPACE1]
}

/// Permissions Set that includes nothing
fn permission_set_empty() -> Vec<SpacePermission> {
    vec![]
}

fn role_update(disabled: Option<bool>, ipfs_hash: Option<Option<Vec<u8>>>, permissions: Option<BTreeSet<SpacePermission>>) -> RoleUpdate {
    RoleUpdate {
        disabled,
        ipfs_hash,
        permissions,
    }
}

fn _create_default_role() -> DispatchResult {
    _create_role(None, None, None, None, None)
}

fn _create_role(
    origin: Option<Origin>,
    space_id: Option<SpaceId>,
    time_to_live: Option<Option<BlockNumber>>,
    ipfs_hash: Option<Option<Vec<u8>>>,
    permissions: Option<Vec<SpacePermission>>,
) -> DispatchResult {
    Roles::create_role(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
        space_id.unwrap_or(SPACE1),
        time_to_live.unwrap_or_default(), // Should return 'None'
        ipfs_hash.unwrap_or_else(self::default_role_ipfs_hash),
        permissions.unwrap_or_else(self::permission_set_default),
    )
}

fn _update_default_role() -> DispatchResult {
    _update_role(None, None, None)
}

fn _update_role(
    origin: Option<Origin>,
    role_id: Option<RoleId>,
    update: Option<RoleUpdate>
) -> DispatchResult {
    Roles::update_role(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
        role_id.unwrap_or(ROLE1),
        update.unwrap_or_else(|| self::role_update(
            Some(true),
            Some(self::updated_role_ipfs_hash()),
            Some(
                BTreeSet::from_iter(self::permission_set_updated().into_iter())
            )
        )),
    )
}

fn _delete_default_role() -> DispatchResult {
    _delete_role(None, None)
}

fn _delete_role(
    origin: Option<Origin>,
    role_id: Option<RoleId>
) -> DispatchResult {
    Roles::delete_role(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
        role_id.unwrap_or(ROLE1)
    )
}

fn _grant_default_role() -> DispatchResult {
    _grant_role(None, None, None)
}

fn _grant_role(
    origin: Option<Origin>,
    role_id: Option<RoleId>,
    users: Option<Vec<User<AccountId>>>
) -> DispatchResult {
    Roles::grant_role(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
        role_id.unwrap_or(ROLE1),
        users.unwrap_or_else(|| vec![User::Account(ACCOUNT2)])
    )
}

fn _revoke_default_role() -> DispatchResult {
    _revoke_role(None, None, None)
}

fn _revoke_role(
    origin: Option<Origin>,
    role_id: Option<RoleId>,
    users: Option<Vec<User<AccountId>>>
) -> DispatchResult {
    Roles::revoke_role(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
        role_id.unwrap_or(ROLE1),
        users.unwrap_or_else(|| vec![User::Account(ACCOUNT2)])
    )
}

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
        assert_eq!(role.ipfs_hash, self::default_role_ipfs_hash());
        assert_eq!(
            role.permissions,
            BTreeSet::from_iter(self::permission_set_default().into_iter())
        );
    });
}

#[test]
fn create_role_should_fail_with_space_not_found() {
    ExtBuilder::build().execute_with(|| {
        assert_noop!(
            _create_role(
                None,
                Some(SPACE2),
                None,
                None,
                None
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
                None,
                None,
                None,
                None
            ), Error::<Test>::NoPermissionToManageRoles
        );
    });
}

#[test]
fn create_role_should_fail_with_no_permissions_provided() {
    ExtBuilder::build().execute_with(|| {
        assert_noop!(
            _create_role(
                None,
                None,
                None,
                None,
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
            None,
            None,
            None,
            Some(self::invalid_role_ipfs_hash()),
            None
        ), UtilsError::<Test>::IpfsIsIncorrect);
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
        assert_eq!(role.ipfs_hash, self::updated_role_ipfs_hash());
        assert_eq!(
            role.permissions,
            BTreeSet::from_iter(self::permission_set_updated().into_iter())
        );
    });
}

#[test]
fn update_role_should_work_empty_set() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_create_default_role()); // RoleId 1
        assert_ok!(
            _update_role(
                None,
                None,
                Some(
                    self::role_update(
                        Some(true),
                        None,
                        Some(BTreeSet::from_iter(self::permission_set_empty().into_iter()))
                    )
                )
            )
        );

        // Check whether Role is stored correctly
        assert!(Roles::role_by_id(ROLE1).is_some());

        // Check whether data in Role structure is correct
        let role = Roles::role_by_id(ROLE1).unwrap();

        assert!(role.updated.is_some());
        assert_eq!(role.space_id, SPACE1);
        assert_eq!(role.disabled, true);
        assert_eq!(role.ipfs_hash, self::default_role_ipfs_hash());
        assert_eq!(
            role.permissions,
            BTreeSet::from_iter(self::permission_set_default().into_iter())
        );
    });
}

#[test]
fn update_role_should_work_not_updated_all_the_same() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_create_default_role()); // RoleId 1
        assert_ok!(
            _update_role(
                None,
                None,
                Some(
                    self::role_update(
                        Some(false),
                        Some(self::default_role_ipfs_hash()),
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
        assert_eq!(role.ipfs_hash, self::default_role_ipfs_hash());
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
                None,
                None
            ), Error::<Test>::NoPermissionToManageRoles
        );
    });
}

#[test]
fn update_role_should_fail_with_no_role_updates() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_create_default_role()); // RoleId 1
        assert_noop!(_update_role(
            None,
            None,
            Some(self::role_update(None, None, None))
        ), Error::<Test>::NoRoleUpdates);
    });
}

#[test]
fn update_role_should_fail_with_ipfs_is_incorrect() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_create_default_role()); // RoleId 1
        assert_noop!(_update_role(
            None,
            None,
            Some(self::role_update(None, Some(self::invalid_role_ipfs_hash()), None))
        ), UtilsError::<Test>::IpfsIsIncorrect);
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
        assert_eq!(Roles::in_space_role_ids_by_user((user, SPACE1)), vec![ROLE1]);
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
                None,
                None
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
                None,
                None,
                Some(vec![])
            ), Error::<Test>::NoUsersProvided
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
        assert!(Roles::in_space_role_ids_by_user((user, SPACE1)).is_empty());
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
                None,
                None
            ),
            Error::<Test>::NoPermissionToManageRoles
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
        assert!(Roles::in_space_role_ids_by_user((User::Account(ACCOUNT2), SPACE1)).is_empty());
        assert_eq!(Roles::next_role_id(), ROLE2);
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
                None
            ), Error::<Test>::NoPermissionToManageRoles
        );
    });
}

#[test]
fn delete_role_should_fail_with_too_many_users_for_delete_role() {
    ExtBuilder::build().execute_with(|| {
        let mut users: Vec<User<AccountId>> = Vec::new();
        for account in 2..22 {
            users.push(User::Account(account));
        }

        assert_ok!(_create_default_role()); // RoleId 1
        assert_ok!(_grant_role(None, None, Some(users))); // Grant RoleId 1 to ACCOUNT2-ACCOUNT20
        assert_noop!(_delete_default_role(), Error::<Test>::TooManyUsersForDeleteRole);
    });
}
