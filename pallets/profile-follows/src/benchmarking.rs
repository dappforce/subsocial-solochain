//! Profile-follows pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use sp_std::vec;
use frame_system::RawOrigin;
use frame_benchmarking::{benchmarks, account, whitelisted_caller};
use sp_runtime::traits::Bounded;
use pallet_utils::{Config as UtilsConfig, BalanceOf, Content};
use pallet_profiles::Module as ProfilesModule;
use frame_support::{
    dispatch::DispatchError,
    traits::Currency,
};

const SEED: u32 = 0;

fn profile_content_ipfs() -> Content {
    Content::IPFS(b"QmRAQB6YaCyidP37UdDnjFY5vQuiaRtqdyoW2CuDgwxkA5".to_vec())
}

fn caller_with_profile_and_balance<T: Config>() -> Result<T::AccountId, DispatchError> {
    let caller: T::AccountId = whitelisted_caller();
    let origin = RawOrigin::Signed(caller.clone());

    <T as UtilsConfig>::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

    ProfilesModule::<T>::create_profile(origin.into(), profile_content_ipfs())?;

    Ok(caller)
}

benchmarks! {
    follow_account {
        let caller: T::AccountId = caller_with_profile_and_balance::<T>()?;

        let follower: T::AccountId = account("user", 0, SEED);
        let follower_origin = RawOrigin::Signed(follower.clone());
    }: _(follower_origin, caller.clone())
    verify {
        assert_eq!(AccountsFollowedByAccount::<T>::get(follower.clone()), vec![caller.clone()]);
        assert_eq!(AccountFollowers::<T>::get(caller.clone()), vec![follower.clone()]);
        assert_eq!(AccountFollowedByAccount::<T>::get((follower, caller)), true);
    }

    unfollow_account {
        let caller: T::AccountId = caller_with_profile_and_balance::<T>()?;

        let follower: T::AccountId = account("user", 0, SEED);
        let follower_origin = RawOrigin::Signed(follower.clone());

        Module::<T>::follow_account(follower_origin.clone().into(), caller.clone())?;
    }: _(follower_origin, caller.clone())
    verify {
        assert!(AccountsFollowedByAccount::<T>::get(follower.clone()).is_empty());
        assert!(AccountFollowers::<T>::get(caller.clone()).is_empty());
        assert_eq!(AccountFollowedByAccount::<T>::get((caller, follower)), false);
    }
}
