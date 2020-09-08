#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod tests {
    use frame_support::{
        assert_ok, assert_noop,
        impl_outer_origin, parameter_types,
        weights::Weight,
        dispatch::DispatchResult,
        storage::StorageMap,
    };
    use sp_core::H256;
    use sp_io::TestExternalities;
    use sp_std::iter::FromIterator;
    use sp_runtime::{
        traits::{BlakeTwo256, IdentityLookup},
        testing::Header,
        Perbill,
    };
    use frame_system::{self as system};

    use pallet_permissions::{
        SpacePermission,
        SpacePermission as SP,
        SpacePermissionSet,
        SpacePermissions,
    };
    use pallet_posts::{PostId, Post, PostUpdate, PostExtension, Comment, Error as PostsError};
    use pallet_profiles::{ProfileUpdate, Error as ProfilesError};
    use pallet_profile_follows::Error as ProfileFollowsError;
    use pallet_reactions::{ReactionId, ReactionKind, PostReactionScores, Error as ReactionsError};
    use pallet_scores::ScoringAction;
    use pallet_spaces::{SpaceById, SpaceUpdate, Error as SpacesError};
    use pallet_space_follows::Error as SpaceFollowsError;
    use pallet_space_ownership::Error as SpaceOwnershipError;
    use pallet_utils::{SpaceId, Error as UtilsError, User, Content};

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

    impl pallet_timestamp::Trait for TestRuntime {
        type Moment = u64;
        type OnTimestampSet = ();
        type MinimumPeriod = MinimumPeriod;
    }

    parameter_types! {
        pub const ExistentialDeposit: u64 = 1;
    }

    impl pallet_balances::Trait for TestRuntime {
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

    impl pallet_utils::Trait for TestRuntime {
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

            SP::UpdateSpaceSettings,

            SP::ManageBadges,
  			SP::ManageAwards,
            
        ].into_iter())),
      };
    }

    impl pallet_permissions::Trait for TestRuntime {
        type DefaultSpacePermissions = DefaultSpacePermissions;
    }

    parameter_types! {
        pub const MaxCommentDepth: u32 = 10;
    }

    impl pallet_posts::Trait for TestRuntime {
        type Event = ();
        type MaxCommentDepth = MaxCommentDepth;
        type PostScores = Scores;
        type AfterPostUpdated = PostHistory;
    }

    parameter_types! {}

    impl pallet_post_history::Trait for TestRuntime {}

    parameter_types! {}

    impl pallet_profile_follows::Trait for TestRuntime {
        type Event = ();
        type BeforeAccountFollowed = Scores;
        type BeforeAccountUnfollowed = Scores;
    }

    parameter_types! {}

    impl pallet_profiles::Trait for TestRuntime {
        type Event = ();
        type AfterProfileUpdated = ProfileHistory;
    }

    parameter_types! {}

    impl pallet_profile_history::Trait for TestRuntime {}

    parameter_types! {}

    impl pallet_reactions::Trait for TestRuntime {
        type Event = ();
        type PostReactionScores = Scores;
    }

    parameter_types! {
        pub const MaxUsersToProcessPerDeleteRole: u16 = 40;
    }

    impl pallet_roles::Trait for TestRuntime {
        type Event = ();
        type MaxUsersToProcessPerDeleteRole = MaxUsersToProcessPerDeleteRole;
        type Spaces = Spaces;
        type SpaceFollows = SpaceFollows;
        type IsAccountBlocked = Moderation;
        type IsContentBlocked = Moderation;
    }

    parameter_types! {
        pub const FollowSpaceActionWeight: i16 = 7;
        pub const FollowAccountActionWeight: i16 = 3;

        pub const SharePostActionWeight: i16 = 7;
        pub const UpvotePostActionWeight: i16 = 5;
        pub const DownvotePostActionWeight: i16 = -3;

        pub const CreateCommentActionWeight: i16 = 5;
        pub const ShareCommentActionWeight: i16 = 5;
        pub const UpvoteCommentActionWeight: i16 = 4;
        pub const DownvoteCommentActionWeight: i16 = -2;
    }

    impl pallet_scores::Trait for TestRuntime {
        type Event = ();

        type FollowSpaceActionWeight = FollowSpaceActionWeight;
        type FollowAccountActionWeight = FollowAccountActionWeight;

        type SharePostActionWeight = SharePostActionWeight;
        type UpvotePostActionWeight = UpvotePostActionWeight;
        type DownvotePostActionWeight = DownvotePostActionWeight;

        type CreateCommentActionWeight = CreateCommentActionWeight;
        type ShareCommentActionWeight = ShareCommentActionWeight;
        type UpvoteCommentActionWeight = UpvoteCommentActionWeight;
        type DownvoteCommentActionWeight = DownvoteCommentActionWeight;
    }

    parameter_types! {}

    impl pallet_space_follows::Trait for TestRuntime {
        type Event = ();
        type BeforeSpaceFollowed = Scores;
        type BeforeSpaceUnfollowed = Scores;
    }

    parameter_types! {}

    impl pallet_space_ownership::Trait for TestRuntime {
        type Event = ();
    }

    parameter_types! {}

    impl pallet_spaces::Trait for TestRuntime {
        type Event = ();
        type Roles = Roles;
        type SpaceFollows = SpaceFollows;
        type BeforeSpaceCreated = SpaceFollows;
        type AfterSpaceUpdated = SpaceHistory;
        type IsAccountBlocked = Moderation;
        type IsContentBlocked = Moderation;
        type SpaceCreationFee = ();
    }

    parameter_types! {}

    impl pallet_space_history::Trait for TestRuntime {}

    parameter_types! {
        pub const DefaultAutoblockThreshold: u16 = 20;
    }

    impl pallet_moderation::Trait for TestRuntime {
        type Event = ();
        type DefaultAutoblockThreshold = DefaultAutoblockThreshold;
    }

    type System = system::Module<TestRuntime>;
    type Balances = pallet_balances::Module<TestRuntime>;

    type Posts = pallet_posts::Module<TestRuntime>;
    type PostHistory = pallet_post_history::Module<TestRuntime>;
    type ProfileFollows = pallet_profile_follows::Module<TestRuntime>;
    type Profiles = pallet_profiles::Module<TestRuntime>;
    type ProfileHistory = pallet_profile_history::Module<TestRuntime>;
    type Reactions = pallet_reactions::Module<TestRuntime>;
    type Roles = pallet_roles::Module<TestRuntime>;
    type Scores = pallet_scores::Module<TestRuntime>;
    type SpaceFollows = pallet_space_follows::Module<TestRuntime>;
    type SpaceHistory = pallet_space_history::Module<TestRuntime>;
    type SpaceOwnership = pallet_space_ownership::Module<TestRuntime>;
    type Spaces = pallet_spaces::Module<TestRuntime>;
    type Moderation = pallet_moderation::Module<TestRuntime>;

    pub type AccountId = u64;
    type BlockNumber = u64;


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

        /// Custom ext configuration with pending ownership transfer without Space
        pub fn build_with_pending_ownership_transfer_no_space() -> TestExternalities {
            let storage = system::GenesisConfig::default()
                .build_storage::<TestRuntime>()
                .unwrap();

            let mut ext = TestExternalities::from(storage);
            ext.execute_with(|| {
                System::set_block_number(1);

                assert_ok!(_create_default_space());
                assert_ok!(_transfer_default_space_ownership());

                <SpaceById<TestRuntime>>::remove(SPACE1);
            });

            ext
        }

        /// Custom ext configuration with specified permissions granted (includes SpaceId 1)
        pub fn build_with_a_few_roles_granted_to_account2(perms: Vec<SP>) -> TestExternalities {
            let storage = system::GenesisConfig::default()
                .build_storage::<TestRuntime>()
                .unwrap();

            let mut ext = TestExternalities::from(storage);
            ext.execute_with(|| {
                System::set_block_number(1);
                let user = User::Account(ACCOUNT2);

                assert_ok!(_create_default_space());

                assert_ok!(_create_role(
                    None,
                    None,
                    None,
                    None,
                    Some(perms)
                ));
                // RoleId 1
                assert_ok!(_create_default_role()); // RoleId 2

                assert_ok!(_grant_role(None, Some(ROLE1), Some(vec![user.clone()])));
                assert_ok!(_grant_role(None, Some(ROLE2), Some(vec![user])));
            });

            ext
        }

        /// Custom ext configuration with space follow without Space
        pub fn build_with_space_follow_no_space() -> TestExternalities {
            let storage = system::GenesisConfig::default()
                .build_storage::<TestRuntime>()
                .unwrap();

            let mut ext = TestExternalities::from(storage);
            ext.execute_with(|| {
                System::set_block_number(1);

                assert_ok!(_create_default_space());
                assert_ok!(_default_follow_space());

                <SpaceById<TestRuntime>>::remove(SPACE1);
            });

            ext
        }
    }


    /* Integrated tests mocks */

    const ACCOUNT1: AccountId = 1;
    const ACCOUNT2: AccountId = 2;
    const ACCOUNT3: AccountId = 3;

    const SPACE1: SpaceId = 1001;
    const SPACE2: SpaceId = 1002;
    const _SPACE3: SpaceId = 1003;

    const POST1: PostId = 1;
    const POST2: PostId = 2;
    const POST3: PostId = 3;

    const REACTION1: ReactionId = 1;
    const REACTION2: ReactionId = 2;
    const _REACTION3: ReactionId = 3;

    fn space_handle() -> Vec<u8> {
        b"space_handle".to_vec()
    }

    fn space_content_ipfs() -> Content {
        Content::IPFS(b"QmRAQB6YaCyidP37UdDnjFY5vQuiBrcqdyoW1CuDgwxkD4".to_vec())
    }

    fn space_update(
        parent_id: Option<Option<SpaceId>>,
        handle: Option<Option<Vec<u8>>>,
        content: Option<Content>,
        hidden: Option<bool>,
        permissions: Option<Option<SpacePermissions>>
    ) -> SpaceUpdate {
        SpaceUpdate {
            parent_id,
            handle,
            content,
            hidden,
            permissions
        }
    }

    fn post_content_ipfs() -> Content {
        Content::IPFS(b"QmRAQB6YaCyidP37UdDnjFY5vQuiBrcqdyoW2CuDgwxkD4".to_vec())
    }

    fn post_update(
        space_id: Option<SpaceId>,
        content: Option<Content>,
        hidden: Option<bool>
    ) -> PostUpdate {
        PostUpdate {
            space_id,
            content,
            hidden,
        }
    }

    fn comment_content_ipfs() -> Content {
        Content::IPFS(b"QmRAQB6YaCyidP37UdDnjFY5vQuiBrcqdyoW1CuDgwxkD4".to_vec())
    }

    fn reply_content_ipfs() -> Content {
        Content::IPFS(b"QmYA2fn8cMbVWo4v95RwcwJVyQsNtnEwHerfWR8UNtEwoE".to_vec())
    }

    fn profile_content_ipfs() -> Content {
        Content::IPFS(b"QmRAQB6YaCyidP37UdDnjFY5vQuiaRtqdyoW2CuDgwxkA5".to_vec())
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
        PostExtension::Comment(Comment { parent_id, root_post_id })
    }

    fn extension_shared_post(post_id: PostId) -> PostExtension {
        PostExtension::SharedPost(post_id)
    }

    fn _create_default_space() -> DispatchResult { _create_space(None, None, None, None) }

    fn _create_space(
        origin: Option<Origin>,
        parent_id_opt: Option<Option<SpaceId>>,
        handle: Option<Option<Vec<u8>>>,
        content: Option<Content>
    ) -> DispatchResult {
        Spaces::create_space(
            origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
            parent_id_opt.unwrap_or(None),
            handle.unwrap_or_else(|| Some(self::space_handle())),
            content.unwrap_or_else(self::space_content_ipfs),
        )
    }

    fn _update_space(
        origin: Option<Origin>,
        space_id: Option<SpaceId>,
        update: Option<SpaceUpdate>
    ) -> DispatchResult {
        Spaces::update_space(
            origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
            space_id.unwrap_or(SPACE1),
            update.unwrap_or_else(|| self::space_update(None, None, None, None, None)),
        )
    }

    fn _default_follow_space() -> DispatchResult {
        _follow_space(None, None)
    }

    fn _follow_space(origin: Option<Origin>, space_id: Option<SpaceId>) -> DispatchResult {
        SpaceFollows::follow_space(
            origin.unwrap_or_else(|| Origin::signed(ACCOUNT2)),
            space_id.unwrap_or(SPACE1),
        )
    }

    fn _default_unfollow_space() -> DispatchResult {
        _unfollow_space(None, None)
    }

    fn _unfollow_space(origin: Option<Origin>, space_id: Option<SpaceId>) -> DispatchResult {
        SpaceFollows::unfollow_space(
            origin.unwrap_or_else(|| Origin::signed(ACCOUNT2)),
            space_id.unwrap_or(SPACE1),
        )
    }

    fn _create_default_post() -> DispatchResult {
        _create_post(None, None, None, None)
    }

    fn _create_post(
        origin: Option<Origin>,
        space_id_opt: Option<Option<SpaceId>>,
        extension: Option<PostExtension>,
        content: Option<Content>
    ) -> DispatchResult {
        Posts::create_post(
            origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
            space_id_opt.unwrap_or(Some(SPACE1)),
            extension.unwrap_or_else(self::extension_regular_post),
            content.unwrap_or_else(self::post_content_ipfs),
        )
    }

    fn _update_post(
        origin: Option<Origin>,
        post_id: Option<PostId>,
        update: Option<PostUpdate>,
    ) -> DispatchResult {
        Posts::update_post(
            origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
            post_id.unwrap_or(POST1),
            update.unwrap_or_else(|| self::post_update(None, None, None)),
        )
    }

    fn _create_default_comment() -> DispatchResult {
        _create_comment(None, None, None, None)
    }

    fn _create_comment(
        origin: Option<Origin>,
        post_id: Option<PostId>,
        parent_id: Option<Option<PostId>>,
        content: Option<Content>,
    ) -> DispatchResult {
        _create_post(
            origin,
            Some(None),
            Some(self::extension_comment(
                parent_id.unwrap_or(None),
                post_id.unwrap_or(POST1)
            )),
            Some(content.unwrap_or_else(self::comment_content_ipfs)),
        )
    }

    fn _update_comment(
        origin: Option<Origin>,
        post_id: Option<PostId>,
        update: Option<PostUpdate>
    ) -> DispatchResult {
        _update_post(
            origin,
            Some(post_id.unwrap_or(POST2)),
            Some(update.unwrap_or_else(||
                self::post_update(None, Some(self::reply_content_ipfs()), None))
            ),
        )
    }

    fn _create_default_post_reaction() -> DispatchResult {
        _create_post_reaction(None, None, None)
    }

    fn _create_default_comment_reaction() -> DispatchResult {
        _create_comment_reaction(None, None, None)
    }

    fn _create_post_reaction(
        origin: Option<Origin>,
        post_id: Option<PostId>,
        kind: Option<ReactionKind>
    ) -> DispatchResult {
        Reactions::create_post_reaction(
            origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
            post_id.unwrap_or(POST1),
            kind.unwrap_or_else(self::reaction_upvote),
        )
    }

    fn _create_comment_reaction(
        origin: Option<Origin>,
        post_id: Option<PostId>,
        kind: Option<ReactionKind>
    ) -> DispatchResult {
        _create_post_reaction(origin, Some(post_id.unwrap_or(2)), kind)
    }

    fn _update_post_reaction(
        origin: Option<Origin>,
        post_id: Option<PostId>,
        reaction_id: ReactionId,
        kind: Option<ReactionKind>
    ) -> DispatchResult {
        Reactions::update_post_reaction(
            origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
            post_id.unwrap_or(POST1),
            reaction_id,
            kind.unwrap_or_else(self::reaction_upvote),
        )
    }

    fn _update_comment_reaction(
        origin: Option<Origin>,
        post_id: Option<PostId>,
        reaction_id: ReactionId,
        kind: Option<ReactionKind>
    ) -> DispatchResult {
        _update_post_reaction(origin, Some(post_id.unwrap_or(2)), reaction_id, kind)
    }

    fn _delete_post_reaction(
        origin: Option<Origin>,
        post_id: Option<PostId>,
        reaction_id: ReactionId
    ) -> DispatchResult {
        Reactions::delete_post_reaction(
            origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
            post_id.unwrap_or(POST1),
            reaction_id,
        )
    }

    fn _delete_comment_reaction(
        origin: Option<Origin>,
        post_id: Option<PostId>,
        reaction_id: ReactionId
    ) -> DispatchResult {
        _delete_post_reaction(origin, Some(post_id.unwrap_or(2)), reaction_id)
    }

    fn _create_default_profile() -> DispatchResult {
        _create_profile(None, None)
    }

    fn _create_profile(
        origin: Option<Origin>,
        content: Option<Content>
    ) -> DispatchResult {
        Profiles::create_profile(
            origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
            content.unwrap_or_else(self::profile_content_ipfs),
        )
    }

    fn _update_profile(
        origin: Option<Origin>,
        content: Option<Content>
    ) -> DispatchResult {
        Profiles::update_profile(
            origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
            ProfileUpdate {
                content,
            },
        )
    }

    fn _default_follow_account() -> DispatchResult {
        _follow_account(None, None)
    }

    fn _follow_account(origin: Option<Origin>, account: Option<AccountId>) -> DispatchResult {
        ProfileFollows::follow_account(
            origin.unwrap_or_else(|| Origin::signed(ACCOUNT2)),
            account.unwrap_or(ACCOUNT1),
        )
    }

    fn _default_unfollow_account() -> DispatchResult {
        _unfollow_account(None, None)
    }

    fn _unfollow_account(origin: Option<Origin>, account: Option<AccountId>) -> DispatchResult {
        ProfileFollows::unfollow_account(
            origin.unwrap_or_else(|| Origin::signed(ACCOUNT2)),
            account.unwrap_or(ACCOUNT1),
        )
    }

    fn _score_post_on_reaction_with_id(
        account: AccountId,
        post_id: PostId,
        kind: ReactionKind
    ) -> DispatchResult {
        if let Some(ref mut post) = Posts::post_by_id(post_id) {
            Scores::score_post_on_reaction(account, post, kind)
        } else {
            panic!("Test error. Post\\Comment with specified ID not found.");
        }
    }

    fn _score_post_on_reaction(
        account: AccountId,
        post: &mut Post<TestRuntime>,
        kind: ReactionKind
    ) -> DispatchResult {
        Scores::score_post_on_reaction(account, post, kind)
    }

    fn _transfer_default_space_ownership() -> DispatchResult {
        _transfer_space_ownership(None, None, None)
    }

    fn _transfer_space_ownership(
        origin: Option<Origin>,
        space_id: Option<SpaceId>,
        transfer_to: Option<AccountId>
    ) -> DispatchResult {
        SpaceOwnership::transfer_space_ownership(
            origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
            space_id.unwrap_or(SPACE1),
            transfer_to.unwrap_or(ACCOUNT2),
        )
    }

    fn _accept_default_pending_ownership() -> DispatchResult {
        _accept_pending_ownership(None, None)
    }

    fn _accept_pending_ownership(origin: Option<Origin>, space_id: Option<SpaceId>) -> DispatchResult {
        SpaceOwnership::accept_pending_ownership(
            origin.unwrap_or_else(|| Origin::signed(ACCOUNT2)),
            space_id.unwrap_or(SPACE1),
        )
    }

    fn _reject_default_pending_ownership() -> DispatchResult {
        _reject_pending_ownership(None, None)
    }

    fn _reject_default_pending_ownership_by_current_owner() -> DispatchResult {
        _reject_pending_ownership(Some(Origin::signed(ACCOUNT1)), None)
    }

    fn _reject_pending_ownership(origin: Option<Origin>, space_id: Option<SpaceId>) -> DispatchResult {
        SpaceOwnership::reject_pending_ownership(
            origin.unwrap_or_else(|| Origin::signed(ACCOUNT2)),
            space_id.unwrap_or(SPACE1),
        )
    }
    /* ---------------------------------------------------------------------------------------------- */

    // TODO: fix copy-paste from pallet_roles
    /* Roles pallet mocks */

    type RoleId = u64;

    const ROLE1: RoleId = 1;
    const ROLE2: RoleId = 2;

    fn default_role_content_ipfs() -> Content {
        Content::IPFS(b"QmRAQB6YaCyidP37UdDnjFY5vQuiBrcqdyoW1CuDgwxkD4".to_vec())
    }

    /// Permissions Set that includes next permission: ManageRoles
    fn permission_set_default() -> Vec<SpacePermission> {
        vec![SP::ManageRoles]
    }


    pub fn _create_default_role() -> DispatchResult {
        _create_role(None, None, None, None, None)
    }

    pub fn _create_role(
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

    pub fn _grant_default_role() -> DispatchResult {
        _grant_role(None, None, None)
    }

    pub fn _grant_role(
        origin: Option<Origin>,
        role_id: Option<RoleId>,
        users: Option<Vec<User<AccountId>>>,
    ) -> DispatchResult {
        Roles::grant_role(
            origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
            role_id.unwrap_or(ROLE1),
            users.unwrap_or_else(|| vec![User::Account(ACCOUNT2)]),
        )
    }

    pub fn _delete_default_role() -> DispatchResult {
        _delete_role(None, None)
    }

    pub fn _delete_role(
        origin: Option<Origin>,
        role_id: Option<RoleId>,
    ) -> DispatchResult {
        Roles::delete_role(
            origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
            role_id.unwrap_or(ROLE1),
        )
    }
    /* ---------------------------------------------------------------------------------------------- */


    // Space tests
    #[test]
    fn create_space_should_work() {
        ExtBuilder::build().execute_with(|| {
            assert_ok!(_create_default_space()); // SpaceId 1

            // Check storages
            assert_eq!(Spaces::space_ids_by_owner(ACCOUNT1), vec![SPACE1]);
            assert_eq!(Spaces::space_id_by_handle(self::space_handle()), Some(SPACE1));
            assert_eq!(Spaces::next_space_id(), SPACE2);

            // Check whether data stored correctly
            let space = Spaces::space_by_id(SPACE1).unwrap();

            assert_eq!(space.created.account, ACCOUNT1);
            assert!(space.updated.is_none());
            assert_eq!(space.hidden, false);

            assert_eq!(space.owner, ACCOUNT1);
            assert_eq!(space.handle, Some(self::space_handle()));
            assert_eq!(space.content, self::space_content_ipfs());

            assert_eq!(space.posts_count, 0);
            assert_eq!(space.followers_count, 1);
            assert!(SpaceHistory::edit_history(space.id).is_empty());
            assert_eq!(space.score, 0);
        });
    }

    #[test]
    fn create_space_should_store_handle_lowercase() {
        ExtBuilder::build().execute_with(|| {
            let handle: Vec<u8> = b"sPaCe_hAnDlE".to_vec();

            assert_ok!(_create_space(None, None, Some(Some(handle.clone())), None)); // SpaceId 1

            // Handle should be lowercase in storage and original in struct
            let space = Spaces::space_by_id(SPACE1).unwrap();
            assert_eq!(space.handle, Some(handle.clone()));
            assert_eq!(Spaces::space_id_by_handle(handle.to_ascii_lowercase()), Some(SPACE1));
        });
    }

    #[test]
    fn create_space_should_fail_with_handle_too_short() {
        ExtBuilder::build().execute_with(|| {
            let handle: Vec<u8> = vec![65; (MinHandleLen::get() - 1) as usize];

            // Try to catch an error creating a space with too short handle
            assert_noop!(_create_space(
                None,
                None,
                Some(Some(handle)),
                None
            ), UtilsError::<TestRuntime>::HandleIsTooShort);
        });
    }

    #[test]
    fn create_space_should_fail_with_handle_too_long() {
        ExtBuilder::build().execute_with(|| {
            let handle: Vec<u8> = vec![65; (MaxHandleLen::get() + 1) as usize];

            // Try to catch an error creating a space with too long handle
            assert_noop!(_create_space(
                None,
                None,
                Some(Some(handle)),
                None
            ), UtilsError::<TestRuntime>::HandleIsTooLong);
        });
    }

    #[test]
    fn create_space_should_fail_with_handle_not_unique() {
        ExtBuilder::build().execute_with(|| {
            assert_ok!(_create_default_space());
            // SpaceId 1
            // Try to catch an error creating a space with not unique handle
            assert_noop!(_create_default_space(), SpacesError::<TestRuntime>::SpaceHandleIsNotUnique);
        });
    }

    #[test]
    fn create_space_should_fail_with_handle_contains_invalid_char_at() {
        ExtBuilder::build().execute_with(|| {
            let handle: Vec<u8> = b"@space_handle".to_vec();

            assert_noop!(_create_space(
                None,
                None,
                Some(Some(handle)),
                None
            ), UtilsError::<TestRuntime>::HandleContainsInvalidChars);
        });
    }

    #[test]
    fn create_space_should_fail_with_handle_contains_invalid_char_minus() {
        ExtBuilder::build().execute_with(|| {
            let handle: Vec<u8> = b"space-handle".to_vec();

            assert_noop!(_create_space(
                None,
                None,
                Some(Some(handle)),
                None
            ), UtilsError::<TestRuntime>::HandleContainsInvalidChars);
        });
    }

    #[test]
    fn create_space_should_fail_with_handle_contains_invalid_char_space() {
        ExtBuilder::build().execute_with(|| {
            let handle: Vec<u8> = b"space handle".to_vec();

            assert_noop!(_create_space(
                None,
                None,
                Some(Some(handle)),
                None
            ), UtilsError::<TestRuntime>::HandleContainsInvalidChars);
        });
    }

    #[test]
    fn create_space_should_fail_with_handle_contains_invalid_chars_unicode() {
        ExtBuilder::build().execute_with(|| {
            let handle: Vec<u8> = String::from("блог_хендл").into_bytes().to_vec();

            assert_noop!(_create_space(
                None,
                None,
                Some(Some(handle)),
                None
            ), UtilsError::<TestRuntime>::HandleContainsInvalidChars);
        });
    }

    #[test]
    fn create_space_should_fail_with_invalid_ipfs_cid() {
        ExtBuilder::build().execute_with(|| {
            let content_ipfs = Content::IPFS(b"QmV9tSDx9UiPeWExXEeH6aoDvmihvx6j".to_vec());

            // Try to catch an error creating a space with invalid content
            assert_noop!(_create_space(
                None,
                None,
                None,
                Some(content_ipfs)
            ), UtilsError::<TestRuntime>::InvalidIpfsCid);
        });
    }

    #[test]
    fn update_space_should_work() {
        ExtBuilder::build_with_space().execute_with(|| {
            let handle: Vec<u8> = b"new_handle".to_vec();
            let content_ipfs = Content::IPFS(b"QmRAQB6YaCyidP37UdDnjFY5vQuiBrcqdyoW2CuDgwxkD4".to_vec());
            // Space update with ID 1 should be fine

            assert_ok!(_update_space(
                None, // From ACCOUNT1 (has permission as he's an owner)
                None,
                Some(
                    self::space_update(
                        None,
                        Some(Some(handle.clone())),
                        Some(content_ipfs.clone()),
                        Some(true),
                        Some(Some(SpacePermissions {
                            none: None,
                            everyone: None,
                            follower: None,
                            space_owner: None
                        })),
                    )
                )
            ));

            // Check whether space updates correctly
            let space = Spaces::space_by_id(SPACE1).unwrap();
            assert_eq!(space.handle, Some(handle));
            assert_eq!(space.content, content_ipfs);
            assert_eq!(space.hidden, true);

            // Check whether history recorded correctly
            let edit_history = &SpaceHistory::edit_history(space.id)[0];
            assert_eq!(edit_history.old_data.handle, Some(Some(self::space_handle())));
            assert_eq!(edit_history.old_data.content, Some(self::space_content_ipfs()));
            assert_eq!(edit_history.old_data.hidden, Some(false));
        });
    }

    #[test]
    fn update_space_should_work_with_a_few_roles() {
        ExtBuilder::build_with_a_few_roles_granted_to_account2(vec![SP::UpdateSpace]).execute_with(|| {
            let space_update = self::space_update(
                None,
                Some(Some(b"new_handle".to_vec())),
                Some(Content::IPFS(
                    b"QmRAQB6YaCyidP37UdDnjFY5vQuiBrcqdyoW2CuDgwxkD4".to_vec()
                )),
                Some(true),
                None,
            );

            assert_ok!(_update_space(
                Some(Origin::signed(ACCOUNT2)),
                Some(SPACE1),
                Some(space_update)
            ));
        });
    }

    #[test]
    fn update_space_should_fail_with_no_updates_for_space() {
        ExtBuilder::build_with_space().execute_with(|| {
            // Try to catch an error updating a space with no changes
            assert_noop!(_update_space(None, None, None), SpacesError::<TestRuntime>::NoUpdatesForSpace);
        });
    }

    #[test]
    fn update_space_should_fail_with_space_not_found() {
        ExtBuilder::build_with_space().execute_with(|| {
            let handle: Vec<u8> = b"new_handle".to_vec();

            // Try to catch an error updating a space with wrong space ID
            assert_noop!(_update_space(
                None,
                Some(SPACE2),
                Some(
                    self::space_update(
                        None,
                        Some(Some(handle)),
                        None,
                        None,
                        None,
                    )
                )
            ), SpacesError::<TestRuntime>::SpaceNotFound);
        });
    }

    #[test]
    fn update_space_should_fail_with_no_permission() {
        ExtBuilder::build_with_space().execute_with(|| {
            let handle: Vec<u8> = b"new_handle".to_vec();

            // Try to catch an error updating a space with an account that it not permitted
            assert_noop!(_update_space(
                Some(Origin::signed(ACCOUNT2)),
                None,
                Some(
                    self::space_update(
                        None,
                        Some(Some(handle)),
                        None,
                        None,
                        None,
                    )
                )
            ), SpacesError::<TestRuntime>::NoPermissionToUpdateSpace);
        });
    }

    #[test]
    fn update_space_should_fail_with_handle_too_short() {
        ExtBuilder::build_with_space().execute_with(|| {
            let handle: Vec<u8> = vec![65; (MinHandleLen::get() - 1) as usize];

            // Try to catch an error updating a space with too short handle
            assert_noop!(_update_space(
                None,
                None,
                Some(
                    self::space_update(
                        None,
                        Some(Some(handle)),
                        None,
                        None,
                        None,
                    )
                )
            ), UtilsError::<TestRuntime>::HandleIsTooShort);
        });
    }

    #[test]
    fn update_space_should_fail_with_handle_too_long() {
        ExtBuilder::build_with_space().execute_with(|| {
            let handle: Vec<u8> = vec![65; (MaxHandleLen::get() + 1) as usize];

            // Try to catch an error updating a space with too long handle
            assert_noop!(_update_space(
                None,
                None,
                Some(
                    self::space_update(
                        None,
                        Some(Some(handle)),
                        None,
                        None,
                        None,
                    )
                )
            ), UtilsError::<TestRuntime>::HandleIsTooLong);
        });
    }

    #[test]
    fn update_space_should_fail_with_handle_is_not_unique() {
        ExtBuilder::build_with_space().execute_with(|| {
            let handle: Vec<u8> = b"unique_handle".to_vec();

            assert_ok!(_create_space(
                None,
                None,
                Some(Some(handle.clone())),
                None
            )); // SpaceId 2 with a custom handle

                // Try to catch an error updating a space on ID 1 with a handle of space on ID 2
                assert_noop!(_update_space(
                None,
                Some(SPACE1),
                Some(
                    self::space_update(
                        None,
                        Some(Some(handle)),
                        None,
                        None,
                        None,
                    )
                )
            ), SpacesError::<TestRuntime>::SpaceHandleIsNotUnique);
        });
    }

    #[test]
    fn update_space_should_fail_with_handle_contains_invalid_char_at() {
        ExtBuilder::build_with_space().execute_with(|| {
            let handle: Vec<u8> = b"@space_handle".to_vec();

            assert_noop!(_update_space(
                None,
                None,
                Some(
                    self::space_update(
                        None,
                        Some(Some(handle)),
                        None,
                        None,
                        None,
                    )
                )
            ), UtilsError::<TestRuntime>::HandleContainsInvalidChars);
        });
    }

    #[test]
    fn update_space_should_fail_with_handle_contains_invalid_char_minus() {
        ExtBuilder::build_with_space().execute_with(|| {
            let handle: Vec<u8> = b"space-handle".to_vec();

            assert_noop!(_update_space(
                None,
                None,
                Some(
                    self::space_update(
                        None,
                        Some(Some(handle)),
                        None,
                        None,
                        None,
                    )
                )
            ), UtilsError::<TestRuntime>::HandleContainsInvalidChars);
        });
    }

    #[test]
    fn update_space_should_fail_with_handle_contains_invalid_space() {
        ExtBuilder::build_with_space().execute_with(|| {
            let handle: Vec<u8> = b"space handle".to_vec();

            assert_noop!(_update_space(
                None,
                None,
                Some(
                    self::space_update(
                        None,
                        Some(Some(handle)),
                        None,
                        None,
                        None,
                    )
                )
            ), UtilsError::<TestRuntime>::HandleContainsInvalidChars);
        });
    }

    #[test]
    fn update_space_should_fail_with_handle_contains_invalid_chars_unicode() {
        ExtBuilder::build_with_space().execute_with(|| {
            let handle: Vec<u8> = String::from("блог_хендл").into_bytes().to_vec();

            assert_noop!(_update_space(
                None,
                None,
                Some(
                    self::space_update(
                        None,
                        Some(Some(handle)),
                        None,
                        None,
                        None,
                    )
                )
            ), UtilsError::<TestRuntime>::HandleContainsInvalidChars);
        });
    }

    #[test]
    fn update_space_should_fail_with_invalid_ipfs_cid() {
        ExtBuilder::build_with_space().execute_with(|| {
            let content_ipfs = Content::IPFS(b"QmV9tSDx9UiPeWExXEeH6aoDvmihvx6j".to_vec());

            // Try to catch an error updating a space with invalid content
            assert_noop!(_update_space(
                None,
                None,
                Some(
                    self::space_update(
                        None,
                        None,
                        Some(content_ipfs),
                        None,
                        None,
                    )
                )
            ), UtilsError::<TestRuntime>::InvalidIpfsCid);
        });
    }

    #[test]
    fn update_space_should_fail_with_a_few_roles_no_permission() {
        ExtBuilder::build_with_a_few_roles_granted_to_account2(vec![SP::UpdateSpace]).execute_with(|| {
            let space_update = self::space_update(
                None,
                Some(Some(b"new_handle".to_vec())),
                Some(Content::IPFS(
                    b"QmRAQB6YaCyidP37UdDnjFY5vQuiBrcqdyoW2CuDgwxkD4".to_vec()
                )),
                Some(true),
                None,
            );

            assert_ok!(_delete_default_role());

            assert_noop!(_update_space(
                Some(Origin::signed(ACCOUNT2)),
                Some(SPACE1),
                Some(space_update)
            ), SpacesError::<TestRuntime>::NoPermissionToUpdateSpace);
        });
    }

    // Post tests
    #[test]
    fn create_post_should_work() {
        ExtBuilder::build_with_space().execute_with(|| {
            assert_ok!(_create_default_post()); // PostId 1 by ACCOUNT1 which is permitted by default

            // Check storages
            assert_eq!(Posts::post_ids_by_space_id(SPACE1), vec![POST1]);
            assert_eq!(Posts::next_post_id(), POST2);

            // Check whether data stored correctly
            let post = Posts::post_by_id(POST1).unwrap();

            assert_eq!(post.created.account, ACCOUNT1);
            assert!(post.updated.is_none());
            assert_eq!(post.hidden, false);

            assert_eq!(post.space_id, Some(SPACE1));
            assert_eq!(post.extension, self::extension_regular_post());

            assert_eq!(post.content, self::post_content_ipfs());

            assert_eq!(post.replies_count, 0);
            assert_eq!(post.hidden_replies_count, 0);
            assert_eq!(post.shares_count, 0);
            assert_eq!(post.upvotes_count, 0);
            assert_eq!(post.downvotes_count, 0);

            assert_eq!(post.score, 0);

            assert!(PostHistory::edit_history(POST1).is_empty());
        });
    }

    #[test]
    fn create_post_should_work_with_a_few_roles() {
        ExtBuilder::build_with_a_few_roles_granted_to_account2(vec![SP::CreatePosts]).execute_with(|| {
            assert_ok!(_create_post(
                Some(Origin::signed(ACCOUNT2)),
                None, // SpaceId 1,
                None, // RegularPost extension
                None, // Default post content
            ));
        });
    }

    #[test]
    fn create_post_should_fail_with_post_has_no_spaceid() {
        ExtBuilder::build_with_space().execute_with(|| {
            assert_noop!(_create_post(
                None,
                Some(None),
                None,
                None
            ), PostsError::<TestRuntime>::PostHasNoSpaceId);
        });
    }

    #[test]
    fn create_post_should_fail_with_space_not_found() {
        ExtBuilder::build().execute_with(|| {
            assert_noop!(_create_default_post(), SpacesError::<TestRuntime>::SpaceNotFound);
        });
    }

    #[test]
    fn create_post_should_fail_with_invalid_ipfs_cid() {
        ExtBuilder::build_with_space().execute_with(|| {
            let content_ipfs = Content::IPFS(b"QmV9tSDx9UiPeWExXEeH6aoDvmihvx6j".to_vec());

            // Try to catch an error creating a regular post with invalid content
            assert_noop!(_create_post(
                None,
                None,
                None,
                Some(content_ipfs)
            ), UtilsError::<TestRuntime>::InvalidIpfsCid);
        });
    }

    #[test]
    fn create_post_should_fail_with_no_permission() {
        ExtBuilder::build_with_space().execute_with(|| {
            assert_noop!(_create_post(
                Some(Origin::signed(ACCOUNT2)),
                None,
                None,
                None
            ), PostsError::<TestRuntime>::NoPermissionToCreatePosts);
        });
    }

    #[test]
    fn create_post_should_fail_with_a_few_roles_no_permission() {
        ExtBuilder::build_with_a_few_roles_granted_to_account2(vec![SP::CreatePosts]).execute_with(|| {
            assert_ok!(_delete_default_role());

            assert_noop!(_create_post(
                Some(Origin::signed(ACCOUNT2)),
                None, // SpaceId 1,
                None, // RegularPost extension
                None, // Default post content
            ), PostsError::<TestRuntime>::NoPermissionToCreatePosts);
        });
    }

    #[test]
    fn update_post_should_work() {
        ExtBuilder::build_with_post().execute_with(|| {
            let content_ipfs = Content::IPFS(b"QmRAQB6YaCyidP37UdDnjFY5vQuiBrcqdyoW1CuDgwxkD4".to_vec());

            // Post update with ID 1 should be fine
            assert_ok!(_update_post(
                None, // From ACCOUNT1 (has default permission to UpdateOwnPosts)
                None,
                Some(
                    self::post_update(
                        None,
                        Some(content_ipfs.clone()),
                        Some(true)
                    )
                )
            ));

            // Check whether post updates correctly
            let post = Posts::post_by_id(POST1).unwrap();
            assert_eq!(post.space_id, Some(SPACE1));
            assert_eq!(post.content, content_ipfs);
            assert_eq!(post.hidden, true);

            // Check whether history recorded correctly
            let post_history = PostHistory::edit_history(POST1)[0].clone();
            assert!(post_history.old_data.space_id.is_none());
            assert_eq!(post_history.old_data.content, Some(self::post_content_ipfs()));
            assert_eq!(post_history.old_data.hidden, Some(false));
        });
    }

    #[test]
    fn update_post_should_work_after_transfer_space_ownership() {
        ExtBuilder::build_with_post().execute_with(|| {
            let post_update = self::post_update(
                None,
                Some(Content::IPFS(
                    b"QmRAQB6YaCyidP37UdDnjFY5vQuiBrcqdyoW1CuDgwxkD4".to_vec()
                )),
                Some(true),
            );

            assert_ok!(_transfer_default_space_ownership());

            // Post update with ID 1 should be fine
            assert_ok!(_update_post(None, None, Some(post_update)));
        });
    }

    #[test]
    fn update_any_post_should_work_with_default_permission() {
        ExtBuilder::build_with_a_few_roles_granted_to_account2(vec![SP::CreatePosts]).execute_with(|| {
            let post_update = self::post_update(
                None,
                Some(Content::IPFS(
                    b"QmRAQB6YaCyidP37UdDnjFY5vQuiBrcqdyoW1CuDgwxkD4".to_vec()
                )),
                Some(true),
            );
            assert_ok!(_create_post(
                Some(Origin::signed(ACCOUNT2)),
                None, // SpaceId 1
                None, // RegularPost extension
                None // Default post content
            )); // PostId 1

            // Post update with ID 1 should be fine
            assert_ok!(_update_post(
                None, // From ACCOUNT1 (has default permission to UpdateAnyPosts as SpaceOwner)
                Some(POST1),
                Some(post_update)
            ));
        });
    }

    #[test]
    fn update_any_post_should_work_with_a_few_roles() {
        ExtBuilder::build_with_a_few_roles_granted_to_account2(vec![SP::UpdateAnyPost]).execute_with(|| {
            let post_update = self::post_update(
                None,
                Some(Content::IPFS(
                    b"QmRAQB6YaCyidP37UdDnjFY5vQuiBrcqdyoW1CuDgwxkD4".to_vec()
                )),
                Some(true),
            );
            assert_ok!(_create_default_post()); // PostId 1

            // Post update with ID 1 should be fine
            assert_ok!(_update_post(
                Some(Origin::signed(ACCOUNT2)),
                Some(POST1),
                Some(post_update)
            ));
        });
    }

    #[test]
    fn update_post_should_fail_with_no_updates_for_post() {
        ExtBuilder::build_with_post().execute_with(|| {
            // Try to catch an error updating a post with no changes
            assert_noop!(_update_post(None, None, None), PostsError::<TestRuntime>::NoUpdatesForPost);
        });
    }

    #[test]
    fn update_post_should_fail_with_post_not_found() {
        ExtBuilder::build_with_post().execute_with(|| {
            assert_ok!(_create_space(None, None, Some(Some(b"space2_handle".to_vec())), None)); // SpaceId 2

            // Try to catch an error updating a post with wrong post ID
            assert_noop!(_update_post(
                None,
                Some(POST2),
                Some(
                    self::post_update(
                        // FIXME: when Post's `space_id` update is fully implemented
                        None/*Some(SPACE2)*/,
                        None,
                        Some(true)/*None*/
                    )
                )
            ), PostsError::<TestRuntime>::PostNotFound);
        });
    }

    #[test]
    fn update_post_should_fail_with_no_permission_to_update_any_post() {
        ExtBuilder::build_with_post().execute_with(|| {
            assert_ok!(_create_space(None, None, Some(Some(b"space2_handle".to_vec())), None)); // SpaceId 2

            // Try to catch an error updating a post with different account
            assert_noop!(_update_post(
                Some(Origin::signed(ACCOUNT2)),
                None,
                Some(
                    self::post_update(
                        // FIXME: when Post's `space_id` update is fully implemented
                        None/*Some(SPACE2)*/,
                        None,
                        Some(true)/*None*/
                    )
                )
            ), PostsError::<TestRuntime>::NoPermissionToUpdateAnyPost);
        });
    }

    #[test]
    fn update_post_should_fail_with_invalid_ipfs_cid() {
        ExtBuilder::build_with_post().execute_with(|| {
            let content_ipfs = Content::IPFS(b"QmV9tSDx9UiPeWExXEeH6aoDvmihvx6j".to_vec());

            // Try to catch an error updating a post with invalid content
            assert_noop!(_update_post(
                None,
                None,
                Some(
                    self::post_update(
                        None,
                        Some(content_ipfs),
                        None
                    )
                )
            ), UtilsError::<TestRuntime>::InvalidIpfsCid);
        });
    }

    #[test]
    fn update_any_post_should_fail_with_a_few_roles_no_permission() {
        ExtBuilder::build_with_a_few_roles_granted_to_account2(vec![SP::UpdateAnyPost]).execute_with(|| {
            let post_update = self::post_update(
                None,
                Some(Content::IPFS(
                    b"QmRAQB6YaCyidP37UdDnjFY5vQuiBrcqdyoW1CuDgwxkD4".to_vec()
                )),
                Some(true),
            );
            assert_ok!(_create_default_post());
            // PostId 1
            assert_ok!(_delete_default_role());

            // Post update with ID 1 should be fine
            assert_noop!(_update_post(
                Some(Origin::signed(ACCOUNT2)),
                Some(POST1),
                Some(post_update)
            ), PostsError::<TestRuntime>::NoPermissionToUpdateAnyPost);
        });
    }

    // Comment tests
    #[test]
    fn create_comment_should_work() {
        ExtBuilder::build_with_post().execute_with(|| {
            assert_ok!(_create_default_comment()); // PostId 2 by ACCOUNT1 which is permitted by default

            // Check storages
            let root_post = Posts::post_by_id(POST1).unwrap();
            assert_eq!(Posts::reply_ids_by_post_id(POST1), vec![POST2]);
            assert_eq!(root_post.replies_count, 1);
            assert_eq!(root_post.hidden_replies_count, 0);

            // Check whether data stored correctly
            let comment = Posts::post_by_id(POST2).unwrap();
            let comment_ext = comment.get_comment_ext().unwrap();

            assert!(comment_ext.parent_id.is_none());
            assert_eq!(comment_ext.root_post_id, POST1);
            assert_eq!(comment.created.account, ACCOUNT1);
            assert!(comment.updated.is_none());
            assert_eq!(comment.content, self::comment_content_ipfs());
            assert_eq!(comment.replies_count, 0);
            assert_eq!(comment.hidden_replies_count, 0);
            assert_eq!(comment.shares_count, 0);
            assert_eq!(comment.upvotes_count, 0);
            assert_eq!(comment.downvotes_count, 0);
            assert_eq!(comment.score, 0);

            assert!(PostHistory::edit_history(POST2).is_empty());
        });
    }

    #[test]
    fn create_comment_should_work_with_parents() {
        ExtBuilder::build_with_comment().execute_with(|| {
            let first_comment_id: PostId = 2;
            let penultimate_comment_id: PostId = 8;
            let last_comment_id: PostId = 9;

            for parent_id in first_comment_id..last_comment_id as PostId {
                // last created = `last_comment_id`; last parent = `penultimate_comment_id`
                assert_ok!(_create_comment(None, None, Some(Some(parent_id)), None));
            }

            for comment_id in first_comment_id..penultimate_comment_id as PostId {
                let comment = Posts::post_by_id(comment_id).unwrap();
                let replies_should_be = last_comment_id-comment_id;
                assert_eq!(comment.replies_count, replies_should_be as u16);
                assert_eq!(Posts::reply_ids_by_post_id(comment_id), vec![comment_id + 1]);

                assert_eq!(comment.hidden_replies_count, 0);
            }

            let last_comment = Posts::post_by_id(last_comment_id).unwrap();
            assert_eq!(last_comment.replies_count, 0);
            assert!(Posts::reply_ids_by_post_id(last_comment_id).is_empty());

            assert_eq!(last_comment.hidden_replies_count, 0);
        });
    }

    #[test]
    fn create_comment_should_fail_with_post_not_found() {
        ExtBuilder::build().execute_with(|| {
            // Try to catch an error creating a comment with wrong post
            assert_noop!(_create_default_comment(), PostsError::<TestRuntime>::PostNotFound);
        });
    }

    #[test]
    fn create_comment_should_fail_with_unknown_parent_comment() {
        ExtBuilder::build_with_post().execute_with(|| {
            // Try to catch an error creating a comment with wrong parent
            assert_noop!(_create_comment(
                None,
                None,
                Some(Some(POST2)),
                None
            ), PostsError::<TestRuntime>::UnknownParentComment);
        });
    }

    #[test]
    fn create_comment_should_fail_with_invalid_ipfs_cid() {
        ExtBuilder::build_with_post().execute_with(|| {
            let content_ipfs = Content::IPFS(b"QmV9tSDx9UiPeWExXEeH6aoDvmihvx6j".to_vec());

            // Try to catch an error creating a comment with wrong parent
            assert_noop!(_create_comment(
                None,
                None,
                None,
                Some(content_ipfs)
            ), UtilsError::<TestRuntime>::InvalidIpfsCid);
        });
    }

    #[test]
    fn create_comment_should_fail_with_cannot_create_in_hidden_space_scope() {
        ExtBuilder::build_with_post().execute_with(|| {
            assert_ok!(_update_space(
                None,
                None,
                Some(self::space_update(None, None, None, Some(true), None))
            ));

            assert_noop!(_create_default_comment(), PostsError::<TestRuntime>::CannotCreateInHiddenScope);
        });
    }

    #[test]
    fn create_comment_should_fail_with_cannot_create_in_hidden_post_scope() {
        ExtBuilder::build_with_post().execute_with(|| {
            assert_ok!(_update_post(
                None,
                None,
                Some(self::post_update(None, None, Some(true)))
            ));

            assert_noop!(_create_default_comment(), PostsError::<TestRuntime>::CannotCreateInHiddenScope);
        });
    }

    #[test]
    fn create_comment_should_fail_with_max_comment_depth_reached() {
        ExtBuilder::build_with_post().execute_with(|| {
            assert_ok!(_create_comment(None, None, Some(None), None)); // PostId 2

            for parent_id in 2..11 as PostId {
                assert_ok!(_create_comment(None, None, Some(Some(parent_id)), None)); // PostId N (last = 10)
            }

            // Some(Some(11)) - here is parent_id 11 of type PostId
            assert_noop!(_create_comment(
                None,
                None,
                Some(Some(11)),
                None
            ), PostsError::<TestRuntime>::MaxCommentDepthReached);
        });
    }

    #[test]
    fn update_comment_should_work() {
        ExtBuilder::build_with_comment().execute_with(|| {
            // Post update with ID 1 should be fine
            assert_ok!(_update_comment(None, None, None));

            // Check whether post updates correctly
            let comment = Posts::post_by_id(POST2).unwrap();
            assert_eq!(comment.content, self::reply_content_ipfs());

            // Check whether history recorded correctly
            assert_eq!(PostHistory::edit_history(POST2)[0].old_data.content, Some(self::comment_content_ipfs()));
        });
    }

    #[test]
    fn update_comment_hidden_should_work_with_parents() {
        ExtBuilder::build_with_comment().execute_with(|| {
            let first_comment_id: PostId = 2;
            let penultimate_comment_id: PostId = 8;
            let last_comment_id: PostId = 9;

            for parent_id in first_comment_id..last_comment_id as PostId {
                // last created = `last_comment_id`; last parent = `penultimate_comment_id`
                assert_ok!(_create_comment(None, None, Some(Some(parent_id)), None));
            }

            assert_ok!(_update_comment(
                None,
                Some(last_comment_id),
                Some(self::post_update(
                    None,
                    None,
                    Some(true) // make comment hidden
                ))
            ));

            for comment_id in first_comment_id..penultimate_comment_id as PostId {
                let comment = Posts::post_by_id(comment_id).unwrap();
                assert_eq!(comment.hidden_replies_count, 1);
            }
            let last_comment = Posts::post_by_id(last_comment_id).unwrap();
            assert_eq!(last_comment.hidden_replies_count, 0);
        });
    }

    #[test]
    // `PostNotFound` here: Post with Comment extension. Means that comment wasn't found.
    fn update_comment_should_fail_with_post_not_found() {
        ExtBuilder::build().execute_with(|| {
            // Try to catch an error updating a comment with wrong PostId
            assert_noop!(_update_comment(None, None, None), PostsError::<TestRuntime>::PostNotFound);
        });
    }

    #[test]
    fn update_comment_should_fail_with_not_a_comment_author() {
        ExtBuilder::build_with_comment().execute_with(|| {
            // Try to catch an error updating a comment with wrong Account
            assert_noop!(_update_comment(
                Some(Origin::signed(ACCOUNT2)),
                None,
                None
            ), PostsError::<TestRuntime>::NotACommentAuthor);
        });
    }

    #[test]
    fn update_comment_should_fail_with_invalid_ipfs_cid() {
        ExtBuilder::build_with_comment().execute_with(|| {
            let content_ipfs = Content::IPFS(b"QmV9tSDx9UiPeWExXEeH6aoDvmihvx6j".to_vec());

            // Try to catch an error updating a comment with invalid content
            assert_noop!(_update_comment(
                None,
                None,
                Some(
                    self::post_update(
                        None,
                        Some(content_ipfs),
                        None
                    )
                )
            ), UtilsError::<TestRuntime>::InvalidIpfsCid);
        });
    }

    // Reaction tests
    #[test]
    fn create_post_reaction_should_work_upvote() {
        ExtBuilder::build_with_post().execute_with(|| {
            assert_ok!(_create_post_reaction(
                Some(Origin::signed(ACCOUNT2)),
                None,
                None
            )); // ReactionId 1 by ACCOUNT2 which is permitted by default

            // Check storages
            assert_eq!(Reactions::reaction_ids_by_post_id(POST1), vec![REACTION1]);
            assert_eq!(Reactions::next_reaction_id(), REACTION2);

            // Check post reaction counters
            let post = Posts::post_by_id(POST1).unwrap();
            assert_eq!(post.upvotes_count, 1);
            assert_eq!(post.downvotes_count, 0);

            // Check whether data stored correctly
            let reaction = Reactions::reaction_by_id(REACTION1).unwrap();
            assert_eq!(reaction.created.account, ACCOUNT2);
            assert_eq!(reaction.kind, self::reaction_upvote());
        });
    }

    #[test]
    fn create_post_reaction_should_work_downvote() {
        ExtBuilder::build_with_post().execute_with(|| {
            assert_ok!(_create_post_reaction(
                Some(Origin::signed(ACCOUNT2)),
                None,
                Some(self::reaction_downvote())
            )); // ReactionId 1 by ACCOUNT2 which is permitted by default

            // Check storages
            assert_eq!(Reactions::reaction_ids_by_post_id(POST1), vec![REACTION1]);
            assert_eq!(Reactions::next_reaction_id(), REACTION2);

            // Check post reaction counters
            let post = Posts::post_by_id(POST1).unwrap();
            assert_eq!(post.upvotes_count, 0);
            assert_eq!(post.downvotes_count, 1);

            // Check whether data stored correctly
            let reaction = Reactions::reaction_by_id(REACTION1).unwrap();
            assert_eq!(reaction.created.account, ACCOUNT2);
            assert_eq!(reaction.kind, self::reaction_downvote());
        });
    }

    #[test]
    fn create_post_reaction_should_fail_with_account_already_reacted() {
        ExtBuilder::build_with_post().execute_with(|| {
            assert_ok!(_create_default_post_reaction()); // ReactionId1

            // Try to catch an error creating reaction by the same account
            assert_noop!(_create_default_post_reaction(), ReactionsError::<TestRuntime>::AccountAlreadyReacted);
        });
    }

    #[test]
    fn create_post_reaction_should_fail_with_post_not_found() {
        ExtBuilder::build().execute_with(|| {
            // Try to catch an error creating reaction by the same account
            assert_noop!(_create_default_post_reaction(), PostsError::<TestRuntime>::PostNotFound);
        });
    }

    #[test]
    fn create_post_reaction_should_fail_with_cannot_react_when_space_hidden() {
        ExtBuilder::build_with_post().execute_with(|| {
            assert_ok!(_update_space(
                None,
                None,
                Some(self::space_update(None, None, None, Some(true), None))
            ));

            assert_noop!(_create_default_post_reaction(), ReactionsError::<TestRuntime>::CannotReactWhenSpaceHidden);
        });
    }

    #[test]
    fn create_post_reaction_should_fail_with_cannot_react_when_post_hidden() {
        ExtBuilder::build_with_post().execute_with(|| {
            assert_ok!(_update_post(
                None,
                None,
                Some(self::post_update(None, None, Some(true)))
            ));

            assert_noop!(_create_default_post_reaction(), ReactionsError::<TestRuntime>::CannotReactWhenPostHidden);
        });
    }

// Rating system tests

    #[test]
    fn check_results_of_score_diff_for_action_with_common_values() {
        ExtBuilder::build().execute_with(|| {
            assert_eq!(Scores::score_diff_for_action(1, self::scoring_action_upvote_post()), UpvotePostActionWeight::get() as i16);
            assert_eq!(Scores::score_diff_for_action(1, self::scoring_action_downvote_post()), DownvotePostActionWeight::get() as i16);
            assert_eq!(Scores::score_diff_for_action(1, self::scoring_action_share_post()), SharePostActionWeight::get() as i16);
            assert_eq!(Scores::score_diff_for_action(1, self::scoring_action_create_comment()), CreateCommentActionWeight::get() as i16);
            assert_eq!(Scores::score_diff_for_action(1, self::scoring_action_upvote_comment()), UpvoteCommentActionWeight::get() as i16);
            assert_eq!(Scores::score_diff_for_action(1, self::scoring_action_downvote_comment()), DownvoteCommentActionWeight::get() as i16);
            assert_eq!(Scores::score_diff_for_action(1, self::scoring_action_share_comment()), ShareCommentActionWeight::get() as i16);
            assert_eq!(Scores::score_diff_for_action(1, self::scoring_action_follow_space()), FollowSpaceActionWeight::get() as i16);
            assert_eq!(Scores::score_diff_for_action(1, self::scoring_action_follow_account()), FollowAccountActionWeight::get() as i16);
        });
    }

    #[test]
    fn check_results_of_score_diff_for_action_with_random_values() {
        ExtBuilder::build().execute_with(|| {
            assert_eq!(Scores::score_diff_for_action(32768, self::scoring_action_upvote_post()), 80); // 2^15
            assert_eq!(Scores::score_diff_for_action(32769, self::scoring_action_upvote_post()), 80); // 2^15 + 1
            assert_eq!(Scores::score_diff_for_action(65535, self::scoring_action_upvote_post()), 80); // 2^16 - 1
            assert_eq!(Scores::score_diff_for_action(65536, self::scoring_action_upvote_post()), 85); // 2^16
        });
    }

//--------------------------------------------------------------------------------------------------

    #[test]
    fn change_space_score_should_work_for_follow_space() {
        ExtBuilder::build_with_space().execute_with(|| {
            assert_ok!(_follow_space(
                Some(Origin::signed(ACCOUNT2)),
                Some(SPACE1)
            ));

            assert_eq!(Spaces::space_by_id(SPACE1).unwrap().score, FollowSpaceActionWeight::get() as i32);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT1).unwrap().reputation, 1 + FollowSpaceActionWeight::get() as u32);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT2).unwrap().reputation, 1);
        });
    }

    #[test]
    fn change_space_score_should_work_for_unfollow_space() {
        ExtBuilder::build_with_space().execute_with(|| {
            assert_ok!(_follow_space(
                Some(Origin::signed(ACCOUNT2)),
                Some(SPACE1)
            ));
            assert_ok!(_unfollow_space(
                Some(Origin::signed(ACCOUNT2)),
                Some(SPACE1)
            ));

            assert_eq!(Spaces::space_by_id(SPACE1).unwrap().score, 0);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT1).unwrap().reputation, 1);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT2).unwrap().reputation, 1);
        });
    }

    #[test]
    fn change_space_score_should_work_for_upvote_post() {
        ExtBuilder::build_with_post().execute_with(|| {
            assert_ok!(_create_post_reaction(Some(Origin::signed(ACCOUNT2)), None, None)); // ReactionId 1

            assert_eq!(Spaces::space_by_id(SPACE1).unwrap().score, UpvotePostActionWeight::get() as i32);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT1).unwrap().reputation, 1 + UpvotePostActionWeight::get() as u32);
        });
    }

    #[test]
    fn change_space_score_should_work_for_downvote_post() {
        ExtBuilder::build_with_post().execute_with(|| {
            assert_ok!(_create_post_reaction(
                Some(Origin::signed(ACCOUNT2)),
                None,
                Some(self::reaction_downvote())
            )); // ReactionId 1

            assert_eq!(Spaces::space_by_id(SPACE1).unwrap().score, DownvotePostActionWeight::get() as i32);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT1).unwrap().reputation, 1);
        });
    }

