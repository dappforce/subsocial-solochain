//! Benchmarks for Template Pallet
#![cfg(feature = "runtime-benchmarks")]

use crate::*;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;
use frame_benchmarking::Box;
use frame_benchmarking::vec;
use sp_runtime::traits::Bounded;
use pallet_locker_mirror::{BalanceOf, LockedInfo, LockedInfoByAccount};

benchmarks!{
    // Individual benchmarks are placed here
    try_free_call {
        let caller: T::AccountId = whitelisted_caller();
		let call = Box::new(frame_system::Call::<T>::remark { remark: vec![] }.into());
        let current_block = <frame_system::Pallet<T>>::block_number();
        <LockedInfoByAccount<T>>::insert(caller.clone(), LockedInfo {
            expires_at: None,
            locked_amount: BalanceOf::<T>::max_value(),
            locked_at: current_block + 1000u32.into(),
        });
    }: try_free_call(RawOrigin::Signed(caller.clone()), call)
    verify {
        let found_stats = <WindowStatsByConsumer<T>>::get(caller.clone()).is_empty() == false;
        ensure!(found_stats, "Stats should be recorded after the call");
        <WindowStatsByConsumer<T>>::remove(caller.clone());
    }

    impl_benchmark_test_suite!(
        Pallet,
        crate::mock::ExtBuilder::default().build(),
        crate::mock::Test,
    );
}