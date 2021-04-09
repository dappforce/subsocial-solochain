#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;
use sp_std::collections::btree_map::BTreeMap;
use sp_std::vec::Vec;

use pallet_posts::rpc::FlatPost;
use pallet_utils::{PostId, SpaceId};

sp_api::decl_runtime_apis! {
    pub trait PostsApi<AccountId, BlockNumber> where
        AccountId: Codec,
        BlockNumber: Codec
    {
        fn get_posts_by_ids(post_ids: Vec<PostId>) -> Vec<FlatPost<AccountId, BlockNumber>>;
    
        fn get_public_posts_by_space(space_id: SpaceId, offset: u64, limit: u16) -> Vec<FlatPost<AccountId, BlockNumber>>;
    
        fn get_unlisted_posts_by_space(space_id: SpaceId, offset: u64, limit: u16) -> Vec<FlatPost<AccountId, BlockNumber>>;
    
        fn get_reply_ids_by_post_id(post_id: PostId) -> Vec<PostId>;
    
        fn get_comment_ids_tree(post_id: PostId) -> BTreeMap<PostId, Vec<PostId>>;
    
        fn get_public_post_ids_by_space(space_id: SpaceId) -> Vec<PostId>;

        fn get_unlisted_post_ids_by_space(space_id: SpaceId) -> Vec<PostId>;

        fn get_next_post_id() -> PostId;

        fn get_feed(account: AccountId, offset: u64, limit: u16) -> Vec<FlatPost<AccountId, BlockNumber>>;
    }
}
