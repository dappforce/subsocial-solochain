use std::sync::Arc;
use codec::Codec;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;
use pallet_utils::SpaceId;
use pallet_permissions::SpacePermission;

pub use roles_runtime_api::RolesApi as RolesRuntimeApi;

#[rpc]
pub trait RolesApi<BlockHash, AccountId> {
    #[rpc(name = "roles_getSpacePermissionsByAccount")]
    fn get_space_permissions_by_user(
        &self,
        at: Option<BlockHash>,
        account: AccountId,
        space_id: SpaceId
    ) -> Result<Vec<SpacePermission>>;
}

pub struct Roles<C, M> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<M>,
}

impl<C, M> Roles<C, M> {
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

impl<C, Block, AccountId> RolesApi<<Block as BlockT>::Hash, AccountId>
    for Roles<C, Block>
where
    Block: BlockT,
    AccountId: Codec,
    C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: RolesRuntimeApi<Block, AccountId>,
{
    fn get_space_permissions_by_user(
        &self, at:
        Option<<Block as BlockT>::Hash>,
        account: AccountId,
        space_id: SpaceId
    ) -> Result<Vec<SpacePermission>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_space_permissions_by_user(&at, account, space_id);
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
