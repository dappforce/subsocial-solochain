//! RPC interface for the transaction payment module.

use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;
use posts_runtime_api::PostsApi as PostsRuntimeApi;

use pallet_utils::SpaceId;
use pallet_posts::PostId;

#[rpc]
pub trait PostsApi<BlockHash> {
    #[rpc(name = "posts_findPublicPostIdsInSpace")]
    fn find_public_post_ids_in_space(
        &self,
        at: Option<BlockHash>,
        space_id: SpaceId,
        limit: u64,
        offset: u64
    ) -> Result<Vec<PostId>>;

    #[rpc(name = "posts_findUnlistedPostIdsInSpace")]
    fn find_unlisted_post_ids_in_space(
        &self,
        at: Option<BlockHash>,
        space_id: SpaceId,
        limit: u64,
        offset: u64
    ) -> Result<Vec<PostId>>;
}

pub struct Posts<C, M> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<M>,
}

impl<C, M> Posts<C, M> {
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

impl<C, Block> PostsApi<<Block as BlockT>::Hash> for Posts<C, Block>
    where
        Block: BlockT,
        C: Send + Sync + 'static,
        C: ProvideRuntimeApi<Block>,
        C: HeaderBackend<Block>,
        C::Api: PostsRuntimeApi<Block>,
{
    fn find_public_post_ids_in_space(
        &self,
        at: Option<<Block as BlockT>::Hash>,
        space_id: SpaceId,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<PostId>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

        let runtime_api_result = api.find_public_post_ids_in_space(&at, space_id, offset, limit);
        runtime_api_result.map_err(|e| RpcError {
            // TODO: research on error codes and change a value
            code: ErrorCode::ServerError(9876), // No real reason for this value
            // TODO: change error message (?use errors macro)
            message: "Something wrong".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }

    fn find_unlisted_post_ids_in_space(
        &self,
        at: Option<<Block as BlockT>::Hash>,
        space_id: SpaceId,
        offset: u64,
        limit: u64
    ) -> Result<Vec<PostId>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

        let runtime_api_result = api.find_unlisted_post_ids_in_space(&at, space_id, offset, limit);
        runtime_api_result.map_err(|e| RpcError {
            // TODO: research on error codes and change a value
            code: ErrorCode::ServerError(9876), // No real reason for this value
            // TODO: change error message (?use errors macro)
            message: "Something wrong".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }
}
