use super::*;

use crate as moderation;

use frame_support::{assert_ok, dispatch::DispatchResult, parameter_types, StorageMap};
use frame_system as system;

use sp_core::H256;
use sp_io::TestExternalities;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};

use pallet_permissions::{
    SpacePermission as SP,
    default_permissions::DefaultSpacePermissions,
};
use pallet_posts::PostExtension;
use pallet_roles::RoleId;
use pallet_spaces::{RESERVED_SPACE_COUNT, SpaceById};

use pallet_utils::{Content, DEFAULT_MAX_HANDLE_LEN, DEFAULT_MIN_HANDLE_LEN, PostId, SpaceId, User};
pub use pallet_utils::mock_functions::valid_content_ipfs;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: system::{Module, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
        Timestamp: pallet_timestamp::{Module, Call, Storage, Inherent},
        Moderation: moderation::{Module, Call, Storage, Event<T>},
		Posts: pallet_posts::{Module, Call, Storage, Event<T>},
        Profiles: pallet_profiles::{Module, Call, Storage, Event<T>},
		Roles: pallet_roles::{Module, Call, Storage, Event<T>},
		SpaceFollows: pallet_space_follows::{Module, Call, Storage, Event<T>},
		Spaces: pallet_spaces::{Module, Call, Storage, Event<T>, Config<T>},
        Utils: pallet_utils::{Module, Event<T>},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
}

impl system::Config for Test {
    type BaseCallFilter = ();
    type BlockWeights = ();
    type BlockLength = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type DbWeight = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
}

parameter_types! {
    pub const MinimumPeriod: u64 = 5;
}

impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

parameter_types! {
    pub const MinHandleLen: u32 = 5;
    pub const MaxHandleLen: u32 = 50;
}

impl pallet_utils::Config for Test {
    type Event = Event;
    type Currency = Balances;
    type MinHandleLen = MinHandleLen;
    type MaxHandleLen = MaxHandleLen;
}

parameter_types! {
    pub const ExistentialDeposit: u64 = 1;
}

impl pallet_balances::Config for Test {
    type Balance = u64;
    type DustRemoval = ();
    type Event = Event;
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxLocks = ();
}

impl pallet_permissions::Config for Test {
    type DefaultSpacePermissions = DefaultSpacePermissions;
}

impl pallet_spaces::Config for Test {
    type Event = Event;
    type Currency = Balances;
    type Roles = Roles;
    type SpaceFollows = SpaceFollows;
    type BeforeSpaceCreated = SpaceFollows;
    type AfterSpaceUpdated = ();
    type IsAccountBlocked = Moderation;
    type IsContentBlocked = Moderation;
    type HandleDeposit = ();
}

impl pallet_space_follows::Config for Test {
    type Event = Event;
    type BeforeSpaceFollowed = ();
    type BeforeSpaceUnfollowed = ();
}

parameter_types! {
    pub const MaxCommentDepth: u32 = 10;
}

impl pallet_posts::Config for Test {
    type Event = Event;
    type MaxCommentDepth = MaxCommentDepth;
    type PostScores = ();
    type AfterPostUpdated = ();
    type IsPostBlocked = Moderation;
}

parameter_types! {
    pub const MaxUsersToProcessPerDeleteRole: u16 = 40;
}

impl pallet_roles::Config for Test {
    type Event = Event;
    type MaxUsersToProcessPerDeleteRole = MaxUsersToProcessPerDeleteRole;
    type Spaces = Spaces;
    type SpaceFollows = SpaceFollows;
    type IsAccountBlocked = Moderation;
    type IsContentBlocked = Moderation;
}

impl pallet_profiles::Config for Test {
    type Event = Event;
    type AfterProfileUpdated = ();
}

parameter_types! {
    pub const DefaultAutoblockThreshold: u16 = 3;
}

impl Config for Test {
    type Event = Event;
    type DefaultAutoblockThreshold = DefaultAutoblockThreshold;
}

pub(crate) type AccountId = u64;

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

    pub fn build_with_report_then_grant_role_to_suggest_entity_status() -> TestExternalities {
        let mut ext = Self::build_with_space_and_post_then_report();

        ext.execute_with(|| {
            // Create a new role for moderators:
            assert_ok!(Roles::create_role(
                Origin::signed(ACCOUNT_SCOPE_OWNER),
                SPACE1,
                None,
                default_role_content_ipfs(),
                vec![SP::SuggestEntityStatus],
            ));

            // Allow the moderator accounts to suggest entity statuses:
            let mods = moderators().into_iter().map(User::Account).collect();
            assert_ok!(Roles::grant_role(
                Origin::signed(ACCOUNT_SCOPE_OWNER),
                MODERATOR_ROLE_ID,
                mods
            ));
        });

        ext
    }
}

pub(crate) const ACCOUNT_SCOPE_OWNER: AccountId = 1;
pub(crate) const ACCOUNT_NOT_MODERATOR: AccountId = 2;
pub(crate) const FIRST_MODERATOR_ID: AccountId = 100;

pub(crate) const SPACE1: SpaceId = RESERVED_SPACE_COUNT + 1;
pub(crate) const SPACE2: SpaceId = SPACE1 + 1;

pub(crate) const POST1: PostId = 1;

pub(crate) const REPORT1: ReportId = 1;
pub(crate) const REPORT2: ReportId = 2;

pub(crate) const MODERATOR_ROLE_ID: RoleId = 1;

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

pub(crate) fn moderators() -> Vec<AccountId> {
    let first_mod_id = FIRST_MODERATOR_ID;
    let last_mod_id = first_mod_id + DefaultAutoblockThreshold::get() as u64 + 2;
    (first_mod_id..last_mod_id).collect()
}

// TODO: replace with common function when benchmarks PR is merged
// TODO: replace with common function when benchmarks PR is merged
pub(crate) fn default_role_content_ipfs() -> Content {
    Content::IPFS(b"QmRAQB6YaCyidP37UdDnjFY5vQuiBrcqdyoW1CuDgwxkD4".to_vec())
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

/// By default (when all options are `None`) makes ACCOUNT1 to suggest 'Blocked' status to the POST1
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
