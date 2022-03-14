//! Benchmarks for Locker Mirror Pallet
#![cfg(feature = "runtime-benchmarks")]

use crate::*;
use frame_benchmarking::{benchmarks, whitelisted_caller, account};
use frame_system::{RawOrigin};
use frame_support::ensure;
use sp_runtime::traits::Bounded;

fn _mock_lock_info<T: Config>() -> LockedInfoOf<T> {
    LockedInfo::<T::BlockNumber, BalanceOf<T>> {
        locked_amount: BalanceOf::<T>::max_value(),
        expires_at: Some(T::BlockNumber::from(11u32)),
        locked_at: T::BlockNumber::from(102u32),
    }
}

benchmarks!{

    set_locked_info {
        let account: T::AccountId = account("BenchAccount", 1, 3);
        let locked_info = _mock_lock_info::<T>();
        let caller: T::AccountId = whitelisted_caller();
        let origin = if cfg!(test) {
            RawOrigin::Signed(caller)
        } else {
            RawOrigin::Root
        };
    }: _(origin, account.clone(), locked_info.clone())
    verify {
        let res = <LockedInfoByAccount<T>>::get(account.clone()).expect("There should be a value stored for this account");
        ensure!(res == locked_info, "stored locked_info is not correct");
    }


    clear_locked_info {
        let caller: T::AccountId = whitelisted_caller();
        let account: T::AccountId = account("BenchAccount", 1, 3);
        let locked_amount = BalanceOf::<T>::max_value();
        let lock_period = T::BlockNumber::from(1223u32);
        let unlocks_at = T::BlockNumber::from(101323u32);
        <LockedInfoByAccount<T>>::insert(account.clone(), _mock_lock_info::<T>());
        let origin = if cfg!(test) {
            RawOrigin::Signed(caller)
        } else {
            RawOrigin::Root
        };
    }: _(origin, account.clone())
    verify {
        ensure!(matches!(<LockedInfoByAccount<T>>::get(account.clone()), None), "There should be no value for this account");
    }

    set_last_processed_parachain_event {
        let caller: T::AccountId = whitelisted_caller();
        let last_event_info = ParachainEvent {
            block_number: 125,
            event_index: 568,
        };
        <LastProcessedParachainEvent<T>>::kill();

        let origin = if cfg!(test) {
            RawOrigin::Signed(caller)
        } else {
            RawOrigin::Root
        };
    }: _(origin, last_event_info)
    verify {
        ensure!(matches!(<LastProcessedParachainEvent<T>>::get(), Some(last_event_info)), "The passed value should be stored");
    }


    impl_benchmark_test_suite!(
        Pallet,
        crate::mock::ExtBuilder::default().build(),
        crate::mock::Test,
    );
}