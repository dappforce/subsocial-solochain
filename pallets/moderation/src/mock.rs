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

use frame_system::{self as system, EnsureRoot};
use sp_io::TestExternalities;

use pallet_utils::{Content, SpaceId, PostId};
use pallet_spaces::{RESERVED_SPACE_COUNT, SpaceById};
use pallet_posts::PostExtension;

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
    type PalletInfo = ();
    type AccountData = pallet_balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
}

parameter_types! {
    pub const MinimumPeriod: u64 = 5;
}

impl pallet_timestamp::Trait for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
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
    type AllowTransfers = EnsureRoot<AccountId>;
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
    type WeightInfo = ();
    type MaxLocks = ();
}

use pallet_permissions::default_permissions::DefaultSpacePermissions;

impl pallet_permissions::Trait for Test {
    type DefaultSpacePermissions = DefaultSpacePermissions;
}

impl pallet_spaces::Trait for Test {
    type Event = ();
    type Currency = Balances;
    type Roles = Roles;
    type SpaceFollows = SpaceFollows;
    type BeforeSpaceCreated = SpaceFollows;
    type AfterSpaceUpdated = ();
    type IsAccountBlocked = Moderation;
    type IsContentBlocked = Moderation;
    type HandleDeposit = ();
}

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
type Posts = pallet_posts::Module<Test>;
type Roles = pallet_roles::Module<Test>;

pub type AccountId = u64;

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
            create_space_and_post();
        });

        ext
    }

    pub fn build_with_space_and_post_then_report() -> TestExternalities {
        let storage = system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();

        let mut ext = TestExternalities::from(storage);
        ext.execute_with(|| {
            System::set_block_number(1);

            create_space_and_post();
            assert_ok!(_report_default_post());
        });

        ext
    }

    pub fn build_with_report_then_remove_scope() -> TestExternalities {
        let storage = system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();

        let mut ext = TestExternalities::from(storage);
        ext.execute_with(|| {
            System::set_block_number(1);

            create_space_and_post();
            assert_ok!(_report_default_post());

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

pub(crate) const REPORT1: ReportId = 1;
pub(crate) const REPORT2: ReportId = 2;

pub(crate) const AUTOBLOCK_THRESHOLD: u16 = 5;

pub(crate) const fn new_autoblock_threshold() -> SpaceModerationSettingsUpdate {
    SpaceModerationSettingsUpdate {
        autoblock_threshold: Some(Some(AUTOBLOCK_THRESHOLD))
    }
}

pub(crate) const fn empty_moderation_settings_update() -> SpaceModerationSettingsUpdate {
    SpaceModerationSettingsUpdate {
        autoblock_threshold: None
    }
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

pub(crate) fn _report_default_post() -> DispatchResult {
    _report_entity(None, None, None, None)
}

pub(crate) fn _report_entity(
    origin: Option<Origin>,
    entity: Option<EntityId<AccountId>>,
    scope: Option<SpaceId>,
    reason: Option<Content>,
) -> DispatchResult {
    Moderation::report_entity(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT_SCOPE_OWNER)),
        entity.unwrap_or(EntityId::Post(POST1)),
        scope.unwrap_or(SPACE1),
        reason.unwrap_or_else(|| valid_content_ipfs()),
    )
}

pub(crate) fn _suggest_blocked_status_for_post() -> DispatchResult {
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
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT_SCOPE_OWNER)),
        entity.unwrap_or(EntityId::Post(POST1)),
        scope.unwrap_or(SPACE1),
        status.unwrap_or(Some(EntityStatus::Blocked)),
        report_id_opt.unwrap_or(Some(REPORT1)),
    )
}

pub(crate) fn _update_post_status_to_allowed() -> DispatchResult {
    _update_entity_status(None, None, None, None)
}

pub(crate) fn _update_entity_status(
    origin: Option<Origin>,
    entity: Option<EntityId<AccountId>>,
    scope: Option<SpaceId>,
    status_opt: Option<Option<EntityStatus>>,
) -> DispatchResult {
    Moderation::update_entity_status(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT_SCOPE_OWNER)),
        entity.unwrap_or(EntityId::Post(POST1)),
        scope.unwrap_or(SPACE1),
        status_opt.unwrap_or(Some(EntityStatus::Allowed)),
    )
}

pub(crate) fn _delete_post_status() -> DispatchResult {
    _delete_entity_status(None, None, None)
}

pub(crate) fn _delete_entity_status(
    origin: Option<Origin>,
    entity: Option<EntityId<AccountId>>,
    scope: Option<SpaceId>,
) -> DispatchResult {
    Moderation::delete_entity_status(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT_SCOPE_OWNER)),
        entity.unwrap_or(EntityId::Post(POST1)),
        scope.unwrap_or(SPACE1),
    )
}

pub(crate) fn _update_autoblock_threshold_in_moderation_settings() -> DispatchResult {
    _update_moderation_settings(None, None, None)
}

pub(crate) fn _update_moderation_settings(
    origin: Option<Origin>,
    space_id: Option<SpaceId>,
    settings_update: Option<SpaceModerationSettingsUpdate>,
) -> DispatchResult {
    Moderation::update_moderation_settings(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT_SCOPE_OWNER)),
        space_id.unwrap_or(SPACE1),
        settings_update.unwrap_or_else(|| new_autoblock_threshold()),
    )
}


