#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;
use sp_std::vec::Vec;
use pallet_utils::SpaceId;
use pallet_permissions::SpacePermission;

sp_api::decl_runtime_apis! {
    pub trait RolesApi<AccountId> where
        AccountId: Codec
    {
        fn get_space_permissions_by_user(account: AccountId, space_id: SpaceId) -> Vec<SpacePermission>;
    }
}
