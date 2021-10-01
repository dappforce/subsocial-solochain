//! Posts pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use sp_std::{vec};
use frame_system::RawOrigin;
use frame_support::ensure;
use sp_runtime::traits::Bounded;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use pallet_utils::{Config as UtilsConfig, BalanceOf};
use frame_support::{
    dispatch::DispatchError,
    traits::Currency,
};
use pallet_spaces::Module as SpaceModule;

const SPACE1: SpaceId = 1001;
const SPACE2: SpaceId = 1002;
const POST: PostId = 1;

fn post_content_ipfs() -> Content {
    Content::IPFS(b"bafyreidzue2dtxpj6n4x5mktrt7las5wz5diqma47zr25uau743dhe76we".to_vec())
}

fn space_content_ipfs() -> Content {
    Content::IPFS(b"bafyreib3mgbou4xln42qqcgj6qlt3cif35x4ribisxgq7unhpun525l54e".to_vec())
}

fn updated_post_content() -> Content {
    Content::IPFS(b"bafyreifw4omlqpr3nqm32bueugbodkrdne7owlkxgg7ul2qkvgrnkt3g3u".to_vec())
}

fn space_handle_1() -> Option<Vec<u8>> {
    Some(b"Space_Handle".to_vec())
}
fn space_handle_2() -> Option<Vec<u8>> {
    Some(b"space_handle_2".to_vec())
}

fn check_if_post_moved_correctly<T: Config>(
    moved_post_id: PostId,
    old_space_id: SpaceId,
    expected_new_space_id: SpaceId
) {
    let post: Post<T> = PostById::<T>::get(moved_post_id).unwrap();
    let new_space_id = post.space_id.unwrap();

    assert_eq!(new_space_id, expected_new_space_id);

    let old_space: Space<T> = SpaceById::<T>::get(old_space_id).unwrap();
    assert_eq!(old_space.posts_count, 0);
    assert_eq!(old_space.hidden_posts_count, 0);
    assert_eq!(old_space.score, 0);

    let new_space: Space<T> = SpaceById::<T>::get(new_space_id).unwrap();
    assert_eq!(new_space.posts_count, 1);
    assert_eq!(new_space.hidden_posts_count, if post.hidden { 1 } else { 0 });
    assert_eq!(new_space.score, post.score);
}

fn add_origin_with_space_post_and_balance<T: Config>() -> Result<RawOrigin<T::AccountId>, DispatchError> {
    let caller: T::AccountId = whitelisted_caller();
    let origin = RawOrigin::Signed(caller.clone());

    <T as UtilsConfig>::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

    SpaceModule::<T>::create_space(origin.clone().into(), None, space_handle_1(), space_content_ipfs(), None)?;
    Module::<T>::create_post(origin.clone().into(), Some(SPACE1), PostExtension::RegularPost, post_content_ipfs())?;

    Ok(origin)
}

benchmarks! {
    create_post {
        let caller: T::AccountId = whitelisted_caller();
        let origin = RawOrigin::Signed(caller.clone());

        <T as UtilsConfig>::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        SpaceModule::<T>::create_space(origin.clone().into(), None, space_handle_1(), space_content_ipfs(), None)?;
    }: _(origin, Some(SPACE1), PostExtension::RegularPost, post_content_ipfs())
    verify {
        ensure!(PostById::<T>::get(POST).is_some(), Error::<T>::PostNotFound)
    }

    update_post {
        let origin = add_origin_with_space_post_and_balance::<T>()?;

        let post_update: PostUpdate = PostUpdate {
            space_id: Some(SPACE1),
            content: Some(updated_post_content()),
            hidden: Some(false),
        };

    }: _(origin, POST, post_update)
    verify {
        let post: Post<T> = PostById::<T>::get(POST).unwrap();
        assert_eq!(post.space_id, Some(SPACE1));
        assert_eq!(post.content, updated_post_content());
        assert_eq!(post.hidden, false);
    }

    move_post {
        let origin = add_origin_with_space_post_and_balance::<T>()?;

        SpaceModule::<T>::create_space(origin.clone().into(), None, space_handle_2(), space_content_ipfs(), None)?;

    }: _(origin, POST, Some(SPACE2))
    verify {
        check_if_post_moved_correctly::<T>(POST, SPACE1, SPACE2)
    }

}
