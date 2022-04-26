use std::sync::Arc;
use codec::{Codec, Encode};
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::{Block as BlockT, Header as HeaderT, Zero}};
use jsonrpc_core::Result;
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;
use free_calls_runtime_api::NumberOfCalls;

use pallet_utils::rpc::map_rpc_error;
pub use free_calls_runtime_api::FreeCallsApi as FreeCallsRuntimeApi;

#[rpc]
pub trait FreeCallsApi<BlockHash, AccountId> {
    #[rpc(name = "freeCalls_getMaxQuota")]
    fn get_max_quota(
        &self,
        at: Option<BlockHash>,
        account: AccountId,
    ) -> Result<NumberOfCalls>;

    #[rpc(name = "freeCalls_canMakeFreeCall")]
    fn can_make_free_call(
        &self,
        at: Option<BlockHash>,
        account: AccountId,
    ) -> Result<bool>;
}

pub struct FreeCalls<C, M> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<M>,
}

impl<C, M> FreeCalls<C, M> {
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

impl<C, Block, AccountId> FreeCallsApi<<Block as BlockT>::Hash, AccountId>
    for FreeCalls<C, Block>
where
    Block: BlockT,
    AccountId: Codec,
    C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: FreeCallsRuntimeApi<Block, AccountId, <<Block as BlockT>::Header as HeaderT>::Number>,
{
    fn get_max_quota(
        &self,
        at: Option<<Block as BlockT>::Hash>,
        account: AccountId,
    ) -> Result<NumberOfCalls> {
        let api = self.client.runtime_api();
        let at_number = self.client.info().best_number;
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_max_quota(&at, account, at_number);
        runtime_api_result.map_err(map_rpc_error)
    }

    fn can_make_free_call(
        &self,
        at: Option<<Block as BlockT>::Hash>,
        account: AccountId,
    ) -> Result<bool> {
        let api = self.client.runtime_api();
        let at_number = self.client.info().best_number;
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.can_make_free_call(&at, account, at_number);
        runtime_api_result.map_err(map_rpc_error)
    }
}
