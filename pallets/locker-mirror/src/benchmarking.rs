//! Benchmarks for Locker Mirror Pallet
#![cfg(feature = "runtime-benchmarks")]

use crate::*;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller, account};
use frame_system::RawOrigin;
use frame_benchmarking::Box;
use frame_benchmarking::vec;
use frame_support::ensure;
use sp_runtime::traits::Bounded;

fn _mock_lock_info<T: Config>() -> LockedInfoOf<T> {
    LockedInfo::<T::BlockNumber, BalanceOf<T>> {
        locked_amount: BalanceOf::<T>::max_value(),
        lock_period: T::BlockNumber::from(11u32),
        unlocks_at: T::BlockNumber::from(102u32),
    }
}

benchmarks!{

    set_locked_info {
        let caller = RawOrigin::Root;
        let account: T::AccountId = account("BenchAccount", 1, 3);
        let locked_info = _mock_lock_info::<T>();
    }: _(caller, account.clone(), locked_info.clone())
    verify {
        let res = <LockedInfoByAccount<T>>::get(account.clone()).expect("There should be a value stored for this account");
        ensure!(res == locked_info, "stored locked_info is not correct");
    }


    clear_locked_info {
        let caller = RawOrigin::Root;
        let account: T::AccountId = account("BenchAccount", 1, 3);
        let locked_amount = BalanceOf::<T>::max_value();
        let lock_period = T::BlockNumber::from(1223u32);
        let unlocks_at = T::BlockNumber::from(101323u32);
        <LockedInfoByAccount<T>>::insert(account.clone(), LockedInfo {
            locked_amount,
            lock_period,
            unlocks_at,
        });
    }: _(caller, account.clone())
    verify {
        ensure!(matches!(<LockedInfoByAccount<T>>::get(account.clone()), None), "There should be no value for this account");
    }
}

impl_benchmark_test_suite!(
    Pallet,
    crate::mock::new_test_ext(),
    crate::mock::Test,
);
