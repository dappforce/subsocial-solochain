//! Spaces pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use sp_std::vec;
use sp_runtime::traits::Bounded;
use frame_system::RawOrigin;
use frame_support::ensure;
use frame_benchmarking::{benchmarks, whitelisted_caller};

const SPACE: SpaceId = 1001;

fn space_content_ipfs() -> Content {
    Content::IPFS(b"bafyreib3mgbou4xln42qqcgj6qlt3cif35x4ribisxgq7unhpun525l54e".to_vec())
}

fn space_handle_1() -> Option<Vec<u8>> {
    Some(b"Space_Handle".to_vec())
}
fn space_handle_2() -> Option<Vec<u8>> {
    Some(b"space_handle_2".to_vec())
}

benchmarks! {
    create_space {
        let caller: T::AccountId = whitelisted_caller();
        <T as Config>::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
    }: _(RawOrigin::Signed(caller), None, space_handle_1(), space_content_ipfs(), None)
    verify {
        ensure!(SpaceById::<T>::get(SPACE).is_some(), Error::<T>::SpaceNotFound)
    }

    update_space {
        let caller: T::AccountId = whitelisted_caller();
        let origin = RawOrigin::Signed(caller.clone());

        let space_update: SpaceUpdate = SpaceUpdate {
            parent_id: None,
            handle: Some(space_handle_2()),
            content: Some(space_content_ipfs()),
            hidden: Some(true),
            permissions: None,
        };
        <T as Config>::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        Module::<T>::create_space(origin.clone().into(), None, space_handle_1(), space_content_ipfs(), None)?;

    }: _(origin, SPACE, space_update)
    verify {
        let space: Space<T> = SpaceById::<T>::get(SPACE).unwrap();
        assert_eq!(space.handle, space_handle_2());
        assert_eq!(space.content, space_content_ipfs());
        assert_eq!(space.hidden, true);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::mock::Test;
    use frame_support::assert_ok;
    use pallet_utils::mock_functions::DefaultExtBuilder;

    #[test]
    fn test_benchmarks() {
        DefaultExtBuilder::<Test>::build().execute_with(|| {
            assert_ok!(test_benchmark_create_space::<Test>());
            assert_ok!(test_benchmark_update_space::<Test>());
        });
    }
}
