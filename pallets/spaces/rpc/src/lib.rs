//! RPC interface for the transaction payment module.

use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;
use spaces_runtime_api::SpacesApi as SpacesRuntimeApi;

use pallet_utils::SpaceId;

#[rpc]
pub trait SpacesApi<BlockHash> {
    #[rpc(name = "spaces_getLastSpaceId")]
    fn get_last_space_id(&self, at: Option<BlockHash>) -> Result<SpaceId>;

    #[rpc(name = "spaces_getHiddenSpaceIds")]
    fn get_hidden_space_ids(
        &self,
        at: Option<BlockHash>,
        limit_opt: Option<u64>,
        offset_opt: Option<u64>
    ) -> Result<Vec<SpaceId>>;
}

pub struct Spaces<C, M> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<M>,
}

impl<C, M> Spaces<C, M> {
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

impl<C, Block> SpacesApi<<Block as BlockT>::Hash> for Spaces<C, Block>
    where
        Block: BlockT,
        C: Send + Sync + 'static,
        C: ProvideRuntimeApi<Block>,
        C: HeaderBackend<Block>,
        C::Api: SpacesRuntimeApi<Block>,
{
    fn get_last_space_id(&self, at: Option<<Block as BlockT>::Hash>) -> Result<SpaceId> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

        let runtime_api_result = api.get_last_space_id(&at);
        runtime_api_result.map_err(|e| RpcError {
            code: ErrorCode::ServerError(9876), // No real reason for this value
            message: "Something wrong".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }

    fn get_hidden_space_ids(
        &self,
        at: Option<<Block as BlockT>::Hash>,
        limit_opt: Option<u64>,
        offset_opt: Option<u64>
    ) -> Result<Vec<SpaceId>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

        let runtime_api_result = api.get_hidden_space_ids(&at, limit_opt, offset_opt);
        runtime_api_result.map_err(|e| RpcError {
            // TODO: research on error codes and change a value
            code: ErrorCode::ServerError(9876), // No real reason for this value
            // TODO: change error message (?use errors macro)
            message: "Something wrong".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }
}