//--------------------------------------------------------------------------------------------------

    #[test]
    fn change_post_score_should_work_for_create_comment() {
        ExtBuilder::build_with_post().execute_with(|| {
            assert_ok!(_create_comment(
                Some(Origin::signed(ACCOUNT2)),
                None,
                None,
                None
            )); // PostId 2

            assert_eq!(Posts::post_by_id(POST1).unwrap().score, CreateCommentActionWeight::get() as i32);
            assert_eq!(Spaces::space_by_id(SPACE1).unwrap().score, CreateCommentActionWeight::get() as i32);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT1).unwrap().reputation, 1 + CreateCommentActionWeight::get() as u32);
            assert_eq!(Scores::post_score_by_account((ACCOUNT2, POST1, self::scoring_action_create_comment())), Some(CreateCommentActionWeight::get()));
        });
    }

    #[test]
    fn change_post_score_should_work_for_upvote_post() {
        ExtBuilder::build_with_post().execute_with(|| {
            assert_ok!(_create_post_reaction(
                Some(Origin::signed(ACCOUNT2)),
                None,
                None
            ));

            assert_eq!(Posts::post_by_id(POST1).unwrap().score, UpvotePostActionWeight::get() as i32);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT1).unwrap().reputation, 1 + UpvotePostActionWeight::get() as u32);
            assert_eq!(Scores::post_score_by_account((ACCOUNT2, POST1, self::scoring_action_upvote_post())), Some(UpvotePostActionWeight::get()));
        });
    }

    #[test]
    fn change_post_score_should_work_for_downvote_post() {
        ExtBuilder::build_with_post().execute_with(|| {
            assert_ok!(_create_post_reaction(
                Some(Origin::signed(ACCOUNT2)),
                None,
                Some(self::reaction_downvote())
            ));

            assert_eq!(Posts::post_by_id(POST1).unwrap().score, DownvotePostActionWeight::get() as i32);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT1).unwrap().reputation, 1);
            assert_eq!(Scores::post_score_by_account((ACCOUNT2, POST1, self::scoring_action_downvote_post())), Some(DownvotePostActionWeight::get()));
        });
    }

    #[test]
    fn change_post_score_should_for_revert_upvote() {
        ExtBuilder::build_with_post().execute_with(|| {
            assert_ok!(_create_post_reaction(
                Some(Origin::signed(ACCOUNT2)),
                None,
                None
            ));
            // ReactionId 1
            assert_ok!(_delete_post_reaction(
                Some(Origin::signed(ACCOUNT2)),
                None,
                REACTION1
            ));

            assert_eq!(Posts::post_by_id(POST1).unwrap().score, 0);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT1).unwrap().reputation, 1);
            assert!(Scores::post_score_by_account((ACCOUNT2, POST1, self::scoring_action_upvote_post())).is_none());
        });
    }

    #[test]
    fn change_post_score_should_for_revert_downvote() {
        ExtBuilder::build_with_post().execute_with(|| {
            assert_ok!(_create_post_reaction(
                Some(Origin::signed(ACCOUNT2)),
                None,
                Some(self::reaction_downvote())
            ));
            // ReactionId 1
            assert_ok!(_delete_post_reaction(
                Some(Origin::signed(ACCOUNT2)),
                None,
                REACTION1
            ));

            assert_eq!(Posts::post_by_id(POST1).unwrap().score, 0);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT1).unwrap().reputation, 1);
            assert!(Scores::post_score_by_account((ACCOUNT2, POST1, self::scoring_action_downvote_post())).is_none());
        });
    }

    #[test]
    fn change_post_score_should_work_for_change_upvote_with_downvote() {
        ExtBuilder::build_with_post().execute_with(|| {
            assert_ok!(_create_post_reaction(
                Some(Origin::signed(ACCOUNT2)),
                None,
                None
            ));
            // ReactionId 1
            assert_ok!(_update_post_reaction(
                Some(Origin::signed(ACCOUNT2)),
                None,
                REACTION1,
                Some(self::reaction_downvote())
            ));

            assert_eq!(Posts::post_by_id(POST1).unwrap().score, DownvotePostActionWeight::get() as i32);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT1).unwrap().reputation, 1);
            assert!(Scores::post_score_by_account((ACCOUNT2, POST1, self::scoring_action_upvote_post())).is_none());
            assert_eq!(Scores::post_score_by_account((ACCOUNT2, POST1, self::scoring_action_downvote_post())), Some(DownvotePostActionWeight::get()));
        });
    }

    #[test]
    fn change_post_score_should_work_for_change_downvote_with_upvote() {
        ExtBuilder::build_with_post().execute_with(|| {
            assert_ok!(_create_post_reaction(
                Some(Origin::signed(ACCOUNT2)),
                None,
                Some(self::reaction_downvote())
            ));
            // ReactionId 1
            assert_ok!(_update_post_reaction(
                Some(Origin::signed(ACCOUNT2)),
                None,
                REACTION1,
                None
            ));

            assert_eq!(Posts::post_by_id(POST1).unwrap().score, UpvotePostActionWeight::get() as i32);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT1).unwrap().reputation, 1 + UpvotePostActionWeight::get() as u32);
            assert!(Scores::post_score_by_account((ACCOUNT2, POST1, self::scoring_action_downvote_post())).is_none());
            assert_eq!(Scores::post_score_by_account((ACCOUNT2, POST1, self::scoring_action_upvote_post())), Some(UpvotePostActionWeight::get()));
        });
    }

