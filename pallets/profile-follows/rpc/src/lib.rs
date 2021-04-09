use std::sync::Arc;
use codec::Codec;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;

pub use profile_follows_runtime_api::ProfileFollowsApi as ProfileFollowsRuntimeApi;

#[rpc]
pub trait ProfileFollowsApi<BlockHash, AccountId> {
    #[rpc(name = "profileFollows_filterFollowedAccounts")]
    fn filter_followed_accounts(
        &self,
        at: Option<BlockHash>,
        account: AccountId,
        other_accounts: Vec<AccountId>,
    ) -> Result<Vec<AccountId>>;
}

pub struct ProfileFollows<C, M> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<M>,
}

impl<C, M> ProfileFollows<C, M> {
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

impl<C, Block, AccountId> ProfileFollowsApi<<Block as BlockT>::Hash, AccountId>
    for ProfileFollows<C, Block>
where
    Block: BlockT,
    AccountId: Codec,
    C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: ProfileFollowsRuntimeApi<Block, AccountId>,
{
    fn filter_followed_accounts(
        &self, at:
        Option<<Block as BlockT>::Hash>,
        account: AccountId,
        other_accounts: Vec<AccountId>,
    ) -> Result<Vec<AccountId>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.filter_followed_accounts(&at, account, other_accounts);
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
