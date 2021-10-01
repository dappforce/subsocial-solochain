//! Reactions pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use sp_std::vec;
use frame_system::RawOrigin;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use sp_runtime::traits::Bounded;
use pallet_utils::{Config as UtilsConfig, BalanceOf, Content, SpaceId};
use pallet_spaces::Module as SpaceModule;
use pallet_posts::{Module as PostsModule, PostExtension};
use frame_support::{
    dispatch::DispatchError,
    traits::Currency,
};

const POST: PostId = 1;
const SPACE: SpaceId = 1001;
const REACTION: ReactionId = 1;

fn reaction_upvote() -> ReactionKind {
    ReactionKind::Upvote
}

fn reaction_downvote() -> ReactionKind {
    ReactionKind::Downvote
}

fn post_content_ipfs() -> Content {
    Content::IPFS(b"bafyreidzue2dtxpj6n4x5mktrt7las5wz5diqma47zr25uau743dhe76we".to_vec())
}

fn space_content_ipfs() -> Content {
    Content::IPFS(b"bafyreib3mgbou4xln42qqcgj6qlt3cif35x4ribisxgq7unhpun525l54e".to_vec())
}

fn space_handle_1() -> Option<Vec<u8>> {
    Some(b"Space_Handle".to_vec())
}

fn origin_with_space_post_and_balance<T: Config>() -> Result<RawOrigin<T::AccountId>, DispatchError> {
    let caller: T::AccountId = whitelisted_caller();
    let origin = RawOrigin::Signed(caller.clone());

    <T as UtilsConfig>::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

    SpaceModule::<T>::create_space(origin.clone().into(), None, space_handle_1(), space_content_ipfs(), None)?;
    PostsModule::<T>::create_post(origin.clone().into(), Some(SPACE), PostExtension::RegularPost, post_content_ipfs())?;

    Ok(origin)
}

benchmarks! {
    create_post_reaction {
        let origin = origin_with_space_post_and_balance::<T>()?;
    }: _(origin, POST, reaction_upvote())
    verify {
        assert_eq!(ReactionIdsByPostId::get(POST), vec![REACTION]);
    }

    update_post_reaction {
        let origin = origin_with_space_post_and_balance::<T>()?;

        Module::<T>::create_post_reaction(origin.clone().into(), POST, reaction_upvote())?;
    }: _(origin, POST, REACTION, reaction_downvote())
    verify {
    }

    delete_post_reaction {
        let origin = origin_with_space_post_and_balance::<T>()?;

        Module::<T>::create_post_reaction(origin.clone().into(), POST, reaction_upvote())?;
    }: _(origin, POST, REACTION)
    verify {
    }
}