//--------------------------------------------------------------------------------------------------

    #[test]
    fn change_social_account_reputation_should_work_with_max_score_diff() {
        ExtBuilder::build_with_space().execute_with(|| {
            assert_ok!(_create_post(Some(Origin::signed(ACCOUNT1)), None, None, None));
            assert_ok!(Scores::change_social_account_reputation(
                ACCOUNT1,
                ACCOUNT2,
                std::i16::MAX,
                self::scoring_action_follow_account())
            );
        });
    }

    #[test]
    fn change_social_account_reputation_should_work_with_min_score_diff() {
        ExtBuilder::build_with_space().execute_with(|| {
            assert_ok!(_create_post(Some(Origin::signed(ACCOUNT1)), None, None, None));
            assert_ok!(Scores::change_social_account_reputation(
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
            assert_ok!(Scores::change_social_account_reputation(
                ACCOUNT1,
                ACCOUNT2,
                DownvotePostActionWeight::get(),
                self::scoring_action_downvote_post())
            );
            assert_eq!(Scores::account_reputation_diff_by_account((ACCOUNT2, ACCOUNT1, self::scoring_action_downvote_post())), Some(0));
            assert_eq!(Profiles::social_account_by_id(ACCOUNT1).unwrap().reputation, 1);

            // To ensure function works correctly, multiply default UpvotePostActionWeight by two
            assert_ok!(Scores::change_social_account_reputation(
                ACCOUNT1,
                ACCOUNT2,
                UpvotePostActionWeight::get() * 2,
                self::scoring_action_upvote_post())
            );

            assert_eq!(
                Scores::account_reputation_diff_by_account(
                    (
                        ACCOUNT2,
                        ACCOUNT1,
                        self::scoring_action_upvote_post()
                    )
                ), Some(UpvotePostActionWeight::get() * 2)
            );

            assert_eq!(Profiles::social_account_by_id(ACCOUNT1).unwrap().reputation, 1 + (UpvotePostActionWeight::get() * 2) as u32);
        });
    }

//--------------------------------------------------------------------------------------------------

    #[test]
    fn change_comment_score_should_work_for_upvote() {
        ExtBuilder::build_with_space().execute_with(|| {
            assert_ok!(_create_post(
                Some(Origin::signed(ACCOUNT1)),
                None,
                None,
                None
            ));
            // PostId 1
            assert_ok!(_create_comment(
                Some(Origin::signed(ACCOUNT2)),
                None,
                None,
                None
            )); // PostId 2

            assert_ok!(_score_post_on_reaction_with_id(
                ACCOUNT3,
                POST2,
                self::reaction_upvote()
            ));

            assert_eq!(Posts::post_by_id(POST2).unwrap().score, UpvoteCommentActionWeight::get() as i32);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT1).unwrap().reputation, 1 + CreateCommentActionWeight::get() as u32);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT2).unwrap().reputation, 1 + UpvoteCommentActionWeight::get() as u32);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT3).unwrap().reputation, 1);
            assert_eq!(Scores::post_score_by_account((ACCOUNT3, POST2, self::scoring_action_upvote_comment())), Some(UpvoteCommentActionWeight::get()));
        });
    }

    #[test]
    fn change_comment_score_should_work_for_downvote() {
        ExtBuilder::build_with_space().execute_with(|| {
            assert_ok!(_create_post(
                Some(Origin::signed(ACCOUNT1)),
                None,
                None,
                None
            ));
            // PostId 1
            assert_ok!(_create_comment(
                Some(Origin::signed(ACCOUNT2)),
                None,
                None,
                None
            )); // PostId 2

            assert_ok!(_score_post_on_reaction_with_id(ACCOUNT3, POST2, self::reaction_downvote()));

            assert_eq!(Posts::post_by_id(POST2).unwrap().score, DownvoteCommentActionWeight::get() as i32);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT1).unwrap().reputation, 1 + CreateCommentActionWeight::get() as u32);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT2).unwrap().reputation, 1);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT3).unwrap().reputation, 1);
            assert_eq!(Scores::post_score_by_account((ACCOUNT3, POST2, self::scoring_action_downvote_comment())), Some(DownvoteCommentActionWeight::get()));
        });
    }

    #[test]
    fn change_comment_score_should_for_revert_upvote() {
        ExtBuilder::build_with_space().execute_with(|| {
            assert_ok!(_create_post(
                Some(Origin::signed(ACCOUNT1)),
                None,
                None,
                None
            ));
            // PostId 1
            assert_ok!(_create_comment(
                Some(Origin::signed(ACCOUNT2)),
                None,
                None,
                None
            )); // PostId 2

            assert_ok!(_score_post_on_reaction_with_id(ACCOUNT3, POST2, self::reaction_upvote()));
            assert_ok!(_score_post_on_reaction_with_id(ACCOUNT3, POST2, self::reaction_upvote()));

            assert_eq!(Posts::post_by_id(POST2).unwrap().score, 0);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT1).unwrap().reputation, 1 + CreateCommentActionWeight::get() as u32);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT2).unwrap().reputation, 1);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT3).unwrap().reputation, 1);
            assert!(Scores::post_score_by_account((ACCOUNT1, POST2, self::scoring_action_upvote_comment())).is_none());
        });
    }

    #[test]
    fn change_comment_score_should_for_revert_downvote() {
        ExtBuilder::build_with_space().execute_with(|| {
            assert_ok!(_create_post(
                Some(Origin::signed(ACCOUNT1)),
                None,
                None,
                None
            ));
            // PostId 1
            assert_ok!(_create_comment(
                Some(Origin::signed(ACCOUNT2)),
                None,
                None,
                None
            )); // PostId 2

            assert_ok!(_score_post_on_reaction_with_id(ACCOUNT3, POST2, self::reaction_downvote()));
            assert_ok!(_score_post_on_reaction_with_id(ACCOUNT3, POST2, self::reaction_downvote()));

            assert_eq!(Posts::post_by_id(POST2).unwrap().score, 0);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT1).unwrap().reputation, 1 + CreateCommentActionWeight::get() as u32);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT2).unwrap().reputation, 1);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT3).unwrap().reputation, 1);
            assert!(Scores::post_score_by_account((ACCOUNT1, POST2, self::scoring_action_downvote_comment())).is_none());
        });
    }

    #[test]
    fn change_comment_score_check_for_cancel_upvote() {
        ExtBuilder::build_with_space().execute_with(|| {
            assert_ok!(_create_post(
                Some(Origin::signed(ACCOUNT1)),
                None,
                None,
                None
            ));
            // PostId 1
            assert_ok!(_create_comment(
                Some(Origin::signed(ACCOUNT2)),
                None,
                None,
                None
            )); // PostId 2

            assert_ok!(_score_post_on_reaction_with_id(ACCOUNT3, POST2, self::reaction_upvote()));
            assert_ok!(_score_post_on_reaction_with_id(ACCOUNT3, POST2, self::reaction_downvote()));

            assert_eq!(Posts::post_by_id(POST2).unwrap().score, DownvoteCommentActionWeight::get() as i32);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT1).unwrap().reputation, 1 + CreateCommentActionWeight::get() as u32);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT2).unwrap().reputation, 1);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT3).unwrap().reputation, 1);
            assert!(Scores::post_score_by_account((ACCOUNT3, POST2, self::scoring_action_upvote_comment())).is_none());
            assert_eq!(Scores::post_score_by_account((ACCOUNT3, POST2, self::scoring_action_downvote_comment())), Some(DownvoteCommentActionWeight::get()));
        });
    }

    #[test]
    fn change_comment_score_check_for_cancel_downvote() {
        ExtBuilder::build_with_space().execute_with(|| {
            assert_ok!(_create_post(
                Some(Origin::signed(ACCOUNT1)),
                None,
                None,
                None
            ));
            // PostId 1
            assert_ok!(_create_comment(
                Some(Origin::signed(ACCOUNT2)),
                None,
                None,
                None
            )); // PostId 2

            assert_ok!(_score_post_on_reaction_with_id(ACCOUNT3, POST2, self::reaction_downvote()));
            assert_ok!(_score_post_on_reaction_with_id(ACCOUNT3, POST2, self::reaction_upvote()));

            assert_eq!(Posts::post_by_id(POST2).unwrap().score, UpvoteCommentActionWeight::get() as i32);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT1).unwrap().reputation, 1 + CreateCommentActionWeight::get() as u32);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT2).unwrap().reputation, 1 + UpvoteCommentActionWeight::get() as u32);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT3).unwrap().reputation, 1);
            assert!(Scores::post_score_by_account((ACCOUNT3, POST2, self::scoring_action_downvote_comment())).is_none());
            assert_eq!(Scores::post_score_by_account((ACCOUNT3, POST2, self::scoring_action_upvote_comment())), Some(UpvoteCommentActionWeight::get()));
        });
    }

