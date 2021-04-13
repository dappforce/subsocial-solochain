use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::SaturatedConversion;
use sp_std::prelude::*;

use crate::{Content, from_bool_to_option, Config, WhoAndWhen};

#[derive(Eq, PartialEq, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct FlatWhoAndWhen<AccountId, BlockNumber> {
    pub created_by: AccountId,
    pub created_at_block: BlockNumber,
    pub created_at_time: u64,
    #[cfg_attr(feature = "std", serde(skip_serializing_if = "ShouldSkip::should_skip"))]
    pub is_updated: Option<bool>,
    #[cfg_attr(feature = "std", serde(skip_serializing_if = "ShouldSkip::should_skip"))]
    pub updated_by: Option<AccountId>,
    #[cfg_attr(feature = "std", serde(skip_serializing_if = "ShouldSkip::should_skip"))]
    pub updated_at_block: Option<BlockNumber>,
    #[cfg_attr(feature = "std", serde(skip_serializing_if = "ShouldSkip::should_skip"))]
    pub updated_at_time: Option<u64>,
}

impl<T: Config> From<(WhoAndWhen<T>, Option<WhoAndWhen<T>>)> for FlatWhoAndWhen<T::AccountId, T::BlockNumber> {
    fn from(created_and_updated: (WhoAndWhen<T>, Option<WhoAndWhen<T>>)) -> Self {
        let (created, updated) = created_and_updated;
        Self {
            created_by: created.account,
            created_at_block: created.block,
            created_at_time: created.time.saturated_into::<u64>(),
            is_updated: from_bool_to_option(updated.is_some()),
            updated_by: updated.clone().map(|value| value.account),
            updated_at_block: updated.clone().map(|value| value.block),
            updated_at_time: updated.map(|value| value.time.saturated_into::<u64>()),
        }
    }
}

impl<T: Config> From<WhoAndWhen<T>> for FlatWhoAndWhen<T::AccountId, T::BlockNumber> {
    fn from(created: WhoAndWhen<T>) -> Self {
        Self {
            created_by: created.account,
            created_at_block: created.block,
            created_at_time: created.time.saturated_into::<u64>(),
            is_updated: None,
            updated_by: None,
            updated_at_block: None,
            updated_at_time: None,
        }
    }
}

#[derive(Eq, PartialEq, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct FlatContent {
    pub content_id: Content,
    #[cfg_attr(feature = "std", serde(skip_serializing_if = "ShouldSkip::should_skip"))]
    pub is_ipfs_content: Option<bool>,
}

#[cfg(feature = "std")]
impl Serialize for Content {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
    {
        let content_vec: Vec<u8> = self.clone().into();

        // If Bytes slice is invalid, then empty string will be returned
        serializer.serialize_str(
            std::str::from_utf8(&content_vec).unwrap_or_default()
        )
    }
}

impl From<Content> for FlatContent {
    fn from(content: Content) -> Self {
        Self {
            content_id: content.clone(),
            is_ipfs_content: from_bool_to_option(content.is_ipfs()),
        }
    }
}

pub trait ShouldSkip {
    fn should_skip(&self) -> bool;
}
impl<T> ShouldSkip for Option<T> {
    fn should_skip(&self) -> bool {
        self.is_none()
    }
}
