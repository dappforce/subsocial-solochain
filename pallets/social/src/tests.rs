use crate::*;
use frame_support::{
    assert_ok, assert_noop,
    impl_outer_origin, parameter_types,
    weights::Weight,
    dispatch::DispatchResult,
};
use sp_core::H256;
use sp_io::TestExternalities;
use sp_std::{
    collections::btree_set::BTreeSet,
    iter::FromIterator
};
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    testing::Header,
    Perbill
};

use pallet_permissions::{
    SpacePermission as SP,
    SpacePermissions,
};

use pallet_utils::{Error as UtilsError};

impl_outer_origin! {
  pub enum Origin for TestRuntime {}
}

#[derive(Clone, Eq, PartialEq)]
pub struct TestRuntime;

parameter_types! {
  pub const BlockHashCount: u64 = 250;
  pub const MaximumBlockWeight: Weight = 1024;
  pub const MaximumBlockLength: u32 = 2 * 1024;
  pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
}

impl system::Trait for TestRuntime {
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

impl pallet_timestamp::Trait for TestRuntime {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
}

parameter_types! {
  pub const IpfsHashLen: u32 = 46;
}

impl pallet_utils::Trait for TestRuntime {
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

impl pallet_permissions::Trait for TestRuntime {
    type DefaultSpacePermissions = DefaultSpacePermissions;
}

parameter_types! {
  pub const MaxUsersToProcessPerDeleteRole: u16 = 20;
}

impl pallet_roles::Trait for TestRuntime {
    type Event = ();
    type MaxUsersToProcessPerDeleteRole = MaxUsersToProcessPerDeleteRole;
    type Spaces = Social;
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

impl Trait for TestRuntime {
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

type System = system::Module<TestRuntime>;
type Social = Module<TestRuntime>;
type Roles = pallet_roles::Module<TestRuntime>;

pub type AccountId = u64;

const ACCOUNT1 : AccountId = 1;
const ACCOUNT2 : AccountId = 2;
const ACCOUNT3: AccountId = 3;

pub struct ExtBuilder;

// TODO: make created space/post/comment configurable or by default
impl ExtBuilder {
    /// Default ext configuration with BlockNumber 1
    pub fn build() -> TestExternalities {
        let storage = system::GenesisConfig::default()
            .build_storage::<TestRuntime>()
            .unwrap();

        let mut ext = TestExternalities::from(storage);
        ext.execute_with(|| System::set_block_number(1));

        ext
    }

    /// Custom ext configuration with SpaceId 1 and BlockNumber 1
    pub fn build_with_space() -> TestExternalities {
        let storage = system::GenesisConfig::default()
            .build_storage::<TestRuntime>()
            .unwrap();

        let mut ext = TestExternalities::from(storage);
        ext.execute_with(|| {
            System::set_block_number(1);
            assert_ok!(_create_default_space());
        });

        ext
    }

    /// Custom ext configuration with SpaceId 1, PostId 1 and BlockNumber 1
    pub fn build_with_post() -> TestExternalities {
        let storage = system::GenesisConfig::default()
            .build_storage::<TestRuntime>()
            .unwrap();

        let mut ext = TestExternalities::from(storage);
        ext.execute_with(|| {
            System::set_block_number(1);
            assert_ok!(_create_default_space());
            assert_ok!(_create_default_post());
        });

        ext
    }

    /// Custom ext configuration with SpaceId 1, PostId 1, PostId 2 (as comment) and BlockNumber 1
    pub fn build_with_comment() -> TestExternalities {
        let storage = system::GenesisConfig::default()
            .build_storage::<TestRuntime>()
            .unwrap();

        let mut ext = TestExternalities::from(storage);
        ext.execute_with(|| {
            System::set_block_number(1);
            assert_ok!(_create_default_space());
            assert_ok!(_create_default_post());
            assert_ok!(_create_default_comment());
        });

        ext
    }
}

// TODO: add new externality for testing transfer ownership

fn space_handle() -> Vec<u8> {
    b"space_handle".to_vec()
}

fn space_ipfs_hash() -> Vec<u8> {
    b"QmRAQB6YaCyidP37UdDnjFY5vQuiBrcqdyoW1CuDgwxkD4".to_vec()
}

fn space_update(handle: Option<Option<Vec<u8>>>, ipfs_hash: Option<Vec<u8>>, hidden: Option<bool>) -> SpaceUpdate {
    SpaceUpdate {
        handle,
        ipfs_hash,
        hidden
    }
}

fn post_ipfs_hash() -> Vec<u8> {
    b"QmRAQB6YaCyidP37UdDnjFY5vQuiBrcqdyoW2CuDgwxkD4".to_vec()
}

fn fake_post(id: PostId, created_by: AccountId, space_id: Option<SpaceId>, extension: PostExtension) -> Post<TestRuntime> {
    Post {
        id,
        created: WhoAndWhen::<TestRuntime>::new(created_by),
        updated: None,
        hidden: false,
        space_id,
        extension,
        ipfs_hash: self::post_ipfs_hash(),
        edit_history: Vec::new(),
        direct_replies_count: 0,
        total_replies_count: 0,
        shares_count: 0,
        upvotes_count: 0,
        downvotes_count: 0,
        score: 0
    }
}

fn post_update(space_id: Option<SpaceId>, ipfs_hash: Option<Vec<u8>>, hidden: Option<bool>) -> PostUpdate {
    PostUpdate {
        space_id,
        ipfs_hash,
        hidden
    }
}

fn comment_ipfs_hash() -> Vec<u8> {
    b"QmRAQB6YaCyidP37UdDnjFY5vQuiBrcqdyoW1CuDgwxkD4".to_vec()
}

fn subcomment_ipfs_hash() -> Vec<u8> {
    b"QmYA2fn8cMbVWo4v95RwcwJVyQsNtnEwHerfWR8UNtEwoE".to_vec()
}

fn alice_username() -> Vec<u8> {
    b"Alice".to_vec()
}
fn bob_username() -> Vec<u8> {
    b"Bob".to_vec()
}

fn profile_ipfs_hash() -> Vec<u8> {
    b"QmRAQB6YaCyidP37UdDnjFY5vQuiaRtqdyoW2CuDgwxkA5".to_vec()
}

fn reaction_upvote() -> ReactionKind {
    ReactionKind::Upvote
}
fn reaction_downvote() -> ReactionKind {
    ReactionKind::Downvote
}

fn scoring_action_upvote_post() -> ScoringAction {
    ScoringAction::UpvotePost
}
fn scoring_action_downvote_post() -> ScoringAction {
    ScoringAction::DownvotePost
}
fn scoring_action_share_post() -> ScoringAction {
    ScoringAction::SharePost
}
fn scoring_action_create_comment() -> ScoringAction {
    ScoringAction::CreateComment
}
fn scoring_action_upvote_comment() -> ScoringAction {
    ScoringAction::UpvoteComment
}
fn scoring_action_downvote_comment() -> ScoringAction {
    ScoringAction::DownvoteComment
}
fn scoring_action_share_comment() -> ScoringAction {
    ScoringAction::ShareComment
}
fn scoring_action_follow_space() -> ScoringAction {
    ScoringAction::FollowSpace
}
fn scoring_action_follow_account() -> ScoringAction {
    ScoringAction::FollowAccount
}

fn extension_regular_post() -> PostExtension {
    PostExtension::RegularPost
}
fn extension_comment(parent_id: Option<PostId>, root_post_id: PostId) -> PostExtension {
    PostExtension::Comment(CommentExt{ parent_id, root_post_id })
}
fn extension_shared_post(post_id: PostId) -> PostExtension {
    PostExtension::SharedPost(post_id)
}

fn _create_default_space() -> DispatchResult {
    _create_space(None, None, None)
}

fn _create_space(origin: Option<Origin>, handle: Option<Option<Vec<u8>>>, ipfs_hash: Option<Vec<u8>>) -> DispatchResult {
    Social::create_space(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
        handle.unwrap_or_else(|| Some(self::space_handle())),
        ipfs_hash.unwrap_or_else(self::space_ipfs_hash)
    )
}

fn _update_space(origin: Option<Origin>, space_id: Option<u32>, update: Option<SpaceUpdate>) -> DispatchResult {
    Social::update_space(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
        space_id.unwrap_or(1).into(),
        update.unwrap_or_else(|| self::space_update(None, None, None))
    )
}

fn _default_follow_space() -> DispatchResult {
    _follow_space(None, None)
}

fn _follow_space(origin: Option<Origin>, space_id: Option<SpaceId>) -> DispatchResult {
    Social::follow_space(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT2)),
        space_id.unwrap_or(1)
    )
}

fn _default_unfollow_space() -> DispatchResult {
    _unfollow_space(None, None)
}

fn _unfollow_space(origin: Option<Origin>, space_id: Option<SpaceId>) -> DispatchResult {
    Social::unfollow_space(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT2)),
        space_id.unwrap_or(1)
    )
}

fn _create_default_post() -> DispatchResult {
    _create_post(None, None, None, None)
}

fn _create_post(origin: Option<Origin>, space_id_opt: Option<Option<SpaceId>>, extension: Option<PostExtension>, ipfs_hash: Option<Vec<u8>>) -> DispatchResult {
    Social::create_post(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
        space_id_opt.unwrap_or(Some(1)),
        extension.unwrap_or_else(self::extension_regular_post),
        ipfs_hash.unwrap_or_else(self::post_ipfs_hash)
    )
}

fn _update_post(origin: Option<Origin>, post_id: Option<PostId>, update: Option<PostUpdate>) -> DispatchResult {
    Social::update_post(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
        post_id.unwrap_or(1),
        update.unwrap_or_else(|| self::post_update(None, None, None))
    )
}

fn _create_default_comment() -> DispatchResult {
    _create_comment(None, None, None, None)
}

fn _create_comment(origin: Option<Origin>, post_id: Option<PostId>,
                   parent_id: Option<Option<PostId>>, ipfs_hash: Option<Vec<u8>>)
                   -> DispatchResult {
    _create_post(
        origin,
        Some(None),
        Some(self::extension_comment(
            parent_id.unwrap_or(None), post_id.unwrap_or(1))
        ),
        Some(ipfs_hash.unwrap_or_else(self::comment_ipfs_hash))
    )
}

fn _update_comment(origin: Option<Origin>, post_id: Option<PostId>, update: Option<PostUpdate>) -> DispatchResult {
    _update_post(
        origin,
        Some(post_id.unwrap_or(2)),
        Some(update.unwrap_or_else(|| self::post_update(None, Some(self::subcomment_ipfs_hash()), None))
        )
    )
}

fn _create_default_post_reaction() -> DispatchResult {
    _create_post_reaction(None, None, None)
}

fn _create_default_comment_reaction() -> DispatchResult {
    _create_comment_reaction(None, None, None)
}

