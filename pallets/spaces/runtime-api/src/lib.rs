#![cfg_attr(not(feature = "std"), no_std)]
// #![allow(clippy::too_many_arguments)]

use sp_std::vec::Vec;

use pallet_utils::SpaceId;

sp_api::decl_runtime_apis! {
    pub trait SpacesApi {
        fn get_last_space_id() -> SpaceId;

        fn get_hidden_space_ids(limit_opt: Option<u64>, offset_opt: Option<u64>) -> Vec<SpaceId>;

        fn get_public_space_ids(limit_opt: Option<u64>, offset_opt: Option<u64>) -> Vec<SpaceId>;

        fn get_unlisted_space_ids(limit_opt: Option<u64>, offset_opt: Option<u64>) -> Vec<SpaceId>;
    }
}
