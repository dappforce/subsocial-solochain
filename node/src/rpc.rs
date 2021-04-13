//! A collection of node-specific RPC methods.
//! Substrate provides the `sc-rpc` crate, which defines the core RPC layer
//! used by Substrate nodes. This file extends those RPC definitions with
//! capabilities that are specific to this project's runtime configuration.

#![warn(missing_docs)]

use std::sync::Arc;

use subsocial_primitives::{Block, AccountId, Balance, Index, BlockNumber};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::{Error as BlockChainError, HeaderMetadata, HeaderBackend};
use sp_block_builder::BlockBuilder;
pub use sc_rpc_api::DenyUnsafe;
use sp_transaction_pool::TransactionPool;


/// Full client dependencies.
pub struct FullDeps<C, P> {
    /// The client instance to use.
    pub client: Arc<C>,
    /// Transaction pool instance.
    pub pool: Arc<P>,
    /// Whether to deny unsafe calls
    pub deny_unsafe: DenyUnsafe,
}

/// Instantiate all full RPC extensions.
pub fn create_full<C, P>(
    deps: FullDeps<C, P>,
) -> jsonrpc_core::IoHandler<sc_rpc::Metadata> where
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block> + HeaderMetadata<Block, Error=BlockChainError> + 'static,
    C: Send + Sync + 'static,
    C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>,
    C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
    C::Api: posts_rpc::PostsRuntimeApi<Block, AccountId, BlockNumber>,
    C::Api: profile_follows_rpc::ProfileFollowsRuntimeApi<Block, AccountId>,
    C::Api: profiles_rpc::ProfilesRuntimeApi<Block, AccountId, BlockNumber>,
    C::Api: reactions_rpc::ReactionsRuntimeApi<Block, AccountId, BlockNumber>,
    C::Api: space_follows_rpc::SpaceFollowsRuntimeApi<Block, AccountId>,
    C::Api: spaces_rpc::SpacesRuntimeApi<Block, AccountId, BlockNumber>,
    C::Api: roles_rpc::RolesRuntimeApi<Block, AccountId>,
    C::Api: BlockBuilder<Block>,
    P: TransactionPool + 'static,
{
    use substrate_frame_rpc_system::{FullSystem, SystemApi};
    use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApi};

    use posts_rpc::{Posts, PostsApi};
    use profile_follows_rpc::{ProfileFollows, ProfileFollowsApi};
    use profiles_rpc::{Profiles, ProfilesApi};
    use reactions_rpc::{Reactions, ReactionsApi};
    use space_follows_rpc::{SpaceFollows, SpaceFollowsApi};
    use spaces_rpc::{Spaces, SpacesApi};
    use roles_rpc::{Roles, RolesApi};

    let mut io = jsonrpc_core::IoHandler::default();
    let FullDeps {
        client,
        pool,
        deny_unsafe,
    } = deps;

    io.extend_with(
        SystemApi::to_delegate(FullSystem::new(client.clone(), pool, deny_unsafe))
    );

    io.extend_with(
        TransactionPaymentApi::to_delegate(TransactionPayment::new(client.clone()))
    );

    io.extend_with(
        SpacesApi::to_delegate(Spaces::new(client.clone()),
    ));

    io.extend_with(
    SpaceFollowsApi::to_delegate(SpaceFollows::new(client.clone()),
    ));

    io.extend_with(
        PostsApi::to_delegate(Posts::new(client.clone()),
    ));

    io.extend_with(
        ProfileFollowsApi::to_delegate(ProfileFollows::new(client.clone()),
    ));

    io.extend_with(
        ProfilesApi::to_delegate(Profiles::new(client.clone()),
    ));

    io.extend_with(
        ReactionsApi::to_delegate(Reactions::new(client.clone()),
    ));

    io.extend_with(
        RolesApi::to_delegate(Roles::new(client.clone()),
    ));

    io
}
