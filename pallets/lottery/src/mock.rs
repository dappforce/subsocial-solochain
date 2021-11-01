use crate::{Module, Trait};
use frame_support::{
	assert_ok, dispatch::DispatchResult, impl_outer_origin, parameter_types, weights::Weight, StorageMap,
};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	Perbill,
};

use frame_system as system;
use sp_io::TestExternalities;

use pallet_posts::PostExtension;
use pallet_spaces::{SpaceById, RESERVED_SPACE_COUNT};
use pallet_utils::{Content, PostId, SpaceId};

pub use pallet_utils::mock_functions::valid_content_ipfs;

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
	type AccountData = pallet_balances::AccountData<u64>;
	type AccountId = u64;
	type AvailableBlockRatio = AvailableBlockRatio;
	type BaseCallFilter = ();
	type BlockExecutionWeight = ();
	type BlockHashCount = BlockHashCount;
	type BlockNumber = u64;
	type Call = ();
	type DbWeight = ();
	type Event = ();
	type ExtrinsicBaseWeight = ();
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type Header = Header;
	type Index = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type MaximumBlockLength = MaximumBlockLength;
	type MaximumBlockWeight = MaximumBlockWeight;
	type MaximumExtrinsicWeight = MaximumBlockWeight;
	type OnKilledAccount = ();
	type OnNewAccount = ();
	type Origin = Origin;
	type PalletInfo = ();
	type SystemWeightInfo = ();
	type Version = ();
}

parameter_types! {
	pub const MinimumPeriod: u64 = 5;
}
impl pallet_roles::Trait for Test {
	type Event = ();
	type IsAccountBlocked = ();
	type IsContentBlocked = ();
	type MaxUsersToProcessPerDeleteRole = MaxUsersToProcessPerDeleteRole;
	type SpaceFollows = SpaceFollows;
	type Spaces = Spaces;
}
impl pallet_timestamp::Trait for Test {
	type MinimumPeriod = MinimumPeriod;
	type Moment = u64;
	type OnTimestampSet = ();
	type WeightInfo = ();
}

parameter_types! {
	pub const MinHandleLen: u32 = 5;
	pub const MaxHandleLen: u32 = 50;
}

impl pallet_utils::Trait for Test {
	type Currency = Balances;
	type Event = ();
	type MaxHandleLen = MaxHandleLen;
	type MinHandleLen = MinHandleLen;
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
}

impl pallet_balances::Trait for Test {
	type AccountStore = System;
	type Balance = u64;
	type DustRemoval = ();
	type Event = ();
	type ExistentialDeposit = ExistentialDeposit;
	type MaxLocks = ();
	type WeightInfo = ();
}

use frame_support::traits::Currency;
use pallet_permissions::default_permissions::DefaultSpacePermissions;

impl pallet_permissions::Trait for Test {
	type DefaultSpacePermissions = DefaultSpacePermissions;
}

impl pallet_spaces::Trait for Test {
	type AfterSpaceUpdated = ();
	type BeforeSpaceCreated = SpaceFollows;
	type Currency = Balances;
	type Event = ();
	type HandleDeposit = ();
	type IsAccountBlocked = ();
	type IsContentBlocked = ();
	type Roles = Roles;
	type SpaceFollows = SpaceFollows;
}

impl pallet_space_follows::Trait for Test {
	type BeforeSpaceFollowed = ();
	type BeforeSpaceUnfollowed = ();
	type Event = ();
}

parameter_types! {
	pub const MaxCommentDepth: u32 = 10;
}

impl pallet_posts::Trait for Test {
	type AfterPostUpdated = ();
	type Event = ();
	type IsPostBlocked = ();
	type MaxCommentDepth = MaxCommentDepth;
	type PostScores = ();
}

parameter_types! {
	pub const MaxUsersToProcessPerDeleteRole: u16 = 40;
}

impl pallet_profiles::Trait for Test {
	type AfterProfileUpdated = ();
	type Event = ();
}

parameter_types! {
	pub const DefaultAutoblockThreshold: u16 = 20;
}

impl Trait for Test {
	type Currency = Balances;
	type Event = ();
}

pub(crate) type System = system::Module<Test>;
pub(crate) type Lottery = Module<Test>;
type SpaceFollows = pallet_space_follows::Module<Test>;
pub(crate) type Balances = pallet_balances::Module<Test>;
type Spaces = pallet_spaces::Module<Test>;
type Posts = pallet_posts::Module<Test>;
type Roles = pallet_roles::Module<Test>;

pub type AccountId = u64;

pub struct ExtBuilder;

impl ExtBuilder {
	pub fn build() -> TestExternalities {
		let storage = system::GenesisConfig::default().build_storage::<Test>().unwrap();

		let mut ext = TestExternalities::from(storage);
		ext.execute_with(|| System::set_block_number(1));

		ext
	}

	pub fn build_with_space_and_post() -> TestExternalities {
		let storage = system::GenesisConfig::default().build_storage::<Test>().unwrap();

		let mut ext = TestExternalities::from(storage);
		ext.execute_with(|| {
			System::set_block_number(1);
			create_space_and_post();
		});

		ext
	}

	pub fn build_and_vote() -> TestExternalities {
		let mut ext = Self::build_with_space_and_post();
		ext.execute_with(|| vote());
		ext
	}

	pub fn build_with_report_then_remove_scope() -> TestExternalities {
		let storage = system::GenesisConfig::default().build_storage::<Test>().unwrap();

		let mut ext = TestExternalities::from(storage);
		ext.execute_with(|| {
			System::set_block_number(1);

			create_space_and_post();
			SpaceById::<Test>::remove(SPACE1);
		});

		ext
	}
}

pub(crate) const ACCOUNT_SCOPE_OWNER: AccountId = 1;
pub(crate) const ACCOUNT_NOT_MODERATOR: AccountId = 2;

pub(crate) const SPACE1: SpaceId = RESERVED_SPACE_COUNT + 1;
pub(crate) const SPACE2: SpaceId = SPACE1 + 1;

pub(crate) const POST1: PostId = 1;

pub(crate) const AUTOBLOCK_THRESHOLD: u16 = 5;

pub(crate) fn init_account_with_balance(account: &AccountId) {
	Balances::make_free_balance_be(account, 100000000000);
}

pub(crate) fn create_space_and_post() {
	assert_ok!(Spaces::create_space(
		Origin::signed(ACCOUNT_SCOPE_OWNER),
		None,
		None,
		Content::None,
		None
	));

	assert_ok!(Posts::create_post(
		Origin::signed(ACCOUNT_SCOPE_OWNER),
		Some(SPACE1),
		PostExtension::RegularPost,
		valid_content_ipfs(),
	));
}

pub(crate) fn vote() {
	init_account_with_balance(&ACCOUNT_SCOPE_OWNER);
	assert_ok!(Lottery::vote_for_post(Origin::signed(ACCOUNT_SCOPE_OWNER), POST1, 1000));
}
