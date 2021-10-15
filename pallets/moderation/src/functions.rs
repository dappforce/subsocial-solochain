use crate::*;

use frame_support::dispatch::DispatchError;
use pallet_posts::Module as Posts;
use pallet_space_follows::Module as SpaceFollows;
use df_traits::moderation::*;

impl<T: Config> Module<T> {
    pub fn require_report(report_id: ReportId) -> Result<Report<T>, DispatchError> {
        Ok(Self::report_by_id(report_id).ok_or(Error::<T>::ReportNotFound)?)
    }

    /// Get entity space_id if it exists.
    /// Content and Account has no scope, consider check with `if let Some`
    fn get_entity_scope(entity: &EntityId<T::AccountId>) -> Result<Option<SpaceId>, DispatchError> {
        match entity {
            EntityId::Content(content) => {
                Utils::<T>::ensure_content_is_some(content).map(|_| None)
            },
            EntityId::Account(_) => Ok(None),
            EntityId::Space(space_id) => {
                let space = Spaces::<T>::require_space(*space_id)?;
                let root_space_id = space.try_get_parent()?;

                Ok(Some(root_space_id))
            },
            EntityId::Post(post_id) => {
                let post = Posts::<T>::require_post(*post_id)?;
                let space_id = post.get_space()?.id;

                Ok(Some(space_id))
            },
        }
    }

    #[allow(dead_code)]
    // fixme: do we need this?
    fn ensure_entity_exists(entity: &EntityId<T::AccountId>) -> DispatchResult {
        match entity {
            EntityId::Content(content) => Utils::<T>::ensure_content_is_some(content),
            EntityId::Account(_) => Ok(()),
            EntityId::Space(space_id) => Spaces::<T>::ensure_space_exists(*space_id),
            EntityId::Post(post_id) => Posts::<T>::ensure_post_exists(*post_id),
        }.map_err(|_| Error::<T>::EntityNotFound.into())
    }

    pub(crate) fn kick_entity_from_scope(who: T::AccountId, entity: EntityId<T::AccountId>, scope: SpaceId) -> DispatchResult {
        Self::ensure_entity_in_scope(&entity, scope)?;

        // TODO: update counters, when entity is moved
        // TODO: think, what and where we should change something if entity is moved
        match &entity {
            EntityId::Content(_) => (),
            EntityId::Account(account_id)
                => SpaceFollows::<T>::unfollow_space_by_account(account_id.clone(), scope)?,
            EntityId::Space(space_id) => Spaces::<T>::try_move_space_to_root(*space_id)?,
            EntityId::Post(post_id) => Posts::<T>::delete_post_from_space(*post_id)?,
        }
        StatusByEntityInSpace::<T>::insert(&entity, scope, EntityStatus::Blocked);
        Self::deposit_event(RawEvent::EntityKickedFromScope(who, scope, entity));
        Ok(())
    }

    pub(crate) fn ensure_account_status_manager(who: T::AccountId, scope: SpaceId) -> DispatchResult {
        let space = Spaces::<T>::require_space(scope).map_err(|_| Error::<T>::ScopeNotFound)?;

        Spaces::<T>::ensure_account_has_space_permission(
            who,
            &space,
            pallet_permissions::SpacePermission::UpdateEntityStatus,
            Error::<T>::NoPermissionToUpdateEntityStatus.into(),
        )
    }

    pub(crate) fn ensure_entity_in_scope(entity: &EntityId<T::AccountId>, scope: SpaceId) -> DispatchResult {
        if let Some(entity_scope) = Self::get_entity_scope(entity)? {
            ensure!(entity_scope == scope, Error::<T>::EntityNotInScope);
        }
        Ok(())
    }

    pub fn default_autoblock_threshold_as_settings() -> SpaceModerationSettings {
        SpaceModerationSettings {
            autoblock_threshold: Some(T::DefaultAutoblockThreshold::get())
        }
    }
}

impl<T: Config> Report<T> {
    pub fn new(
        id: ReportId,
        created_by: T::AccountId,
        reported_entity: EntityId<T::AccountId>,
        scope: SpaceId,
        reason: Reason,
        details: Content
    ) -> Self {
        Self {
            id,
            created: WhoAndWhen::<T>::new(created_by),
            reported_entity,
            reported_within: scope,
            reason,
            details
        }
    }
}

impl<T: Config> SuggestedStatus<T> {
    pub fn new(who: T::AccountId, status: Option<EntityStatus>, report_id: Option<ReportId>) -> Self {
        Self {
            suggested: WhoAndWhen::<T>::new(who),
            status,
            report_id
        }
    }
}

// TODO: maybe simplify using one common trait?
impl<T: Config> IsAccountBlocked<T::AccountId> for Module<T> {
    fn is_blocked_account(account: T::AccountId, scope: SpaceId) -> bool {
        let entity = EntityId::Account(account);

        Self::status_by_entity_in_space(entity, scope) == Some(EntityStatus::Blocked)
    }

    fn is_allowed_account(account: T::AccountId, scope: SpaceId) -> bool {
        let entity = EntityId::Account(account);

        Self::status_by_entity_in_space(entity, scope) != Some(EntityStatus::Blocked)
    }
}

impl<T: Config> IsSpaceBlocked for Module<T> {
    fn is_blocked_space(space_id: SpaceId, scope: SpaceId) -> bool {
        let entity = EntityId::Space(space_id);

        Self::status_by_entity_in_space(entity, scope) == Some(EntityStatus::Blocked)
    }

    fn is_allowed_space(space_id: SpaceId, scope: SpaceId) -> bool {
        let entity = EntityId::Space(space_id);

        Self::status_by_entity_in_space(entity, scope) != Some(EntityStatus::Blocked)
    }
}

impl<T: Config> IsPostBlocked<PostId> for Module<T> {
    fn is_blocked_post(post_id: PostId, scope: SpaceId) -> bool {
        let entity = EntityId::Post(post_id);

        Self::status_by_entity_in_space(entity, scope) == Some(EntityStatus::Blocked)
    }

    fn is_allowed_post(post_id: PostId, scope: SpaceId) -> bool {
        let entity = EntityId::Post(post_id);

        Self::status_by_entity_in_space(entity, scope) != Some(EntityStatus::Blocked)
    }
}

impl<T: Config> IsContentBlocked for Module<T> {
    fn is_blocked_content(content: Content, scope: SpaceId) -> bool {
        let entity = EntityId::Content(content);

        Self::status_by_entity_in_space(entity, scope) == Some(EntityStatus::Blocked)
    }

    fn is_allowed_content(content: Content, scope: SpaceId) -> bool {
        let entity = EntityId::Content(content);

        Self::status_by_entity_in_space(entity, scope) != Some(EntityStatus::Blocked)
    }
}
