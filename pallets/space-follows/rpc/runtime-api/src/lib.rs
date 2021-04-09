#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;
use sp_std::vec::Vec;

use pallet_utils::SpaceId;

sp_api::decl_runtime_apis! {
    pub trait SpaceFollowsApi<AccountId> where
        AccountId: Codec
    {
        fn get_space_ids_followed_by_account(account: AccountId) -> Vec<SpaceId>;

        fn filter_followed_spaces(account: AccountId, space_ids: Vec<SpaceId>) -> Vec<SpaceId>;
    }
}
