#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::string_lit_as_bytes)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::DispatchResult, ensure, traits::Get,
};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;

use pallet_posts::{PostScores, Post, PostById, PostExtension, PostId};
use pallet_profile_follows::{BeforeAccountFollowed, BeforeAccountUnfollowed};
use pallet_profiles::{Module as Profiles, SocialAccountById};
use pallet_reactions::{PostReactionScores, ReactionKind};
use pallet_space_follows::{BeforeSpaceFollowed, BeforeSpaceUnfollowed};
use pallet_spaces::{Space, SpaceById};
use pallet_utils::log_2;

// mod tests;

#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, RuntimeDebug)]
pub enum ScoringAction {
    UpvotePost,
    DownvotePost,
    SharePost,
    CreateComment,
    UpvoteComment,
    DownvoteComment,
    ShareComment,
    FollowSpace,
    FollowAccount,
}

impl Default for ScoringAction {
    fn default() -> Self {
        ScoringAction::FollowAccount
    }
}

/// The pallet's configuration trait.
pub trait Trait: system::Trait
    + pallet_utils::Trait
    + pallet_profiles::Trait
    + pallet_profile_follows::Trait
    + pallet_posts::Trait
    + pallet_spaces::Trait
    + pallet_space_follows::Trait
    + pallet_reactions::Trait
{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    // Weights of the social actions
    type FollowSpaceActionWeight: Get<i16>;
    type FollowAccountActionWeight: Get<i16>;

    type SharePostActionWeight: Get<i16>;
    type UpvotePostActionWeight: Get<i16>;
    type DownvotePostActionWeight: Get<i16>;

    type CreateCommentActionWeight: Get<i16>;
    type ShareCommentActionWeight: Get<i16>;
    type UpvoteCommentActionWeight: Get<i16>;
    type DownvoteCommentActionWeight: Get<i16>;
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Scored account reputation difference by account and action not found.
        ReputationDiffNotFound,
        /// Post extension is a comment.
        NotRootPost,
        /// Post extension is not a comment.
        NotComment,
    }
}

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as TemplateModule {

        // TODO shorten name? (refactor)
        pub AccountReputationDiffByAccount get(fn account_reputation_diff_by_account):
            map (/* actor */ T::AccountId, /* subject */ T::AccountId, ScoringAction) => Option<i16>;

        pub PostScoreByAccount get(fn post_score_by_account):
            map (/* actor */ T::AccountId, /* subject */ PostId, ScoringAction) => Option<i16>;
    }
}

decl_event!(
    pub enum Event<T> where
        <T as system::Trait>::AccountId,
    {
        AccountReputationChanged(AccountId, ScoringAction, u32),
    }
);

// The pallet's dispatchable functions.
decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        /// Weights of the related social account actions
        const FollowSpaceActionWeight: i16 = T::FollowSpaceActionWeight::get();
        const FollowAccountActionWeight: i16 = T::FollowAccountActionWeight::get();
        const UpvotePostActionWeight: i16 = T::UpvotePostActionWeight::get();
        const DownvotePostActionWeight: i16 = T::DownvotePostActionWeight::get();
        const SharePostActionWeight: i16 = T::SharePostActionWeight::get();
        const CreateCommentActionWeight: i16 = T::CreateCommentActionWeight::get();
        const UpvoteCommentActionWeight: i16 = T::UpvoteCommentActionWeight::get();
        const DownvoteCommentActionWeight: i16 = T::DownvoteCommentActionWeight::get();
        const ShareCommentActionWeight: i16 = T::ShareCommentActionWeight::get();

        // Initializing events
        fn deposit_event() = default;
    }
}

impl<T: Trait> Module<T> {