// Shares tests

    #[test]
    fn share_post_should_work() {
        ExtBuilder::build_with_post().execute_with(|| {
            assert_ok!(_create_space(
                Some(Origin::signed(ACCOUNT2)),
                None,
                Some(Some(b"space2_handle".to_vec())),
                None
            )); // SpaceId 2 by ACCOUNT2

            assert_ok!(_create_post(
                Some(Origin::signed(ACCOUNT2)),
                Some(Some(SPACE2)),
                Some(self::extension_shared_post(POST1)),
                None
            )); // Share PostId 1 on SpaceId 2 by ACCOUNT2 which is permitted by default in both spaces

            // Check storages
            assert_eq!(Posts::post_ids_by_space_id(SPACE1), vec![POST1]);
            assert_eq!(Posts::post_ids_by_space_id(SPACE2), vec![POST2]);
            assert_eq!(Posts::next_post_id(), POST3);

            assert_eq!(Posts::shared_post_ids_by_original_post_id(POST1), vec![POST2]);

            // Check whether data stored correctly
            assert_eq!(Posts::post_by_id(POST1).unwrap().shares_count, 1);

            let shared_post = Posts::post_by_id(POST2).unwrap();

            assert_eq!(shared_post.space_id, Some(SPACE2));
            assert_eq!(shared_post.created.account, ACCOUNT2);
            assert_eq!(shared_post.extension, self::extension_shared_post(POST1));
        });
    }

    #[test]
    fn share_post_should_work_with_a_few_roles() {
        ExtBuilder::build_with_a_few_roles_granted_to_account2(vec![SP::CreatePosts]).execute_with(|| {
            assert_ok!(_create_space(
                None, // From ACCOUNT1
                None, // With no parent_id provided
                Some(None), // Provided without any handle
                None // With default space content,
            ));
            // SpaceId 2
            assert_ok!(_create_post(
                None, // From ACCOUNT1
                Some(Some(SPACE2)),
                None, // With RegularPost extension
                None // With default post content
            )); // PostId 1 on SpaceId 2

            assert_ok!(_create_post(
                Some(Origin::signed(ACCOUNT2)),
                Some(Some(SPACE1)),
                Some(self::extension_shared_post(POST1)),
                None
            )); // Share PostId 1 on SpaceId 1 by ACCOUNT2 which is permitted by RoleId 1 from ext
        });
    }

    #[test]
    fn share_post_should_work_for_share_own_post_in_same_own_space() {
        ExtBuilder::build_with_post().execute_with(|| {
            assert_ok!(_create_post(
                Some(Origin::signed(ACCOUNT1)),
                Some(Some(SPACE1)),
                Some(self::extension_shared_post(POST1)),
                None
            )); // Share PostId 1

            // Check storages
            assert_eq!(Posts::post_ids_by_space_id(SPACE1), vec![POST1, POST2]);
            assert_eq!(Posts::next_post_id(), POST3);

            assert_eq!(Posts::shared_post_ids_by_original_post_id(POST1), vec![POST2]);

            // Check whether data stored correctly
            assert_eq!(Posts::post_by_id(POST1).unwrap().shares_count, 1);

            let shared_post = Posts::post_by_id(POST2).unwrap();
            assert_eq!(shared_post.space_id, Some(SPACE1));
            assert_eq!(shared_post.created.account, ACCOUNT1);
            assert_eq!(shared_post.extension, self::extension_shared_post(POST1));
        });
    }

    #[test]
    fn share_post_should_change_score() {
        ExtBuilder::build_with_post().execute_with(|| {
            assert_ok!(_create_space(
                Some(Origin::signed(ACCOUNT2)),
                None,
                Some(Some(b"space2_handle".to_vec())),
                None
            )); // SpaceId 2 by ACCOUNT2

            assert_ok!(_create_post(
                Some(Origin::signed(ACCOUNT2)),
                Some(Some(SPACE2)),
                Some(self::extension_shared_post(POST1)),
                None
            )); // Share PostId 1 on SpaceId 2 by ACCOUNT2

            assert_eq!(Posts::post_by_id(POST1).unwrap().score, SharePostActionWeight::get() as i32);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT1).unwrap().reputation, 1 + SharePostActionWeight::get() as u32);
            assert_eq!(Scores::post_score_by_account((ACCOUNT2, POST1, self::scoring_action_share_post())), Some(SharePostActionWeight::get()));
        });
    }

    #[test]
    fn share_post_should_not_change_score() {
        ExtBuilder::build_with_post().execute_with(|| {
            assert_ok!(_create_post(
                Some(Origin::signed(ACCOUNT1)),
                Some(Some(SPACE1)),
                Some(self::extension_shared_post(POST1)),
                None
            )); // Share PostId

            assert_eq!(Posts::post_by_id(POST1).unwrap().score, 0);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT1).unwrap().reputation, 1);
            assert!(Scores::post_score_by_account((ACCOUNT1, POST1, self::scoring_action_share_post())).is_none());
        });
    }

    #[test]
    fn share_post_should_fail_with_original_post_not_found() {
        ExtBuilder::build_with_space().execute_with(|| {
            assert_ok!(_create_space(
                Some(Origin::signed(ACCOUNT2)),
                None,
                Some(Some(b"space2_handle".to_vec())),
                None
            )); // SpaceId 2 by ACCOUNT2

            // Skipped creating PostId 1
            assert_noop!(_create_post(
                Some(Origin::signed(ACCOUNT2)),
                Some(Some(SPACE2)),
                Some(self::extension_shared_post(POST1)),
                None
            ), PostsError::<TestRuntime>::OriginalPostNotFound);
        });
    }

    #[test]
    fn share_post_should_fail_with_cannot_share_sharing_post() {
        ExtBuilder::build_with_post().execute_with(|| {
            assert_ok!(_create_space(
                Some(Origin::signed(ACCOUNT2)),
                None,
                Some(Some(b"space2_handle".to_vec())),
                None
            )); // SpaceId 2 by ACCOUNT2

            assert_ok!(_create_post(
                Some(Origin::signed(ACCOUNT2)),
                Some(Some(SPACE2)),
                Some(self::extension_shared_post(POST1)),
                None)
            );

            // Try to share post with extension SharedPost
            assert_noop!(_create_post(
                Some(Origin::signed(ACCOUNT1)),
                Some(Some(SPACE1)),
                Some(self::extension_shared_post(POST2)),
                None
            ), PostsError::<TestRuntime>::CannotShareSharingPost);
        });
    }

    #[test]
    fn share_post_should_fail_with_no_permission_to_create_posts() {
        ExtBuilder::build_with_post().execute_with(|| {
            assert_ok!(_create_space(
                Some(Origin::signed(ACCOUNT1)),
                None, // With no parent_id provided
                Some(None), // No space_handle provided (ok)
                None // Default space content,
            )); // SpaceId 2 by ACCOUNT1

            // Try to share post with extension SharedPost
            assert_noop!(_create_post(
                Some(Origin::signed(ACCOUNT2)),
                Some(Some(SPACE2)),
                Some(self::extension_shared_post(POST1)),
                None
            ), PostsError::<TestRuntime>::NoPermissionToCreatePosts);
        });
    }

    #[test]
    fn share_post_should_fail_with_a_few_roles_no_permission() {
        ExtBuilder::build_with_a_few_roles_granted_to_account2(vec![SP::CreatePosts]).execute_with(|| {
            assert_ok!(_create_space(
                None, // From ACCOUNT1
                None, // With no parent_id provided
                Some(None), // Provided without any handle
                None // With default space content
            ));
            // SpaceId 2
            assert_ok!(_create_post(
                None, // From ACCOUNT1
                Some(Some(SPACE2)),
                None, // With RegularPost extension
                None // With default post content
            )); // PostId 1 on SpaceId 2

            assert_ok!(_delete_default_role());

            assert_noop!(_create_post(
                Some(Origin::signed(ACCOUNT2)),
                Some(Some(SPACE1)),
                Some(self::extension_shared_post(POST1)),
                None
            ), PostsError::<TestRuntime>::NoPermissionToCreatePosts);
        });
    }

