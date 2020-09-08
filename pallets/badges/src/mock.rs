// Creating mock runtime here

use crate::{Module, Trait};
use sp_std::iter::FromIterator;
use sp_core::H256;
use sp_io::TestExternalities;
use sp_runtime::{traits::{BlakeTwo256, IdentityLookup}, testing::Header, Perbill, DispatchResult};
use frame_support::{impl_outer_origin, parameter_types, weights::Weight, StorageMap};
use frame_system as system;

use pallet_permissions::{
	SpacePermission as SP,
	SpacePermissionSet,
	SpacePermissions,
};
use pallet_spaces::{Space, SpaceById};
use pallet_utils::{SpaceId, Content};

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
	pub const MinimumPeriod: u64 = 5;
}

impl pallet_timestamp::Trait for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
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

            SP::UpdateSpaceSettings,

            SP::ManageBadges,
  			SP::ManageAwards,

        ].into_iter())),
      };
    }

impl pallet_permissions::Trait for Test {
	type DefaultSpacePermissions = DefaultSpacePermissions;
}

impl pallet_profiles::Trait for Test {
	type Event = ();
	type AfterProfileUpdated = ();
}

parameter_types! {}

impl pallet_space_follows::Trait for Test {
	type Event = ();
	type BeforeSpaceFollowed = ();
	type BeforeSpaceUnfollowed = ();
}

parameter_types! {}

impl pallet_spaces::Trait for Test {
	type Event = ();
	type Roles = Roles;
	type SpaceFollows = SpaceFollows;
	type BeforeSpaceCreated = ();
	type AfterSpaceUpdated = ();
	type IsAccountBlocked = Moderation;
	type IsContentBlocked = Moderation;
}

parameter_types! {
      pub const IpfsCidLen: u32 = 46;
      pub const MinHandleLen: u32 = 5;
      pub const MaxHandleLen: u32 = 50;
    }

impl pallet_utils::Trait for Test {
	type Event = ();
	type Currency = Balances;
	type IpfsCidLen = IpfsCidLen;
	type MinHandleLen = MinHandleLen;
	type MaxHandleLen = MaxHandleLen;
}

parameter_types! {
        pub const MaxUsersToProcessPerDeleteRole: u16 = 40;
    }

impl pallet_roles::Trait for Test {
	type Event = ();
	type MaxUsersToProcessPerDeleteRole = MaxUsersToProcessPerDeleteRole;
	type Spaces = Spaces;
	type SpaceFollows = SpaceFollows;
	type IsAccountBlocked = Moderation;
	type IsContentBlocked = Moderation;
}

parameter_types! {
	pub const DefaultAutoblockThreshold: u16 = 20;
}

impl pallet_moderation::Trait for Test {
	type Event = ();
	type DefaultAutoblockThreshold = DefaultAutoblockThreshold;
}

parameter_types! {
	pub const MaxCommentDepth: u32 = 10;
}

impl pallet_posts::Trait for Test {
	type Event = ();
	type MaxCommentDepth = MaxCommentDepth;
	type PostScores = ();
	type AfterPostUpdated = ();
}

impl Trait for Test {
}

pub type Badge = Module<Test>;
type System = system::Module<Test>;
type Balances = pallet_balances::Module<Test>;
type Spaces = pallet_spaces::Module<Test>;
type Roles = pallet_roles::Module<Test>;
type SpaceFollows = pallet_space_follows::Module<Test>;
type Moderation = pallet_moderation::Module<Test>;

pub type AccountId = u64;
pub type BadgeId = u64;
pub type SpaceAwardId = u64;
pub type BlockNumber = u64;

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

	pub fn build_with_space() -> TestExternalities {
		let storage = system::GenesisConfig::default()
			.build_storage::<Test>()
			.unwrap();

		let mut ext = TestExternalities::from(storage);
		ext.execute_with(|| {
			System::set_block_number(1);
			let space = Space::<Test>::new(SPACE1, None, ACCOUNT1, Content::None, None);
			SpaceById::insert(space.id, space);
		});

		ext
	}
}

pub const ACCOUNT1: AccountId = 1;
pub const SPACE1: SpaceId = 1;
pub const SPACE2: SpaceId = 2;
pub const BADGEID1: BadgeId = 1;
pub const BADGEID2: BadgeId = 2;
pub const SPACEAWARDID: SpaceAwardId = 1;


pub fn default_badge_content_ipfs() -> Content {
	Content::IPFS(b"QmRAQB6YaCyidP37UdDnjFY5vQuiBrcqdyoW1CuDgwxkD4".to_vec())
}

pub fn updated_badge_content_ipfs() -> Content {
	Content::IPFS(b"QmZENA8YaCyidP37UdDnjFY5vQuiBrcqdyoW1CuDaazhR8".to_vec())
}

pub fn _create_default_badge() -> DispatchResult {
	_create_badge(None, None, None)
}

pub fn _create_badge(
	origin: Option<Origin>,
	space_id: Option<SpaceId>,
	content: Option<Content>
) -> DispatchResult {
	Badge::create_badge(
		origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
		space_id.unwrap_or(SPACE1),
		content.unwrap_or_else(self::default_badge_content_ipfs)
	)
}

pub fn _update_default_badge() -> DispatchResult {
	_update_badge(None, None, None)
}

pub fn _update_badge(
	origin: Option<Origin>,
	badge_id: Option<BadgeId>,
	content: Option<Content>
) -> DispatchResult {
	Badge::update_badge(
		origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
		badge_id.unwrap_or(BADGEID1),
		content.unwrap_or_else(self::updated_badge_content_ipfs)
	)
}

pub fn _delete_default_badge() -> DispatchResult {
	_delete_badge(None, None)
}

pub fn _delete_badge(
	origin: Option<Origin>,
	badge_id: Option<BadgeId>
) -> DispatchResult {
	Badge::delete_badge(
		origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
		badge_id.unwrap_or(BADGEID1)
	)
}

pub fn _award_default_badge() -> DispatchResult {
	_award_badge(None, None, None, None)
}

pub fn _award_badge(
	origin: Option<Origin>,
	badge_id: Option<BadgeId>,
	recipient: Option<SpaceId>,
	expire_after_opt: Option<Option<BlockNumber>>,
) -> DispatchResult {
	Badge::award_badge(
		origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
		badge_id.unwrap_or(BADGEID1),
		recipient.unwrap_or(SPACE2),
		expire_after_opt.unwrap_or_default(),

	)
}

pub fn _accept_default_award() -> DispatchResult {
	_accept_award(None, None)
}

pub fn _accept_award(
	origin: Option<Origin>,
	award_id: Option<SpaceAwardId>,
) -> DispatchResult {
	Badge::accept_award(
		origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
		award_id.unwrap_or(SPACEAWARDID)
	)
}

pub fn _delete_default_award() -> DispatchResult {
	_delete_badge_award(None, None)
}

pub fn _delete_badge_award(
	origin: Option<Origin>,
	space_award_id: Option<SpaceAwardId>,
) -> DispatchResult {
	Badge::delete_badge_award(
		origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
		space_award_id.unwrap_or(SPACEAWARDID)
	)
}


