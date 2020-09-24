#![cfg_attr(not(feature = "std"), no_std)]
// #![allow(clippy::too_many_arguments)]

use sp_std::vec::Vec;

use pallet_utils::SpaceId;
use pallet_posts::PostId;

sp_api::decl_runtime_apis! {
    pub trait PostsApi {
        fn find_public_post_ids_in_space(space_id: SpaceId, offset: u64, limit: u64) -> Vec<PostId>;

        fn find_unlisted_post_ids_in_space(space_id: SpaceId, offset: u64, limit: u64) -> Vec<PostId>;
    }
}
