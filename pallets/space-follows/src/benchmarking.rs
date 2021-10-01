//! Space-follows pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use sp_std::vec;
use frame_system::RawOrigin;
use frame_benchmarking::{benchmarks, account, whitelisted_caller};
use sp_runtime::traits::Bounded;
use pallet_utils::{Config as UtilsConfig, BalanceOf, Content, SpaceId};
use pallet_spaces::Module as SpaceModule;
use frame_support::{
    dispatch::DispatchError,
    traits::Currency,
};

const SPACE: SpaceId = 1001;
const SEED: u32 = 0;

fn space_content_ipfs() -> Content {
    Content::IPFS(b"bafyreib3mgbou4xln42qqcgj6qlt3cif35x4ribisxgq7unhpun525l54e".to_vec())
}

fn space_handle() -> Option<Vec<u8>> {
    Some(b"Space_Handle".to_vec())
}

fn add_origin_with_space_and_balance<T: Config>(caller: T::AccountId, follower: T::AccountId) -> Result<RawOrigin<T::AccountId>, DispatchError> {
    let origin = RawOrigin::Signed(caller.clone());
    let follower_origin = RawOrigin::Signed(follower);

    <T as UtilsConfig>::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

    SpaceModule::<T>::create_space(origin.into(), None, space_handle(), space_content_ipfs(), None)?;

    Ok(follower_origin)
}

benchmarks! {
    follow_space {
        let caller: T::AccountId = whitelisted_caller();
        let follower: T::AccountId = account("user", 0, SEED);

        let follower_origin = add_origin_with_space_and_balance::<T>(caller.clone(), follower.clone())?;
    }: _(follower_origin, SPACE)
    verify {
        assert_eq!(SpacesFollowedByAccount::<T>::get(follower.clone()), vec![SPACE]);
        assert_eq!(SpaceFollowers::<T>::get(SPACE), vec![caller, follower.clone()]);
        assert_eq!(SpaceFollowedByAccount::<T>::get((follower, SPACE)), true);
    }

    unfollow_space {
        let caller: T::AccountId = whitelisted_caller();
        let follower: T::AccountId = account("user", 0, SEED);

        let follower_origin = add_origin_with_space_and_balance::<T>(caller.clone(), follower.clone())?;

        Module::<T>::follow_space(follower_origin.clone().into(), SPACE)?;
    }: _(follower_origin, SPACE)
    verify {
        assert!(SpacesFollowedByAccount::<T>::get(follower.clone()).is_empty());
        assert_eq!(SpaceFollowers::<T>::get(SPACE), vec![caller]);
    }
}
