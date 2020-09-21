use crate::{Module, Trait, EntityId, EntityStatus, ReportId, SpaceModerationSettingsUpdate};
use sp_core::H256;
use frame_support::{
    impl_outer_origin, parameter_types, assert_ok, StorageMap,
    weights::Weight,
    dispatch::{DispatchResult},
};
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup}, testing::Header, Perbill,
};

use frame_system as system;
use sp_io::TestExternalities;

use pallet_utils::{Content, SpaceId};
use pallet_spaces::{Space, SpaceById};
use pallet_posts::{Post, PostId, PostById, PostExtension};

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
        pub const ExistentialDeposit: u64 = 1;
    }

impl pallet_balances::Trait for Test {
    type Balance = u64;
    type DustRemoval = ();
    type Event = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
}

use pallet_permissions::default_permissions::DefaultSpacePermissions;

impl pallet_permissions::Trait for Test {
    type DefaultSpacePermissions = DefaultSpacePermissions;
}

parameter_types! {
    pub const DefaultRPCLimit: u64 = 20;
}

impl pallet_spaces::Trait for Test {
    type Event = ();
    type Roles = Roles;
    type SpaceFollows = SpaceFollows;
    type BeforeSpaceCreated = SpaceFollows;
    type AfterSpaceUpdated = ();
    type IsAccountBlocked = Moderation;
    type IsContentBlocked = Moderation;
    type SpaceCreationFee = ();
    type DefaultRPCLimit = DefaultRPCLimit;
}

parameter_types! {}

impl pallet_space_follows::Trait for Test {
    type Event = ();
    type BeforeSpaceFollowed = ();
    type BeforeSpaceUnfollowed = ();
}

parameter_types! {
        pub const MaxCommentDepth: u32 = 10;
    }

impl pallet_posts::Trait for Test {
    type Event = ();
    type MaxCommentDepth = MaxCommentDepth;
    type PostScores = ();
    type AfterPostUpdated = ();
    type IsPostBlocked = Moderation;
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

parameter_types! {}

impl pallet_profiles::Trait for Test {
    type Event = ();
    type AfterProfileUpdated = ();
}

parameter_types! {
	pub const DefaultAutoblockThreshold: u16 = 20;
}

impl Trait for Test {
    type Event = ();
    type DefaultAutoblockThreshold = DefaultAutoblockThreshold;
}

type System = system::Module<Test>;
pub(crate) type Moderation = Module<Test>;
type SpaceFollows = pallet_space_follows::Module<Test>;
type Balances = pallet_balances::Module<Test>;
type Spaces = pallet_spaces::Module<Test>;
type Roles = pallet_roles::Module<Test>;

pub type AccountId = u64;
pub type AutoblockThreshold = u16;

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

    pub fn build_with_space_and_post() -> TestExternalities {
        let storage = system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();

        let mut ext = TestExternalities::from(storage);
        ext.execute_with(|| {
            System::set_block_number(1);

            let space = Space::<Test>::new(SPACE1, None, ACCOUNT1, Content::None, None);
            SpaceById::insert(space.id, space);
            let post = Post::<Test>::new(
                POST1, ACCOUNT1, Some(SPACE1), PostExtension::SharedPost(POST1), valid_content_ipfs_1());
            PostById::insert(post.id, post);
        });

        ext
    }

    pub fn build_with_report_no_space() -> TestExternalities {
        let storage = system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();

        let mut ext = TestExternalities::from(storage);
        ext.execute_with(|| {
            System::set_block_number(1);

            let space = Space::<Test>::new(SPACE1, None, ACCOUNT1, Content::None, None);
            SpaceById::insert(space.id, space);
            let post = Post::<Test>::new(
                POST1,
                ACCOUNT1,
                Some(SPACE1),
                PostExtension::SharedPost(POST1),
                valid_content_ipfs_1(),
            );
            PostById::insert(post.id, post);

            assert_ok!(_report_default_entity());

            SpaceById::<Test>::remove(SPACE1);
        });

        ext
    }
}

