use crate::{Module, Trait, RoleId, RoleUpdate};

use sp_core::H256;
use sp_std::{
    collections::btree_set::BTreeSet,
    prelude::Vec,
    iter::FromIterator
};
use sp_io::TestExternalities;

use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup}, testing::Header, Perbill,
};
use frame_support::{
    impl_outer_origin, parameter_types, assert_ok,
    weights::Weight,
    dispatch::{DispatchResult, DispatchError}
};
use frame_system as system;

use pallet_permissions::{
    SpacePermission,
    SpacePermission as SP,
    SpacePermissions,
    SpacePermissionSet
};
use df_traits::{SpaceForRoles, SpaceFollowsProvider, SpaceForRolesProvider};
use pallet_utils::{SpaceId, User, Content};

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
    type BaseCallFilter = ();
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
    type DbWeight = ();
    type BlockExecutionWeight = ();
    type ExtrinsicBaseWeight = ();
    type MaximumExtrinsicWeight = MaximumBlockWeight;
    type MaximumBlockLength = MaximumBlockLength;
    type AvailableBlockRatio = AvailableBlockRatio;
    type Version = ();
    type ModuleToIndex = ();
    type AccountData = pallet_balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
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
	pub const ExistentialDeposit: u64 = 1;
}

impl pallet_balances::Trait for Test {
    type Balance = u64;
    type DustRemoval = ();
    type Event = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
}

parameter_types! {
    pub const MinHandleLen: u32 = 5;
    pub const MaxHandleLen: u32 = 50;
}

impl pallet_utils::Trait for Test {
    type Event = ();
    type Currency = Balances;
    type MinHandleLen = MinHandleLen;
    type MaxHandleLen = MaxHandleLen;
}

parameter_types! {
  pub DefaultSpacePermissions: SpacePermissions = SpacePermissions {

    // No permissions disabled by default
    none: None,

    everyone: Some(SpacePermissionSet::from_iter(vec![
	  SP::UpdateOwnSubspaces,
	  SP::DeleteOwnSubspaces,
	  SP::HideOwnSubspaces,

	  SP::UpdateOwnPosts,
	  SP::DeleteOwnPosts,
	  SP::HideOwnPosts,

	  SP::CreateComments,
	  SP::UpdateOwnComments,
	  SP::DeleteOwnComments,
	  SP::HideOwnComments,

	  SP::Upvote,
	  SP::Downvote,
	  SP::Share,
    ].into_iter())),

    // Followers can do everything that everyone else can.
    follower: None,

    space_owner: Some(SpacePermissionSet::from_iter(vec![
      SP::ManageRoles,
      SP::RepresentSpaceInternally,
      SP::RepresentSpaceExternally,
      SP::OverrideSubspacePermissions,
      SP::OverridePostPermissions,

      SP::CreateSubspaces,
      SP::CreatePosts,

      SP::UpdateSpace,
      SP::UpdateAnySubspace,
      SP::UpdateAnyPost,

      SP::DeleteAnySubspace,
      SP::DeleteAnyPost,

      SP::HideAnySubspace,
      SP::HideAnyPost,
      SP::HideAnyComment,

      SP::SuggestEntityStatus,
      SP::UpdateEntityStatus,
    ].into_iter())),
  };
}

impl pallet_permissions::Trait for Test {
    type DefaultSpacePermissions = DefaultSpacePermissions;
}

parameter_types! {
  pub const MaxUsersToProcessPerDeleteRole: u16 = 20;
}

impl df_traits::moderation::IsAccountBlocked for Test {
    type AccountId = u64;

    fn is_account_blocked(_account: Self::AccountId, _scope: SpaceId) -> bool {
        false
    }
}

impl Trait for Test {
    type Event = ();
    type MaxUsersToProcessPerDeleteRole = MaxUsersToProcessPerDeleteRole;
    type Spaces = Roles;
    type SpaceFollows = Roles;
    type IsAccountBlocked = Self;
    type IsContentBlocked = ();
}

type System = system::Module<Test>;
type Balances = pallet_balances::Module<Test>;
pub(crate) type Roles = Module<Test>;

pub type AccountId = u64;
pub type BlockNumber = u64;

impl<T: Trait> SpaceForRolesProvider for Module<T> {
    type AccountId = AccountId;

    // This function should return an error every time Space doesn't exist by SpaceId
    // Currently, we have a list of valid space id's to check
    fn get_space(id: SpaceId) -> Result<SpaceForRoles<Self::AccountId>, DispatchError> {
        if self::valid_space_ids().contains(&id) {
            return Ok(SpaceForRoles { owner: ACCOUNT1, permissions: None })
        }

        Err("SpaceNotFound".into())
    }
}

impl<T: Trait> SpaceFollowsProvider for Module<T> {
    type AccountId = AccountId;

