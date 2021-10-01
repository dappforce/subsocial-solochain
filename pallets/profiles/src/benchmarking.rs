//! Profiles pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use sp_std::{vec};
use frame_system::{RawOrigin};
use frame_benchmarking::{benchmarks, whitelisted_caller};
use sp_runtime::traits::Bounded;
use pallet_utils::{Config as UtilsConfig, BalanceOf};
use frame_support::traits::Currency;

fn profile_content_ipfs() -> Content {
    Content::IPFS(b"QmRAQB6YaCyidP37UdDnjFY5vQuiaRtqdyoW2CuDgwxkA5".to_vec())
}

fn updated_profile_content_ipfs() -> Content {
    Content::IPFS(b"QmRAQB6YaCyidP37UdDnjFY5vQuiajthdyeW2CuagwxkA5".to_vec())
}

fn caller_with_balance<T: Config>() -> T::AccountId {
    let caller: T::AccountId = whitelisted_caller();
    <T as UtilsConfig>::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

    caller
}

benchmarks! {
    create_profile {
        let caller = caller_with_balance::<T>();
        let origin = RawOrigin::Signed(caller.clone());
    }: _(origin, profile_content_ipfs())
    verify {
        let profile = SocialAccountById::<T>::get(caller.clone()).unwrap().profile.unwrap();
        assert_eq!(profile.created.account, caller);
        assert!(profile.updated.is_none());
        assert_eq!(profile.content, profile_content_ipfs());
    }

    update_profile {
         let caller = caller_with_balance::<T>();
        let origin = RawOrigin::Signed(caller.clone());

        Module::<T>::create_profile(origin.clone().into(), profile_content_ipfs())?;
    }: _(origin, ProfileUpdate {
        content: Some(updated_profile_content_ipfs())
    })
    verify {
        let profile = SocialAccountById::<T>::get(caller).unwrap().profile.unwrap();
        assert!(profile.updated.is_some());
        assert_eq!(profile.content, updated_profile_content_ipfs());
    }
}