fn _create_post_reaction(origin: Option<Origin>, post_id: Option<PostId>, kind: Option<ReactionKind>) -> DispatchResult {
    Social::create_post_reaction(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
        post_id.unwrap_or(1),
        kind.unwrap_or_else(self::reaction_upvote)
    )
}

fn _create_comment_reaction(origin: Option<Origin>, post_id: Option<PostId>, kind: Option<ReactionKind>) -> DispatchResult {
    _create_post_reaction(origin, Some(post_id.unwrap_or(2)), kind)
}

fn _update_post_reaction(origin: Option<Origin>, post_id: Option<PostId>, reaction_id: ReactionId, kind: Option<ReactionKind>) -> DispatchResult {
    Social::update_post_reaction(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
        post_id.unwrap_or(1),
        reaction_id,
        kind.unwrap_or_else(self::reaction_upvote)
    )
}

fn _update_comment_reaction(origin: Option<Origin>, post_id: Option<PostId>, reaction_id: ReactionId, kind: Option<ReactionKind>) -> DispatchResult {
    _update_post_reaction(origin, Some(post_id.unwrap_or(2)), reaction_id, kind)
}

fn _delete_post_reaction(origin: Option<Origin>, post_id: Option<PostId>, reaction_id: ReactionId) -> DispatchResult {
    Social::delete_post_reaction(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
        post_id.unwrap_or(1),
        reaction_id
    )
}

fn _delete_comment_reaction(origin: Option<Origin>, post_id: Option<PostId>, reaction_id: ReactionId) -> DispatchResult {
    _delete_post_reaction(origin, Some(post_id.unwrap_or(2)), reaction_id)
}

fn _create_default_profile() -> DispatchResult {
    _create_profile(None, None, None)
}

fn _create_profile(origin: Option<Origin>, username: Option<Vec<u8>>, ipfs_hash: Option<Vec<u8>>) -> DispatchResult {
    Social::create_profile(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
        username.unwrap_or_else(self::alice_username),
        ipfs_hash.unwrap_or_else(self::profile_ipfs_hash)
    )
}

fn _update_profile(origin: Option<Origin>, username: Option<Vec<u8>>, ipfs_hash: Option<Vec<u8>>) -> DispatchResult {
    Social::update_profile(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
        ProfileUpdate {
            username,
            ipfs_hash
        }
    )
}

fn _default_follow_account() -> DispatchResult {
    _follow_account(None, None)
}

fn _follow_account(origin: Option<Origin>, account: Option<AccountId>) -> DispatchResult {
    Social::follow_account(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT2)),
        account.unwrap_or(ACCOUNT1)
    )
}

fn _default_unfollow_account() -> DispatchResult {
    _unfollow_account(None, None)
}

fn _unfollow_account(origin: Option<Origin>, account: Option<AccountId>) -> DispatchResult {
    Social::unfollow_account(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT2)),
        account.unwrap_or(ACCOUNT1)
    )
}

fn _change_post_score_by_extension_with_id(account: AccountId, post_id: PostId, action: ScoringAction) -> DispatchResult {
    if let Some(ref mut post) = Social::post_by_id(post_id) {
        Social::change_post_score_by_extension(account, post, action)
    } else {
        panic!("Test error. Post\\Comment with specified ID not found.");
    }
}

fn _change_post_score_by_extension(account: AccountId, post: &mut Post<TestRuntime>, action: ScoringAction) -> DispatchResult {
    Social::change_post_score_by_extension(account, post, action)
}

fn _transfer_default_space_ownership() -> DispatchResult {
    _transfer_space_ownership(None, None, None)
}

fn _transfer_space_ownership(origin: Option<Origin>, space_id: Option<SpaceId>, transfer_to: Option<AccountId>) -> DispatchResult {
    Social::transfer_space_ownership(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
        space_id.unwrap_or(1),
        transfer_to.unwrap_or(ACCOUNT2)
    )
}

fn _accept_default_pending_ownership() -> DispatchResult {
    _accept_pending_ownership(None, None)
}

fn _accept_pending_ownership(origin: Option<Origin>, space_id: Option<SpaceId>) -> DispatchResult {
    Social::accept_pending_ownership(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT2)),
        space_id.unwrap_or(1)
    )
}

fn _reject_default_pending_ownership() -> DispatchResult {
    _reject_pending_ownership(None, None)
}

fn _reject_default_pending_ownership_by_current_owner() -> DispatchResult {
    _reject_pending_ownership(Some(Origin::signed(ACCOUNT1)), None)
}

fn _reject_pending_ownership(origin: Option<Origin>, space_id: Option<SpaceId>) -> DispatchResult {
    Social::reject_pending_ownership(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT2)),
        space_id.unwrap_or(1)
    )
}

// Space tests
#[test]
fn create_space_should_work() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_create_default_space()); // SpaceId 1

        // Check storages
        assert_eq!(Social::space_ids_by_owner(ACCOUNT1), vec![1]);
        assert_eq!(Social::space_id_by_handle(self::space_handle()), Some(1));
        assert_eq!(Social::next_space_id(), 2);

        // Check whether data stored correctly
        let space = Social::space_by_id(1).unwrap();

        assert_eq!(space.created.account, ACCOUNT1);
        assert!(space.updated.is_none());
        assert_eq!(space.hidden, false);

        assert_eq!(space.owner, ACCOUNT1);
        assert_eq!(space.handle, Some(self::space_handle()));
        assert_eq!(space.ipfs_hash, self::space_ipfs_hash());

        assert_eq!(space.posts_count, 0);
        assert_eq!(space.followers_count, 1);
        assert!(space.edit_history.is_empty());
        assert_eq!(space.score, 0);
    });
}

#[test]
fn create_space_should_make_handle_lowercase() {
    ExtBuilder::build().execute_with(|| {
        let handle : Vec<u8> = b"sPaCe_hAnDlE".to_vec();

        assert_ok!(_create_space(None, Some(Some(handle.clone())), None)); // BlodId 1

        // Handle should be lowercase in storage and original in struct
        let space = Social::space_by_id(1).unwrap();
        assert_eq!(space.handle, Some(handle.clone()));
        assert_eq!(Social::space_id_by_handle(handle.to_ascii_lowercase()), Some(1));
    });
}

#[test]
fn create_space_should_fail_short_handle() {
    ExtBuilder::build().execute_with(|| {
        let handle : Vec<u8> = vec![65; (MinHandleLen::get() - 1) as usize];

        // Try to catch an error creating a space with too short handle
        assert_noop!(_create_space(None, Some(Some(handle)), None), Error::<TestRuntime>::HandleIsTooShort);
    });
}

#[test]
fn create_space_should_fail_long_handle() {
    ExtBuilder::build().execute_with(|| {
        let handle : Vec<u8> = vec![65; (MaxHandleLen::get() + 1) as usize];

        // Try to catch an error creating a space with too long handle
        assert_noop!(_create_space(None, Some(Some(handle)), None), Error::<TestRuntime>::HandleIsTooLong);
    });
}

#[test]
fn create_space_should_fail_not_unique_handle() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_create_default_space()); // SpaceId 1
        // Try to catch an error creating a space with not unique handle
        assert_noop!(_create_default_space(), Error::<TestRuntime>::HandleIsNotUnique);
    });
}

#[test]
fn create_space_should_fail_invalid_at_char() {
    ExtBuilder::build().execute_with(|| {
        let handle : Vec<u8> = b"@space_handle".to_vec();

        assert_noop!(_create_space(None, Some(Some(handle)), None), Error::<TestRuntime>::HandleContainsInvalidChars);
    });
}

#[test]
fn create_space_should_fail_invalid_minus_char() {
    ExtBuilder::build().execute_with(|| {
        let handle : Vec<u8> = b"space-handle".to_vec();

        assert_noop!(_create_space(None, Some(Some(handle)), None), Error::<TestRuntime>::HandleContainsInvalidChars);
    });
}

#[test]
fn create_space_should_fail_invalid_space_char() {
    ExtBuilder::build().execute_with(|| {
        let handle : Vec<u8> = b"space handle".to_vec();

        assert_noop!(_create_space(None, Some(Some(handle.clone())), None), Error::<TestRuntime>::HandleContainsInvalidChars);
    });
}

#[test]
fn create_space_should_fail_invalid_unicode_char() {
    ExtBuilder::build().execute_with(|| {
        let handle : Vec<u8> = String::from("блог_хендл").into_bytes().to_vec();

        assert_noop!(_create_space(None, Some(Some(handle.clone())), None), Error::<TestRuntime>::HandleContainsInvalidChars);
    });
}

#[test]
fn create_space_should_fail_invalid_ipfs_hash() {
    ExtBuilder::build().execute_with(|| {
        let ipfs_hash : Vec<u8> = b"QmV9tSDx9UiPeWExXEeH6aoDvmihvx6j".to_vec();

        // Try to catch an error creating a space with invalid ipfs_hash
        assert_noop!(_create_space(None, None, Some(ipfs_hash)), UtilsError::<TestRuntime>::IpfsIsIncorrect);
    });
}

#[test]
fn update_space_should_work() {
    ExtBuilder::build_with_space().execute_with(|| {
        let handle : Vec<u8> = b"new_handle".to_vec();
        let ipfs_hash : Vec<u8> = b"QmRAQB6YaCyidP37UdDnjFY5vQuiBrcqdyoW2CuDgwxkD4".to_vec();        // Space update with ID 1 should be fine

        assert_ok!(_update_space(None, None,
      Some(
        self::space_update(
          Some(Some(handle.clone())),
          Some(ipfs_hash.clone()),
          Some(true)
        )
      )
    ));

        // Check whether space updates correctly
        let space = Social::space_by_id(1).unwrap();
        assert_eq!(space.handle, Some(handle));
        assert_eq!(space.ipfs_hash, ipfs_hash);
        assert_eq!(space.hidden, true);

        // Check whether history recorded correctly
        assert_eq!(space.edit_history[0].old_data.handle, Some(Some(self::space_handle())));
        assert_eq!(space.edit_history[0].old_data.ipfs_hash, Some(self::space_ipfs_hash()));
        assert_eq!(space.edit_history[0].old_data.hidden, Some(false));
    });
}

#[test]
fn update_space_should_fail_nothing_to_update() {
    ExtBuilder::build_with_space().execute_with(|| {
        // Try to catch an error updating a space with no changes
        assert_noop!(_update_space(None, None, None), Error::<TestRuntime>::NoUpdatesInSpace);
    });
}

#[test]
fn update_space_should_fail_space_not_found() {
    ExtBuilder::build_with_space().execute_with(|| {
        let handle : Vec<u8> = b"new_handle".to_vec();

        // Try to catch an error updating a space with wrong space ID
        assert_noop!(_update_space(None, Some(2),
      Some(
        self::space_update(
          Some(Some(handle)),
          None,
          None
        )
      )
    ), Error::<TestRuntime>::SpaceNotFound);
    });
}

