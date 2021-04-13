use sp_std::prelude::*;

use crate::{Module, Config};

impl<T: Config> Module<T> {
    pub fn filter_followed_accounts(account: T::AccountId, other_accounts: Vec<T::AccountId>) -> Vec<T::AccountId> {
        other_accounts.iter()
                      .filter(|following_account| Self::account_followed_by_account((&account, following_account)))
                      .cloned().collect()
    }
}