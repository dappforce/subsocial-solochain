use std::sync::Arc;
use codec::Codec;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;

use pallet_spaces::rpc::FlatSpace;
use pallet_utils::SpaceId;
pub use spaces_runtime_api::SpacesApi as SpacesRuntimeApi;

#[rpc]
pub trait SpacesApi<BlockHash, AccountId, BlockNumber> {
    #[rpc(name = "spaces_getSpaces")]
    fn get_spaces(
        &self,
        at: Option<BlockHash>,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<FlatSpace<AccountId, BlockNumber>>>;

    #[rpc(name = "spaces_getSpacesByIds")]
    fn get_spaces_by_ids(
        &self,
        at: Option<BlockHash>,
        space_ids: Vec<SpaceId>,
    ) -> Result<Vec<FlatSpace<AccountId, BlockNumber>>>;

    #[rpc(name = "spaces_getPublicSpaces")]
    fn get_public_spaces(
        &self,
        at: Option<BlockHash>,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<FlatSpace<AccountId, BlockNumber>>>;

    #[rpc(name = "spaces_getUnlistedSpaces")]
    fn get_unlisted_spaces(
        &self,
        at: Option<BlockHash>,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<FlatSpace<AccountId, BlockNumber>>>;

    #[rpc(name = "spaces_getSpaceIdByHandle")]
    fn get_space_id_by_handle(
        &self,
        at: Option<BlockHash>,
        handle: Vec<u8>,
    ) -> Result<Option<SpaceId>>;

    #[rpc(name = "spaces_getSpaceByHandle")]
    fn get_space_by_handle(
        &self,
        at: Option<BlockHash>,
        handle: Vec<u8>,
    ) -> Result<Option<FlatSpace<AccountId, BlockNumber>>>;

    #[rpc(name = "spaces_getPublicSpaceIdsByOwner")]
    fn get_public_space_ids_by_owner(
        &self,
        at: Option<BlockHash>,
        owner: AccountId,
    ) -> Result<Vec<SpaceId>>;

    #[rpc(name = "spaces_getUnlistedSpaceIdsByOwner")]
    fn get_unlisted_space_ids_by_owner(
        &self,
        at: Option<BlockHash>,
        owner: AccountId,
    ) -> Result<Vec<SpaceId>>;

    #[rpc(name = "spaces_nextSpaceId")]
    fn get_next_space_id(&self, at: Option<BlockHash>) -> Result<SpaceId>;
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

impl<C, Block, AccountId, BlockNumber> SpacesApi<<Block as BlockT>::Hash, AccountId, BlockNumber>
    for Spaces<C, Block>
where
    Block: BlockT,
    AccountId: Codec,
    BlockNumber: Codec,
    C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: SpacesRuntimeApi<Block, AccountId, BlockNumber>,
{
    fn get_spaces(
        &self,
        at: Option<<Block as BlockT>::Hash>,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<FlatSpace<AccountId, BlockNumber>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_spaces(&at, offset, limit);
        runtime_api_result.map_err(map_rpc_error)
    }

    fn get_spaces_by_ids(
        &self,
        at: Option<<Block as BlockT>::Hash>,
        space_ids: Vec<SpaceId>,
    ) -> Result<Vec<FlatSpace<AccountId, BlockNumber>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_spaces_by_ids(&at, space_ids);
        runtime_api_result.map_err(map_rpc_error)
    }

    fn get_public_spaces(
        &self,
        at: Option<<Block as BlockT>::Hash>,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<FlatSpace<AccountId, BlockNumber>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_public_spaces(&at, offset, limit);
        runtime_api_result.map_err(map_rpc_error)
    }

    fn get_unlisted_spaces(
        &self,
        at: Option<<Block as BlockT>::Hash>,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<FlatSpace<AccountId, BlockNumber>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_unlisted_spaces(&at, offset, limit);
        runtime_api_result.map_err(map_rpc_error)
    }

    fn get_space_id_by_handle(&self, at: Option<<Block as BlockT>::Hash>, handle: Vec<u8>) -> Result<Option<u64>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_space_id_by_handle(&at, handle);
        runtime_api_result.map_err(map_rpc_error)
    }

    fn get_space_by_handle(
        &self,
        at: Option<<Block as BlockT>::Hash>,
        handle: Vec<u8>,
    ) -> Result<Option<FlatSpace<AccountId, BlockNumber>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_space_by_handle(&at, handle);
        runtime_api_result.map_err(map_rpc_error)
    }

    fn get_public_space_ids_by_owner(&self, at: Option<<Block as BlockT>::Hash>, owner: AccountId) -> Result<Vec<u64>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_public_space_ids_by_owner(&at, owner);
        runtime_api_result.map_err(map_rpc_error)
    }

    fn get_unlisted_space_ids_by_owner(&self, at: Option<<Block as BlockT>::Hash>, owner: AccountId) -> Result<Vec<u64>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_unlisted_space_ids_by_owner(&at, owner);
        runtime_api_result.map_err(map_rpc_error)
    }

    fn get_next_space_id(&self, at: Option<<Block as BlockT>::Hash>) -> Result<u64> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_next_space_id(&at);
        runtime_api_result.map_err(map_rpc_error)
    }
}

// TODO: move this copy-paste code to a common file
fn map_rpc_error(err: impl std::fmt::Debug) -> RpcError {
    RpcError {
        code: ErrorCode::ServerError(1),
        message: "An RPC error occurred".into(),
        data: Some(format!("{:?}", err).into()),
    }
}
