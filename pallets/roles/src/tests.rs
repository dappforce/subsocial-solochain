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
    traits::{BlakeTwo256, IdentityLookup}, testing::Header, Perbill,
};

use pallet_permissions::{
    SpacePermission as SP,
    SpacePermissions,
};

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
  pub const MinHandleLen: u32 = 5;
  pub const MaxHandleLen: u32 = 50;
  pub const MinUsernameLen: u32 = 3;
  pub const MaxUsernameLen: u32 = 50;
  pub const FollowSpaceActionWeight: i16 = 7;
  pub const FollowAccountActionWeight: i16 = 3;
  pub const UpvotePostActionWeight: i16 = 5;
  pub const DownvotePostActionWeight: i16 = -3;
  pub const SharePostActionWeight: i16 = 5;
  pub const CreateCommentActionWeight: i16 = 5;
  pub const UpvoteCommentActionWeight: i16 = 4;
  pub const DownvoteCommentActionWeight: i16 = -2;
  pub const ShareCommentActionWeight: i16 = 3;
  pub const MaxCommentDepth: u32 = 10;
}

impl pallet_social::Trait for Test {
    type Event = ();
    type MinHandleLen = MinHandleLen;
    type MaxHandleLen = MaxHandleLen;
    type MinUsernameLen = MinUsernameLen;
    type MaxUsernameLen = MaxUsernameLen;
    type FollowSpaceActionWeight = FollowSpaceActionWeight;
    type FollowAccountActionWeight = FollowAccountActionWeight;
    type UpvotePostActionWeight = UpvotePostActionWeight;
    type DownvotePostActionWeight = DownvotePostActionWeight;
    type SharePostActionWeight = SharePostActionWeight;
    type CreateCommentActionWeight = CreateCommentActionWeight;
    type UpvoteCommentActionWeight = UpvoteCommentActionWeight;
    type DownvoteCommentActionWeight = DownvoteCommentActionWeight;
    type ShareCommentActionWeight = ShareCommentActionWeight;
    type MaxCommentDepth = MaxCommentDepth;
    type Roles = Roles;
}

parameter_types! {
  pub const MaxUsersToProcessPerDeleteRole: u16 = 20;
}

impl Trait for Test {
    type Event = ();
    type MaxUsersToProcessPerDeleteRole = MaxUsersToProcessPerDeleteRole;
    type Spaces = Social;
}

type System = system::Module<Test>;
type Roles = Module<Test>;
type Social = pallet_social::Module<Test>;

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

pub type AccountId = u64;
pub type BlockNumber = u64;

const ACCOUNT1: AccountId = 1;
const _ACCOUNT2: AccountId = 2;
const _ACCOUNT3: AccountId = 3;

fn role_ipfs_hash() -> Option<Vec<u8>> {
    Option::from(b"QmRAQB6YaCyidP37UdDnjFY5vQuiBrcqdyoW1CuDgwxkD4".to_vec())
}

fn permissions_manage_roles() -> Vec<SpacePermission> {
    vec![SpacePermission::ManageRoles]
}

fn permissions_empty() -> Vec<SpacePermission> {
    vec![]
}

fn _role_update(disabled: Option<bool>, ipfs_hash: Option<Option<Vec<u8>>>, permissions: Option<BTreeSet<SpacePermission>>) -> RoleUpdate {
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
        space_id.unwrap_or(1),
        time_to_live.unwrap_or_default(),
        ipfs_hash.unwrap_or_else(self::role_ipfs_hash),
        permissions.unwrap_or_else(self::permissions_manage_roles),
    )
}

#[test]
fn create_role_should_work() {
    ExtBuilder::build().execute_with(|| {
        // assert_ok!()
        assert_ok!(_create_default_role()); // RoleId 1

        // Check whether Role is stored correctly
        assert!(Roles::role_by_id(1).is_some());

        // Check whether data in Role structure is correct
        let role = Roles::role_by_id(1).unwrap();
        assert_eq!(Roles::next_role_id(), 2);

        assert!(role.updated.is_none());
        assert_eq!(role.space_id, 1);
        assert_eq!(role.disabled, false);
        assert_eq!(role.ipfs_hash, self::role_ipfs_hash());
        // assert_eq!(role.roles, self::permission_manage_roles());
    });
}

#[test]
fn create_role_should_fail_post_not_found() {
    ExtBuilder::build().execute_with(|| {
        // TODO: import an error or use SpaceNotFound from this pallet
        // assert_noop!(_create_default_role(), Error::<Test>::SpaceNotFound);
    });
}

#[test]
fn create_role_should_fail_empty_permissions_provided() {
    ExtBuilder::build().execute_with(|| {
        assert_noop!(
            _create_role(
                None,
                None,
                None,
                None,
                Some(self::permissions_empty())
            ),
            Error::<Test>::NoPermissionsProvided
        );
    });
}