// Profiles tests

    #[test]
    fn create_profile_should_work() {
        ExtBuilder::build().execute_with(|| {
            assert_ok!(_create_default_profile()); // AccountId 1

            let profile = Profiles::social_account_by_id(ACCOUNT1).unwrap().profile.unwrap();
            assert_eq!(profile.created.account, ACCOUNT1);
            assert!(profile.updated.is_none());
            assert_eq!(profile.content, self::profile_content_ipfs());

            assert!(ProfileHistory::edit_history(ACCOUNT1).is_empty());
        });
    }

    #[test]
    fn create_profile_should_fail_with_profile_already_created() {
        ExtBuilder::build().execute_with(|| {
            assert_ok!(_create_default_profile());
            // AccountId 1
            assert_noop!(_create_default_profile(), ProfilesError::<TestRuntime>::ProfileAlreadyCreated);
        });
    }

    #[test]
    fn create_profile_should_fail_with_invalid_ipfs_cid() {
        ExtBuilder::build().execute_with(|| {
            let content_ipfs = Content::IPFS(b"QmV9tSDx9UiPeWExXEeH6aoDvmihvx6j".to_vec());

            assert_noop!(_create_profile(
                None,
                Some(content_ipfs)
            ), UtilsError::<TestRuntime>::InvalidIpfsCid);
        });
    }

    #[test]
    fn update_profile_should_work() {
        ExtBuilder::build().execute_with(|| {
            assert_ok!(_create_default_profile());
            // AccountId 1
            assert_ok!(_update_profile(
                None,
                Some(self::space_content_ipfs())
            ));

            // Check whether profile updated correctly
            let profile = Profiles::social_account_by_id(ACCOUNT1).unwrap().profile.unwrap();
            assert!(profile.updated.is_some());
            assert_eq!(profile.content, self::space_content_ipfs());

            // Check whether profile history is written correctly
            let profile_history = ProfileHistory::edit_history(ACCOUNT1)[0].clone();
            assert_eq!(profile_history.old_data.content, Some(self::profile_content_ipfs()));
        });
    }

    #[test]
    fn update_profile_should_fail_with_social_account_not_found() {
        ExtBuilder::build().execute_with(|| {
            assert_noop!(_update_profile(
                None,
                Some(self::profile_content_ipfs())
            ), ProfilesError::<TestRuntime>::SocialAccountNotFound);
        });
    }

    #[test]
    fn update_profile_should_fail_with_account_has_no_profile() {
        ExtBuilder::build().execute_with(|| {
            assert_ok!(ProfileFollows::follow_account(Origin::signed(ACCOUNT1), ACCOUNT2));
            assert_noop!(_update_profile(
                None,
                Some(self::profile_content_ipfs())
            ), ProfilesError::<TestRuntime>::AccountHasNoProfile);
        });
    }

    #[test]
    fn update_profile_should_fail_with_no_updates_for_profile() {
        ExtBuilder::build().execute_with(|| {
            assert_ok!(_create_default_profile());
            // AccountId 1
            assert_noop!(_update_profile(
                None,
                None
            ), ProfilesError::<TestRuntime>::NoUpdatesForProfile);
        });
    }

    #[test]
    fn update_profile_should_fail_with_invalid_ipfs_cid() {
        ExtBuilder::build().execute_with(|| {
            let content_ipfs = Content::IPFS(b"QmV9tSDx9UiPeWExXEeH6aoDvmihvx6j".to_vec());

            assert_ok!(_create_default_profile());
            assert_noop!(_update_profile(
                None,
                Some(content_ipfs)
            ), UtilsError::<TestRuntime>::InvalidIpfsCid);
        });
    }

