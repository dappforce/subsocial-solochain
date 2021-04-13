use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_std::prelude::*;

use pallet_utils::{SpaceId, rpc::{FlatContent, FlatWhoAndWhen, ShouldSkip}};

use crate::{Module, Space, Config};

#[derive(Eq, PartialEq, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct FlatSpace<AccountId, BlockNumber> {
    pub id: SpaceId,
    #[cfg_attr(feature = "std", serde(flatten))]
    pub who_and_when: FlatWhoAndWhen<AccountId, BlockNumber>,

    pub owner_id: AccountId,

    #[cfg_attr(feature = "std", serde(skip_serializing_if = "ShouldSkip::should_skip"))]
    pub parent_id: Option<SpaceId>,
    #[cfg_attr(feature = "std", serde(skip_serializing_if = "ShouldSkip::should_skip"))]
    #[cfg_attr(feature = "std", serde(serialize_with = "bytes_to_string"))]
    pub handle: Option<Vec<u8>>,
    #[cfg_attr(feature = "std", serde(flatten))]
    pub content: FlatContent,

    // #[cfg_attr(feature = "std", serde(skip_serializing_if = "ShouldSkip::should_skip"))]
    pub is_hidden: bool,

    pub posts_count: u32,
    pub hidden_posts_count: u32,
    pub visible_posts_count: u32,
    pub followers_count: u32,

    pub score: i32,
}

#[cfg(feature = "std")]
fn bytes_to_string<S>(field: &Option<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
    let field_unwrapped = field.clone().unwrap_or_default();
    // If Bytes slice is invalid, then empty string will be returned
    serializer.serialize_str(
        std::str::from_utf8(&field_unwrapped).unwrap_or_default()
    )
}

impl<T: Config> From<Space<T>> for FlatSpace<T::AccountId, T::BlockNumber> {
    fn from(from: Space<T>) -> Self {
        let Space {
            id, created, updated, owner,
            parent_id, handle, content, hidden, posts_count,
            hidden_posts_count, followers_count, score, ..
        } = from;

        Self {
            id,
            who_and_when: (created, updated).into(),
            owner_id: owner,
            parent_id,
            handle,
            content: content.into(),
            is_hidden: hidden,
            posts_count,
            hidden_posts_count,
            visible_posts_count: posts_count.saturating_sub(hidden_posts_count),
            followers_count,
            score,
        }
    }
}

impl<T: Config> Module<T> {
    pub fn get_spaces_by_ids(space_ids: Vec<SpaceId>) -> Vec<FlatSpace<T::AccountId, T::BlockNumber>> {
        space_ids.iter()
            .filter_map(|id| Self::require_space(*id).ok())
            .map(|space| space.into())
            .collect()
    }

    fn get_spaces_slice<F: FnMut(&Space<T>) -> bool>(
        offset: u64,
        limit: u64,
        mut filter: F,
    ) -> Vec<FlatSpace<T::AccountId, T::BlockNumber>> {
        let mut start_from = offset;
        let mut iterate_until = offset;
        let last_space_id = Self::next_space_id().saturating_sub(1);

        let mut spaces = Vec::new();

        'outer: loop {
            iterate_until = iterate_until.saturating_add(limit);

            if start_from > last_space_id {
                break;
            }

            if iterate_until > last_space_id {
                iterate_until = last_space_id;
            }

            for space_id in start_from..=iterate_until {
                if let Some(space) = Self::require_space(space_id).ok() {
                    if filter(&space) {
                        spaces.push(space.into());
                        if spaces.len() >= limit as usize { break 'outer; }
                    }
                }
            }
            start_from = iterate_until;
        }

        spaces
    }

    pub fn get_spaces(offset: u64, limit: u64) -> Vec<FlatSpace<T::AccountId, T::BlockNumber>> {
        Self::get_spaces_slice(offset, limit, |_| true)
    }

    pub fn get_public_spaces(offset: u64, limit: u64) -> Vec<FlatSpace<T::AccountId, T::BlockNumber>> {
        Self::get_spaces_slice(offset, limit, |space| space.is_public())
    }

    pub fn get_unlisted_spaces(offset: u64, limit: u64) -> Vec<FlatSpace<T::AccountId, T::BlockNumber>> {
        Self::get_spaces_slice(offset, limit, |space| !space.is_public())
    }

    pub fn get_space_id_by_handle(handle: Vec<u8>) -> Option<SpaceId> {
        Self::space_id_by_handle(handle)
    }

    pub fn get_space_by_handle(handle: Vec<u8>) -> Option<FlatSpace<T::AccountId, T::BlockNumber>> {
        Self::space_id_by_handle(handle)
            .and_then(|space_id| Self::require_space(space_id).ok())
            .map(|space| space.into())
    }

    fn get_space_ids_by_owner<F: FnMut(&Space<T>) -> bool>(owner: T::AccountId, mut compare_fn: F) -> Vec<SpaceId> {
        Self::space_ids_by_owner(owner)
            .iter()
            .filter_map(|space_id| Self::require_space(*space_id).ok())
            .filter(|space| compare_fn(space))
            .map(|space| space.id)
            .collect()
    }

    pub fn get_public_space_ids_by_owner(owner: T::AccountId) -> Vec<SpaceId> {
        Self::get_space_ids_by_owner(owner, |space| !space.hidden)
    }

    pub fn get_unlisted_space_ids_by_owner(owner: T::AccountId) -> Vec<SpaceId> {
        Self::get_space_ids_by_owner(owner, |space| space.hidden)
    }


    pub fn get_next_space_id() -> SpaceId {
        Self::next_space_id()
    }
}