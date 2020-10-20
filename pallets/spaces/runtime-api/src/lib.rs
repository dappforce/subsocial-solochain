#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]

use sp_std::vec::Vec;
use codec::Codec;
use sp_runtime::traits::{MaybeDisplay, MaybeFromStr};

use pallet_utils::SpaceId;

sp_api::decl_runtime_apis! {
    pub trait SpacesApi<Space>
        where Space: core::fmt::Debug + Codec + MaybeDisplay + MaybeFromStr
    {
        fn get_last_space() -> Option<Space>;

        fn get_hidden_space_ids(limit_opt: Option<u64>, offset_opt: Option<u64>) -> Vec<SpaceId>;
    }
}
