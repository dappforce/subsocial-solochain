#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;
use sp_std::vec::Vec;

use pallet_spaces::rpc::FlatSpace;
use pallet_utils::SpaceId;

sp_api::decl_runtime_apis! {
    pub trait SpacesApi<AccountId, BlockNumber> where
        AccountId: Codec,
        BlockNumber: Codec
    {
        fn get_spaces(offset: u64, limit: u64) -> Vec<FlatSpace<AccountId, BlockNumber>>;

        fn get_spaces_by_ids(space_ids: Vec<SpaceId>) -> Vec<FlatSpace<AccountId, BlockNumber>>;

        fn get_public_spaces(offset: u64, limit: u64) -> Vec<FlatSpace<AccountId, BlockNumber>>;

        fn get_unlisted_spaces(offset: u64, limit: u64) -> Vec<FlatSpace<AccountId, BlockNumber>>;

        fn get_space_id_by_handle(handle: Vec<u8>) -> Option<SpaceId>;

        fn get_space_by_handle(handle: Vec<u8>) -> Option<FlatSpace<AccountId, BlockNumber>>;

        fn get_public_space_ids_by_owner(owner: AccountId) -> Vec<SpaceId>;

        fn get_unlisted_space_ids_by_owner(owner: AccountId) -> Vec<SpaceId>;

        fn get_next_space_id() -> SpaceId;
    }
}
