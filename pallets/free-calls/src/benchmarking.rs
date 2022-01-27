//! Benchmarks for Template Pallet
#![cfg(feature = "runtime-benchmarks")]

use crate::*;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller, account};
use frame_system::RawOrigin;
use frame_benchmarking::Box;
use frame_benchmarking::vec;
use frame_support::traits::Currency;
use sp_runtime::traits::Bounded;
use pallet_locker_mirror::{BalanceOf, LockedInfo, LockedInfoByAccount};

pub mod currency {
    type Balance = u64;

    pub const UNITS: Balance = 100_000_000_000;
    pub const DOLLARS: Balance = UNITS;            // 100_000_000_000
    pub const CENTS: Balance = DOLLARS / 100;      // 1_000_000_000
    pub const MILLICENTS: Balance = CENTS / 1_000; // 1_000_000

    pub const fn deposit(items: u32, bytes: u32) -> Balance {
        items as Balance * 15 * CENTS + (bytes as Balance) * 6 * CENTS
    }
}

benchmarks!{
    // Individual benchmarks are placed here
    try_free_call {
        let caller: T::AccountId = whitelisted_caller();
		let call = Box::new(frame_system::Call::<T>::remark(vec![]).into());
        let current_block = <frame_system::Pallet<T>>::block_number();
        <LockedInfoByAccount<T>>::insert(caller.clone(), LockedInfo {
            lock_period: 1000u32.into(),
            locked_amount: BalanceOf::<T>::max_value(),
            unlocks_at: current_block + 1000u32.into(),
        });
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
