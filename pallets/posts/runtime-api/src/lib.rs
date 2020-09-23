#![cfg_attr(not(feature = "std"), no_std)]
// #![allow(clippy::too_many_arguments)]

use sp_std::vec::Vec;

use pallet_utils::SpaceId;
use pallet_posts::PostId;

sp_api::decl_runtime_apis! {
    pub trait PostsApi {
        fn get_public_post_ids_in_space(space_id: SpaceId, limit: u64, offset: u64) -> Vec<PostId>;

        fn get_unlisted_post_ids_in_space(space_id: SpaceId, limit: u64, offset: u64) -> Vec<PostId>;
    }
}