    pub fn scoring_action_by_post_extension(
        extension: PostExtension,
        reaction_kind: ReactionKind,
    ) -> ScoringAction {
        match extension {
            PostExtension::RegularPost | PostExtension::SharedPost(_) => match reaction_kind {
                ReactionKind::Upvote => ScoringAction::UpvotePost,
                ReactionKind::Downvote => ScoringAction::DownvotePost,
            },
            PostExtension::Comment(_) => match reaction_kind {
                ReactionKind::Upvote => ScoringAction::UpvoteComment,
                ReactionKind::Downvote => ScoringAction::DownvoteComment,
            },
        }
    }

    fn change_post_score_with_reaction(
        actor: T::AccountId,
        post: &mut Post<T>,
        reaction_kind: ReactionKind,
    ) -> DispatchResult {

        // Post owner should not be able to change the score of their post.
        if post.is_owner(&actor) {
            return Ok(())
        }

        let action = Self::scoring_action_by_post_extension(post.extension, reaction_kind);
        Self::change_post_score(actor, post, action)
    }

    pub fn change_post_score(
        account: T::AccountId,
        post: &mut Post<T>,
        action: ScoringAction,
    ) -> DispatchResult {
        if post.is_comment() {
            Self::change_comment_score(account, post, action)
        } else {
            Self::change_root_post_score(account, post, action)
        }
    }

    fn change_root_post_score(
        account: T::AccountId,
        post: &mut Post<T>,
        action: ScoringAction,
    ) -> DispatchResult {
        ensure!(post.is_root_post(), Error::<T>::NotRootPost);

        let social_account = Profiles::get_or_new_social_account(account.clone());

        // TODO inspect: this insert could be redundant if the account already exists.
        <SocialAccountById<T>>::insert(account.clone(), social_account.clone());

        let post_id = post.id;

        // TODO inspect: maybe this check is redundant such as we use change_root_post_score() internally and post was already loaded.
        // Posts::<T>::ensure_post_exists(post_id)?;

        // Post owner should not have any impact on their post score.
        if post.is_owner(&account) {
            return Ok(())
        }

        let mut space = post.get_space()?;

        if let Some(score_diff) = Self::post_score_by_account((account.clone(), post_id, action)) {
            let reputation_diff = Self::account_reputation_diff_by_account((account.clone(), post.created.account.clone(), action))
                .ok_or(Error::<T>::ReputationDiffNotFound)?;

            // Revert this score diff:
            post.change_score(-score_diff);
            space.change_score(-score_diff);
            Self::change_social_account_reputation(post.created.account.clone(), account.clone(), -reputation_diff, action)?;
            <PostScoreByAccount<T>>::remove((account, post_id, action));
        } else {
            match action {
                ScoringAction::UpvotePost => {
                    if Self::post_score_by_account((account.clone(), post_id, ScoringAction::DownvotePost)).is_some() {
                        // TODO inspect this recursion. Doesn't look good:
                        Self::change_root_post_score(account.clone(), post, ScoringAction::DownvotePost)?;
                    }
                }
                ScoringAction::DownvotePost => {
                    if Self::post_score_by_account((account.clone(), post_id, ScoringAction::UpvotePost)).is_some() {
                        // TODO inspect this recursion. Doesn't look good:
                        Self::change_root_post_score(account.clone(), post, ScoringAction::UpvotePost)?;
                    }
                }
                _ => (),
            }
            let score_diff = Self::score_diff_for_action(social_account.reputation, action);
            post.change_score(score_diff);
            space.change_score(score_diff);
            Self::change_social_account_reputation(post.created.account.clone(), account.clone(), score_diff, action)?;
            <PostScoreByAccount<T>>::insert((account, post_id, action), score_diff);
        }

        <PostById<T>>::insert(post_id, post.clone());
        <SpaceById<T>>::insert(space.id, space);

        Ok(())
    }

