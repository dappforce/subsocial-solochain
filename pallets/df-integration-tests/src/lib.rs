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
    use sp_runtime::{
        traits::{BlakeTwo256, IdentityLookup, Zero},
        testing::Header,
        Perbill,
        Storage,
    };
    use frame_system::{self as system, EnsureRoot};

    use pallet_permissions::{
        SpacePermission,
        SpacePermission as SP,
        SpacePermissions,
    };
    use pallet_posts::{Post, PostUpdate, PostExtension, Comment, Error as PostsError};
    use pallet_profiles::{ProfileUpdate, Error as ProfilesError};
    use pallet_profile_follows::Error as ProfileFollowsError;
    use pallet_reactions::{ReactionId, ReactionKind, PostReactionScores, Error as ReactionsError};
    use pallet_scores::ScoringAction;
    use pallet_spaces::{SpaceById, SpaceUpdate, Error as SpacesError};
    use pallet_space_follows::Error as SpaceFollowsError;
    use pallet_space_ownership::Error as SpaceOwnershipError;
    use pallet_moderation::{EntityId, EntityStatus, ReportId};
    use pallet_utils::{
        mock_functions::*,
        Error as UtilsError, Module as Utils,
        SpaceId, PostId, User, Content,
    };

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
        type PalletInfo = ();
        type AccountData = pallet_balances::AccountData<u64>;
        type OnNewAccount = ();
        type OnKilledAccount = ();
        type SystemWeightInfo = ();
    }

    parameter_types! {
        pub const MinimumPeriod: u64 = 5;
    }

    impl pallet_timestamp::Trait for TestRuntime {
        type Moment = u64;
        type OnTimestampSet = ();
        type MinimumPeriod = MinimumPeriod;
        type WeightInfo = ();
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
        type WeightInfo = ();
        type MaxLocks = ();
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
        type AllowTransfers = EnsureRoot<AccountId>;
    }

    use pallet_permissions::default_permissions::DefaultSpacePermissions;

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
        type IsPostBlocked = Moderation;
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

    const HANDLE_DEPOSIT: u64 = 0;

    impl pallet_spaces::Trait for TestRuntime {
        type Event = ();
        type Currency = Balances;
        type Roles = Roles;
        type SpaceFollows = SpaceFollows;
        type BeforeSpaceCreated = SpaceFollows;
        type AfterSpaceUpdated = SpaceHistory;
        type IsAccountBlocked = Moderation;
        type IsContentBlocked = Moderation;
        type HandleDeposit = ();
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
        fn configure_storages(storage: &mut Storage) {
            let mut accounts = Vec::new();
            for account in ACCOUNT1..=ACCOUNT3 {
                accounts.push(account);
            }

            let _ = pallet_balances::GenesisConfig::<TestRuntime> {
                balances: accounts.iter().cloned().map(|k|(k, 100)).collect()
            }.assimilate_storage(storage);
        }

        /// Default ext configuration with BlockNumber 1
        pub fn build() -> TestExternalities {
            let mut storage = system::GenesisConfig::default()
                .build_storage::<TestRuntime>()
                .unwrap();

            Self::configure_storages(&mut storage);

            let mut ext = TestExternalities::from(storage);
            ext.execute_with(|| System::set_block_number(1));

            ext
        }

        fn add_default_space() {
            assert_ok!(_create_default_space());
        }

        fn add_space_with_no_handle() {
            assert_ok!(_create_space(None, Some(None), None, None));
        }

        fn add_post() {
            Self::add_default_space();
            assert_ok!(_create_default_post());
        }

        fn add_comment() {
            Self::add_post();
            assert_ok!(_create_default_comment());
        }

        /// Custom ext configuration with SpaceId 1 and BlockNumber 1
        pub fn build_with_space() -> TestExternalities {
            let mut ext = Self::build();
            ext.execute_with(|| Self::add_default_space());
            ext
        }

        /// Custom ext configuration with SpaceId 1, PostId 1 and BlockNumber 1
        pub fn build_with_post() -> TestExternalities {
            let mut ext = Self::build();
            ext.execute_with(|| Self::add_post());
            ext
        }

        /// Custom ext configuration with SpaceId 1, PostId 1, PostId 2 (as comment) and BlockNumber 1
        pub fn build_with_comment() -> TestExternalities {
            let mut ext = Self::build();
            ext.execute_with(|| Self::add_comment());
            ext
        }

        /// Custom ext configuration with SpaceId 1-2, PostId 1 where BlockNumber 1
        pub fn build_with_post_and_two_spaces() -> TestExternalities {
            let mut ext = Self::build_with_post();
            ext.execute_with(|| Self::add_space_with_no_handle());
            ext
        }

        /// Custom ext configuration with SpaceId 1, PostId 1 and ReactionId 1 (on post) where BlockNumber is 1
        pub fn build_with_reacted_post_and_two_spaces() -> TestExternalities {
            let mut ext = Self::build_with_post_and_two_spaces();
            ext.execute_with(|| { assert_ok!(_create_default_post_reaction()); });
            ext
        }

        /// Custom ext configuration with pending ownership transfer without Space
        pub fn build_with_pending_ownership_transfer_no_space() -> TestExternalities {
            let mut ext = Self::build_with_space();
            ext.execute_with(|| {
                assert_ok!(_transfer_default_space_ownership());
                <SpaceById<TestRuntime>>::remove(SPACE1);
            });
            ext
        }

        /// Custom ext configuration with specified permissions granted (includes SpaceId 1)
        pub fn build_with_a_few_roles_granted_to_account2(perms: Vec<SP>) -> TestExternalities {
            let mut ext = Self::build_with_space();

            ext.execute_with(|| {
                let user = User::Account(ACCOUNT2);
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
            let mut ext = Self::build_with_space();

            ext.execute_with(|| {
                assert_ok!(_default_follow_space());
                <SpaceById<TestRuntime>>::remove(SPACE1);
            });

            ext
        }
    }

    /* Integration tests mocks */

    const ACCOUNT1: AccountId = 1;
    const ACCOUNT2: AccountId = 2;
    const ACCOUNT3: AccountId = 3;

    const SPACE1: SpaceId = 1001;
    const SPACE2: SpaceId = 1002;

    const POST1: PostId = 1;
    const POST2: PostId = 2;
    const POST3: PostId = 3;

    const REACTION1: ReactionId = 1;
    const REACTION2: ReactionId = 2;

    /// Lowercase a handle and then try to find a space id by it.
    fn find_space_id_by_handle(handle: Vec<u8>) -> Option<SpaceId> {
        let lc_handle = Utils::<TestRuntime>::lowercase_handle(handle);
        Spaces::space_id_by_handle(lc_handle)
    }

    fn space_handle() -> Vec<u8> {
        b"Space_Handle".to_vec()
    }

    fn space_handle_2() -> Vec<u8> {
        b"space_handle_2".to_vec()
    }

    fn space_content_ipfs() -> Content {
        Content::IPFS(b"bafyreib3mgbou4xln42qqcgj6qlt3cif35x4ribisxgq7unhpun525l54e".to_vec())
    }

    fn updated_space_content() -> Content {
        Content::IPFS(b"QmRAQB6YaCyidP37UdDnjFY5vQuiBrcqdyoW2CuDgwxkD4".to_vec())
    }

    fn update_for_space_handle(
        new_handle: Option<Vec<u8>>,
    ) -> SpaceUpdate {
        space_update(Some(new_handle), None, None)
    }

    fn space_update(
        handle: Option<Option<Vec<u8>>>,
        content: Option<Content>,
        hidden: Option<bool>,
    ) -> SpaceUpdate {
        SpaceUpdate {
            parent_id: None,
            handle,
            content,
            hidden,
            permissions: None,
        }
    }

    fn post_content_ipfs() -> Content {
        Content::IPFS(b"bafyreidzue2dtxpj6n4x5mktrt7las5wz5diqma47zr25uau743dhe76we".to_vec())
    }

    fn updated_post_content() -> Content {
        Content::IPFS(b"bafyreifw4omlqpr3nqm32bueugbodkrdne7owlkxgg7ul2qkvgrnkt3g3u".to_vec())
    }

    fn post_update(
        space_id: Option<SpaceId>,
        content: Option<Content>,
        hidden: Option<bool>,
    ) -> PostUpdate {
        PostUpdate {
            space_id,
            content,
            hidden,
        }
    }

    fn comment_content_ipfs() -> Content {
        Content::IPFS(b"bafyreib6ceowavccze22h2x4yuwagsnym2c66gs55mzbupfn73kd6we7eu".to_vec())
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

    fn _create_default_space() -> DispatchResult {
        _create_space(None, None, None, None)
    }

    fn _create_space(
        origin: Option<Origin>,
        handle: Option<Option<Vec<u8>>>,
        content: Option<Content>,
        permissions: Option<Option<SpacePermissions>>
    ) -> DispatchResult {
        _create_space_with_parent_id(
            origin,
            None,
            handle,
            content,
            permissions,
        )
    }

    fn _create_subspace(
        origin: Option<Origin>,
        parent_id_opt: Option<Option<SpaceId>>,
        handle: Option<Option<Vec<u8>>>,
        content: Option<Content>,
        permissions: Option<Option<SpacePermissions>>
    ) -> DispatchResult {
        _create_space_with_parent_id(
            origin,
            parent_id_opt,
            handle,
            content,
            permissions,
        )
    }

    fn _create_space_with_parent_id(
        origin: Option<Origin>,
        parent_id_opt: Option<Option<SpaceId>>,
        handle: Option<Option<Vec<u8>>>,
        content: Option<Content>,
        permissions: Option<Option<SpacePermissions>>
    ) -> DispatchResult {
        Spaces::create_space(
            origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
            parent_id_opt.unwrap_or(None),
            handle.unwrap_or_else(|| Some(space_handle())),
            content.unwrap_or_else(space_content_ipfs),
            permissions.unwrap_or(None)
        )
    }

    fn _update_space(
        origin: Option<Origin>,
        space_id: Option<SpaceId>,
        update: Option<SpaceUpdate>,
    ) -> DispatchResult {
        Spaces::update_space(
            origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
            space_id.unwrap_or(SPACE1),
            update.unwrap_or_else(|| space_update(None, None, None)),
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
        content: Option<Content>,
    ) -> DispatchResult {
        Posts::create_post(
            origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
            space_id_opt.unwrap_or(Some(SPACE1)),
            extension.unwrap_or_else(extension_regular_post),
            content.unwrap_or_else(post_content_ipfs),
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
            update.unwrap_or_else(|| post_update(None, None, None)),
        )
    }

    fn _move_post_1_to_space_2() -> DispatchResult {
        _move_post(None, None, None)
    }

    /// Move the post out of this space to nowhere (space = None).
    fn _move_post_to_nowhere(post_id: PostId) -> DispatchResult {
        _move_post(None, Some(post_id), Some(None))
    }

    fn _move_post(
        origin: Option<Origin>,
        post_id: Option<PostId>,
        new_space_id: Option<Option<SpaceId>>,
    ) -> DispatchResult {
        Posts::move_post(
            origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
            post_id.unwrap_or(POST1),
            new_space_id.unwrap_or(Some(SPACE2)),
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
            Some(extension_comment(
                parent_id.unwrap_or(None),
                post_id.unwrap_or(POST1),
            )),
            Some(content.unwrap_or_else(comment_content_ipfs)),
        )
    }

    fn _update_comment(
        origin: Option<Origin>,
        post_id: Option<PostId>,
        update: Option<PostUpdate>,
    ) -> DispatchResult {
        _update_post(
            origin,
            Some(post_id.unwrap_or(POST2)),
            Some(update.unwrap_or_else(||
                post_update(None, Some(reply_content_ipfs()), None))
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
        kind: Option<ReactionKind>,
    ) -> DispatchResult {
        Reactions::create_post_reaction(
            origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
            post_id.unwrap_or(POST1),
            kind.unwrap_or_else(reaction_upvote),
        )
    }

    fn _create_comment_reaction(
        origin: Option<Origin>,
        post_id: Option<PostId>,
        kind: Option<ReactionKind>,
    ) -> DispatchResult {
        _create_post_reaction(origin, Some(post_id.unwrap_or(2)), kind)
    }

    fn _update_post_reaction(
        origin: Option<Origin>,
        post_id: Option<PostId>,
        reaction_id: ReactionId,
        kind: Option<ReactionKind>,
    ) -> DispatchResult {
        Reactions::update_post_reaction(
            origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
            post_id.unwrap_or(POST1),
            reaction_id,
            kind.unwrap_or_else(reaction_upvote),
        )
    }

    fn _update_comment_reaction(
        origin: Option<Origin>,
        post_id: Option<PostId>,
        reaction_id: ReactionId,
        kind: Option<ReactionKind>,
    ) -> DispatchResult {
        _update_post_reaction(origin, Some(post_id.unwrap_or(2)), reaction_id, kind)
    }

    fn _delete_post_reaction(
        origin: Option<Origin>,
        post_id: Option<PostId>,
        reaction_id: ReactionId,
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
        reaction_id: ReactionId,
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
            content.unwrap_or_else(profile_content_ipfs),
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
        kind: ReactionKind,
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
        kind: ReactionKind,
    ) -> DispatchResult {
        Scores::score_post_on_reaction(account, post, kind)
    }

    fn _transfer_default_space_ownership() -> DispatchResult {
        _transfer_space_ownership(None, None, None)
    }

    fn _transfer_space_ownership(
        origin: Option<Origin>,
        space_id: Option<SpaceId>,
        transfer_to: Option<AccountId>,
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
            content.unwrap_or_else(default_role_content_ipfs),
            permissions.unwrap_or_else(permission_set_default),
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
    // Moderation pallet mocks
    // FIXME: remove when linter error is fixed
    #[allow(dead_code)]
    const REPORT1: ReportId = 1;

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
            origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
            entity.unwrap_or(EntityId::Post(POST1)),
            scope.unwrap_or(SPACE1),
            reason.unwrap_or_else(|| valid_content_ipfs()),
        )
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

    /*------------------------------------------------------------------------------------------------*/
    // Moderation tests

    fn block_account_in_space_1() {
        assert_ok!(
            _update_entity_status(
                None,
                Some(EntityId::Account(ACCOUNT1)),
                Some(SPACE1),
                Some(Some(EntityStatus::Blocked))
            )
        );
    }

    fn block_content_in_space_1() {
        assert_ok!(
            _update_entity_status(
                None,
                Some(EntityId::Content(valid_content_ipfs())),
                Some(SPACE1),
                Some(Some(EntityStatus::Blocked))
            )
        );
    }

    #[test]
    fn create_subspace_should_fail_when_content_is_blocked() {
        ExtBuilder::build_with_post().execute_with(|| {
            block_content_in_space_1();
            assert_noop!(
                _create_subspace(
                    None,
                    Some(Some(SPACE1)),
                    None,
                    Some(valid_content_ipfs()),
                    None,
                ), UtilsError::<TestRuntime>::ContentIsBlocked
            );
        });
    }

    #[test]
    fn create_subspace_should_fail_when_account_is_blocked() {
        ExtBuilder::build_with_post().execute_with(|| {
            block_account_in_space_1();
            assert_noop!(
                _create_subspace(
                    None,
                    Some(Some(SPACE1)),
                    Some(Some(space_handle_2())),
                    None,
                    None,
                ), UtilsError::<TestRuntime>::AccountIsBlocked
            );
        });
    }

    #[test]
    fn update_space_should_fail_when_account_is_blocked() {
        ExtBuilder::build_with_post().execute_with(|| {
            block_account_in_space_1();
            assert_noop!(
                _update_space(
                    None,
                    None,
                    Some(update_for_space_handle(Some(space_handle_2())))
                ), UtilsError::<TestRuntime>::AccountIsBlocked
            );
        });
    }

    #[test]
    fn update_space_should_fail_when_content_is_blocked() {
        ExtBuilder::build_with_post().execute_with(|| {
            block_content_in_space_1();
            assert_noop!(
                _update_space(
                    None,
                    None,
                    Some(space_update(
                        None,
                        Some(valid_content_ipfs()),
                        None
                    ))
                ),
                UtilsError::<TestRuntime>::ContentIsBlocked
            );
        });
    }

    #[test]
    fn create_post_should_fail_when_content_is_blocked() {
        ExtBuilder::build_with_post().execute_with(|| {
            block_content_in_space_1();
            assert_noop!(
                _create_post(
                    None,
                    None,
                    None,
                    Some(valid_content_ipfs()),
                ), UtilsError::<TestRuntime>::ContentIsBlocked
            );
        });
    }

    #[test]
    fn create_post_should_fail_when_account_is_blocked() {
        ExtBuilder::build_with_post().execute_with(|| {
            block_account_in_space_1();
            assert_noop!(
                _create_post(
                    None,
                    None,
                    None,
                    Some(valid_content_ipfs()),
                ), UtilsError::<TestRuntime>::AccountIsBlocked
            );
        });
    }

    #[test]
    fn update_post_should_fail_when_content_is_blocked() {
        ExtBuilder::build_with_post().execute_with(|| {
            block_content_in_space_1();
            assert_noop!(
                _update_post(
                    None, // From ACCOUNT1 (has default permission to UpdateOwnPosts)
                    None,
                    Some(
                        post_update(
                            None,
                            Some(valid_content_ipfs()),
                            Some(true)
                        )
                    )
                ), UtilsError::<TestRuntime>::ContentIsBlocked
            );
        });
    }

    #[test]
    fn update_post_should_fail_when_account_is_blocked() {
        ExtBuilder::build_with_post().execute_with(|| {
            block_account_in_space_1();
            assert_noop!(
                _update_post(
                    None, // From ACCOUNT1 (has default permission to UpdateOwnPosts)
                    None,
                    Some(
                        post_update(
                            None,
                            Some(valid_content_ipfs()),
                            Some(true)
                        )
                    )
                ), UtilsError::<TestRuntime>::AccountIsBlocked
            );
        });
    }

    // FIXME: uncomment when `update_post` will be able to move post from one space to another
    /*
    #[test]
    fn update_post_should_fail_when_post_is_blocked() {
        ExtBuilder::build_with_post().execute_with(|| {
            assert_ok!(
                _update_entity_status(
                    None,
                    Some(EntityId::Post(POST1)),
                    Some(SPACE1),
                    Some(Some(EntityStatus::Blocked))
                )
            );
            assert_noop!(
                _update_post(
                    None, // From ACCOUNT1 (has default permission to UpdateOwnPosts)
                    Some(POST1),
                    Some(
                        post_update(
                            Some(SPACE1),
                            None,
                            None
                        )
                    )
                ), UtilsError::<TestRuntime>::PostIsBlocked
            );
        });
    }
    */

    /*---------------------------------------------------------------------------------------------------*/
    // Space tests

    #[test]
    fn create_space_should_work() {
        ExtBuilder::build().execute_with(|| {
            assert_ok!(_create_default_space()); // SpaceId 1

            // Check storages
            assert_eq!(Spaces::space_ids_by_owner(ACCOUNT1), vec![SPACE1]);
            assert_eq!(find_space_id_by_handle(space_handle()), Some(SPACE1));
            assert_eq!(Spaces::next_space_id(), SPACE2);

            // Check whether data stored correctly
            let space = Spaces::space_by_id(SPACE1).unwrap();

            assert_eq!(space.created.account, ACCOUNT1);
            assert!(space.updated.is_none());
            assert_eq!(space.hidden, false);

            assert_eq!(space.owner, ACCOUNT1);
            assert_eq!(space.handle, Some(space_handle()));
            assert_eq!(space.content, space_content_ipfs());

            assert_eq!(space.posts_count, 0);
            assert_eq!(space.followers_count, 1);
            assert!(SpaceHistory::edit_history(space.id).is_empty());
            assert_eq!(space.score, 0);

            // Check that the handle deposit has been reserved:
            let reserved_balance = Balances::reserved_balance(ACCOUNT1);
            assert_eq!(reserved_balance, HANDLE_DEPOSIT);
        });
    }

    #[test]
    fn create_space_should_store_handle_lowercase() {
        ExtBuilder::build().execute_with(|| {
            let new_handle: Vec<u8> = b"sPaCe_hAnDlE".to_vec();

            assert_ok!(_create_space(None, Some(Some(new_handle.clone())), None, None)); // SpaceId 1

            // Handle should be lowercase in storage and original in struct
            let space = Spaces::space_by_id(SPACE1).unwrap();
            assert_eq!(space.handle, Some(new_handle.clone()));
            assert_eq!(find_space_id_by_handle(new_handle), Some(SPACE1));
        });
    }

    #[test]
    fn create_space_should_fail_when_too_short_handle_provided() {
        ExtBuilder::build().execute_with(|| {
            let short_handle: Vec<u8> = vec![65; (MinHandleLen::get() - 1) as usize];

            // Try to catch an error creating a space with too short handle
            assert_noop!(_create_space(
                None,
                Some(Some(short_handle)),
                None,
                None
            ), UtilsError::<TestRuntime>::HandleIsTooShort);
        });
    }

    #[test]
    fn create_space_should_fail_when_too_long_handle_provided() {
        ExtBuilder::build().execute_with(|| {
            let long_handle: Vec<u8> = vec![65; (MaxHandleLen::get() + 1) as usize];

            // Try to catch an error creating a space with too long handle
            assert_noop!(_create_space(
                None,
                Some(Some(long_handle)),
                None,
                None
            ), UtilsError::<TestRuntime>::HandleIsTooLong);
        });
    }

    #[test]
    fn create_space_should_fail_when_not_unique_handle_provided() {
        ExtBuilder::build().execute_with(|| {
            assert_ok!(_create_default_space());
            // SpaceId 1
            // Try to catch an error creating a space with not unique handle
            assert_noop!(_create_default_space(), SpacesError::<TestRuntime>::SpaceHandleIsNotUnique);
        });
    }

    #[test]
    fn create_space_should_fail_when_handle_contains_at_char() {
        ExtBuilder::build().execute_with(|| {
            let invalid_handle: Vec<u8> = b"@space_handle".to_vec();

            assert_noop!(_create_space(
                None,
                Some(Some(invalid_handle)),
                None,
                None
            ), UtilsError::<TestRuntime>::HandleContainsInvalidChars);
        });
    }

    #[test]
    fn create_space_should_fail_when_handle_contains_minus_char() {
        ExtBuilder::build().execute_with(|| {
            let invalid_handle: Vec<u8> = b"space-handle".to_vec();

            assert_noop!(_create_space(
                None,
                Some(Some(invalid_handle)),
                None,
                None
            ), UtilsError::<TestRuntime>::HandleContainsInvalidChars);
        });
    }

    #[test]
    fn create_space_should_fail_when_handle_contains_space_char() {
        ExtBuilder::build().execute_with(|| {
            let invalid_handle: Vec<u8> = b"space handle".to_vec();

            assert_noop!(_create_space(
                None,
                Some(Some(invalid_handle)),
                None,
                None
            ), UtilsError::<TestRuntime>::HandleContainsInvalidChars);
        });
    }

    #[test]
    fn create_space_should_fail_when_handle_contains_unicode() {
        ExtBuilder::build().execute_with(|| {
            let invalid_handle: Vec<u8> = String::from("блог_хендл").into_bytes().to_vec();

            assert_noop!(_create_space(
                None,
                Some(Some(invalid_handle)),
                None,
                None
            ), UtilsError::<TestRuntime>::HandleContainsInvalidChars);
        });
    }

    #[test]
    fn create_space_should_fail_when_ipfs_cid_is_invalid() {
        ExtBuilder::build().execute_with(|| {
            // Try to catch an error creating a space with invalid content
            assert_noop!(_create_space(
                None,
                None,
                Some(invalid_content_ipfs()),
                None
            ), UtilsError::<TestRuntime>::InvalidIpfsCid);
        });
    }

    #[test]
    fn update_space_should_work() {
        ExtBuilder::build_with_space().execute_with(|| {
            let new_handle: Vec<u8> = b"new_handle".to_vec();
            let expected_content_ipfs = updated_space_content();
            // Space update with ID 1 should be fine

            assert_ok!(_update_space(
                None, // From ACCOUNT1 (has permission as he's an owner)
                None,
                Some(
                    space_update(
                        Some(Some(new_handle.clone())),
                        Some(expected_content_ipfs.clone()),
                        Some(true),
                    )
                )
            ));

            // Check whether space updates correctly
            let space = Spaces::space_by_id(SPACE1).unwrap();
            assert_eq!(space.handle, Some(new_handle.clone()));
            assert_eq!(space.content, expected_content_ipfs);
            assert_eq!(space.hidden, true);

            // Check whether history recorded correctly
            let edit_history = &SpaceHistory::edit_history(space.id)[0];
            assert_eq!(edit_history.old_data.handle, Some(Some(space_handle())));
            assert_eq!(edit_history.old_data.content, Some(space_content_ipfs()));
            assert_eq!(edit_history.old_data.hidden, Some(false));

            assert_eq!(find_space_id_by_handle(space_handle()), None);
            assert_eq!(find_space_id_by_handle(new_handle), Some(SPACE1));

            // Check that the handle deposit has been reserved:
            let reserved_balance = Balances::reserved_balance(ACCOUNT1);
            assert_eq!(reserved_balance, HANDLE_DEPOSIT);
        });
    }

    #[test]
    fn update_space_should_work_when_one_of_roles_is_permitted() {
        ExtBuilder::build_with_a_few_roles_granted_to_account2(vec![SP::UpdateSpace]).execute_with(|| {
            let space_update = space_update(
                Some(Some(b"new_handle".to_vec())),
                Some(updated_space_content()),
                Some(true),
            );

            assert_ok!(_update_space(
                Some(Origin::signed(ACCOUNT2)),
                Some(SPACE1),
                Some(space_update)
            ));
        });
    }

    #[test]
    fn update_space_should_work_when_unreserving_handle() {
        ExtBuilder::build_with_space().execute_with(|| {
            let no_handle = None;
            let space_update = update_for_space_handle(no_handle);
            assert_ok!(_update_space(None, None, Some(space_update)));

            // Check that the space handle is unreserved after this update:
            let space = Spaces::space_by_id(SPACE1).unwrap();
            assert_eq!(space.handle, None);

            // Check that the previous space handle has been added to the space history:
            let edit_history = &SpaceHistory::edit_history(space.id)[0];
            assert_eq!(edit_history.old_data.handle, Some(Some(space_handle())));

            // Check that the previous space handle is not reserved in storage anymore:
            assert_eq!(find_space_id_by_handle(space_handle()), None);

            // Check that the handle deposit has been unreserved:
            let reserved_balance = Balances::reserved_balance(ACCOUNT1);
            assert!(reserved_balance.is_zero());
        });
    }

    #[test]
    fn update_space_should_fail_when_no_updates_for_space_provided() {
        ExtBuilder::build_with_space().execute_with(|| {
            // Try to catch an error updating a space with no changes
            assert_noop!(
                _update_space(None, None, None),
                SpacesError::<TestRuntime>::NoUpdatesForSpace
            );
        });
    }

    #[test]
    fn update_space_should_fail_when_space_not_found() {
        ExtBuilder::build_with_space().execute_with(|| {
            let new_handle: Vec<u8> = b"new_handle".to_vec();

            // Try to catch an error updating a space with wrong space ID
            assert_noop!(_update_space(
                None,
                Some(SPACE2),
                Some(
                    update_for_space_handle(Some(new_handle))
                )
            ), SpacesError::<TestRuntime>::SpaceNotFound);
        });
    }

    #[test]
    fn update_space_should_fail_when_account_has_no_permission_to_update_space() {
        ExtBuilder::build_with_space().execute_with(|| {
            let new_handle: Vec<u8> = b"new_handle".to_vec();

            // Try to catch an error updating a space with an account that it not permitted
            assert_noop!(_update_space(
                Some(Origin::signed(ACCOUNT2)),
                None,
                Some(
                    update_for_space_handle(Some(new_handle))
                )
            ), SpacesError::<TestRuntime>::NoPermissionToUpdateSpace);
        });
    }

    #[test]
    fn update_space_should_fail_when_too_short_handle_provided() {
        ExtBuilder::build_with_space().execute_with(|| {
            let short_handle: Vec<u8> = vec![65; (MinHandleLen::get() - 1) as usize];

            // Try to catch an error updating a space with too short handle
            assert_noop!(_update_space(
                None,
                None,
                Some(
                    update_for_space_handle(Some(short_handle))
                )
            ), UtilsError::<TestRuntime>::HandleIsTooShort);
        });
    }

    #[test]
    fn update_space_should_fail_when_too_long_handle_provided() {
        ExtBuilder::build_with_space().execute_with(|| {
            let long_handle: Vec<u8> = vec![65; (MaxHandleLen::get() + 1) as usize];

            // Try to catch an error updating a space with too long handle
            assert_noop!(_update_space(
                None,
                None,
                Some(
                    update_for_space_handle(Some(long_handle))
                )
            ), UtilsError::<TestRuntime>::HandleIsTooLong);
        });
    }

    #[test]
    fn update_space_should_fail_when_not_unique_handle_provided() {
        ExtBuilder::build_with_space().execute_with(|| {
            let handle: Vec<u8> = b"unique_handle".to_vec();

            assert_ok!(_create_space(
                None,
                Some(Some(handle.clone())),
                None,
                None
            )); // SpaceId 2 with a custom handle

            // Should fail when updating a space 1 with a handle of a space 2:
            assert_noop!(_update_space(
                None,
                Some(SPACE1),
                Some(
                    update_for_space_handle(Some(handle))
                )
            ), SpacesError::<TestRuntime>::SpaceHandleIsNotUnique);
        });
    }

    #[test]
    fn update_space_should_fail_when_handle_contains_at_char() {
        ExtBuilder::build_with_space().execute_with(|| {
            let invalid_handle: Vec<u8> = b"@space_handle".to_vec();

            assert_noop!(_update_space(
                None,
                None,
                Some(
                    update_for_space_handle(Some(invalid_handle))
                )
            ), UtilsError::<TestRuntime>::HandleContainsInvalidChars);
        });
    }

    #[test]
    fn update_space_should_fail_when_handle_contains_minus_char() {
        ExtBuilder::build_with_space().execute_with(|| {
            let invalid_handle: Vec<u8> = b"space-handle".to_vec();

            assert_noop!(_update_space(
                None,
                None,
                Some(
                    update_for_space_handle(Some(invalid_handle))
                )
            ), UtilsError::<TestRuntime>::HandleContainsInvalidChars);
        });
    }

    #[test]
    fn update_space_should_fail_when_handle_contains_space_char() {
        ExtBuilder::build_with_space().execute_with(|| {
            let invalid_handle: Vec<u8> = b"space handle".to_vec();

            assert_noop!(_update_space(
                None,
                None,
                Some(
                    update_for_space_handle(Some(invalid_handle))
                )
            ), UtilsError::<TestRuntime>::HandleContainsInvalidChars);
        });
    }

    #[test]
    fn update_space_should_fail_when_handle_contains_unicode() {
        ExtBuilder::build_with_space().execute_with(|| {
            let invalid_handle: Vec<u8> = String::from("блог_хендл").into_bytes().to_vec();

            assert_noop!(_update_space(
                None,
                None,
                Some(
                    update_for_space_handle(Some(invalid_handle))
                )
            ), UtilsError::<TestRuntime>::HandleContainsInvalidChars);
        });
    }

    #[test]
    fn update_space_should_fail_when_ipfs_cid_is_invalid() {
        ExtBuilder::build_with_space().execute_with(|| {

            // Try to catch an error updating a space with invalid content
            assert_noop!(_update_space(
                None,
                None,
                Some(
                    space_update(
                        None,
                        Some(invalid_content_ipfs()),
                        None,
                    )
                )
            ), UtilsError::<TestRuntime>::InvalidIpfsCid);
        });
    }

    #[test]
    fn update_space_should_fail_when_no_right_permission_in_account_roles() {
        ExtBuilder::build_with_a_few_roles_granted_to_account2(vec![SP::UpdateSpace]).execute_with(|| {
            let space_update = space_update(
                Some(Some(b"new_handle".to_vec())),
                Some(updated_space_content()),
                Some(true),
            );

            assert_ok!(_delete_default_role());

            assert_noop!(_update_space(
                Some(Origin::signed(ACCOUNT2)),
                Some(SPACE1),
                Some(space_update)
            ), SpacesError::<TestRuntime>::NoPermissionToUpdateSpace);
        });
    }

    // TODO: refactor or remove. Deprecated tests
    // Find public space ids tests
    // --------------------------------------------------------------------------------------------
    /*#[test]
    fn find_public_space_ids_should_work() {
        ExtBuilder::build_with_space().execute_with(|| {
            assert_ok!(_create_space(None, None, Some(Some(space_handle1())), None, None));

            let space_ids = Spaces::find_public_space_ids(0, 3);
            assert_eq!(space_ids, vec![SPACE1, SPACE2]);
        });
    }

    #[test]
    fn find_public_space_ids_should_work_with_zero_offset() {
        ExtBuilder::build_with_space().execute_with(|| {
            let space_ids = Spaces::find_public_space_ids(0, 1);
            assert_eq!(space_ids, vec![SPACE1]);
        });
    }

    #[test]
    fn find_public_space_ids_should_work_with_zero_limit() {
        ExtBuilder::build_with_space().execute_with(|| {
            let space_ids = Spaces::find_public_space_ids(1, 0);
            assert_eq!(space_ids, vec![SPACE1]);
        });
    }

    #[test]
    fn find_public_space_ids_should_work_with_zero_offset_and_zero_limit() {
        ExtBuilder::build_with_space().execute_with(|| {
            let space_ids = Spaces::find_public_space_ids(0, 0);
            assert_eq!(space_ids, vec![]);
        });
    }

    // Find unlisted space ids tests
    // --------------------------------------------------------------------------------------------

    #[test]
    fn find_unlisted_space_ids_should_work() {
        ExtBuilder::build_with_space().execute_with(|| {
            assert_ok!(_create_space(None, None, Some(Some(space_handle1())), None, None));
            assert_ok!(
                _update_space(
                    None,
                    Some(SPACE1),
                    Some(
                        space_update(
                            None,
                            None,
                            Some(Content::None),
                            Some(true),
                            None
                        )
                    )
                )
            );

            assert_ok!(
                _update_space(
                    None,
                    Some(SPACE2),
                    Some(
                        space_update(
                            None,
                            None,
                            Some(Content::None),
                            Some(true),
                            None
                        )
                    )
                )
            );


            let space_ids = Spaces::find_unlisted_space_ids(0, 2);
            assert_eq!(space_ids, vec![SPACE1, SPACE2]);
        });
    }

    #[test]
    fn find_unlisted_space_ids_should_work_with_zero_offset() {
        ExtBuilder::build_with_space().execute_with(|| {
            assert_ok!(
                _update_space(
                    None,
                    Some(SPACE1),
                    Some(
                        space_update(
                            None,
                            None,
                            Some(Content::None),
                            Some(true),
                            None
                        )
                    )
                )
            );

            let space_ids = Spaces::find_unlisted_space_ids(0, 1);
            assert_eq!(space_ids, vec![SPACE1]);
        });
    }

    #[test]
    fn find_unlisted_space_ids_should_work_with_zero_limit() {
        ExtBuilder::build_with_space().execute_with(|| {
            assert_ok!(
                _update_space(
                    None,
                    Some(SPACE1),
                    Some(
                        space_update(
                            None,
                            None,
                            Some(Content::None),
                            Some(true),
                            None
                        )
                    )
                )
            );

            let space_ids = Spaces::find_unlisted_space_ids(1, 0);
            assert_eq!(space_ids, vec![]);
        });
    }

    #[test]
    fn find_unlisted_space_ids_should_work_with_zero_offset_and_zero_limit() {
        ExtBuilder::build_with_space().execute_with(|| {
            assert_ok!(
                _update_space(
                    None,
                    Some(SPACE1),
                    Some(
                        space_update(
                            None,
                            None,
                            Some(Content::None),
                            Some(true),
                            None
                        )
                    )
                )
            );

            let space_ids = Spaces::find_unlisted_space_ids(0, 0);
            assert_eq!(space_ids, vec![]);
        });
    }*/

    // --------------------------------------------------------------------------------------------

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
            assert_eq!(post.extension, extension_regular_post());

            assert_eq!(post.content, post_content_ipfs());

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
    fn create_post_should_work_when_one_of_roles_is_permitted() {
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
    fn create_post_should_fail_when_post_has_no_space_id() {
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
    fn create_post_should_fail_when_space_not_found() {
        ExtBuilder::build().execute_with(|| {
            assert_noop!(_create_default_post(), SpacesError::<TestRuntime>::SpaceNotFound);
        });
    }

    #[test]
    fn create_post_should_fail_when_ipfs_cid_is_invalid() {
        ExtBuilder::build_with_space().execute_with(|| {
            // Try to catch an error creating a regular post with invalid content
            assert_noop!(_create_post(
                None,
                None,
                None,
                Some(invalid_content_ipfs())
            ), UtilsError::<TestRuntime>::InvalidIpfsCid);
        });
    }

    #[test]
    fn create_post_should_fail_when_account_has_no_permission() {
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
    fn create_post_should_fail_when_no_right_permission_in_account_roles() {
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
            let expected_content_ipfs = updated_post_content();

            // Post update with ID 1 should be fine
            assert_ok!(_update_post(
                None, // From ACCOUNT1 (has default permission to UpdateOwnPosts)
                None,
                Some(
                    post_update(
                        None,
                        Some(expected_content_ipfs.clone()),
                        Some(true)
                    )
                )
            ));

            // Check whether post updates correctly
            let post = Posts::post_by_id(POST1).unwrap();
            assert_eq!(post.space_id, Some(SPACE1));
            assert_eq!(post.content, expected_content_ipfs);
            assert_eq!(post.hidden, true);

            // Check whether history recorded correctly
            let post_history = PostHistory::edit_history(POST1)[0].clone();
            assert!(post_history.old_data.space_id.is_none());
            assert_eq!(post_history.old_data.content, Some(post_content_ipfs()));
            assert_eq!(post_history.old_data.hidden, Some(false));
        });
    }

    fn check_if_post_moved_correctly(
        moved_post_id: PostId,
        old_space_id: SpaceId,
        expected_new_space_id: SpaceId
    ) {
        let post: Post<TestRuntime> = Posts::post_by_id(moved_post_id).unwrap(); // `POST2` is a comment
        let new_space_id = post.space_id.unwrap();

        // Check that space id of the post has been updated from 1 to 2
        assert_eq!(new_space_id, expected_new_space_id);

        // Check that stats on the old space have been decreased
        let old_space = Spaces::space_by_id(old_space_id).unwrap();
        assert_eq!(old_space.posts_count, 0);
        assert_eq!(old_space.hidden_posts_count, 0);
        assert_eq!(old_space.score, 0);

        // Check that stats on the new space have been increased
        let new_space = Spaces::space_by_id(new_space_id).unwrap();
        assert_eq!(new_space.posts_count, 1);
        assert_eq!(new_space.hidden_posts_count, if post.hidden { 1 } else { 0 });
        assert_eq!(new_space.score, post.score);
    }

    #[test]
    fn move_post_should_work() {
        ExtBuilder::build_with_reacted_post_and_two_spaces().execute_with(|| {
            assert_ok!(_move_post_1_to_space_2());

            let moved_post_id = POST1;
            let old_space_id = SPACE1;
            let expected_new_space_id = SPACE2;
            check_if_post_moved_correctly(moved_post_id, old_space_id, expected_new_space_id);

            // Check that there are no posts ids in the old space
            assert!(Posts::post_ids_by_space_id(old_space_id).is_empty());

            // Check that there is the post id in the new space
            assert_eq!(Posts::post_ids_by_space_id(expected_new_space_id), vec![moved_post_id]);
        });
    }

    #[test]
    fn move_post_should_work_when_space_id_none() {
        ExtBuilder::build_with_reacted_post_and_two_spaces().execute_with(|| {
            let moved_post_id = POST1;
            let old_space_id = SPACE1; // Where post were before moving to `SpaceId:None`
            let expected_new_space_id = SPACE2;

            assert_ok!(_move_post_to_nowhere(moved_post_id));
            assert_ok!(_move_post_1_to_space_2());

            check_if_post_moved_correctly(moved_post_id, old_space_id, expected_new_space_id);

            // Check that there are no posts ids in the old space
            assert!(Posts::post_ids_by_space_id(old_space_id).is_empty());

            // Check that there is the post id in the new space
            assert_eq!(Posts::post_ids_by_space_id(expected_new_space_id), vec![moved_post_id]);
        });
    }

    #[test]
    fn move_hidden_post_should_work() {
        ExtBuilder::build_with_reacted_post_and_two_spaces().execute_with(|| {
            let moved_post_id = POST1;
            let old_space_id = SPACE1;
            let expected_new_space_id = SPACE2;

            // Hide the post before moving it
            assert_ok!(_update_post(
                None,
                Some(moved_post_id),
                Some(post_update(
                    None,
                    None,
                    Some(true)
                ))
            ));

            assert_ok!(_move_post_1_to_space_2());

            check_if_post_moved_correctly(moved_post_id, old_space_id, expected_new_space_id);

            // Check that there are no posts ids in the old space
            assert!(Posts::post_ids_by_space_id(old_space_id).is_empty());

            // Check that there is the post id in the new space
            assert_eq!(Posts::post_ids_by_space_id(expected_new_space_id), vec![moved_post_id]);
        });
    }

    #[test]
    fn move_hidden_post_should_fail_when_post_not_found() {
        ExtBuilder::build().execute_with(|| {
            // Note that we have not created a post that we are trying to move
            assert_noop!(
                _move_post_1_to_space_2(),
                PostsError::<TestRuntime>::PostNotFound
            );
        });
    }

    #[test]
    fn move_hidden_post_should_fail_when_provided_space_not_found() {
        ExtBuilder::build_with_post().execute_with(|| {
            // Note that we have not created a new space #2 before moving the post
            assert_noop!(
                _move_post_1_to_space_2(),
                SpacesError::<TestRuntime>::SpaceNotFound
            );
        });
    }

    #[test]
    fn move_hidden_post_should_fail_origin_has_no_permission_to_create_posts() {
        ExtBuilder::build_with_post().execute_with(|| {
            // Create a space #2 from account #2
            assert_ok!(_create_space(Some(Origin::signed(ACCOUNT2)), Some(None), None, None));

            // Should not be possible to move the post b/c it's owner is account #1
            // when the space #2 is owned by account #2
            assert_noop!(
                _move_post_1_to_space_2(),
                PostsError::<TestRuntime>::NoPermissionToCreatePosts
            );
        });
    }

    #[test]
    fn move_post_should_fail_when_account_has_no_permission() {
        ExtBuilder::build_with_post_and_two_spaces().execute_with(|| {
            assert_noop!(
                _move_post(Some(Origin::signed(ACCOUNT2)), None, None),
                PostsError::<TestRuntime>::NoPermissionToUpdateAnyPost
            );
        });
    }

    #[test]
    fn move_post_should_fail_when_space_none_and_account_is_not_post_owner() {
        ExtBuilder::build_with_post_and_two_spaces().execute_with(|| {
            assert_ok!(_move_post_to_nowhere(POST1));
            assert_noop!(
                _move_post(Some(Origin::signed(ACCOUNT2)), None, None),
                PostsError::<TestRuntime>::NotAPostOwner
            );
        });
    }

    #[test]
    fn should_fail_when_trying_to_move_comment() {
        ExtBuilder::build_with_comment().execute_with(|| {
            assert_ok!(_create_space(None, Some(None), None, None));

            // Comments cannot be moved, they stick to their parent post
            assert_noop!(
                _move_post(None, Some(POST2), None),
                PostsError::<TestRuntime>::CannotUpdateSpaceIdOnComment
            );
        });
    }

    #[test]
    fn update_post_should_work_after_transfer_space_ownership() {
        ExtBuilder::build_with_post().execute_with(|| {
            let post_update = post_update(
                None,
                Some(updated_post_content()),
                Some(true),
            );

            assert_ok!(_transfer_default_space_ownership());

            // Post update with ID 1 should be fine
            assert_ok!(_update_post(None, None, Some(post_update)));
        });
    }

    #[test]
    fn update_any_post_should_work_when_account_has_default_permission() {
        ExtBuilder::build_with_a_few_roles_granted_to_account2(vec![SP::CreatePosts]).execute_with(|| {
            let post_update = post_update(
                None,
                Some(updated_post_content()),
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
    fn update_any_post_should_work_when_one_of_roles_is_permitted() {
        ExtBuilder::build_with_a_few_roles_granted_to_account2(vec![SP::UpdateAnyPost]).execute_with(|| {
            let post_update = post_update(
                None,
                Some(updated_post_content()),
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
    fn update_post_should_fail_when_no_updates_for_post_provided() {
        ExtBuilder::build_with_post().execute_with(|| {
            // Try to catch an error updating a post with no changes
            assert_noop!(_update_post(None, None, None), PostsError::<TestRuntime>::NoUpdatesForPost);
        });
    }

    #[test]
    fn update_post_should_fail_when_post_not_found() {
        ExtBuilder::build_with_post().execute_with(|| {
            assert_ok!(_create_space(None, Some(Some(b"space2_handle".to_vec())), None, None)); // SpaceId 2

            // Try to catch an error updating a post with wrong post ID
            assert_noop!(_update_post(
                None,
                Some(POST2),
                Some(
                    post_update(
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
    fn update_post_should_fail_when_account_has_no_permission_to_update_any_post() {
        ExtBuilder::build_with_post().execute_with(|| {
            assert_ok!(_create_space(None, Some(Some(b"space2_handle".to_vec())), None, None)); // SpaceId 2

            // Try to catch an error updating a post with different account
            assert_noop!(_update_post(
                Some(Origin::signed(ACCOUNT2)),
                None,
                Some(
                    post_update(
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
    fn update_post_should_fail_when_ipfs_cid_is_invalid() {
        ExtBuilder::build_with_post().execute_with(|| {
            // Try to catch an error updating a post with invalid content
            assert_noop!(_update_post(
                None,
                None,
                Some(
                    post_update(
                        None,
                        Some(invalid_content_ipfs()),
                        None
                    )
                )
            ), UtilsError::<TestRuntime>::InvalidIpfsCid);
        });
    }

    #[test]
    fn update_post_should_fail_when_no_right_permission_in_account_roles() {
        ExtBuilder::build_with_a_few_roles_granted_to_account2(vec![SP::UpdateAnyPost]).execute_with(|| {
            let post_update = post_update(
                None,
                Some(updated_post_content()),
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

    // TODO: refactor or remove. Deprecated tests
    // Find public post ids tests
    // --------------------------------------------------------------------------------------------
    /*#[test]
    fn find_public_post_ids_in_space_should_work() {
        ExtBuilder::build_with_post().execute_with(|| {
            assert_ok!(_create_post(None, Some(Some(SPACE1)), None, None));

            let post_ids = Posts::find_public_post_ids_in_space(SPACE1, 0, 3);
            assert_eq!(post_ids, vec![POST1, POST2]);
        });
    }

    #[test]
    fn find_public_post_ids_in_space_should_work_with_zero_offset() {
        ExtBuilder::build_with_post().execute_with(|| {
            let post_ids = Posts::find_public_post_ids_in_space(SPACE1, 0, 1);
            assert_eq!(post_ids, vec![POST1]);
        });
    }

    #[test]
    fn find_public_post_ids_in_space_should_work_with_zero_limit() {
        ExtBuilder::build_with_post().execute_with(|| {
            let post_ids = Posts::find_public_post_ids_in_space(SPACE1, 1, 0);
            assert_eq!(post_ids, vec![POST1]);
        });
    }

    #[test]
    fn find_public_post_ids_in_space_should_work_with_zero_offset_and_zero_limit() {
        ExtBuilder::build_with_post().execute_with(|| {
            let post_ids = Posts::find_public_post_ids_in_space(SPACE1, 0, 0);
            assert_eq!(post_ids, vec![]);
        });
    }

    // Find unlisted post ids tests
    // --------------------------------------------------------------------------------------------

    #[test]
    fn find_unlisted_post_ids_in_space_should_work() {
        ExtBuilder::build_with_post().execute_with(|| {
            assert_ok!(_create_post(None, Some(Some(SPACE1)), None, None));
            assert_ok!(
                _update_post(
                    None,
                    None,
                    Some(
                        post_update(
                            None,
                            Some(Content::None),
                            Some(true))
                    )
                )
            );
            assert_ok!(
                _update_post(
                    None,
                    Some(POST2),
                    Some(
                        post_update(
                            None,
                            Some(Content::None),
                            Some(true))
                    )
                )
            );

            let post_ids = Posts::find_unlisted_post_ids_in_space(SPACE1, 0, 3);
            assert_eq!(post_ids, vec![POST1, POST2]);
        });
    }

    #[test]
    fn find_unlisted_post_ids_in_space_should_work_with_zero_offset() {
        ExtBuilder::build_with_post().execute_with(|| {
            assert_ok!(
                _update_post(
                    None,
                    None,
                    Some(
                        post_update(
                            None,
                            Some(Content::None),
                            Some(true))
                    )
                )
            );

            let post_ids = Posts::find_unlisted_post_ids_in_space(SPACE1, 0, 1);
            assert_eq!(post_ids, vec![POST1]);
        });
    }

    #[test]
    fn find_unlisted_post_ids_in_space_should_work_with_zero_limit() {
        ExtBuilder::build_with_post().execute_with(|| {
            assert_ok!(
                _update_post(
                    None,
                    None,
                    Some(
                        post_update(
                            None,
                            Some(Content::None),
                            Some(true))
                    )
                )
            );

            let post_ids = Posts::find_unlisted_post_ids_in_space(SPACE1, 1, 0);
            assert_eq!(post_ids, vec![POST1]);
        });
    }

    #[test]
    fn find_unlisted_post_ids_in_space_should_work_with_zero_offset_and_zero_limit() {
        ExtBuilder::build_with_post().execute_with(|| {
            assert_ok!(
                _update_post(
                    None,
                    None,
                    Some(
                        post_update(
                            None,
                            Some(Content::None),
                            Some(true))
                    )
                )
            );

            let post_ids = Posts::find_unlisted_post_ids_in_space(SPACE1, 0, 0);
            assert_eq!(post_ids, vec![]);
        });
    }*/
    // --------------------------------------------------------------------------------------------

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
            assert_eq!(comment.content, comment_content_ipfs());
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
    fn create_comment_should_work_when_comment_has_parents() {
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
                let replies_should_be = last_comment_id - comment_id;
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
    fn create_comment_should_fail_when_post_not_found() {
        ExtBuilder::build().execute_with(|| {
            // Try to catch an error creating a comment with wrong post
            assert_noop!(_create_default_comment(), PostsError::<TestRuntime>::PostNotFound);
        });
    }

    #[test]
    fn create_comment_should_fail_when_parent_comment_is_unknown() {
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
    fn create_comment_should_fail_when_ipfs_cid_is_invalid() {
        ExtBuilder::build_with_post().execute_with(|| {
            // Try to catch an error creating a comment with wrong parent
            assert_noop!(_create_comment(
                None,
                None,
                None,
                Some(invalid_content_ipfs())
            ), UtilsError::<TestRuntime>::InvalidIpfsCid);
        });
    }

    #[test]
    fn create_comment_should_fail_when_trying_to_create_in_hidden_space_scope() {
        ExtBuilder::build_with_post().execute_with(|| {
            assert_ok!(_update_space(
                None,
                None,
                Some(space_update(None, None, Some(true)))
            ));

            assert_noop!(_create_default_comment(), PostsError::<TestRuntime>::CannotCreateInHiddenScope);
        });
    }

    #[test]
    fn create_comment_should_fail_when_trying_create_in_hidden_post_scope() {
        ExtBuilder::build_with_post().execute_with(|| {
            assert_ok!(_update_post(
                None,
                None,
                Some(post_update(None, None, Some(true)))
            ));

            assert_noop!(_create_default_comment(), PostsError::<TestRuntime>::CannotCreateInHiddenScope);
        });
    }

    #[test]
    fn create_comment_should_fail_when_max_comment_depth_reached() {
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
            assert_eq!(comment.content, reply_content_ipfs());

            // Check whether history recorded correctly
            assert_eq!(PostHistory::edit_history(POST2)[0].old_data.content, Some(comment_content_ipfs()));
        });
    }

    #[test]
    fn update_comment_hidden_should_work_when_comment_has_parents() {
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
                Some(post_update(
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
    fn update_comment_should_fail_when_post_not_found() {
        ExtBuilder::build().execute_with(|| {
            // Try to catch an error updating a comment with wrong PostId
            assert_noop!(_update_comment(None, None, None), PostsError::<TestRuntime>::PostNotFound);
        });
    }

    #[test]
    fn update_comment_should_fail_when_account_is_not_a_comment_author() {
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
    fn update_comment_should_fail_when_ipfs_cid_is_invalid() {
        ExtBuilder::build_with_comment().execute_with(|| {
            // Try to catch an error updating a comment with invalid content
            assert_noop!(_update_comment(
                None,
                None,
                Some(
                    post_update(
                        None,
                        Some(invalid_content_ipfs()),
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
            assert_eq!(reaction.kind, reaction_upvote());
        });
    }

    #[test]
    fn create_post_reaction_should_work_downvote() {
        ExtBuilder::build_with_post().execute_with(|| {
            assert_ok!(_create_post_reaction(
                Some(Origin::signed(ACCOUNT2)),
                None,
                Some(reaction_downvote())
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
            assert_eq!(reaction.kind, reaction_downvote());
        });
    }

    #[test]
    fn create_post_reaction_should_fail_when_account_has_already_reacted() {
        ExtBuilder::build_with_reacted_post_and_two_spaces().execute_with(|| {
            // Try to catch an error creating reaction by the same account
            assert_noop!(_create_default_post_reaction(), ReactionsError::<TestRuntime>::AccountAlreadyReacted);
        });
    }

    #[test]
    fn create_post_reaction_should_fail_when_post_not_found() {
        ExtBuilder::build().execute_with(|| {
            // Try to catch an error creating reaction by the same account
            assert_noop!(_create_default_post_reaction(), PostsError::<TestRuntime>::PostNotFound);
        });
    }

    #[test]
    fn create_post_reaction_should_fail_when_trying_to_react_in_hidden_space() {
        ExtBuilder::build_with_post().execute_with(|| {

            // Hide the space
            assert_ok!(_update_space(
                None,
                None,
                Some(space_update(None, None, Some(true)))
            ));

            assert_noop!(_create_default_post_reaction(), ReactionsError::<TestRuntime>::CannotReactWhenSpaceHidden);
        });
    }

    #[test]
    fn create_post_reaction_should_fail_when_trying_to_react_on_hidden_post() {
        ExtBuilder::build_with_post().execute_with(|| {

            // Hide the post
            assert_ok!(_update_post(
                None,
                None,
                Some(post_update(None, None, Some(true)))
            ));

            assert_noop!(_create_default_post_reaction(), ReactionsError::<TestRuntime>::CannotReactWhenPostHidden);
        });
    }

// Rating system tests

    #[test]
    fn check_results_of_score_diff_for_action_with_common_values() {
        ExtBuilder::build().execute_with(|| {
            assert_eq!(Scores::score_diff_for_action(1, scoring_action_upvote_post()), UpvotePostActionWeight::get() as i16);
            assert_eq!(Scores::score_diff_for_action(1, scoring_action_downvote_post()), DownvotePostActionWeight::get() as i16);
            assert_eq!(Scores::score_diff_for_action(1, scoring_action_share_post()), SharePostActionWeight::get() as i16);
            assert_eq!(Scores::score_diff_for_action(1, scoring_action_create_comment()), CreateCommentActionWeight::get() as i16);
            assert_eq!(Scores::score_diff_for_action(1, scoring_action_upvote_comment()), UpvoteCommentActionWeight::get() as i16);
            assert_eq!(Scores::score_diff_for_action(1, scoring_action_downvote_comment()), DownvoteCommentActionWeight::get() as i16);
            assert_eq!(Scores::score_diff_for_action(1, scoring_action_share_comment()), ShareCommentActionWeight::get() as i16);
            assert_eq!(Scores::score_diff_for_action(1, scoring_action_follow_space()), FollowSpaceActionWeight::get() as i16);
            assert_eq!(Scores::score_diff_for_action(1, scoring_action_follow_account()), FollowAccountActionWeight::get() as i16);
        });
    }

    #[test]
    fn check_results_of_score_diff_for_action_with_random_values() {
        ExtBuilder::build().execute_with(|| {
            assert_eq!(Scores::score_diff_for_action(32768, scoring_action_upvote_post()), 80); // 2^15
            assert_eq!(Scores::score_diff_for_action(32769, scoring_action_upvote_post()), 80); // 2^15 + 1
            assert_eq!(Scores::score_diff_for_action(65535, scoring_action_upvote_post()), 80); // 2^16 - 1
            assert_eq!(Scores::score_diff_for_action(65536, scoring_action_upvote_post()), 85); // 2^16
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
                Some(reaction_downvote())
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
            assert_eq!(Scores::post_score_by_account((ACCOUNT2, POST1, scoring_action_create_comment())), Some(CreateCommentActionWeight::get()));
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
            assert_eq!(Scores::post_score_by_account((ACCOUNT2, POST1, scoring_action_upvote_post())), Some(UpvotePostActionWeight::get()));
        });
    }

    #[test]
    fn change_post_score_should_work_for_downvote_post() {
        ExtBuilder::build_with_post().execute_with(|| {
            assert_ok!(_create_post_reaction(
                Some(Origin::signed(ACCOUNT2)),
                None,
                Some(reaction_downvote())
            ));

            assert_eq!(Posts::post_by_id(POST1).unwrap().score, DownvotePostActionWeight::get() as i32);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT1).unwrap().reputation, 1);
            assert_eq!(Scores::post_score_by_account((ACCOUNT2, POST1, scoring_action_downvote_post())), Some(DownvotePostActionWeight::get()));
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
            assert!(Scores::post_score_by_account((ACCOUNT2, POST1, scoring_action_upvote_post())).is_none());
        });
    }

    #[test]
    fn change_post_score_should_for_revert_downvote() {
        ExtBuilder::build_with_post().execute_with(|| {
            assert_ok!(_create_post_reaction(
                Some(Origin::signed(ACCOUNT2)),
                None,
                Some(reaction_downvote())
            ));
            // ReactionId 1
            assert_ok!(_delete_post_reaction(
                Some(Origin::signed(ACCOUNT2)),
                None,
                REACTION1
            ));

            assert_eq!(Posts::post_by_id(POST1).unwrap().score, 0);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT1).unwrap().reputation, 1);
            assert!(Scores::post_score_by_account((ACCOUNT2, POST1, scoring_action_downvote_post())).is_none());
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
                Some(reaction_downvote())
            ));

            assert_eq!(Posts::post_by_id(POST1).unwrap().score, DownvotePostActionWeight::get() as i32);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT1).unwrap().reputation, 1);
            assert!(Scores::post_score_by_account((ACCOUNT2, POST1, scoring_action_upvote_post())).is_none());
            assert_eq!(Scores::post_score_by_account((ACCOUNT2, POST1, scoring_action_downvote_post())), Some(DownvotePostActionWeight::get()));
        });
    }

    #[test]
    fn change_post_score_should_work_for_change_downvote_with_upvote() {
        ExtBuilder::build_with_post().execute_with(|| {
            assert_ok!(_create_post_reaction(
                Some(Origin::signed(ACCOUNT2)),
                None,
                Some(reaction_downvote())
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
            assert!(Scores::post_score_by_account((ACCOUNT2, POST1, scoring_action_downvote_post())).is_none());
            assert_eq!(Scores::post_score_by_account((ACCOUNT2, POST1, scoring_action_upvote_post())), Some(UpvotePostActionWeight::get()));
        });
    }

//--------------------------------------------------------------------------------------------------

    #[test]
    fn change_social_account_reputation_should_work_when_max_score_diff_provided() {
        ExtBuilder::build_with_space().execute_with(|| {
            assert_ok!(_create_post(Some(Origin::signed(ACCOUNT1)), None, None, None));
            assert_ok!(Scores::change_social_account_reputation(
                ACCOUNT1,
                ACCOUNT2,
                std::i16::MAX,
                scoring_action_follow_account())
            );
        });
    }

    #[test]
    fn change_social_account_reputation_should_work_when_min_score_diff_provided() {
        ExtBuilder::build_with_space().execute_with(|| {
            assert_ok!(_create_post(Some(Origin::signed(ACCOUNT1)), None, None, None));
            assert_ok!(Scores::change_social_account_reputation(
                ACCOUNT1,
                ACCOUNT2,
                std::i16::MIN,
                scoring_action_follow_account())
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
                scoring_action_downvote_post())
            );
            assert_eq!(Scores::account_reputation_diff_by_account((ACCOUNT2, ACCOUNT1, scoring_action_downvote_post())), Some(0));
            assert_eq!(Profiles::social_account_by_id(ACCOUNT1).unwrap().reputation, 1);

            // To ensure function works correctly, multiply default UpvotePostActionWeight by two
            assert_ok!(Scores::change_social_account_reputation(
                ACCOUNT1,
                ACCOUNT2,
                UpvotePostActionWeight::get() * 2,
                scoring_action_upvote_post())
            );

            assert_eq!(
                Scores::account_reputation_diff_by_account(
                    (
                        ACCOUNT2,
                        ACCOUNT1,
                        scoring_action_upvote_post()
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
                reaction_upvote()
            ));

            assert_eq!(Posts::post_by_id(POST2).unwrap().score, UpvoteCommentActionWeight::get() as i32);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT1).unwrap().reputation, 1 + CreateCommentActionWeight::get() as u32);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT2).unwrap().reputation, 1 + UpvoteCommentActionWeight::get() as u32);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT3).unwrap().reputation, 1);
            assert_eq!(Scores::post_score_by_account((ACCOUNT3, POST2, scoring_action_upvote_comment())), Some(UpvoteCommentActionWeight::get()));
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

            assert_ok!(_score_post_on_reaction_with_id(ACCOUNT3, POST2, reaction_downvote()));

            assert_eq!(Posts::post_by_id(POST2).unwrap().score, DownvoteCommentActionWeight::get() as i32);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT1).unwrap().reputation, 1 + CreateCommentActionWeight::get() as u32);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT2).unwrap().reputation, 1);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT3).unwrap().reputation, 1);
            assert_eq!(Scores::post_score_by_account((ACCOUNT3, POST2, scoring_action_downvote_comment())), Some(DownvoteCommentActionWeight::get()));
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

            assert_ok!(_score_post_on_reaction_with_id(ACCOUNT3, POST2, reaction_upvote()));
            assert_ok!(_score_post_on_reaction_with_id(ACCOUNT3, POST2, reaction_upvote()));

            assert_eq!(Posts::post_by_id(POST2).unwrap().score, 0);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT1).unwrap().reputation, 1 + CreateCommentActionWeight::get() as u32);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT2).unwrap().reputation, 1);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT3).unwrap().reputation, 1);
            assert!(Scores::post_score_by_account((ACCOUNT1, POST2, scoring_action_upvote_comment())).is_none());
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

            assert_ok!(_score_post_on_reaction_with_id(ACCOUNT3, POST2, reaction_downvote()));
            assert_ok!(_score_post_on_reaction_with_id(ACCOUNT3, POST2, reaction_downvote()));

            assert_eq!(Posts::post_by_id(POST2).unwrap().score, 0);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT1).unwrap().reputation, 1 + CreateCommentActionWeight::get() as u32);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT2).unwrap().reputation, 1);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT3).unwrap().reputation, 1);
            assert!(Scores::post_score_by_account((ACCOUNT1, POST2, scoring_action_downvote_comment())).is_none());
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

            assert_ok!(_score_post_on_reaction_with_id(ACCOUNT3, POST2, reaction_upvote()));
            assert_ok!(_score_post_on_reaction_with_id(ACCOUNT3, POST2, reaction_downvote()));

            assert_eq!(Posts::post_by_id(POST2).unwrap().score, DownvoteCommentActionWeight::get() as i32);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT1).unwrap().reputation, 1 + CreateCommentActionWeight::get() as u32);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT2).unwrap().reputation, 1);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT3).unwrap().reputation, 1);
            assert!(Scores::post_score_by_account((ACCOUNT3, POST2, scoring_action_upvote_comment())).is_none());
            assert_eq!(Scores::post_score_by_account((ACCOUNT3, POST2, scoring_action_downvote_comment())), Some(DownvoteCommentActionWeight::get()));
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

            assert_ok!(_score_post_on_reaction_with_id(ACCOUNT3, POST2, reaction_downvote()));
            assert_ok!(_score_post_on_reaction_with_id(ACCOUNT3, POST2, reaction_upvote()));

            assert_eq!(Posts::post_by_id(POST2).unwrap().score, UpvoteCommentActionWeight::get() as i32);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT1).unwrap().reputation, 1 + CreateCommentActionWeight::get() as u32);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT2).unwrap().reputation, 1 + UpvoteCommentActionWeight::get() as u32);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT3).unwrap().reputation, 1);
            assert!(Scores::post_score_by_account((ACCOUNT3, POST2, scoring_action_downvote_comment())).is_none());
            assert_eq!(Scores::post_score_by_account((ACCOUNT3, POST2, scoring_action_upvote_comment())), Some(UpvoteCommentActionWeight::get()));
        });
    }

// Shares tests

    #[test]
    fn share_post_should_work() {
        ExtBuilder::build_with_post().execute_with(|| {
            assert_ok!(_create_space(
                Some(Origin::signed(ACCOUNT2)),
                Some(Some(b"space2_handle".to_vec())),
                None,
                None
            )); // SpaceId 2 by ACCOUNT2

            assert_ok!(_create_post(
                Some(Origin::signed(ACCOUNT2)),
                Some(Some(SPACE2)),
                Some(extension_shared_post(POST1)),
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
            assert_eq!(shared_post.extension, extension_shared_post(POST1));
        });
    }

    #[test]
    fn share_post_should_work_when_one_of_roles_is_permitted() {
        ExtBuilder::build_with_a_few_roles_granted_to_account2(vec![SP::CreatePosts]).execute_with(|| {
            assert_ok!(_create_space(
                None, // From ACCOUNT1
                Some(None), // Provided without any handle
                None, // With default space content,
                None
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
                Some(extension_shared_post(POST1)),
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
                Some(extension_shared_post(POST1)),
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
            assert_eq!(shared_post.extension, extension_shared_post(POST1));
        });
    }

    #[test]
    fn share_post_should_change_score() {
        ExtBuilder::build_with_post().execute_with(|| {
            assert_ok!(_create_space(
                Some(Origin::signed(ACCOUNT2)),
                Some(Some(b"space2_handle".to_vec())),
                None,
                None
            )); // SpaceId 2 by ACCOUNT2

            assert_ok!(_create_post(
                Some(Origin::signed(ACCOUNT2)),
                Some(Some(SPACE2)),
                Some(extension_shared_post(POST1)),
                None
            )); // Share PostId 1 on SpaceId 2 by ACCOUNT2

            assert_eq!(Posts::post_by_id(POST1).unwrap().score, SharePostActionWeight::get() as i32);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT1).unwrap().reputation, 1 + SharePostActionWeight::get() as u32);
            assert_eq!(Scores::post_score_by_account((ACCOUNT2, POST1, scoring_action_share_post())), Some(SharePostActionWeight::get()));
        });
    }

    #[test]
    fn share_post_should_not_change_score() {
        ExtBuilder::build_with_post().execute_with(|| {
            assert_ok!(_create_post(
                Some(Origin::signed(ACCOUNT1)),
                Some(Some(SPACE1)),
                Some(extension_shared_post(POST1)),
                None
            )); // Share PostId

            assert_eq!(Posts::post_by_id(POST1).unwrap().score, 0);
            assert_eq!(Profiles::social_account_by_id(ACCOUNT1).unwrap().reputation, 1);
            assert!(Scores::post_score_by_account((ACCOUNT1, POST1, scoring_action_share_post())).is_none());
        });
    }

    #[test]
    fn share_post_should_fail_when_original_post_not_found() {
        ExtBuilder::build_with_space().execute_with(|| {
            assert_ok!(_create_space(
                Some(Origin::signed(ACCOUNT2)),
                Some(Some(b"space2_handle".to_vec())),
                None,
                None
            )); // SpaceId 2 by ACCOUNT2

            // Skipped creating PostId 1
            assert_noop!(_create_post(
                Some(Origin::signed(ACCOUNT2)),
                Some(Some(SPACE2)),
                Some(extension_shared_post(POST1)),
                None
            ), PostsError::<TestRuntime>::OriginalPostNotFound);
        });
    }

    #[test]
    fn share_post_should_fail_when_trying_to_share_shared_post() {
        ExtBuilder::build_with_post().execute_with(|| {
            assert_ok!(_create_space(
                Some(Origin::signed(ACCOUNT2)),
                Some(Some(b"space2_handle".to_vec())),
                None,
                None
            )); // SpaceId 2 by ACCOUNT2

            assert_ok!(_create_post(
                Some(Origin::signed(ACCOUNT2)),
                Some(Some(SPACE2)),
                Some(extension_shared_post(POST1)),
                None)
            );

            // Try to share post with extension SharedPost
            assert_noop!(_create_post(
                Some(Origin::signed(ACCOUNT1)),
                Some(Some(SPACE1)),
                Some(extension_shared_post(POST2)),
                None
            ), PostsError::<TestRuntime>::CannotShareSharingPost);
        });
    }

    #[test]
    fn share_post_should_fail_when_account_has_no_permission_to_create_posts_in_new_space() {
        ExtBuilder::build_with_post().execute_with(|| {
            assert_ok!(_create_space(
                Some(Origin::signed(ACCOUNT1)),
                Some(None), // No space_handle provided (ok)
                None, // Default space content,
                None
            )); // SpaceId 2 by ACCOUNT1

            // Try to share post with extension SharedPost
            assert_noop!(_create_post(
                Some(Origin::signed(ACCOUNT2)),
                Some(Some(SPACE2)),
                Some(extension_shared_post(POST1)),
                None
            ), PostsError::<TestRuntime>::NoPermissionToCreatePosts);
        });
    }

    #[test]
    fn share_post_should_fail_when_no_right_permission_in_account_roles() {
        ExtBuilder::build_with_a_few_roles_granted_to_account2(vec![SP::CreatePosts]).execute_with(|| {
            assert_ok!(_create_space(
                None, // From ACCOUNT1
                Some(None), // Provided without any handle
                None, // With default space content
                None
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
                Some(extension_shared_post(POST1)),
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
            assert_eq!(profile.content, profile_content_ipfs());

            assert!(ProfileHistory::edit_history(ACCOUNT1).is_empty());
        });
    }

    #[test]
    fn create_profile_should_fail_when_profile_is_already_created() {
        ExtBuilder::build().execute_with(|| {
            assert_ok!(_create_default_profile());
            // AccountId 1
            assert_noop!(_create_default_profile(), ProfilesError::<TestRuntime>::ProfileAlreadyCreated);
        });
    }

    #[test]
    fn create_profile_should_fail_when_ipfs_cid_is_invalid() {
        ExtBuilder::build().execute_with(|| {
            assert_noop!(_create_profile(
                None,
                Some(invalid_content_ipfs())
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
                Some(space_content_ipfs())
            ));

            // Check whether profile updated correctly
            let profile = Profiles::social_account_by_id(ACCOUNT1).unwrap().profile.unwrap();
            assert!(profile.updated.is_some());
            assert_eq!(profile.content, space_content_ipfs());

            // Check whether profile history is written correctly
            let profile_history = ProfileHistory::edit_history(ACCOUNT1)[0].clone();
            assert_eq!(profile_history.old_data.content, Some(profile_content_ipfs()));
        });
    }

    #[test]
    fn update_profile_should_fail_when_social_account_not_found() {
        ExtBuilder::build().execute_with(|| {
            assert_noop!(_update_profile(
                None,
                Some(profile_content_ipfs())
            ), ProfilesError::<TestRuntime>::SocialAccountNotFound);
        });
    }

    #[test]
    fn update_profile_should_fail_when_account_has_no_profile() {
        ExtBuilder::build().execute_with(|| {
            assert_ok!(ProfileFollows::follow_account(Origin::signed(ACCOUNT1), ACCOUNT2));
            assert_noop!(_update_profile(
                None,
                Some(profile_content_ipfs())
            ), ProfilesError::<TestRuntime>::AccountHasNoProfile);
        });
    }

    #[test]
    fn update_profile_should_fail_when_no_updates_for_profile_provided() {
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
    fn update_profile_should_fail_when_ipfs_cid_is_invalid() {
        ExtBuilder::build().execute_with(|| {
            assert_ok!(_create_default_profile());
            assert_noop!(_update_profile(
                None,
                Some(invalid_content_ipfs())
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
    fn follow_space_should_fail_when_space_not_found() {
        ExtBuilder::build().execute_with(|| {
            assert_noop!(_default_follow_space(), SpacesError::<TestRuntime>::SpaceNotFound);
        });
    }

    #[test]
    fn follow_space_should_fail_when_account_is_already_space_follower() {
        ExtBuilder::build_with_space().execute_with(|| {
            assert_ok!(_default_follow_space()); // Follow SpaceId 1 by ACCOUNT2

            assert_noop!(_default_follow_space(), SpaceFollowsError::<TestRuntime>::AlreadySpaceFollower);
        });
    }

    #[test]
    fn follow_space_should_fail_when_trying_to_follow_hidden_space() {
        ExtBuilder::build_with_space().execute_with(|| {
            assert_ok!(_update_space(
                None,
                None,
                Some(space_update(None, None, Some(true)))
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
    fn unfollow_space_should_fail_when_space_not_found() {
        ExtBuilder::build_with_space_follow_no_space().execute_with(|| {
            assert_noop!(_default_unfollow_space(), SpacesError::<TestRuntime>::SpaceNotFound);
        });
    }

    #[test]
    fn unfollow_space_should_fail_when_account_is_not_space_follower_yet() {
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
    fn follow_account_should_fail_when_account_tries_to_follow_themself() {
        ExtBuilder::build().execute_with(|| {
            assert_noop!(_follow_account(
                None,
                Some(ACCOUNT2)
            ), ProfileFollowsError::<TestRuntime>::AccountCannotFollowItself);
        });
    }

    #[test]
    fn follow_account_should_fail_when_account_is_already_following_account() {
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
    fn unfollow_account_should_fail_when_account_tries_to_unfollow_themself() {
        ExtBuilder::build().execute_with(|| {
            assert_noop!(_unfollow_account(
                None,
                Some(ACCOUNT2)
            ), ProfileFollowsError::<TestRuntime>::AccountCannotUnfollowItself);
        });
    }

    #[test]
    fn unfollow_account_should_fail_when_account_is_not_following_another_account_yet() {
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
    fn transfer_space_ownership_should_fail_when_space_not_found() {
        ExtBuilder::build().execute_with(|| {
            assert_noop!(_transfer_default_space_ownership(), SpacesError::<TestRuntime>::SpaceNotFound);
        });
    }

    #[test]
    fn transfer_space_ownership_should_fail_when_account_is_not_space_owner() {
        ExtBuilder::build_with_space().execute_with(|| {
            assert_noop!(_transfer_space_ownership(
                Some(Origin::signed(ACCOUNT2)),
                None,
                Some(ACCOUNT1)
            ), SpacesError::<TestRuntime>::NotASpaceOwner);
        });
    }

    #[test]
    fn transfer_space_ownership_should_fail_when_trying_to_transfer_to_current_owner() {
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
    fn accept_pending_ownership_should_fail_when_space_not_found() {
        ExtBuilder::build_with_pending_ownership_transfer_no_space().execute_with(|| {
            assert_noop!(_accept_default_pending_ownership(), SpacesError::<TestRuntime>::SpaceNotFound);
        });
    }

    #[test]
    fn accept_pending_ownership_should_fail_when_no_pending_transfer_for_space() {
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
    fn reject_pending_ownership_should_work_when_proposal_rejected_by_current_space_owner() {
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
    fn reject_pending_ownership_should_fail_when_space_not_found() {
        ExtBuilder::build_with_pending_ownership_transfer_no_space().execute_with(|| {
            assert_noop!(_reject_default_pending_ownership(), SpacesError::<TestRuntime>::SpaceNotFound);
        });
    }

    #[test]
    fn reject_pending_ownership_should_fail_when_no_pending_transfer_on_space() {
        ExtBuilder::build_with_space().execute_with(|| {
            assert_noop!(_reject_default_pending_ownership(), SpaceOwnershipError::<TestRuntime>::NoPendingTransferOnSpace); // Rejecting a transfer from ACCOUNT2
        });
    }

    #[test]
    fn reject_pending_ownership_should_fail_when_account_is_not_allowed_to_reject() {
        ExtBuilder::build_with_space().execute_with(|| {
            assert_ok!(_transfer_default_space_ownership()); // Transfer SpaceId 1 owned by ACCOUNT1 to ACCOUNT2

            assert_noop!(_reject_pending_ownership(
                Some(Origin::signed(ACCOUNT3)),
                None
            ), SpaceOwnershipError::<TestRuntime>::NotAllowedToRejectOwnershipTransfer); // Rejecting a transfer from ACCOUNT2
        });
    }
}
