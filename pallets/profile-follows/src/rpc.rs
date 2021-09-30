use sp_std::prelude::*;

use crate::{Module, Config};

impl<T: Config> Module<T> {
    pub fn filter_followed_accounts(account: T::AccountId, maybe_following: Vec<T::AccountId>) -> Vec<T::AccountId> {
        maybe_following.iter()
            .filter(|maybe_following| Self::account_followed_by_account((&account, maybe_following)))
            .cloned().collect()
    }
}