#[test]
fn update_space_should_fail_not_an_owner() {
    ExtBuilder::build_with_space().execute_with(|| {
        let handle : Vec<u8> = b"new_handle".to_vec();

        // Try to catch an error updating a space with different account
        assert_noop!(_update_space(Some(Origin::signed(ACCOUNT2)), None,
      Some(
        self::space_update(
          Some(Some(handle)),
          None,
          None
        )
      )
    ), Error::<TestRuntime>::NoPermissionToUpdateSpace);
    });
}

#[test]
fn update_space_should_fail_short_handle() {
    ExtBuilder::build_with_space().execute_with(|| {
        let handle : Vec<u8> = vec![65; (MinHandleLen::get() - 1) as usize];

        // Try to catch an error updating a space with too short handle
        assert_noop!(_update_space(None, None,
      Some(
        self::space_update(
          Some(Some(handle)),
          None,
          None
        )
      )
    ), Error::<TestRuntime>::HandleIsTooShort);
    });
}

#[test]
fn update_space_should_fail_long_handle() {
    ExtBuilder::build_with_space().execute_with(|| {
        let handle : Vec<u8> = vec![65; (MaxHandleLen::get() + 1) as usize];

        // Try to catch an error updating a space with too long handle
        assert_noop!(_update_space(None, None,
      Some(
        self::space_update(
          Some(Some(handle)),
          None,
          None
        )
      )
    ), Error::<TestRuntime>::HandleIsTooLong);
    });
}

#[test]
fn update_space_should_fail_not_unique_handle() {
    ExtBuilder::build_with_space().execute_with(|| {
        let handle : Vec<u8> = b"unique_handle".to_vec();

        assert_ok!(_create_space(
      None,
      Some(Some(handle.clone())),
      None
    )); // SpaceId 2 with a custom handle

        // Try to catch an error updating a space on ID 1 with a handle of space on ID 2
        assert_noop!(_update_space(None, Some(1),
      Some(
        self::space_update(
          Some(Some(handle)),
          None,
          None
        )
      )
    ), Error::<TestRuntime>::HandleIsNotUnique);
    });
}

#[test]
fn update_space_should_fail_invalid_at_char() {
    ExtBuilder::build_with_space().execute_with(|| {
        let handle : Vec<u8> = b"@space_handle".to_vec();

        assert_noop!(_update_space(None, None,
      Some(
        self::space_update(
          Some(Some(handle)),
          None,
          None
        )
      )
    ), Error::<TestRuntime>::HandleContainsInvalidChars);
    });
}

#[test]
fn update_space_should_fail_invalid_minus_char() {
    ExtBuilder::build_with_space().execute_with(|| {
        let handle : Vec<u8> = b"space-handle".to_vec();

        assert_noop!(_update_space(None, None,
      Some(
        self::space_update(
          Some(Some(handle)),
          None,
          None
        )
      )
    ), Error::<TestRuntime>::HandleContainsInvalidChars);
    });
}

#[test]
fn update_space_should_fail_invalid_space_char() {
    ExtBuilder::build_with_space().execute_with(|| {
        let handle : Vec<u8> = b"space handle".to_vec();

        assert_noop!(_update_space(None, None,
      Some(
        self::space_update(
          Some(Some(handle)),
          None,
          None
        )
      )
    ), Error::<TestRuntime>::HandleContainsInvalidChars);
    });
}

#[test]
fn update_space_should_fail_invalid_unicode_char() {
    ExtBuilder::build_with_space().execute_with(|| {
        let handle : Vec<u8> = String::from("блог_хендл").into_bytes().to_vec();

        assert_noop!(_update_space(None, None,
      Some(
        self::space_update(
          Some(Some(handle)),
          None,
          None
        )
      )
    ), Error::<TestRuntime>::HandleContainsInvalidChars);
    });
}

#[test]
fn update_space_should_fail_invalid_ipfs_hash() {
    ExtBuilder::build_with_space().execute_with(|| {
        let ipfs_hash : Vec<u8> = b"QmV9tSDx9UiPeWExXEeH6aoDvmihvx6j".to_vec();

        // Try to catch an error updating a space with invalid ipfs_hash
        assert_noop!(_update_space(None, None,
      Some(
        self::space_update(
          None,
          Some(ipfs_hash),
          None
        )
      )
    ), UtilsError::<TestRuntime>::IpfsIsIncorrect);
    });
}

// Post tests
#[test]
fn create_post_should_work() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(_create_default_post()); // PostId 1

        // Check storages
        assert_eq!(Social::post_ids_by_space_id(1), vec![1]);
        assert_eq!(Social::next_post_id(), 2);

        // Check whether data stored correctly
        let post = Social::post_by_id(1).unwrap();

        assert_eq!(post.created.account, ACCOUNT1);
        assert!(post.updated.is_none());
        assert_eq!(post.hidden, false);

        assert_eq!(post.space_id, Some(1));
        assert_eq!(post.extension, self::extension_regular_post());

        assert_eq!(post.ipfs_hash, self::post_ipfs_hash());
        assert!(post.edit_history.is_empty());

        assert_eq!(post.total_replies_count, 0);
        assert_eq!(post.shares_count, 0);
        assert_eq!(post.upvotes_count, 0);
        assert_eq!(post.downvotes_count, 0);

        assert_eq!(post.score, 0);
    });
}

#[test]
fn create_post_should_fail_space_not_defined() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_noop!(_create_post(None, Some(None), None, None), Error::<TestRuntime>::SpaceIdIsUndefined);
    });
}

#[test]
fn create_post_should_fail_space_not_found() {
    ExtBuilder::build().execute_with(|| {
        assert_noop!(_create_default_post(), Error::<TestRuntime>::SpaceNotFound);
    });
}

#[test]
fn create_post_should_fail_invalid_ipfs_hash() {
    ExtBuilder::build_with_space().execute_with(|| {
        let ipfs_hash : Vec<u8> = b"QmV9tSDx9UiPeWExXEeH6aoDvmihvx6j".to_vec();

        // Try to catch an error creating a regular post with invalid ipfs_hash
        assert_noop!(_create_post(None, None, None, Some(ipfs_hash)), UtilsError::<TestRuntime>::IpfsIsIncorrect);
    });
}

#[test]
fn create_post_should_fail_not_a_space_owner() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_noop!(_create_post(Some(Origin::signed(ACCOUNT2)), None, None, None), Error::<TestRuntime>::NoPermissionToCreatePosts);
    });
}

#[test]
fn update_post_should_work() {
    ExtBuilder::build_with_post().execute_with(|| {
        let ipfs_hash: Vec<u8> = b"QmRAQB6YaCyidP37UdDnjFY5vQuiBrcqdyoW1CuDgwxkD4".to_vec();

        // Post update with ID 1 should be fine
        assert_ok!(_update_post(None, None,
            Some(
                self::post_update(
                  None,
                  Some(ipfs_hash.clone()),
                  Some(true)
                )
            )
        ));

        // Check whether post updates correctly
        let post = Social::post_by_id(1).unwrap();
        assert_eq!(post.space_id, Some(1));
        assert_eq!(post.ipfs_hash, ipfs_hash);
        assert_eq!(post.hidden, true);

        // Check whether history recorded correctly
        assert_eq!(post.edit_history[0].old_data.space_id, None);
        assert_eq!(post.edit_history[0].old_data.ipfs_hash, Some(self::post_ipfs_hash()));
        assert_eq!(post.edit_history[0].old_data.hidden, Some(false));
    });
}

#[test]
fn update_post_should_work_after_transfer_space_ownership() {
    ExtBuilder::build_with_post().execute_with(|| {
        let ipfs_hash: Vec<u8> = b"QmRAQB6YaCyidP37UdDnjFY5vQuiBrcqdyoW1CuDgwxkD4".to_vec();

        assert_ok!(_transfer_default_space_ownership());

        // Post update with ID 1 should be fine
        assert_ok!(_update_post(None, None,
            Some(
                self::post_update(
                    None,
                    Some(ipfs_hash.clone()),
                    Some(true)
                )
            )
        ));
    });
}

#[test]
fn update_post_should_fail_nothing_to_update() {
    ExtBuilder::build_with_post().execute_with(|| {
        // Try to catch an error updating a post with no changes
        assert_noop!(_update_post(None, None, None), Error::<TestRuntime>::NoUpdatesInPost);
    });
}

#[test]
fn update_post_should_fail_post_not_found() {
    ExtBuilder::build_with_post().execute_with(|| {
        assert_ok!(_create_space(None, Some(Some(b"space2_handle".to_vec())), None)); // SpaceId 2

        // Try to catch an error updating a post with wrong post ID
        assert_noop!(_update_post(None, Some(2),
      Some(
        self::post_update(
          Some(2),
          None,
          None
        )
      )
    ), Error::<TestRuntime>::PostNotFound);
    });
}

#[test]
fn update_post_should_fail_not_an_owner() {
    ExtBuilder::build_with_post().execute_with(|| {
        assert_ok!(_create_space(None, Some(Some(b"space2_handle".to_vec())), None)); // SpaceId 2

        // Try to catch an error updating a post with different account
        assert_noop!(_update_post(Some(Origin::signed(ACCOUNT2)), None,
      Some(
        self::post_update(
          Some(2),
          None,
          None
        )
      )
    ), Error::<TestRuntime>::NoPermissionToUpdateAnyPosts);
    });
}

#[test]
fn update_post_should_fail_invalid_ipfs_hash() {
    ExtBuilder::build_with_post().execute_with(|| {
        let ipfs_hash : Vec<u8> = b"QmV9tSDx9UiPeWExXEeH6aoDvmihvx6j".to_vec();

        // Try to catch an error updating a post with invalid ipfs_hash
        assert_noop!(_update_post(None, None,
            Some(
                self::post_update(
                    None,
                    Some(ipfs_hash),
                    None
                )
            )
        ), UtilsError::<TestRuntime>::IpfsIsIncorrect);
    });
}

// Comment tests
#[test]
fn create_comment_should_work() {
    ExtBuilder::build_with_post().execute_with(|| {
        assert_ok!(_create_default_comment()); // PostId 2

        // Check storages
        let root_post = Social::post_by_id(1).unwrap();
        assert_eq!(Social::reply_ids_by_post_id(1), vec![2]);
        assert_eq!(root_post.total_replies_count, 1);
        assert_eq!(root_post.direct_replies_count, 1);

        // Check whether data stored correctly
        let comment = Social::post_by_id(2).unwrap();
        let comment_ext = comment.get_comment_ext().unwrap();

        assert_eq!(comment_ext.parent_id, None);
        assert_eq!(comment_ext.root_post_id, 1);
        assert_eq!(comment.created.account, ACCOUNT1);
        assert!(comment.updated.is_none());
        assert_eq!(comment.ipfs_hash, self::comment_ipfs_hash());
        assert!(comment.edit_history.is_empty());
        assert_eq!(comment.total_replies_count, 0);
        assert_eq!(comment.shares_count, 0);
        assert_eq!(comment.upvotes_count, 0);
        assert_eq!(comment.downvotes_count, 0);
        assert_eq!(comment.score, 0);
    });
}

