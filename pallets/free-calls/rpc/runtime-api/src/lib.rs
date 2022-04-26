#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;
use sp_std::vec::Vec;
pub use pallet_free_calls::quota::NumberOfCalls;

sp_api::decl_runtime_apis! {
    pub trait FreeCallsApi<AccountId, BlockNumber> where
        AccountId: Codec,
        BlockNumber: Codec,
    {
        fn get_max_quota(account: AccountId, block_number: BlockNumber) -> NumberOfCalls;

        fn can_make_free_call(account: AccountId, block_number: BlockNumber) -> bool;
    }
}
