//! # Moderation Module
//!
//! The Moderation module allows any user (account) to report an account, space, post or even
//! IPFS CID, if they think it's a spam, abuse or inappropriate for a specific space.
//!
//! Moderators of a space can review reported entities and suggest a moderation status for them:
//! `Block` or `Allowed`. A space owner can make a final decision: either block or allow any entity
//! within the space they control.
//!
//! This pallet also has a setting to auto-block the content after a specific number of statuses
//! from moderators that suggest to block the entity. If the entity is added to allow list,
//! then the entity cannot be blocked.
//!
//! The next rules applied to the blocked entities:
//!
//! - A post cannot be added to a space if an IPFS CID of this post is blocked in this space.
//! - An account cannot create posts in a space if this account is blocked in this space.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};
use sp_std::prelude::*;
use sp_runtime::RuntimeDebug;
use frame_support::{
    decl_module, decl_storage, decl_event, decl_error, ensure,
    dispatch::DispatchResult,
    traits::Get,
};
use frame_system::{self as system, ensure_signed};

use pallet_utils::{Content, WhoAndWhen, SpaceId, Module as Utils};
use pallet_posts::PostId;
use pallet_spaces::Module as Spaces;

/*
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;
*/

pub mod functions;

pub type ReportId = u64;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum EntityId<AccountId> {
    Content(Content),
    Account(AccountId),
    Space(SpaceId),
    Post(PostId),
}

/// Entity status is used in two cases: when moderators suggest a moderation status
/// for a reported entity; or when a space owner makes a final decision to either block
/// or allow this entity within the space.
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum EntityStatus {
    Allowed,
    Blocked,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Report<T: Trait> {
    id: ReportId,
    created: WhoAndWhen<T>,
    /// An id of reported entity: account, space, post or IPFS CID.
    reported_entity: EntityId<T::AccountId>,
    /// Within what space (scope) this entity has been reported.
    reported_within: SpaceId, // TODO rename: reported_in_space
    /// A reason should describe why this entity should be blocked in this space.
    reason: Content,
}

// TODO rename to SuggestedEntityStatus
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct SuggestedStatus<T: Trait> {
    /// An account id of a moderator who suggested this status.
    suggested: WhoAndWhen<T>,
    /// `None` if a moderator wants to signal that they have reviewed the entity,
    /// but they are not sure about what status should be applied to it.
    status: Option<EntityStatus>,
    /// `None` if a suggested status is not based on any reports.
    report_id: Option<ReportId>,
}

// TODO rename to ModerationSettings?
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct SpaceModerationSettings {
    autoblock_threshold: Option<u16>
}

// TODO rename to ModerationSettingsUpdate?
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct SpaceModerationSettingsUpdate {
    autoblock_threshold: Option<Option<u16>>
}

/// The pallet's configuration trait.
pub trait Trait: system::Trait
    + pallet_posts::Trait
    + pallet_spaces::Trait
    + pallet_space_follows::Trait
    + pallet_utils::Trait
{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    type DefaultAutoblockThreshold: Get<u16>;
}

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as ModerationModule {

        /// An id for the next report.
        pub NextReportId get(fn next_report_id): ReportId = 1;

        /// Report details by its id (key).
        pub ReportById get(fn report_by_id):
            map hasher(twox_64_concat) ReportId
            => Option<Report<T>>;

        /// Report id if entity (key 1) was reported by a specific account (key 2)
        pub ReportIdByAccount get(fn report_id_by_account):
            map hasher(twox_64_concat) (EntityId<T::AccountId>, T::AccountId)
            => Option<ReportId>;

        /// Ids of all reports in this space (key).
        pub ReportIdsBySpaceId get(fn report_ids_by_space_id):
            map hasher(twox_64_concat) SpaceId
            => Vec<ReportId>;

        /// Ids of all reports related to a specific entity (key 1) sent to this space (key 2).
        pub ReportIdsByEntityInSpace get(fn report_ids_by_entity_in_space): double_map
            hasher(twox_64_concat) EntityId<T::AccountId>,
            hasher(twox_64_concat) SpaceId
            => Vec<ReportId>;

        /// An entity (key 1) status (`Blocked` or `Allowed`) in this space (key 2).
        pub StatusByEntityInSpace get(fn status_by_entity_in_space): double_map
            hasher(twox_64_concat) EntityId<T::AccountId>,
            hasher(twox_64_concat) SpaceId
            => Option<EntityStatus>;

        /// Entity (key 1) statuses suggested by space (key 2) moderators.
        pub SuggestedStatusesByEntityInSpace get(fn suggested_statuses): double_map
            hasher(twox_64_concat) EntityId<T::AccountId>,
            hasher(twox_64_concat) SpaceId
            => Vec<SuggestedStatus<T>>;

        /// A custom moderation settings for a certain space (key).
        pub ModerationSettings get(fn moderation_settings):
            map hasher(twox_64_concat) SpaceId
            => Option<SpaceModerationSettings>;
    }
}