// Space following tests

    #[test]
    fn follow_space_should_work() {
        ExtBuilder::build_with_space().execute_with(|| {
            assert_ok!(_default_follow_space()); // Follow SpaceId 1 by ACCOUNT2

            assert_eq!(Spaces::space_by_id(SPACE1).unwrap().followers_count, 2);
            assert_eq!(SpaceFollows::spaces_followed_by_account(ACCOUNT2), vec![SPACE1]);
            assert_eq!(SpaceFollows::space_followers(SPACE1), vec![ACCOUNT1, ACCOUNT2]);
            assert_eq!(SpaceFollows::space_followed_by_account((ACCOUNT2, SPACE1)), true);
        });
    }

    #[test]
    fn follow_space_should_fail_with_space_not_found() {
        ExtBuilder::build().execute_with(|| {
            assert_noop!(_default_follow_space(), SpacesError::<TestRuntime>::SpaceNotFound);
        });
    }

    #[test]
    fn follow_space_should_fail_with_already_space_follower() {
        ExtBuilder::build_with_space().execute_with(|| {
            assert_ok!(_default_follow_space()); // Follow SpaceId 1 by ACCOUNT2

            assert_noop!(_default_follow_space(), SpaceFollowsError::<TestRuntime>::AlreadySpaceFollower);
        });
    }

    #[test]
    fn follow_space_should_fail_with_cannot_follow_hidden_space() {
        ExtBuilder::build_with_space().execute_with(|| {
            assert_ok!(_update_space(
                None,
                None,
                Some(self::space_update(None, None, None, Some(true), None))
            ));

            assert_noop!(_default_follow_space(), SpaceFollowsError::<TestRuntime>::CannotFollowHiddenSpace);
        });
    }

    #[test]
    fn unfollow_space_should_work() {
        ExtBuilder::build_with_space().execute_with(|| {
            assert_ok!(_default_follow_space());
            // Follow SpaceId 1 by ACCOUNT2
            assert_ok!(_default_unfollow_space());

            assert_eq!(Spaces::space_by_id(SPACE1).unwrap().followers_count, 1);
            assert!(SpaceFollows::spaces_followed_by_account(ACCOUNT2).is_empty());
            assert_eq!(SpaceFollows::space_followers(SPACE1), vec![ACCOUNT1]);
        });
    }

    #[test]
    fn unfollow_space_should_fail_with_space_not_found() {
        ExtBuilder::build_with_space_follow_no_space().execute_with(|| {
            assert_noop!(_default_unfollow_space(), SpacesError::<TestRuntime>::SpaceNotFound);
        });
    }

    #[test]
    fn unfollow_space_should_fail_with_not_space_follower() {
        ExtBuilder::build_with_space().execute_with(|| {
            assert_noop!(_default_unfollow_space(), SpaceFollowsError::<TestRuntime>::NotSpaceFollower);
        });
    }