#[test]
fn create_comment_should_work_with_parent() {
    ExtBuilder::build_with_post().execute_with(|| {
        assert_ok!(_create_default_comment()); // PostId 2
        assert_ok!(_create_comment(None, None, Some(Some(2)), None)); // PostId 3 with parent comment with PostId 2

        // Check storages
        assert_eq!(Social::reply_ids_by_post_id(1), vec![2]);
        assert_eq!(Social::reply_ids_by_post_id(2), vec![3]);
        let root_post = Social::post_by_id(1).unwrap();
        let parent_post = Social::post_by_id(2).unwrap();
        assert_eq!(root_post.total_replies_count, 2);
        assert_eq!(root_post.direct_replies_count, 1);
        assert_eq!(parent_post.total_replies_count, 1);
        assert_eq!(parent_post.direct_replies_count, 1);

        // Check whether data stored correctly
        let comment_ext = Social::post_by_id(3).unwrap().get_comment_ext().unwrap();
        assert_eq!(comment_ext.parent_id, Some(2));
    });
}

#[test]
fn create_comment_should_fail_post_not_found() {
    ExtBuilder::build().execute_with(|| {
        // Try to catch an error creating a comment with wrong post
        assert_noop!(_create_default_comment(), Error::<TestRuntime>::PostNotFound);
    });
}

#[test]
fn create_comment_should_fail_parent_not_found() {
    ExtBuilder::build_with_post().execute_with(|| {
        // Try to catch an error creating a comment with wrong parent
        assert_noop!(_create_comment(None, None, Some(Some(2)), None), Error::<TestRuntime>::UnknownParentComment);
    });
}

#[test]
fn create_comment_should_fail_invalid_ipfs_hash() {
    ExtBuilder::build_with_post().execute_with(|| {
        let ipfs_hash : Vec<u8> = b"QmV9tSDx9UiPeWExXEeH6aoDvmihvx6j".to_vec();

        // Try to catch an error creating a comment with wrong parent
        assert_noop!(_create_comment(None, None, None, Some(ipfs_hash)), UtilsError::<TestRuntime>::IpfsIsIncorrect);
    });
}

#[test]
fn create_comment_should_fail_space_is_hidden() {
    ExtBuilder::build_with_post().execute_with(|| {
        assert_ok!(_update_space(
          None,
          None,
          Some(self::space_update(None, None, Some(true)))
        ));

        assert_noop!(_create_default_comment(), Error::<TestRuntime>::BannedToCreateWhenHidden);
    });
}

#[test]
fn create_comment_should_fail_post_is_hidden() {
    ExtBuilder::build_with_post().execute_with(|| {
        assert_ok!(_update_post(
          None,
          None,
          Some(self::post_update(None, None, Some(true)))
        ));

        assert_noop!(_create_default_comment(), Error::<TestRuntime>::BannedToCreateWhenHidden);
    });
}

#[test]
fn create_comment_should_fail_max_depth_reached() {
    ExtBuilder::build_with_post().execute_with(|| {
        assert_ok!(_create_comment(None, None, Some(None), None)); // PostId 2

        for parent_id in 2..11 as PostId {
            assert_ok!(_create_comment(None, None, Some(Some(parent_id)), None)); // PostId N (last = 10)
        }

        assert_noop!(_create_comment(None, None, Some(Some(11)), None), Error::<TestRuntime>::MaxCommentDepthReached);
    });
}

#[test]
fn update_comment_should_work() {
    ExtBuilder::build_with_comment().execute_with(|| {
        // Post update with ID 1 should be fine
        assert_ok!(_update_comment(None, None, None));

        // Check whether post updates correctly
        let comment = Social::post_by_id(2).unwrap();
        assert_eq!(comment.ipfs_hash, self::subcomment_ipfs_hash());

        // Check whether history recorded correctly
        assert_eq!(comment.edit_history[0].old_data.ipfs_hash, Some(self::comment_ipfs_hash()));
    });
}

#[test]
fn update_comment_should_fail_comment_not_found() {
    ExtBuilder::build().execute_with(|| {
        // Try to catch an error updating a comment with wrong PostId
        assert_noop!(_update_comment(None, None, None), Error::<TestRuntime>::PostNotFound);
    });
}

#[test]
fn update_comment_should_fail_not_an_owner() {
    ExtBuilder::build_with_comment().execute_with(|| {
        // Try to catch an error updating a comment with wrong Account
        assert_noop!(_update_comment(
      Some(Origin::signed(2)), None, None
    ), Error::<TestRuntime>::NotACommentAuthor);
    });
}

#[test]
fn update_comment_should_fail_invalid_ipfs_hash() {
    ExtBuilder::build_with_comment().execute_with(|| {
        let ipfs_hash : Vec<u8> = b"QmV9tSDx9UiPeWExXEeH6aoDvmihvx6j".to_vec();

        // Try to catch an error updating a comment with invalid ipfs_hash
        assert_noop!(_update_comment(
          None, None, Some(self::post_update(None, Some(ipfs_hash), None))
        ), UtilsError::<TestRuntime>::IpfsIsIncorrect);
    });
}

#[test]
fn update_comment_should_fail_ipfs_hash_dont_differ() {
    ExtBuilder::build_with_comment().execute_with(|| {
        // Try to catch an error updating a comment with the same ipfs_hash
        assert_noop!(_update_comment(
      None, None, Some(self::post_update(None, Some(self::comment_ipfs_hash()), None))
    ), Error::<TestRuntime>::CommentIPFSHashNotDiffer);
    });
}

// Reaction tests
#[test]
fn create_post_reaction_should_work_upvote() {
    ExtBuilder::build_with_post().execute_with(|| {
        assert_ok!(_create_post_reaction(Some(Origin::signed(ACCOUNT2)), None, None)); // ReactionId 1 by ACCOUNT2

        // Check storages
        assert_eq!(Social::reaction_ids_by_post_id(1), vec![1]);
        assert_eq!(Social::next_reaction_id(), 2);

        // Check post reaction counters
        let post = Social::post_by_id(1).unwrap();
        assert_eq!(post.upvotes_count, 1);
        assert_eq!(post.downvotes_count, 0);

        // Check whether data stored correctly
        let reaction = Social::reaction_by_id(1).unwrap();
        assert_eq!(reaction.created.account, ACCOUNT2);
        assert_eq!(reaction.kind, self::reaction_upvote());
    });
}

#[test]
fn create_post_reaction_should_work_downvote() {
    ExtBuilder::build_with_post().execute_with(|| {
        assert_ok!(_create_post_reaction(Some(Origin::signed(ACCOUNT2)), None, Some(self::reaction_downvote()))); // ReactionId 1 by ACCOUNT2

        // Check storages
        assert_eq!(Social::reaction_ids_by_post_id(1), vec![1]);
        assert_eq!(Social::next_reaction_id(), 2);

        // Check post reaction counters
        let post = Social::post_by_id(1).unwrap();
        assert_eq!(post.upvotes_count, 0);
        assert_eq!(post.downvotes_count, 1);

        // Check whether data stored correctly
        let reaction = Social::reaction_by_id(1).unwrap();
        assert_eq!(reaction.created.account, ACCOUNT2);
        assert_eq!(reaction.kind, self::reaction_downvote());
    });
}

#[test]
fn create_post_reaction_should_fail_already_reacted() {
    ExtBuilder::build_with_post().execute_with(|| {
        assert_ok!(_create_default_post_reaction()); // ReactionId1

        // Try to catch an error creating reaction by the same account
        assert_noop!(_create_default_post_reaction(), Error::<TestRuntime>::AccountAlreadyReacted);
    });
}

#[test]
fn create_post_reaction_should_fail_post_not_found() {
    ExtBuilder::build().execute_with(|| {
        // Try to catch an error creating reaction by the same account
        assert_noop!(_create_default_post_reaction(), Error::<TestRuntime>::PostNotFound);
    });
}

#[test]
fn create_post_reaction_should_fail_space_is_hidden() {
    ExtBuilder::build_with_post().execute_with(|| {
        assert_ok!(_update_space(
          None,
          None,
          Some(self::space_update(None, None, Some(true)))
        ));

        assert_noop!(_create_default_post_reaction(), Error::<TestRuntime>::BannedToChangeReactionWhenHidden);
    });
}

#[test]
fn create_post_reaction_should_fail_post_is_hidden() {
    ExtBuilder::build_with_post().execute_with(|| {
        assert_ok!(_update_post(
          None,
          None,
          Some(self::post_update(None, None, Some(true)))
        ));

        assert_noop!(_create_default_post_reaction(), Error::<TestRuntime>::BannedToChangeReactionWhenHidden);
    });
}

#[test]
fn create_comment_reaction_should_work_upvote() {
    ExtBuilder::build_with_comment().execute_with(|| {
        assert_ok!(_create_comment_reaction(Some(Origin::signed(ACCOUNT2)), None, None)); // ReactionId 1 by ACCOUNT2

        // Check storages
        assert_eq!(Social::reaction_ids_by_post_id(2), vec![1]);
        assert_eq!(Social::next_reaction_id(), 2);

        // Check comment reaction counters
        let comment = Social::post_by_id(2).unwrap();
        assert_eq!(comment.upvotes_count, 1);
        assert_eq!(comment.downvotes_count, 0);

        // Check whether data stored correctly
        let reaction = Social::reaction_by_id(1).unwrap();
        assert_eq!(reaction.created.account, ACCOUNT2);
        assert_eq!(reaction.kind, self::reaction_upvote());
    });
}

#[test]
fn create_comment_reaction_should_work_downvote() {
    ExtBuilder::build_with_comment().execute_with(|| {
        assert_ok!(_create_comment_reaction(Some(Origin::signed(ACCOUNT2)), None, Some(self::reaction_downvote()))); // ReactionId 1 by ACCOUNT2

        // Check storages
        assert_eq!(Social::reaction_ids_by_post_id(2), vec![1]);
        assert_eq!(Social::next_reaction_id(), 2);

        // Check comment reaction counters
        let comment = Social::post_by_id(2).unwrap();
        assert_eq!(comment.upvotes_count, 0);
        assert_eq!(comment.downvotes_count, 1);

        // Check whether data stored correctly
        let reaction = Social::reaction_by_id(1).unwrap();
        assert_eq!(reaction.created.account, ACCOUNT2);
        assert_eq!(reaction.kind, self::reaction_downvote());
    });
}

#[test]
fn create_comment_reaction_should_fail_already_reacted() {
    ExtBuilder::build_with_comment().execute_with(|| {
        assert_ok!(_create_default_comment_reaction()); // ReactionId 1

        // Try to catch an error creating reaction by the same account
        assert_noop!(_create_default_comment_reaction(), Error::<TestRuntime>::AccountAlreadyReacted);
    });
}