    fn change_comment_score(
        account: T::AccountId,
        comment: &mut Post<T>,
        action: ScoringAction,
    ) -> DispatchResult {
        ensure!(comment.is_comment(), Error::<T>::NotComment);

        let social_account = Profiles::get_or_new_social_account(account.clone());

        // TODO inspect: this insert could be redundant if the account already exists.
        <SocialAccountById<T>>::insert(account.clone(), social_account.clone());

        let comment_id = comment.id;

        // TODO inspect: maybe this check is redundant such as we use change_comment_score() internally and comment was already loaded.
        // Posts::<T>::ensure_post_exists(comment_id)?;

        // Comment owner should not have any impact on their comment score.
        if comment.is_owner(&account) {
            return Ok(())
        }

        if let Some(score_diff) = Self::post_score_by_account((account.clone(), comment_id, action)) {
            let reputation_diff = Self::account_reputation_diff_by_account((account.clone(), comment.created.account.clone(), action))
                .ok_or(Error::<T>::ReputationDiffNotFound)?;

            // Revert this score diff:
            comment.change_score(-score_diff);
            Self::change_social_account_reputation(comment.created.account.clone(), account.clone(), -reputation_diff, action)?;
            <PostScoreByAccount<T>>::remove((account, comment_id, action));
        } else {
            match action {
                ScoringAction::UpvoteComment => {
                    if Self::post_score_by_account((account.clone(), comment_id, ScoringAction::DownvoteComment)).is_some() {
                        Self::change_comment_score(account.clone(), comment, ScoringAction::DownvoteComment)?;
                    }
                }
                ScoringAction::DownvoteComment => {
                    if Self::post_score_by_account((account.clone(), comment_id, ScoringAction::UpvoteComment)).is_some() {
                        Self::change_comment_score(account.clone(), comment, ScoringAction::UpvoteComment)?;
                    }
                }
                ScoringAction::CreateComment => {
                    let root_post = &mut comment.get_root_post()?;
                    Self::change_root_post_score(account.clone(), root_post, action)?;
                }
                _ => (),
            }
            let score_diff = Self::score_diff_for_action(social_account.reputation, action);
            comment.change_score(score_diff);
            Self::change_social_account_reputation(comment.created.account.clone(), account.clone(), score_diff, action)?;
            <PostScoreByAccount<T>>::insert((account, comment_id, action), score_diff);
        }
        <PostById<T>>::insert(comment_id, comment.clone());

        Ok(())
    }

    // TODO change order of args to: actor (scorer), subject (account), ...
    pub fn change_social_account_reputation(
        account: T::AccountId,
        scorer: T::AccountId,
        mut score_diff: i16,
        action: ScoringAction,
    ) -> DispatchResult {

        // TODO return Ok(()) if score_diff == 0?

        // TODO seems like we can pass a &mut social account as an arg to this func
        let mut social_account = Profiles::get_or_new_social_account(account.clone());

        if social_account.reputation as i64 + score_diff as i64 <= 1 {
            social_account.reputation = 1;
            score_diff = 0;
        }

        social_account.change_reputation(score_diff);

        if Self::account_reputation_diff_by_account((scorer.clone(), account.clone(), action)).is_some() {
            <AccountReputationDiffByAccount<T>>::remove((scorer, account.clone(), action));
        } else {
            <AccountReputationDiffByAccount<T>>::insert((scorer, account.clone(), action), score_diff);
        }

        <SocialAccountById<T>>::insert(account.clone(), social_account.clone());

        Self::deposit_event(RawEvent::AccountReputationChanged(account, action, social_account.reputation));

        Ok(())
    }

    pub fn score_diff_for_action(reputation: u32, action: ScoringAction) -> i16 {
        Self::smooth_reputation(reputation) as i16 * Self::weight_of_scoring_action(action)
    }

    fn smooth_reputation(reputation: u32) -> u8 {
        log_2(reputation).map_or(1, |r| {
            let d = (reputation as u64 - (2 as u64).pow(r)) * 100
                / (2 as u64).pow(r);

            // We can safely cast this result to i16 because a score diff for u32::MAX is 32.
            (((r + 1) * 100 + d as u32) / 100) as u8
        })
    }