// Account following tests

    #[test]
    fn follow_account_should_work() {
        ExtBuilder::build().execute_with(|| {
            assert_ok!(_default_follow_account()); // Follow ACCOUNT1 by ACCOUNT2

            assert_eq!(ProfileFollows::accounts_followed_by_account(ACCOUNT2), vec![ACCOUNT1]);
            assert_eq!(ProfileFollows::account_followers(ACCOUNT1), vec![ACCOUNT2]);
            assert_eq!(ProfileFollows::account_followed_by_account((ACCOUNT2, ACCOUNT1)), true);
        });
    }

    #[test]
    fn follow_account_should_fail_with_account_cannot_follow_itself() {
        ExtBuilder::build().execute_with(|| {
            assert_noop!(_follow_account(
                None,
                Some(ACCOUNT2)
            ), ProfileFollowsError::<TestRuntime>::AccountCannotFollowItself);
        });
    }

    #[test]
    fn follow_account_should_fail_with_already_account_follower() {
        ExtBuilder::build().execute_with(|| {
            assert_ok!(_default_follow_account());

            assert_noop!(_default_follow_account(), ProfileFollowsError::<TestRuntime>::AlreadyAccountFollower);
        });
    }

    #[test]
    fn unfollow_account_should_work() {
        ExtBuilder::build().execute_with(|| {
            assert_ok!(_default_follow_account());
            // Follow ACCOUNT1 by ACCOUNT2
            assert_ok!(_default_unfollow_account());

            assert!(ProfileFollows::accounts_followed_by_account(ACCOUNT2).is_empty());
            assert!(ProfileFollows::account_followers(ACCOUNT1).is_empty());
            assert_eq!(ProfileFollows::account_followed_by_account((ACCOUNT2, ACCOUNT1)), false);
        });
    }

    #[test]
    fn unfollow_account_should_fail_with_account_cannot_unfollow_itself() {
        ExtBuilder::build().execute_with(|| {
            assert_noop!(_unfollow_account(
                None,
                Some(ACCOUNT2)
            ), ProfileFollowsError::<TestRuntime>::AccountCannotUnfollowItself);
        });
    }

    #[test]
    fn unfollow_account_should_fail_with_not_account_follower() {
        ExtBuilder::build().execute_with(|| {
            assert_ok!(_default_follow_account());
            assert_ok!(_default_unfollow_account());

            assert_noop!(_default_unfollow_account(), ProfileFollowsError::<TestRuntime>::NotAccountFollower);
        });
    }