#[test]
fn create_comment_reaction_should_fail_comment_not_found() {
    ExtBuilder::build().execute_with(|| {
        // Try to catch an error creating reaction by the same account
        assert_noop!(_create_default_comment_reaction(), Error::<TestRuntime>::PostNotFound);
    });
}

#[test]
fn create_comment_reaction_should_fail_space_is_hidden() {
    ExtBuilder::build_with_comment().execute_with(|| {
        assert_ok!(_update_space(
          None,
          None,
          Some(self::space_update(None, None, Some(true)))
        ));

        assert_noop!(_create_default_comment_reaction(), Error::<TestRuntime>::BannedToChangeReactionWhenHidden);
    });
}

#[test]
fn create_comment_reaction_should_fail_post_is_hidden() {
    ExtBuilder::build_with_comment().execute_with(|| {
        assert_ok!(_update_post(
          None,
          None,
          Some(self::post_update(None, None, Some(true)))
        ));

        assert_noop!(_create_default_comment_reaction(), Error::<TestRuntime>::BannedToChangeReactionWhenHidden);
    });
}

// Rating system tests

#[test]
fn score_diff_by_weights_check_result() {
    ExtBuilder::build().execute_with(|| {
        assert_eq!(Social::get_score_diff(1, self::scoring_action_upvote_post()), UpvotePostActionWeight::get() as i16);
        assert_eq!(Social::get_score_diff(1, self::scoring_action_downvote_post()), DownvotePostActionWeight::get() as i16);
        assert_eq!(Social::get_score_diff(1, self::scoring_action_share_post()), SharePostActionWeight::get() as i16);
        assert_eq!(Social::get_score_diff(1, self::scoring_action_create_comment()), CreateCommentActionWeight::get() as i16);
        assert_eq!(Social::get_score_diff(1, self::scoring_action_upvote_comment()), UpvoteCommentActionWeight::get() as i16);
        assert_eq!(Social::get_score_diff(1, self::scoring_action_downvote_comment()), DownvoteCommentActionWeight::get() as i16);
        assert_eq!(Social::get_score_diff(1, self::scoring_action_share_comment()), ShareCommentActionWeight::get() as i16);
        assert_eq!(Social::get_score_diff(1, self::scoring_action_follow_space()), FollowSpaceActionWeight::get() as i16);
        assert_eq!(Social::get_score_diff(1, self::scoring_action_follow_account()), FollowAccountActionWeight::get() as i16);
    });
}

#[test]
fn random_score_diff_check_result() {
    ExtBuilder::build().execute_with(|| {
        assert_eq!(Social::get_score_diff(32768, self::scoring_action_upvote_post()), 80); // 2^15
        assert_eq!(Social::get_score_diff(32769, self::scoring_action_upvote_post()), 80); // 2^15 + 1
        assert_eq!(Social::get_score_diff(65535, self::scoring_action_upvote_post()), 80); // 2^16 - 1
        assert_eq!(Social::get_score_diff(65536, self::scoring_action_upvote_post()), 85); // 2^16
    });
}

//--------------------------------------------------------------------------------------------------

#[test]
fn change_space_score_should_work_follow_space() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(Social::follow_space(Origin::signed(ACCOUNT2), 1));

        assert_eq!(Social::space_by_id(1).unwrap().score, FollowSpaceActionWeight::get() as i32);
        assert_eq!(Social::social_account_by_id(ACCOUNT1).unwrap().reputation, 1 + FollowSpaceActionWeight::get() as u32);
        assert_eq!(Social::social_account_by_id(ACCOUNT2).unwrap().reputation, 1);
    });
}

#[test]
fn change_space_score_should_work_revert_follow_space() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(Social::follow_space(Origin::signed(ACCOUNT2), 1));
        assert_ok!(Social::unfollow_space(Origin::signed(ACCOUNT2), 1));

        assert_eq!(Social::space_by_id(1).unwrap().score, 0);
        assert_eq!(Social::social_account_by_id(ACCOUNT1).unwrap().reputation, 1);
        assert_eq!(Social::social_account_by_id(ACCOUNT2).unwrap().reputation, 1);
    });
}

#[test]
fn change_space_score_should_work_upvote_post() {
    ExtBuilder::build_with_post().execute_with(|| {
        assert_ok!(_create_post_reaction(Some(Origin::signed(ACCOUNT2)), None, None)); // ReactionId 1

        assert_eq!(Social::space_by_id(1).unwrap().score, UpvotePostActionWeight::get() as i32);
        assert_eq!(Social::social_account_by_id(ACCOUNT1).unwrap().reputation, 1 + UpvotePostActionWeight::get() as u32);
    });
}

#[test]
fn change_space_score_should_work_downvote_post() {
    ExtBuilder::build_with_post().execute_with(|| {
        assert_ok!(_create_post_reaction(Some(Origin::signed(ACCOUNT2)), None, Some(self::reaction_downvote()))); // ReactionId 1

        assert_eq!(Social::space_by_id(1).unwrap().score, DownvotePostActionWeight::get() as i32);
        assert_eq!(Social::social_account_by_id(ACCOUNT1).unwrap().reputation, 1);
    });
}

//--------------------------------------------------------------------------------------------------

#[test]
fn change_post_score_should_work_create_comment() {
    ExtBuilder::build_with_post().execute_with(|| {
        assert_ok!(_create_comment(Some(Origin::signed(ACCOUNT2)), None, None, None)); // PostId 2

        assert_eq!(Social::post_by_id(1).unwrap().score, CreateCommentActionWeight::get() as i32);
        assert_eq!(Social::space_by_id(1).unwrap().score, CreateCommentActionWeight::get() as i32);
        assert_eq!(Social::social_account_by_id(ACCOUNT1).unwrap().reputation, 1 + CreateCommentActionWeight::get() as u32);
        assert_eq!(Social::post_score_by_account((ACCOUNT2, 1, self::scoring_action_create_comment())), Some(CreateCommentActionWeight::get()));
    });
}

#[test]
fn change_post_score_should_work_upvote() {
    ExtBuilder::build_with_post().execute_with(|| {
        assert_ok!(_create_post_reaction(Some(Origin::signed(ACCOUNT2)), None, None));

        assert_eq!(Social::post_by_id(1).unwrap().score, UpvotePostActionWeight::get() as i32);
        assert_eq!(Social::social_account_by_id(ACCOUNT1).unwrap().reputation, 1 + UpvotePostActionWeight::get() as u32);
        assert_eq!(Social::post_score_by_account((ACCOUNT2, 1, self::scoring_action_upvote_post())), Some(UpvotePostActionWeight::get()));
    });
}

#[test]
fn change_post_score_should_work_downvote() {
    ExtBuilder::build_with_post().execute_with(|| {
        assert_ok!(_create_post_reaction(Some(Origin::signed(ACCOUNT2)), None, Some(self::reaction_downvote())));

        assert_eq!(Social::post_by_id(1).unwrap().score, DownvotePostActionWeight::get() as i32);
        assert_eq!(Social::social_account_by_id(ACCOUNT1).unwrap().reputation, 1);
        assert_eq!(Social::post_score_by_account((ACCOUNT2, 1, self::scoring_action_downvote_post())), Some(DownvotePostActionWeight::get()));
    });
}

#[test]
fn change_post_score_should_revert_upvote() {
    ExtBuilder::build_with_post().execute_with(|| {
        assert_ok!(_create_post_reaction(Some(Origin::signed(ACCOUNT2)), None, None)); // ReactionId 1
        assert_ok!(_delete_post_reaction(Some(Origin::signed(ACCOUNT2)), None, 1));

        assert_eq!(Social::post_by_id(1).unwrap().score, 0);
        assert_eq!(Social::social_account_by_id(ACCOUNT1).unwrap().reputation, 1);
        assert_eq!(Social::post_score_by_account((ACCOUNT2, 1, self::scoring_action_upvote_post())), None);
    });
}

#[test]
fn change_post_score_should_revert_downvote() {
    ExtBuilder::build_with_post().execute_with(|| {
        assert_ok!(_create_post_reaction(Some(Origin::signed(ACCOUNT2)), None, Some(self::reaction_downvote()))); // ReactionId 1
        assert_ok!(_delete_post_reaction(Some(Origin::signed(ACCOUNT2)), None, 1));

        assert_eq!(Social::post_by_id(1).unwrap().score, 0);
        assert_eq!(Social::social_account_by_id(ACCOUNT1).unwrap().reputation, 1);
        assert_eq!(Social::post_score_by_account((ACCOUNT2, 1, self::scoring_action_downvote_post())), None);
    });
}

#[test]
fn change_post_score_cancel_upvote_with_downvote() {
    ExtBuilder::build_with_post().execute_with(|| {
        assert_ok!(_create_post_reaction(Some(Origin::signed(ACCOUNT2)), None, None)); // ReactionId 1
        assert_ok!(_update_post_reaction(Some(Origin::signed(ACCOUNT2)), None, 1, Some(self::reaction_downvote())));

        assert_eq!(Social::post_by_id(1).unwrap().score, DownvotePostActionWeight::get() as i32);
        assert_eq!(Social::social_account_by_id(ACCOUNT1).unwrap().reputation, 1);
        assert_eq!(Social::post_score_by_account((ACCOUNT2, 1, self::scoring_action_upvote_post())), None);
        assert_eq!(Social::post_score_by_account((ACCOUNT2, 1, self::scoring_action_downvote_post())), Some(DownvotePostActionWeight::get()));
    });
}

#[test]
fn change_post_score_cancel_downvote_with_upvote() {
    ExtBuilder::build_with_post().execute_with(|| {
        assert_ok!(_create_post_reaction(Some(Origin::signed(ACCOUNT2)), None, Some(self::reaction_downvote()))); // ReactionId 1
        assert_ok!(_update_post_reaction(Some(Origin::signed(ACCOUNT2)), None, 1, None));

        assert_eq!(Social::post_by_id(1).unwrap().score, UpvotePostActionWeight::get() as i32);
        assert_eq!(Social::social_account_by_id(ACCOUNT1).unwrap().reputation, 1 + UpvotePostActionWeight::get() as u32);
        assert_eq!(Social::post_score_by_account((ACCOUNT2, 1, self::scoring_action_downvote_post())), None);
        assert_eq!(Social::post_score_by_account((ACCOUNT2, 1, self::scoring_action_upvote_post())), Some(UpvotePostActionWeight::get()));
    });
}

#[test]
fn change_post_score_should_fail_post_not_found() {
    ExtBuilder::build().execute_with(|| {
        let fake_post = &mut self::fake_post(
            1,
            ACCOUNT1,
            None,
            PostExtension::RegularPost
        );

        assert_ok!(_create_default_space()); // SpaceId 1
        assert_noop!(_change_post_score_by_extension(
          ACCOUNT1, fake_post, self::scoring_action_upvote_post()
        ), Error::<TestRuntime>::PostNotFound);
    });
}

//--------------------------------------------------------------------------------------------------