pub(crate) const ACCOUNT1: AccountId = 1;
pub(crate) const ACCOUNT2: AccountId = 2;
pub(crate) const POST1: PostId = 1;
pub(crate) const SPACE1: SpaceId = 1;
pub(crate) const SPACE2: SpaceId = 2;
pub(crate) const REPORT1: ReportId = 1;
pub(crate) const REPORT2: ReportId = 2;
pub(crate) const AUTOBLOCK_THRESHOLD: AutoblockThreshold = 5;


pub(crate) fn valid_content_ipfs_1() -> Content {
    Content::IPFS(b"QmRAQB6YaCyidP37UdDnjFY5vQuiBrcqdyoW1CuDgwxkD4".to_vec())
}

pub(crate) fn invalid_content_ipfs() -> Content {
    Content::IPFS(b"QmRAQB6DaazhR8".to_vec())
}

pub(crate) const fn default_moderation_settings_update() -> SpaceModerationSettingsUpdate {
    SpaceModerationSettingsUpdate {
        autoblock_threshold: Some(Some(AUTOBLOCK_THRESHOLD))
    }
}

pub(crate) const fn empty_moderation_settings_update() -> SpaceModerationSettingsUpdate {
    SpaceModerationSettingsUpdate {
        autoblock_threshold: None
    }
}

pub(crate) fn _report_default_entity() -> DispatchResult {
    _report_entity(None, None, None, None)
}

pub(crate) fn _report_entity(
    origin: Option<Origin>,
    entity: Option<EntityId<AccountId>>,
    scope: Option<SpaceId>,
    reason: Option<Content>,
) -> DispatchResult {
    Moderation::report_entity(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
        entity.unwrap_or(EntityId::Post(POST1)),
        scope.unwrap_or(SPACE1),
        reason.unwrap_or_else(|| self::valid_content_ipfs_1()),
    )
}

pub(crate) fn _suggest_default_entity_status() -> DispatchResult {
    _suggest_entity_status(None, None, None, None, None)
}

pub(crate) fn _suggest_entity_status(
    origin: Option<Origin>,
    entity: Option<EntityId<AccountId>>,
    scope: Option<SpaceId>,
    status: Option<Option<EntityStatus>>,
    report_id_opt: Option<Option<ReportId>>,
) -> DispatchResult {
    Moderation::suggest_entity_status(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
        entity.unwrap_or(EntityId::Post(POST1)),
        scope.unwrap_or(SPACE1),
        status.unwrap_or(Some(EntityStatus::Blocked)),
        report_id_opt.unwrap_or(Some(REPORT1)),
    )
}

pub(crate) fn _update_default_entity_status() -> DispatchResult {
    _update_entity_status(None, None, None, None)
}

pub(crate) fn _update_entity_status(
    origin: Option<Origin>,
    entity: Option<EntityId<AccountId>>,
    scope: Option<SpaceId>,
    status_opt: Option<Option<EntityStatus>>,
) -> DispatchResult {
    Moderation::update_entity_status(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
        entity.unwrap_or(EntityId::Post(POST1)),
        scope.unwrap_or(SPACE1),
        status_opt.unwrap_or(Some(EntityStatus::Allowed)),
    )
}

pub(crate) fn _delete_default_entity_status() -> DispatchResult {
    _delete_entity_status(None, None, None)
}

pub(crate) fn _delete_entity_status(
    origin: Option<Origin>,
    entity: Option<EntityId<AccountId>>,
    scope: Option<SpaceId>,
) -> DispatchResult {
    Moderation::delete_entity_status(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
        entity.unwrap_or(EntityId::Post(POST1)),
        scope.unwrap_or(SPACE1),
    )
}

pub(crate) fn _update_default_moderation_settings() -> DispatchResult {
    _update_moderation_settings(None, None, None)
}

pub(crate) fn _update_moderation_settings(
    origin: Option<Origin>,
    space_id: Option<SpaceId>,
    settings_update: Option<SpaceModerationSettingsUpdate>,
) -> DispatchResult {
    Moderation::update_moderation_settings(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
        space_id.unwrap_or(SPACE1),
        settings_update.unwrap_or_else(|| default_moderation_settings_update()),
    )
}