// Transfer ownership tests

    #[test]
    fn transfer_space_ownership_should_work() {
        ExtBuilder::build_with_space().execute_with(|| {
            assert_ok!(_transfer_default_space_ownership()); // Transfer SpaceId 1 owned by ACCOUNT1 to ACCOUNT2

            assert_eq!(SpaceOwnership::pending_space_owner(SPACE1).unwrap(), ACCOUNT2);
        });
    }

    #[test]
    fn transfer_space_ownership_should_fail_with_space_not_found() {
        ExtBuilder::build().execute_with(|| {
            assert_noop!(_transfer_default_space_ownership(), SpacesError::<TestRuntime>::SpaceNotFound);
        });
    }

    #[test]
    fn transfer_space_ownership_should_fail_with_not_a_space_owner() {
        ExtBuilder::build_with_space().execute_with(|| {
            assert_noop!(_transfer_space_ownership(
                Some(Origin::signed(ACCOUNT2)),
                None,
                Some(ACCOUNT1)
            ), SpacesError::<TestRuntime>::NotASpaceOwner);
        });
    }

    #[test]
    fn transfer_space_ownership_should_fail_with_cannot_transfer_to_current_owner() {
        ExtBuilder::build_with_space().execute_with(|| {
            assert_noop!(_transfer_space_ownership(
                Some(Origin::signed(ACCOUNT1)),
                None,
                Some(ACCOUNT1)
            ), SpaceOwnershipError::<TestRuntime>::CannotTranferToCurrentOwner);
        });
    }

    #[test]
    fn accept_pending_ownership_should_work() {
        ExtBuilder::build_with_space().execute_with(|| {
            assert_ok!(_transfer_default_space_ownership());
            // Transfer SpaceId 1 owned by ACCOUNT1 to ACCOUNT2
            assert_ok!(_accept_default_pending_ownership()); // Accepting a transfer from ACCOUNT2
            // Check whether owner was changed
            let space = Spaces::space_by_id(SPACE1).unwrap();
            assert_eq!(space.owner, ACCOUNT2);

            // Check whether storage state is correct
            assert!(SpaceOwnership::pending_space_owner(SPACE1).is_none());
        });
    }

    #[test]
    fn accept_pending_ownership_should_fail_with_space_not_found() {
        ExtBuilder::build_with_pending_ownership_transfer_no_space().execute_with(|| {
            assert_noop!(_accept_default_pending_ownership(), SpacesError::<TestRuntime>::SpaceNotFound);
        });
    }

    #[test]
    fn accept_pending_ownership_should_fail_with_no_pending_transfer_on_space() {
        ExtBuilder::build_with_space().execute_with(|| {
            assert_noop!(_accept_default_pending_ownership(), SpaceOwnershipError::<TestRuntime>::NoPendingTransferOnSpace);
        });
    }

    #[test]
    fn accept_pending_ownership_should_fail_if_origin_is_already_an_owner() {
        ExtBuilder::build_with_space().execute_with(|| {
            assert_ok!(_transfer_default_space_ownership());

            assert_noop!(_accept_pending_ownership(
                Some(Origin::signed(ACCOUNT1)),
                None
            ), SpaceOwnershipError::<TestRuntime>::AlreadyASpaceOwner);
        });
    }

    #[test]
    fn accept_pending_ownership_should_fail_if_origin_is_not_equal_to_pending_account() {
        ExtBuilder::build_with_space().execute_with(|| {
            assert_ok!(_transfer_default_space_ownership());

            assert_noop!(_accept_pending_ownership(
                Some(Origin::signed(ACCOUNT3)),
                None
            ), SpaceOwnershipError::<TestRuntime>::NotAllowedToAcceptOwnershipTransfer);
        });
    }

    #[test]
    fn reject_pending_ownership_should_work() {
        ExtBuilder::build_with_space().execute_with(|| {
            assert_ok!(_transfer_default_space_ownership());
            // Transfer SpaceId 1 owned by ACCOUNT1 to ACCOUNT2
            assert_ok!(_reject_default_pending_ownership()); // Rejecting a transfer from ACCOUNT2

            // Check whether owner was not changed
            let space = Spaces::space_by_id(SPACE1).unwrap();
            assert_eq!(space.owner, ACCOUNT1);

            // Check whether storage state is correct
            assert!(SpaceOwnership::pending_space_owner(SPACE1).is_none());
        });
    }

    #[test]
    fn reject_pending_ownership_should_work_with_reject_by_current_space_owner() {
        ExtBuilder::build_with_space().execute_with(|| {
            assert_ok!(_transfer_default_space_ownership());
            // Transfer SpaceId 1 owned by ACCOUNT1 to ACCOUNT2
            assert_ok!(_reject_default_pending_ownership_by_current_owner()); // Rejecting a transfer from ACCOUNT2

            // Check whether owner was not changed
            let space = Spaces::space_by_id(SPACE1).unwrap();
            assert_eq!(space.owner, ACCOUNT1);

            // Check whether storage state is correct
            assert!(SpaceOwnership::pending_space_owner(SPACE1).is_none());
        });
    }

    #[test]
    fn reject_pending_ownership_should_fail_with_space_not_found() {
        ExtBuilder::build_with_pending_ownership_transfer_no_space().execute_with(|| {
            assert_noop!(_reject_default_pending_ownership(), SpacesError::<TestRuntime>::SpaceNotFound);
        });
    }

    #[test]
    fn reject_pending_ownership_should_fail_with_no_pending_transfer_on_space() {
        ExtBuilder::build_with_space().execute_with(|| {
            assert_noop!(_reject_default_pending_ownership(), SpaceOwnershipError::<TestRuntime>::NoPendingTransferOnSpace); // Rejecting a transfer from ACCOUNT2
        });
    }

    #[test]
    fn reject_pending_ownership_should_fail_with_not_allowed_to_reject() {
        ExtBuilder::build_with_space().execute_with(|| {
            assert_ok!(_transfer_default_space_ownership()); // Transfer SpaceId 1 owned by ACCOUNT1 to ACCOUNT2

            assert_noop!(_reject_pending_ownership(
                Some(Origin::signed(ACCOUNT3)),
                None
            ), SpaceOwnershipError::<TestRuntime>::NotAllowedToRejectOwnershipTransfer); // Rejecting a transfer from ACCOUNT2
        });
    }
}