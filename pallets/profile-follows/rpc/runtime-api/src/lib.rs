#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;
use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
    pub trait ProfileFollowsApi<AccountId> where
        AccountId: Codec
    {
        fn filter_followed_accounts(account: AccountId, other_accounts: Vec<AccountId>) -> Vec<AccountId>;
    }
}
