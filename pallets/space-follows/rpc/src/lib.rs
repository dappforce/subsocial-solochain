use std::sync::Arc;
use codec::Codec;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;

use pallet_utils::SpaceId;
pub use space_follows_runtime_api::SpaceFollowsApi as SpaceFollowsRuntimeApi;

#[rpc]
pub trait SpaceFollowsApi<BlockHash, AccountId> {
    #[rpc(name = "spaceFollows_getSpaceIdsFollowedByAccount")]
    fn get_space_ids_followed_by_account(
        &self,
        at: Option<BlockHash>,
        account: AccountId,
    ) -> Result<Vec<SpaceId>>;

    #[rpc(name = "spaceFollows_filterSpacesFollowed")]
    fn filter_followed_spaces(
        &self,
        at: Option<BlockHash>,
        account: AccountId,
        space_ids: Vec<SpaceId>,
    ) -> Result<Vec<SpaceId>>;
}

pub struct SpaceFollows<C, M> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<M>,
}

impl<C, M> SpaceFollows<C, M> {
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

impl<C, Block, AccountId> SpaceFollowsApi<<Block as BlockT>::Hash, AccountId>
    for SpaceFollows<C, Block>
where
    Block: BlockT,
    AccountId: Codec,
    C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: SpaceFollowsRuntimeApi<Block, AccountId>,
{
    fn get_space_ids_followed_by_account(
        &self,
        at: Option<<Block as BlockT>::Hash>,
        account: AccountId,
    ) -> Result<Vec<SpaceId>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_space_ids_followed_by_account(&at, account);
        runtime_api_result.map_err(map_rpc_error)
    }

    fn filter_followed_spaces(
        &self,
        at: Option<<Block as BlockT>::Hash>,
        account: AccountId,
        space_ids: Vec<u64>,
    ) -> Result<Vec<SpaceId>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.filter_followed_spaces(&at, account, space_ids);
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