#[test]
fn change_social_account_reputation_should_work_max_score_diff() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(_create_post(Some(Origin::signed(ACCOUNT1)), None, None, None));
        assert_ok!(Social::change_social_account_reputation(
          ACCOUNT1,
          ACCOUNT2,
          std::i16::MAX,
          self::scoring_action_follow_account())
        );
    });
}

#[test]
fn change_social_account_reputation_should_work_min_score_diff() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(_create_post(Some(Origin::signed(ACCOUNT1)), None, None, None));
        assert_ok!(Social::change_social_account_reputation(
          ACCOUNT1,
          ACCOUNT2,
          std::i16::MIN,
          self::scoring_action_follow_account())
        );
    });
}

#[test]
fn change_social_account_reputation_should_work() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(_create_post(Some(Origin::signed(ACCOUNT1)), None, None, None));
        assert_ok!(Social::change_social_account_reputation(
          ACCOUNT1,
          ACCOUNT2,
          DownvotePostActionWeight::get(),
          self::scoring_action_downvote_post())
        );
        assert_eq!(Social::account_reputation_diff_by_account((ACCOUNT2, ACCOUNT1, self::scoring_action_downvote_post())), Some(0));
        assert_eq!(Social::social_account_by_id(ACCOUNT1).unwrap().reputation, 1);

        // To ensure function works correctly, multiply default UpvotePostActionWeight by two
        assert_ok!(Social::change_social_account_reputation(
          ACCOUNT1,
          ACCOUNT2,
          UpvotePostActionWeight::get() * 2,
          self::scoring_action_upvote_post())
        );

        assert_eq!(Social::account_reputation_diff_by_account(
            (ACCOUNT2, ACCOUNT1, self::scoring_action_upvote_post())
        ), Some(UpvotePostActionWeight::get() * 2));

        assert_eq!(Social::social_account_by_id(ACCOUNT1).unwrap().reputation, 1 + (UpvotePostActionWeight::get() * 2) as u32);
    });
}

//--------------------------------------------------------------------------------------------------

#[test]
fn change_comment_score_should_work_upvote() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(_create_post(Some(Origin::signed(ACCOUNT1)), None, None, None)); // PostId 1
        assert_ok!(_create_comment(Some(Origin::signed(ACCOUNT2)), None, None, None)); // PostId 2

        assert_ok!(_change_post_score_by_extension_with_id(ACCOUNT3, 2, self::scoring_action_upvote_comment()));

        assert_eq!(Social::post_by_id(2).unwrap().score, UpvoteCommentActionWeight::get() as i32);
        assert_eq!(Social::social_account_by_id(ACCOUNT1).unwrap().reputation, 1 + CreateCommentActionWeight::get() as u32);
        assert_eq!(Social::social_account_by_id(ACCOUNT2).unwrap().reputation, 1 + UpvoteCommentActionWeight::get() as u32);
        assert_eq!(Social::social_account_by_id(ACCOUNT3).unwrap().reputation, 1);
        assert_eq!(Social::post_score_by_account((ACCOUNT3, 2, self::scoring_action_upvote_comment())), Some(UpvoteCommentActionWeight::get()));
    });
}

#[test]
fn change_comment_score_should_work_downvote() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(_create_post(Some(Origin::signed(ACCOUNT1)), None, None, None)); // PostId 1
        assert_ok!(_create_comment(Some(Origin::signed(ACCOUNT2)), None, None, None)); // PostId 2

        assert_ok!(_change_post_score_by_extension_with_id(ACCOUNT3, 2, self::scoring_action_downvote_comment()));

        assert_eq!(Social::post_by_id(2).unwrap().score, DownvoteCommentActionWeight::get() as i32);
        assert_eq!(Social::social_account_by_id(ACCOUNT1).unwrap().reputation, 1 + CreateCommentActionWeight::get() as u32);
        assert_eq!(Social::social_account_by_id(ACCOUNT2).unwrap().reputation, 1);
        assert_eq!(Social::social_account_by_id(ACCOUNT3).unwrap().reputation, 1);
        assert_eq!(Social::post_score_by_account((ACCOUNT3, 2, self::scoring_action_downvote_comment())), Some(DownvoteCommentActionWeight::get()));
    });
}

#[test]
fn change_comment_score_should_revert_upvote() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(_create_post(Some(Origin::signed(ACCOUNT1)), None, None, None)); // PostId 1
        assert_ok!(_create_comment(Some(Origin::signed(ACCOUNT2)), None, None, None)); // PostId 2

        assert_ok!(_change_post_score_by_extension_with_id(ACCOUNT3, 2, self::scoring_action_upvote_comment()));
        assert_ok!(_change_post_score_by_extension_with_id(ACCOUNT3, 2, self::scoring_action_upvote_comment()));

        assert_eq!(Social::post_by_id(2).unwrap().score, 0);
        assert_eq!(Social::social_account_by_id(ACCOUNT1).unwrap().reputation, 1 + CreateCommentActionWeight::get() as u32);
        assert_eq!(Social::social_account_by_id(ACCOUNT2).unwrap().reputation, 1);
        assert_eq!(Social::social_account_by_id(ACCOUNT3).unwrap().reputation, 1);
        assert_eq!(Social::post_score_by_account((ACCOUNT1, 2, self::scoring_action_upvote_comment())), None);
    });
}

#[test]
fn change_comment_score_should_revert_downvote() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(_create_post(Some(Origin::signed(ACCOUNT1)), None, None, None)); // PostId 1
        assert_ok!(_create_comment(Some(Origin::signed(ACCOUNT2)), None, None, None)); // PostId 2

        assert_ok!(_change_post_score_by_extension_with_id(ACCOUNT3, 2, self::scoring_action_downvote_comment()));
        assert_ok!(_change_post_score_by_extension_with_id(ACCOUNT3, 2, self::scoring_action_downvote_comment()));

        assert_eq!(Social::post_by_id(2).unwrap().score, 0);
        assert_eq!(Social::social_account_by_id(ACCOUNT1).unwrap().reputation, 1 + CreateCommentActionWeight::get() as u32);
        assert_eq!(Social::social_account_by_id(ACCOUNT2).unwrap().reputation, 1);
        assert_eq!(Social::social_account_by_id(ACCOUNT3).unwrap().reputation, 1);
        assert_eq!(Social::post_score_by_account((ACCOUNT1, 2, self::scoring_action_downvote_comment())), None);
    });
}

#[test]
fn change_comment_score_check_cancel_upvote() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(_create_post(Some(Origin::signed(ACCOUNT1)), None, None, None)); // PostId 1
        assert_ok!(_create_comment(Some(Origin::signed(ACCOUNT2)), None, None, None)); // PostId 2

        assert_ok!(_change_post_score_by_extension_with_id(ACCOUNT3, 2, self::scoring_action_upvote_comment()));
        assert_ok!(_change_post_score_by_extension_with_id(ACCOUNT3, 2, self::scoring_action_downvote_comment()));

        assert_eq!(Social::post_by_id(2).unwrap().score, DownvoteCommentActionWeight::get() as i32);
        assert_eq!(Social::social_account_by_id(ACCOUNT1).unwrap().reputation, 1 + CreateCommentActionWeight::get() as u32);
        assert_eq!(Social::social_account_by_id(ACCOUNT2).unwrap().reputation, 1);
        assert_eq!(Social::social_account_by_id(ACCOUNT3).unwrap().reputation, 1);
        assert_eq!(Social::post_score_by_account((ACCOUNT3, 2, self::scoring_action_upvote_comment())), None);
        assert_eq!(Social::post_score_by_account((ACCOUNT3, 2, self::scoring_action_downvote_comment())), Some(DownvoteCommentActionWeight::get()));
    });
}

#[test]
fn change_comment_score_check_cancel_downvote() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(_create_post(Some(Origin::signed(ACCOUNT1)), None, None, None)); // PostId 1
        assert_ok!(_create_comment(Some(Origin::signed(ACCOUNT2)), None, None, None)); // PostId 2

        assert_ok!(_change_post_score_by_extension_with_id(ACCOUNT3, 2, self::scoring_action_downvote_comment()));
        assert_ok!(_change_post_score_by_extension_with_id(ACCOUNT3, 2, self::scoring_action_upvote_comment()));

        assert_eq!(Social::post_by_id(2).unwrap().score, UpvoteCommentActionWeight::get() as i32);
        assert_eq!(Social::social_account_by_id(ACCOUNT1).unwrap().reputation, 1 + CreateCommentActionWeight::get() as u32);
        assert_eq!(Social::social_account_by_id(ACCOUNT2).unwrap().reputation, 1 + UpvoteCommentActionWeight::get() as u32);
        assert_eq!(Social::social_account_by_id(ACCOUNT3).unwrap().reputation, 1);
        assert_eq!(Social::post_score_by_account((ACCOUNT3, 2, self::scoring_action_downvote_comment())), None);
        assert_eq!(Social::post_score_by_account((ACCOUNT3, 2, self::scoring_action_upvote_comment())), Some(UpvoteCommentActionWeight::get()));
    });
}

#[test]
fn change_comment_score_should_fail_comment_not_found() {
    ExtBuilder::build_with_post().execute_with(|| {
        let fake_post = &mut self::fake_post(
            3,
            ACCOUNT1,
            None,
            PostExtension::Comment(CommentExt{
                parent_id: None,
                root_post_id: 1
            })
        );

        assert_ok!(_create_comment(Some(Origin::signed(ACCOUNT2)), None, None, None)); // PostId 2

        assert_noop!(_change_post_score_by_extension(
          ACCOUNT1, fake_post, self::scoring_action_upvote_comment()
        ), Error::<TestRuntime>::PostNotFound);
    });
}

// Shares tests

#[test]
fn share_post_should_work() {
    ExtBuilder::build_with_post().execute_with(|| {
        assert_ok!(_create_space(Some(Origin::signed(ACCOUNT2)), Some(Some(b"space2_handle".to_vec())), None)); // SpaceId 2 by ACCOUNT2

        assert_ok!(_create_post(
          Some(Origin::signed(ACCOUNT2)),
          Some(Some(2)),
          Some(self::extension_shared_post(1)),
          None
        )); // Share PostId 1 on SpaceId 2 by ACCOUNT2

        // Check storages
        assert_eq!(Social::post_ids_by_space_id(1), vec![1]);
        assert_eq!(Social::post_ids_by_space_id(2), vec![2]);
        assert_eq!(Social::next_post_id(), 3);

        assert_eq!(Social::post_shares_by_account((ACCOUNT2, 1)), 1);
        assert_eq!(Social::shared_post_ids_by_original_post_id(1), vec![2]);

        // Check whether data stored correctly
        assert_eq!(Social::post_by_id(1).unwrap().shares_count, 1);

        let shared_post = Social::post_by_id(2).unwrap();

        assert_eq!(shared_post.space_id, Some(2));
        assert_eq!(shared_post.created.account, ACCOUNT2);
        assert_eq!(shared_post.extension, self::extension_shared_post(1));
    });
}