    fn is_space_follower(_account: Self::AccountId, _space_id: u64) -> bool {
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

    pub fn build_with_a_few_roles_granted_to_account2() -> TestExternalities {
        let storage = system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();

        let mut ext = TestExternalities::from(storage);
        ext.execute_with(|| {
            System::set_block_number(1);
            let user = User::Account(ACCOUNT2);

            assert_ok!(
            _create_role(
                None,
                None,
                None,
                None,
                Some(self::permission_set_random())
            )
        ); // RoleId 1
            assert_ok!(_create_default_role()); // RoleId 2

            assert_ok!(_grant_role(None, Some(ROLE1), Some(vec![user.clone()])));
            assert_ok!(_grant_role(None, Some(ROLE2), Some(vec![user])));
        });

        ext
    }
}


pub(crate) const ACCOUNT1: AccountId = 1;
pub(crate) const ACCOUNT2: AccountId = 2;
pub(crate) const ACCOUNT3: AccountId = 3;

pub(crate) const ROLE1: RoleId = 1;
pub(crate) const ROLE2: RoleId = 2;
pub(crate) const ROLE3: RoleId = 3;
pub(crate) const ROLE4: RoleId = 4;

pub(crate) const SPACE1: SpaceId = 1;
pub(crate) const SPACE2: SpaceId = 2;

pub(crate) fn default_role_content_ipfs() -> Content {
    Content::IPFS(b"QmRAQB6YaCyidP37UdDnjFY5vQuiBrcqdyoW1CuDgwxkD4".to_vec())
}

pub(crate) fn updated_role_content_ipfs() -> Content {
    Content::IPFS(b"QmZENA8YaCyidP37UdDnjFY5vQuiBrcqdyoW1CuDaazhR8".to_vec())
}

pub(crate) fn invalid_role_content_ipfs() -> Content {
    Content::IPFS(b"QmRAQB6DaazhR8".to_vec())
}

/// Permissions Set that includes next permission: ManageRoles
pub(crate) fn permission_set_default() -> Vec<SpacePermission> {
    vec![SP::ManageRoles]
}

/// Permissions Set that includes next permissions: ManageRoles, CreatePosts
pub(crate) fn permission_set_updated() -> Vec<SpacePermission> {
    vec![SP::ManageRoles, SP::CreatePosts]
}

/// Permissions Set that includes random permissions
pub(crate) fn permission_set_random() -> Vec<SpacePermission> {
    vec![SP::CreatePosts, SP::UpdateOwnPosts, SP::UpdateAnyPost, SP::UpdateEntityStatus]
}

pub(crate) fn valid_space_ids() -> Vec<SpaceId> {
    vec![SPACE1]
}

/// Permissions Set that includes nothing
pub(crate) fn permission_set_empty() -> Vec<SpacePermission> {
    vec![]
}

pub(crate) fn role_update(disabled: Option<bool>, content: Option<Content>, permissions: Option<BTreeSet<SpacePermission>>) -> RoleUpdate {
    RoleUpdate {
        disabled,
        content,
        permissions,
    }
}


pub(crate) fn _create_default_role() -> DispatchResult {
    _create_role(None, None, None, None, None)
}

pub(crate) fn _create_role(
    origin: Option<Origin>,
    space_id: Option<SpaceId>,
    time_to_live: Option<Option<BlockNumber>>,
    content: Option<Content>,
    permissions: Option<Vec<SpacePermission>>,
) -> DispatchResult {
    Roles::create_role(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
        space_id.unwrap_or(SPACE1),
        time_to_live.unwrap_or_default(), // Should return 'None'
        content.unwrap_or_else(self::default_role_content_ipfs),
        permissions.unwrap_or_else(self::permission_set_default),
    )
}

pub(crate) fn _update_default_role() -> DispatchResult {
    _update_role(None, None, None)
}

pub(crate) fn _update_role(
    origin: Option<Origin>,
    role_id: Option<RoleId>,
    update: Option<RoleUpdate>
) -> DispatchResult {
    Roles::update_role(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
        role_id.unwrap_or(ROLE1),
        update.unwrap_or_else(|| self::role_update(
            Some(true),
            Some(self::updated_role_content_ipfs()),
            Some(
                BTreeSet::from_iter(self::permission_set_updated().into_iter())
            )
        )),
    )
}

pub(crate) fn _grant_default_role() -> DispatchResult {
    _grant_role(None, None, None)
}

pub(crate) fn _grant_role(
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

pub(crate) fn _revoke_default_role() -> DispatchResult {
    _revoke_role(None, None, None)
}

pub(crate) fn _revoke_role(
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

pub(crate) fn _delete_default_role() -> DispatchResult {
    _delete_role(None, None)
}

pub(crate) fn _delete_role(
    origin: Option<Origin>,
    role_id: Option<RoleId>
) -> DispatchResult {
    Roles::delete_role(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
        role_id.unwrap_or(ROLE1)
    )
}