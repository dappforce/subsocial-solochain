//! Benchmarks for Template Pallet
#![cfg(feature = "runtime-benchmarks")]

use crate::*;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller, account};
use frame_system::RawOrigin;
use frame_benchmarking::Box;
use frame_benchmarking::vec;

benchmarks!{
    // Individual benchmarks are placed here
    try_free_call {
        let caller: T::AccountId = whitelisted_caller();
		let call = Box::new(frame_system::Call::<T>::remark(vec![]).into());
        <QuotaByAccount<T>>::insert(caller.clone(), 1000);
    }: try_free_call(RawOrigin::Signed(caller.clone()), call)
    verify {
        let num_of_usages = <WindowStatsByAccount<T>>::iter_prefix_values(caller.clone()).count();
        ensure!(num_of_usages != 0, "Usage must not be empty after the call");
        <WindowStatsByAccount<T>>::remove_prefix(caller.clone(), None);
    }
}

impl_benchmark_test_suite!(
    Pallet,
    crate::mock::new_test_ext(),
    crate::mock::Test,
);