#[test]
fn share_post_should_work_share_own_post() {
    ExtBuilder::build_with_post().execute_with(|| {
        assert_ok!(_create_post(
          Some(Origin::signed(ACCOUNT1)),
          Some(Some(1)),
          Some(self::extension_shared_post(1)),
          None
        )); // Share PostId 1

        // Check storages
        assert_eq!(Social::post_ids_by_space_id(1), vec![1, 2]);
        assert_eq!(Social::next_post_id(), 3);

        assert_eq!(Social::post_shares_by_account((ACCOUNT1, 1)), 1);
        assert_eq!(Social::shared_post_ids_by_original_post_id(1), vec![2]);

        // Check whether data stored correctly
        assert_eq!(Social::post_by_id(1).unwrap().shares_count, 1);

        let shared_post = Social::post_by_id(2).unwrap();
        assert_eq!(shared_post.space_id, Some(1));
        assert_eq!(shared_post.created.account, ACCOUNT1);
        assert_eq!(shared_post.extension, self::extension_shared_post(1));
    });
}

#[test]
fn share_post_should_change_score() {
    ExtBuilder::build_with_post().execute_with(|| {
        assert_ok!(_create_space(Some(Origin::signed(ACCOUNT2)), Some(Some(b"space2_handle".to_vec())), None)); // SpaceId 2 by ACCOUNT2

        assert_ok!(_create_post(
          Some(Origin::signed(ACCOUNT2)),
          Some(Some(2)),
          Some(self::extension_shared_post(1)),
          None
        )); // Share PostId 1 on SpaceId 2 by ACCOUNT2

        assert_eq!(Social::post_by_id(1).unwrap().score, SharePostActionWeight::get() as i32);
        assert_eq!(Social::social_account_by_id(ACCOUNT1).unwrap().reputation, 1 + SharePostActionWeight::get() as u32);
        assert_eq!(Social::post_score_by_account((ACCOUNT2, 1, self::scoring_action_share_post())), Some(SharePostActionWeight::get()));
    });
}

#[test]
fn share_post_should_not_change_score() {
    ExtBuilder::build_with_post().execute_with(|| {
        assert_ok!(_create_post(
          Some(Origin::signed(ACCOUNT1)),
          Some(Some(1)),
          Some(self::extension_shared_post(1)),
          None
        )); // Share PostId

        assert_eq!(Social::post_by_id(1).unwrap().score, 0);
        assert_eq!(Social::social_account_by_id(ACCOUNT1).unwrap().reputation, 1);
        assert_eq!(Social::post_score_by_account((ACCOUNT1, 1, self::scoring_action_share_post())), None);
    });
}

#[test]
fn share_post_should_fail_original_post_not_found() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(_create_space(Some(Origin::signed(ACCOUNT2)), Some(Some(b"space2_handle".to_vec())), None)); // SpaceId 2 by ACCOUNT2
        // Skipped creating PostId 1
        assert_noop!(_create_post(
          Some(Origin::signed(ACCOUNT2)),
          Some(Some(2)),
          Some(self::extension_shared_post(1)),
          None
        ), Error::<TestRuntime>::OriginalPostNotFound);
    });
}

#[test]
fn share_post_should_fail_share_shared_post() {
    ExtBuilder::build_with_post().execute_with(|| {
        assert_ok!(_create_space(Some(Origin::signed(ACCOUNT2)), Some(Some(b"space2_handle".to_vec())), None)); // SpaceId 2 by ACCOUNT2

        assert_ok!(_create_post(
          Some(Origin::signed(ACCOUNT2)),
          Some(Some(2)),
          Some(self::extension_shared_post(1)),
          None)
        );

        // Try to share post with extension SharedPost
        assert_noop!(_create_post(
          Some(Origin::signed(ACCOUNT1)),
          Some(Some(1)),
          Some(self::extension_shared_post(2)),
          None
        ), Error::<TestRuntime>::CannotShareSharedPost);
    });
}

#[test]
fn share_comment_should_work() {
    ExtBuilder::build_with_comment().execute_with(|| {
        assert_ok!(_create_space(Some(Origin::signed(ACCOUNT2)), Some(Some(b"space2_handle".to_vec())), None)); // SpaceId 2 by ACCOUNT2

        assert_ok!(_create_post(
          Some(Origin::signed(ACCOUNT2)),
          Some(Some(2)),
          Some(self::extension_shared_post(2)),
          None
        )); // Share PostId 2 comment on SpaceId 2 by ACCOUNT2

        // Check storages
        assert_eq!(Social::post_ids_by_space_id(1), vec![1]);
        assert_eq!(Social::post_ids_by_space_id(2), vec![3]);
        assert_eq!(Social::next_post_id(), 4);

        assert_eq!(Social::post_shares_by_account((ACCOUNT2, 2)), 1);
        assert_eq!(Social::shared_post_ids_by_original_post_id(2), vec![3]);

        // Check whether data stored correctly
        assert_eq!(Social::post_by_id(2).unwrap().shares_count, 1);

        let shared_post = Social::post_by_id(3).unwrap();

        assert_eq!(shared_post.space_id, Some(2));
        assert_eq!(shared_post.created.account, ACCOUNT2);
        assert_eq!(shared_post.extension, self::extension_shared_post(2));
    });
}

#[test]
fn share_comment_should_change_score() {
    ExtBuilder::build_with_comment().execute_with(|| {
        assert_ok!(_create_space(Some(Origin::signed(ACCOUNT2)), Some(Some(b"space2_handle".to_vec())), None)); // SpaceId 2 by ACCOUNT2

        assert_ok!(_create_post(
          Some(Origin::signed(ACCOUNT2)),
          Some(Some(2)),
          Some(self::extension_shared_post(2)),
          None
        )); // Share PostId 2 comment on SpaceId 2 by ACCOUNT2

        assert_eq!(Social::post_by_id(2).unwrap().score, ShareCommentActionWeight::get() as i32);
        assert_eq!(Social::social_account_by_id(ACCOUNT1).unwrap().reputation, 1 + ShareCommentActionWeight::get() as u32);
        assert_eq!(Social::post_score_by_account((ACCOUNT2, 2, self::scoring_action_share_comment())), Some(ShareCommentActionWeight::get()));
    });
}

#[test]
fn share_comment_should_fail_original_comment_not_found() {
    ExtBuilder::build_with_post().execute_with(|| {
        assert_ok!(_create_space(Some(Origin::signed(ACCOUNT2)), Some(Some(b"space2_handle".to_vec())), None)); // SpaceId 2 by ACCOUNT2

        // Skipped creating comment with PostId 2
        assert_noop!(_create_post(
          Some(Origin::signed(ACCOUNT2)),
          Some(Some(2)),
          Some(self::extension_shared_post(2)),
          None
         ), Error::<TestRuntime>::OriginalPostNotFound);
    });
}

// Profiles tests

#[test]
fn create_profile_should_work() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_create_default_profile()); // AccountId 1

        let profile = Social::social_account_by_id(ACCOUNT1).unwrap().profile.unwrap();
        assert_eq!(profile.created.account, ACCOUNT1);
        // TODO: Fix unresolved error
        // assert_eq!(profile.updated, None);
        assert_eq!(profile.username, self::alice_username());
        assert_eq!(profile.ipfs_hash, self::profile_ipfs_hash());
        assert!(profile.edit_history.is_empty());
        assert_eq!(Social::account_by_profile_username(self::alice_username()), Some(ACCOUNT1));
    });
}

#[test]
fn create_profile_should_fail_profile_exists() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_create_default_profile()); // AccountId 1
        assert_noop!(_create_default_profile(), Error::<TestRuntime>::ProfileAlreadyExists);
    });
}

#[test]
fn create_profile_should_fail_invalid_ipfs_hash() {
    ExtBuilder::build().execute_with(|| {
        let ipfs_hash : Vec<u8> = b"QmV9tSDx9UiPeWExXEeH6aoDvmihvx6j".to_vec();
        assert_noop!(_create_profile(None, None, Some(ipfs_hash)), UtilsError::<TestRuntime>::IpfsIsIncorrect);
    });
}

#[test]
fn create_profile_should_fail_username_is_busy() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_create_default_profile()); // AccountId 1
        assert_noop!(_create_profile(Some(Origin::signed(ACCOUNT2)), None, None), Error::<TestRuntime>::UsernameIsBusy);
    });
}

#[test]
fn create_profile_should_fail_too_short_username() {
    ExtBuilder::build().execute_with(|| {
        let username : Vec<u8> = vec![97; (MinUsernameLen::get() - 1) as usize];

        assert_ok!(_create_default_profile()); // AccountId 1
        assert_noop!(_create_profile(Some(Origin::signed(ACCOUNT2)), Some(username), None), Error::<TestRuntime>::UsernameIsTooShort);
    });
}

#[test]
fn create_profile_should_fail_too_long_username() {
    ExtBuilder::build().execute_with(|| {
        let username : Vec<u8> = vec![97; (MaxUsernameLen::get() + 1) as usize];

        assert_ok!(_create_default_profile()); // AccountId 1
        assert_noop!(_create_profile(Some(Origin::signed(ACCOUNT2)), Some(username), None), Error::<TestRuntime>::UsernameIsTooLong);
    });
}

#[test]
fn create_profile_should_fail_invalid_username() {
    ExtBuilder::build().execute_with(|| {
        let username : Vec<u8> = b"{}sername".to_vec();

        assert_ok!(_create_default_profile()); // AccountId 1
        assert_noop!(_create_profile(Some(Origin::signed(ACCOUNT2)), Some(username), None), Error::<TestRuntime>::UsernameIsNotAlphanumeric);
    });
}

#[test]
fn update_profile_should_work() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_create_default_profile()); // AccountId 1
        assert_ok!(_update_profile(None, Some(self::bob_username()), Some(self::space_ipfs_hash())));

        // Check whether profile updated correctly
        let profile = Social::social_account_by_id(ACCOUNT1).unwrap().profile.unwrap();
        assert!(profile.updated.is_some());
        assert_eq!(profile.username, self::bob_username());
        assert_eq!(profile.ipfs_hash, self::space_ipfs_hash());

        // Check storages
        assert_eq!(Social::account_by_profile_username(self::alice_username()), None);
        assert_eq!(Social::account_by_profile_username(self::bob_username()), Some(ACCOUNT1));

        // Check whether profile history is written correctly
        assert_eq!(profile.edit_history[0].old_data.username, Some(self::alice_username()));
        assert_eq!(profile.edit_history[0].old_data.ipfs_hash, Some(self::profile_ipfs_hash()));
    });
}

#[test]
fn update_profile_should_fail_no_social_account() {
    ExtBuilder::build().execute_with(|| {
        assert_noop!(_update_profile(None, Some(self::bob_username()), None), Error::<TestRuntime>::SocialAccountNotFound);
    });
}

