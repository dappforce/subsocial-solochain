//! Benchmarks for Template Pallet
#![cfg(feature = "runtime-benchmarks")]

use crate::*;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_system::RawOrigin;
use frame_benchmarking::Box;
use frame_benchmarking::vec;
use sp_runtime::traits::Bounded;
use frame_support::traits::Get;
use frame_support::BoundedVec;
use sp_std::convert::TryInto;
use sp_std::prelude;
use crate::benchmarking::prelude::Vec;
use pallet_locker_mirror::{BalanceOf, LockedInfo, LockedInfoByAccount};

fn _create_eligible_account<T: Config>(index: u32) -> T::AccountId {
    account("eligible_account", index, 1)
}

fn _create_eligible_accounts<T: Config>(accounts_number: u32) -> Vec<T::AccountId> {
    let mut eligible_accounts: Vec<T::AccountId> = Vec::new();
    for i in 0..accounts_number {
        eligible_accounts.push(_create_eligible_account::<T>(i));
    }

    eligible_accounts
}

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

    add_eligible_accounts {
        let a in 1 .. T::AccountsSetLimit::get() => ();
        let eligible_accounts = _create_eligible_accounts::<T>(a);
    }: _(RawOrigin::Root, eligible_accounts.try_into().unwrap())
    verify {
        ensure!(EligibleAccounts::<T>::iter().count() as u32 == a, "Eligible accounts not added");
    }

    impl_benchmark_test_suite!(
        Pallet,
        crate::mock::ExtBuilder::default().build(),
        crate::mock::Test,
    );
}