// The pallet's events
decl_event!(
    pub enum Event<T> where
        AccountId = <T as system::Trait>::AccountId,
        EntityId = EntityId<<T as system::Trait>::AccountId>
    {
        EntityReported(AccountId, SpaceId, EntityId, ReportId),
        EntityStatusSuggested(AccountId, SpaceId, EntityId, Option<EntityStatus>),
        EntityStatusUpdated(AccountId, SpaceId, EntityId, Option<EntityStatus>),
        EntityStatusDeleted(AccountId, SpaceId, EntityId),
        ModerationSettingsUpdated(AccountId, SpaceId),
    }
);

// The pallet's errors
decl_error! {
    pub enum Error for Module<T: Trait> {
        /// The account has already reported this entity.
        AlreadyReportedEntity,
        /// The entity has no status in this space. Nothing to delete.
        EntityHasNoStatusInScope,
        /// Entity scope differs from the scope provided.
        EntityNotInScope,
        /// Entity was not found by its id.
        EntityNotFound,
        /// Entity status is already as suggested one.
        SuggestedSameEntityStatus,
        /// Provided entity scope does not exist.
        InvalidScope,
        /// Account does not have a permission to suggest a new entity status.
        NoPermissionToSuggestEntityStatus,
        /// Account does not have a permission to update an entity status.
        NoPermissionToUpdateEntityStatus,
        /// Account does not have a permission to update the moderation settings.
        NoPermissionToUpdateModerationSettings,
        /// No updates provided for the space settings.
        NoUpdatesForModerationSettings,
        /// Report reason should not be empty.
        ReasonIsEmpty,
        /// Report was not found by its id.
        ReportNotFound,
        /// Trying to suggest an entity status in a scope that is different from the scope
        /// the entity was reported in.
        SuggestedStatusInWrongScope,
        /// Entity status has already been suggested by this moderator account.
        AlreadySuggestedEntityStatus,
    }
}

