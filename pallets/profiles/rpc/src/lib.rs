use std::sync::Arc;
use codec::Codec;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;

use pallet_profiles::rpc::FlatSocialAccount;
pub use profiles_runtime_api::ProfilesApi as ProfilesRuntimeApi;

#[rpc]
pub trait ProfilesApi<BlockHash, AccountId, BlockNumber> {
    #[rpc(name = "profiles_getSocialAccountsByIds")]
    fn get_social_accounts_by_ids(
        &self,
        at: Option<BlockHash>,
        account_ids: Vec<AccountId>,
    ) -> Result<Vec<FlatSocialAccount<AccountId, BlockNumber>>>;
}

pub struct Profiles<C, M> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<M>,
}

impl<C, M> Profiles<C, M> {
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

impl<C, Block, AccountId, BlockNumber> ProfilesApi<<Block as BlockT>::Hash, AccountId, BlockNumber>
    for Profiles<C, Block>
where
    Block: BlockT,
    AccountId: Codec,
    BlockNumber: Codec,
    C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: ProfilesRuntimeApi<Block, AccountId, BlockNumber>,
{
    fn get_social_accounts_by_ids(&self, at: Option<<Block as BlockT>::Hash>, account_ids: Vec<AccountId>) -> Result<Vec<FlatSocialAccount<AccountId, BlockNumber>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_social_accounts_by_ids(&at, account_ids);
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
