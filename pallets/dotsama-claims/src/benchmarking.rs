//! Dotsama Claims pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use sp_std::vec;
use crate::Module as Pallet;
use frame_system::{RawOrigin};
use frame_benchmarking::{benchmarks, account};
use frame_support::{
    ensure, traits::{Currency, Get},
};
use sp_runtime::traits::Bounded;
use pallet_utils::BalanceOf;
use sp_std::{
    vec::Vec,
    boxed::Box,
};

const REWARDS_SENDER_SEED: u32 = 0;
const ELIGIBLE_ACCOUNT_SEED: u32 = 1;

fn rewards_sender_with_free_balance<T: Config>() -> T::AccountId {
    let rewards_sender: T::AccountId = account("rewards_sender", REWARDS_SENDER_SEED, REWARDS_SENDER_SEED);

    T::Currency::make_free_balance_be(&rewards_sender, BalanceOf::<T>::max_value());

    rewards_sender
}

fn create_eligible_account<T: Config>(index: u32) -> T::AccountId {
    account("eligible_account", index, ELIGIBLE_ACCOUNT_SEED)
}

fn create_eligible_accounts<T: Config>(accounts_number: u32) -> Vec<T::AccountId> {
    let mut eligible_accounts: Vec<T::AccountId> = Vec::new();
    for i in 0..accounts_number {
        eligible_accounts.push(create_eligible_account::<T>(i));
    }

    eligible_accounts
}

benchmarks! {
    claim_tokens {
        let rewards_sender: T::AccountId = rewards_sender_with_free_balance::<T>();
        Pallet::<T>::set_rewards_sender(RawOrigin::Root.into(), Some(rewards_sender))?;

        let eligible_account: T::AccountId = create_eligible_account::<T>(1);
        Pallet::<T>::add_eligible_accounts(RawOrigin::Root.into(), vec![eligible_account.clone()])?;
    }: _(RawOrigin::Signed(eligible_account.clone()))
    verify {
        let initial_claim_amount = T::InitialClaimAmount::get();
        assert_eq!(T::Currency::free_balance(&eligible_account), initial_claim_amount);
        assert_eq!(Pallet::<T>::tokens_claimed_by_account(eligible_account), initial_claim_amount);
    }

    set_rewards_sender {
        let rewards_sender: T::AccountId = rewards_sender_with_free_balance::<T>();
    }: _(RawOrigin::Root, Some(rewards_sender.clone()))
    verify {
        if let Some(rewards_sender_from_storage) = RewardsSender::<T>::get() {
            assert_eq!(rewards_sender_from_storage, rewards_sender);
        };
    }

    add_eligible_accounts {
        let a in 1 .. T::AccountsSetLimit::get() => ();
        let eligible_accounts = create_eligible_accounts::<T>(a);
    }: _(RawOrigin::Root, eligible_accounts)
    verify {
        ensure!(EligibleAccounts::<T>::iter().count() as u32 == a, "Eligible accounts not added");
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::mock::{Test, ExtBuilder};
    use frame_support::assert_ok;

    #[test]
    fn test_benchmarks() {
        ExtBuilder::build().execute_with(|| {
            assert_ok!(test_benchmark_claim_tokens::<Test>());
            assert_ok!(test_benchmark_set_rewards_sender::<Test>());
            assert_ok!(test_benchmark_add_eligible_accounts::<Test>());
        });
    }
}