#[test]
fn update_profile_should_fail_no_profile() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(Social::follow_account(Origin::signed(ACCOUNT1), ACCOUNT2));
        assert_noop!(_update_profile(None, Some(self::bob_username()), None), Error::<TestRuntime>::ProfileDoesNotExist);
    });
}

#[test]
fn update_profile_should_fail_nothing_to_update() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_create_default_profile()); // AccountId 1
        assert_noop!(_update_profile(None, None, None), Error::<TestRuntime>::NoUpdatesInProfile);
    });
}

#[test]
fn update_profile_should_fail_username_is_busy() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_create_default_profile()); // AccountId 1
        assert_ok!(_create_profile(Some(Origin::signed(ACCOUNT2)), Some(self::bob_username()), None));
        assert_noop!(_update_profile(None, Some(self::bob_username()), None), Error::<TestRuntime>::UsernameIsBusy);
    });
}

#[test]
fn update_profile_should_fail_too_short_username() {
    ExtBuilder::build().execute_with(|| {
        let username : Vec<u8> = vec![97; (MinUsernameLen::get() - 1) as usize];

        assert_ok!(_create_default_profile()); // AccountId 1
        assert_noop!(_update_profile(None, Some(username), None), Error::<TestRuntime>::UsernameIsTooShort);
    });
}

#[test]
fn update_profile_should_fail_too_long_username() {
    ExtBuilder::build().execute_with(|| {
        let username : Vec<u8> = vec![97; (MaxUsernameLen::get() + 1) as usize];

        assert_ok!(_create_default_profile()); // AccountId 1
        assert_noop!(_update_profile(None, Some(username), None), Error::<TestRuntime>::UsernameIsTooLong);
    });
}

#[test]
fn update_profile_should_fail_invalid_username() {
    ExtBuilder::build().execute_with(|| {
        let username : Vec<u8> = b"{}sername".to_vec();

        assert_ok!(_create_default_profile()); // AccountId 1
        assert_noop!(_update_profile(None, Some(username), None), Error::<TestRuntime>::UsernameIsNotAlphanumeric);
    });
}

#[test]
fn update_profile_should_fail_invalid_ipfs_hash() {
    ExtBuilder::build().execute_with(|| {
        let ipfs_hash : Vec<u8> = b"QmV9tSDx9UiPeWExXEeH6aoDvmihvx6j".to_vec();

        assert_ok!(_create_default_profile());
        assert_noop!(_update_profile(None, None, Some(ipfs_hash)), UtilsError::<TestRuntime>::IpfsIsIncorrect);
    });
}

// Space following tests

#[test]
fn follow_space_should_work() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(_default_follow_space()); // Follow SpaceId 1 by ACCOUNT2

        assert_eq!(Social::space_by_id(1).unwrap().followers_count, 2);
        assert_eq!(Social::spaces_followed_by_account(ACCOUNT2), vec![1]);
        assert_eq!(Social::space_followers(1), vec![ACCOUNT1, ACCOUNT2]);
        assert_eq!(Social::space_followed_by_account((ACCOUNT2, 1)), true);
    });
}

#[test]
fn follow_space_should_fail_space_not_found() {
    ExtBuilder::build().execute_with(|| {
        assert_noop!(_default_follow_space(), Error::<TestRuntime>::SpaceNotFound);
    });
}

#[test]
fn follow_space_should_fail_already_following() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(_default_follow_space()); // Follow SpaceId 1 by ACCOUNT2

        assert_noop!(_default_follow_space(), Error::<TestRuntime>::AccountIsFollowingSpace);
    });
}

#[test]
fn follow_space_should_fail_space_is_hidden() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(_update_space(
          None,
          None,
          Some(self::space_update(None, None, Some(true)))
        ));

        assert_noop!(_default_follow_space(), Error::<TestRuntime>::BannedToFollowWhenHidden);
    });
}

#[test]
fn unfollow_space_should_work() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(_default_follow_space()); // Follow SpaceId 1 by ACCOUNT2
        assert_ok!(_default_unfollow_space());

        assert_eq!(Social::space_by_id(1).unwrap().followers_count, 1);
        assert!(Social::spaces_followed_by_account(ACCOUNT2).is_empty());
        assert_eq!(Social::space_followers(1), vec![ACCOUNT1]);
    });
}

#[test]
fn unfollow_space_should_fail_space_not_found() {
    ExtBuilder::build().execute_with(|| {
        assert_noop!(_default_unfollow_space(), Error::<TestRuntime>::SpaceNotFound);
    });
}

#[test]
fn unfollow_space_should_fail_already_following() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_noop!(_default_unfollow_space(), Error::<TestRuntime>::AccountIsNotFollowingSpace);
    });
}

// Account following tests

#[test]
fn follow_account_should_work() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_default_follow_account()); // Follow ACCOUNT1 by ACCOUNT2

        assert_eq!(Social::accounts_followed_by_account(ACCOUNT2), vec![ACCOUNT1]);
        assert_eq!(Social::account_followers(ACCOUNT1), vec![ACCOUNT2]);
        assert_eq!(Social::account_followed_by_account((ACCOUNT2, ACCOUNT1)), true);
    });
}

#[test]
fn follow_account_should_fail_follow_itself() {
    ExtBuilder::build().execute_with(|| {
        assert_noop!(_follow_account(None, Some(ACCOUNT2)), Error::<TestRuntime>::AccountCannotFollowItself);
    });
}

#[test]
fn follow_account_should_fail_already_followed() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_default_follow_account());

        assert_noop!(_default_follow_account(), Error::<TestRuntime>::AccountIsAlreadyFollowed);
    });
}

#[test]
fn unfollow_account_should_work() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_default_follow_account()); // Follow ACCOUNT1 by ACCOUNT2
        assert_ok!(_default_unfollow_account());

        assert!(Social::accounts_followed_by_account(ACCOUNT2).is_empty());
        assert!(Social::account_followers(ACCOUNT1).is_empty());
        assert_eq!(Social::account_followed_by_account((ACCOUNT2, ACCOUNT1)), false);
    });
}

#[test]
fn unfollow_account_should_fail_unfollow_itself() {
    ExtBuilder::build().execute_with(|| {
        assert_noop!(_unfollow_account(None, Some(ACCOUNT2)), Error::<TestRuntime>::AccountCannotUnfollowItself);
    });
}

#[test]
fn unfollow_account_should_fail_is_not_followed() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_default_follow_account());
        assert_ok!(_default_unfollow_account());

        assert_noop!(_default_unfollow_account(), Error::<TestRuntime>::AccountIsNotFollowed);
    });
}

// Transfer ownership tests

#[test]
fn transfer_space_ownership_should_work() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(_transfer_default_space_ownership()); // Transfer SpaceId 1 owned by ACCOUNT1 to ACCOUNT2

        assert_eq!(Social::pending_space_owner(1).unwrap(), ACCOUNT2);
    });
}

#[test]
fn transfer_space_ownership_should_fail_space_not_found() {
    ExtBuilder::build().execute_with(|| {
        assert_noop!(_transfer_default_space_ownership(), Error::<TestRuntime>::SpaceNotFound);
    });
}

#[test]
fn transfer_space_ownership_should_fail_not_an_owner() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_noop!(_transfer_space_ownership(
          Some(Origin::signed(ACCOUNT2)),
          None,
          Some(ACCOUNT1)
        ), Error::<TestRuntime>::NotASpaceOwner);
    });
}

#[test]
fn transfer_space_ownership_should_fail_transferring_to_current_owner() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_noop!(_transfer_space_ownership(
          Some(Origin::signed(ACCOUNT1)),
          None,
          Some(ACCOUNT1)
        ), Error::<TestRuntime>::CannotTranferToCurrentOwner);
    });
}

#[test]
fn accept_pending_ownership_should_work() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(_transfer_default_space_ownership()); // Transfer SpaceId 1 owned by ACCOUNT1 to ACCOUNT2
        assert_ok!(_accept_default_pending_ownership()); // Accepting a transfer from ACCOUNT2
        // Check whether owner was changed
        let space = Social::space_by_id(1).unwrap();
        assert_eq!(space.owner, ACCOUNT2);

        // Check whether storage state is correct
        assert!(Social::pending_space_owner(1).is_none());
    });
}

#[test]
fn accept_pending_ownership_should_fail_space_not_found() {
    ExtBuilder::build().execute_with(|| {
        // TODO: after adding new test externality
    });
}

#[test]
fn accept_pending_ownership_should_fail_no_pending_transfer() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_noop!(_accept_default_pending_ownership(), Error::<TestRuntime>::NoPendingTransferOnSpace);
    });
}

#[test]
fn accept_pending_ownership_should_fail_not_allowed_to_accept() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(_transfer_default_space_ownership());

        assert_noop!(_accept_pending_ownership(
      Some(Origin::signed(ACCOUNT1)),
      None
    ), Error::<TestRuntime>::NotAllowedToAcceptOwnershipTransfer);
    });
}

#[test]
fn reject_pending_ownership_should_work() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(_transfer_default_space_ownership()); // Transfer SpaceId 1 owned by ACCOUNT1 to ACCOUNT2
        assert_ok!(_reject_default_pending_ownership()); // Rejecting a transfer from ACCOUNT2

        // Check whether owner was not changed
        let space = Social::space_by_id(1).unwrap();
        assert_eq!(space.owner, ACCOUNT1);

        // Check whether storage state is correct
        assert!(Social::pending_space_owner(1).is_none());
    });
}

#[test]
fn reject_pending_ownership_should_work_when_rejected_by_current_owner() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(_transfer_default_space_ownership()); // Transfer SpaceId 1 owned by ACCOUNT1 to ACCOUNT2
        assert_ok!(_reject_default_pending_ownership_by_current_owner()); // Rejecting a transfer from ACCOUNT2

        // Check whether owner was not changed
        let space = Social::space_by_id(1).unwrap();
        assert_eq!(space.owner, ACCOUNT1);

        // Check whether storage state is correct
        assert!(Social::pending_space_owner(1).is_none());
    });
}

#[test]
fn reject_pending_ownership_should_fail_space_not_found() {
    ExtBuilder::build_with_space().execute_with(|| {
        // TODO: after adding new test externality
    });
}

#[test]
fn reject_pending_ownership_should_fail_no_pending_transfer() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_noop!(_reject_default_pending_ownership(), Error::<TestRuntime>::NoPendingTransferOnSpace); // Rejecting a transfer from ACCOUNT2
    });
}

#[test]
fn reject_pending_ownership_should_fail_not_allowed_to_reject() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(_transfer_default_space_ownership()); // Transfer SpaceId 1 owned by ACCOUNT1 to ACCOUNT2

        assert_noop!(_reject_pending_ownership(
      Some(Origin::signed(ACCOUNT3)),
      None
    ), Error::<TestRuntime>::NotAllowedToRejectOwnershipTransfer); // Rejecting a transfer from ACCOUNT2
    });
}