// The pallet's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        const DefaultAutoblockThreshold: u16 = T::DefaultAutoblockThreshold::get();

        // Initializing errors
        type Error = Error<T>;

        // Initializing events
        fn deposit_event() = default;

        /// Report any entity by any person with mandatory reason.
        /// `entity` scope and the `scope` provided mustn't differ
        #[weight = 10_000 + T::DbWeight::get().reads_writes(6, 5)]
        pub fn report_entity(
            origin,
            entity: EntityId<T::AccountId>,
            scope: SpaceId,
            reason: Content
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // TODO check this func, if looks strange
            Utils::<T>::ensure_content_is_some(&reason).map_err(|_| Error::<T>::ReasonIsEmpty)?;
            
            Utils::<T>::is_valid_content(reason.clone())?;

            ensure!(Spaces::<T>::require_space(scope).is_ok(), Error::<T>::InvalidScope);
            Self::ensure_entity_in_scope(&entity, scope)?;

            let not_reported_yet = Self::report_id_by_account((&entity, &who)).is_none();
            ensure!(not_reported_yet, Error::<T>::AlreadyReportedEntity);

            let report_id = Self::next_report_id();
            let new_report = Report::<T>::new(report_id, who.clone(), entity.clone(), scope, reason);

            ReportById::<T>::insert(report_id, new_report);
            ReportIdByAccount::<T>::insert((&entity, &who), report_id);
            ReportIdsBySpaceId::mutate(scope, |ids| ids.push(report_id));
            ReportIdsByEntityInSpace::<T>::mutate(&entity, scope, |ids| ids.push(report_id));
            NextReportId::mutate(|n| { *n += 1; });

            Self::deposit_event(RawEvent::EntityReported(who, scope, entity, report_id));
            Ok(())
        }

        /// Leave a feedback on the report either it's confirmation or ignore.
        /// `origin` - any permitted account (e.g. Space owner or moderator that's set via role)
        #[weight = 10_000 /* TODO + T::DbWeight::get().reads_writes(_, _) */]
        pub fn suggest_entity_status(
            origin,
            entity: EntityId<T::AccountId>,
            scope: SpaceId,
            status: Option<EntityStatus>,
            report_id_opt: Option<ReportId>
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            if let Some(report_id) = report_id_opt {
                let report = Self::require_report(report_id)?;
                ensure!(scope == report.reported_within, Error::<T>::SuggestedStatusInWrongScope);
            }

            let entity_status = StatusByEntityInSpace::<T>::get(&entity, scope);
            ensure!(!(entity_status.is_some() && status == entity_status), Error::<T>::SuggestedSameEntityStatus);

            let space = Spaces::<T>::require_space(scope).map_err(|_| Error::<T>::InvalidScope)?;
            Spaces::<T>::ensure_account_has_space_permission(
                who.clone(),
                &space,
                pallet_permissions::SpacePermission::SuggestEntityStatus,
                Error::<T>::NoPermissionToSuggestEntityStatus.into(),
            )?;

            let mut suggestions = SuggestedStatusesByEntityInSpace::<T>::get(&entity, scope);
            let is_already_suggested = suggestions.iter().any(|suggestion| suggestion.suggested.account == who);
            ensure!(!is_already_suggested, Error::<T>::AlreadySuggestedEntityStatus);
            suggestions.push(SuggestedStatus::new(who.clone(), status.clone(), report_id_opt));

            let block_suggestions_total = suggestions.iter()
                .filter(|suggestion| suggestion.status == Some(EntityStatus::Blocked))
                .count();

            let autoblock_threshold_opt = Self::moderation_settings(scope)
                .unwrap_or_else(Self::default_autoblock_threshold_as_settings)
                .autoblock_threshold;

            if let Some(autoblock_threshold) = autoblock_threshold_opt {
                if block_suggestions_total >= autoblock_threshold as usize {
                    Self::block_entity_in_scope(&entity, scope)?;
                }
            }

            Self::deposit_event(RawEvent::EntityStatusSuggested(who, scope, entity.clone(), status));
            SuggestedStatusesByEntityInSpace::<T>::insert(entity, scope, suggestions);
            Ok(())
        }

        /// Allows a space owner/admin to update the final moderation status of a reported entity.
        #[weight = 10_000 /* TODO + T::DbWeight::get().reads_writes(_, _) */]
        pub fn update_entity_status(
            origin,
            entity: EntityId<T::AccountId>,
            scope: SpaceId,
            status_opt: Option<EntityStatus>
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // TODO: add `forbid_content` parameter and track entity Content blocking via OCW
            //  - `forbid_content` - whether to block `Content` provided with entity.

            let space = Spaces::<T>::require_space(scope).map_err(|_| Error::<T>::InvalidScope)?;
            Self::ensure_account_status_manager(who.clone(), &space)?;

            if let Some(status) = &status_opt {
                let is_entity_in_scope = Self::ensure_entity_in_scope(&entity, scope).is_ok();

                if is_entity_in_scope && status == &EntityStatus::Blocked {
                    Self::block_entity_in_scope(&entity, scope)?;
                } else {
                    StatusByEntityInSpace::<T>::insert(entity.clone(), scope, status);
                }
            } else {
                StatusByEntityInSpace::<T>::remove(entity.clone(), scope);
            }

            Self::deposit_event(RawEvent::EntityStatusUpdated(who, scope, entity, status_opt));
            Ok(())
        }

        /// Allows a space owner/admin to delete a current status of a reported entity.
        #[weight = 10_000 /* TODO + T::DbWeight::get().reads_writes(_, _) */]
        pub fn delete_entity_status(
            origin,
            entity: EntityId<T::AccountId>,
            scope: SpaceId
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let status = Self::status_by_entity_in_space(&entity, scope);
            ensure!(status.is_some(), Error::<T>::EntityHasNoStatusInScope);

            let space = Spaces::<T>::require_space(scope).map_err(|_| Error::<T>::InvalidScope)?;
            Self::ensure_account_status_manager(who.clone(), &space)?;

            StatusByEntityInSpace::<T>::remove(&entity, scope);

            Self::deposit_event(RawEvent::EntityStatusDeleted(who, scope, entity));
            Ok(())
        }

        // todo: add ability to delete report_ids

        // TODO rename to update_settings?
        #[weight = 10_000 /* TODO + T::DbWeight::get().reads_writes(_, _) */]
        fn update_moderation_settings(
            origin,
            space_id: SpaceId,
            update: SpaceModerationSettingsUpdate
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let has_updates = update.autoblock_threshold.is_some();
            ensure!(has_updates, Error::<T>::NoUpdatesForModerationSettings);

            let space = Spaces::<T>::require_space(space_id)?;

            Spaces::<T>::ensure_account_has_space_permission(
                who.clone(),
                &space,
                pallet_permissions::SpacePermission::UpdateSpaceSettings,
                Error::<T>::NoPermissionToUpdateModerationSettings.into(),
            )?;

            // `true` if there is at least one updated field.
            let mut should_update = false;

            let mut settings = Self::moderation_settings(space_id)
                .unwrap_or_else(Self::default_autoblock_threshold_as_settings);

            if let Some(autoblock_threshold) = update.autoblock_threshold {
                if autoblock_threshold != settings.autoblock_threshold {
                    settings.autoblock_threshold = autoblock_threshold;
                    should_update = true;
                }
            }

            if should_update {
                ModerationSettings::insert(space_id, settings);
                Self::deposit_event(RawEvent::ModerationSettingsUpdated(who, space_id));
            }
            Ok(())
        }
    }
}
