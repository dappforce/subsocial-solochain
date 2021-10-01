use sp_std::prelude::*;

use crate::pallet::{Pallet, Config};

impl<T: Config> Pallet<T> {
    pub fn filter_followed_accounts(account: T::AccountId, maybe_following: Vec<T::AccountId>) -> Vec<T::AccountId> {
        maybe_following.iter()
            .filter(|maybe_following| Self::account_followed_by_account((&account, maybe_following)))
            .cloned().collect()
    }
}