    fn weight_of_scoring_action(action: ScoringAction) -> i16 {
        use ScoringAction::*;
        match action {
            UpvotePost => T::UpvotePostActionWeight::get(),
            DownvotePost => T::DownvotePostActionWeight::get(),
            SharePost => T::SharePostActionWeight::get(),
            CreateComment => T::CreateCommentActionWeight::get(),
            UpvoteComment => T::UpvoteCommentActionWeight::get(),
            DownvoteComment => T::DownvoteCommentActionWeight::get(),
            ShareComment => T::ShareCommentActionWeight::get(),
            FollowSpace => T::FollowSpaceActionWeight::get(),
            FollowAccount => T::FollowAccountActionWeight::get(),
        }
    }
}

impl<T: Trait> BeforeSpaceFollowed<T> for Module<T> {
    fn before_space_followed(follower: T::AccountId, follower_reputation: u32, space: &mut Space<T>) {
        // Change a space score only if the follower is NOT a space owner.
        if !space.is_owner(&follower) {
            let space_owner = space.owner.clone();
            let action = ScoringAction::FollowSpace;
            let score_diff = Self::score_diff_for_action(follower_reputation, action);
            if Self::change_social_account_reputation(
                space_owner, follower, score_diff, action
            ).is_ok() {
                space.change_score(score_diff);
            }
        }
    }
}

impl<T: Trait> BeforeSpaceUnfollowed<T> for Module<T> {
    fn before_space_unfollowed(follower: T::AccountId, space: &mut Space<T>) {
        // Change a space score only if the follower is NOT a space owner.
        if !space.is_owner(&follower) {
            let space_owner = space.owner.clone();
            let action = ScoringAction::FollowSpace;
            if let Some(score_diff) = Self::account_reputation_diff_by_account(
                (follower.clone(), space_owner.clone(), action)
            ) {
                // Subtract a score diff that was added when this user followed this space in the past:
                if Self::change_social_account_reputation(
                    space_owner, follower, -score_diff, action
                ).is_ok() {
                    space.change_score(-score_diff);
                }
            }
        }
    }
}

impl<T: Trait> BeforeAccountFollowed<T> for Module<T> {
    fn before_account_followed(follower: T::AccountId, follower_reputation: u32, following: T::AccountId) -> DispatchResult {
        let action = ScoringAction::FollowAccount;
        let score_diff = Self::score_diff_for_action(follower_reputation, action);
        Self::change_social_account_reputation(following, follower, score_diff, action)
    }
}

impl<T: Trait> BeforeAccountUnfollowed<T> for Module<T> {
    fn before_account_unfollowed(follower: T::AccountId, following: T::AccountId) -> DispatchResult {
        let action = ScoringAction::FollowAccount;

        let rep_diff = Self::account_reputation_diff_by_account(
            (follower.clone(), following.clone(), action)
        ).ok_or(Error::<T>::ReputationDiffNotFound)?;

        Self::change_social_account_reputation(following, follower, rep_diff, action)
    }
}

impl<T: Trait> PostScores<T> for Module<T> {
    fn score_post_on_new_share(account: T::AccountId, original_post: &mut Post<T>) -> DispatchResult {
        let action =
            if original_post.is_comment() { ScoringAction::ShareComment }
            else { ScoringAction::SharePost };

        let account_never_shared_this_post =
            Self::post_score_by_account(
                (account.clone(), original_post.id, action)
            ).is_none();

        // It makes sense to change a score of this post only once:
        // i.e. when this account sharing it for the first time.
        if account_never_shared_this_post {
            Self::change_post_score(account, original_post, action)
        } else {
            Ok(())
        }
    }

    fn score_root_post_on_new_comment(account: T::AccountId, root_post: &mut Post<T>) -> DispatchResult {
        Self::change_post_score(account, root_post, ScoringAction::CreateComment)
    }
}

impl<T: Trait> PostReactionScores<T> for Module<T> {
    fn score_post_on_reaction(
        actor: T::AccountId,
        post: &mut Post<T>,
        reaction_kind: ReactionKind,
    ) -> DispatchResult {
        Self::change_post_score_with_reaction(actor, post, reaction_kind)
    }
}
