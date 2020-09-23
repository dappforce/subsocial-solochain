#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, ensure,
    dispatch::{DispatchError, DispatchResult},
    traits::{Get, Currency, ExistenceRequirement},
};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;
use frame_system::{self as system, ensure_signed};

use df_traits::{
    SpaceForRoles, SpaceForRolesProvider, PermissionChecker, SpaceFollowsProvider,
    moderation::{IsAccountBlocked, IsContentBlocked},
};
use pallet_permissions::{SpacePermission, SpacePermissions, SpacePermissionsContext};
use pallet_utils::{Module as Utils, Error as UtilsError, SpaceId, WhoAndWhen, Content};

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Space<T: Trait> {
    pub id: SpaceId,
    pub created: WhoAndWhen<T>,
    pub updated: Option<WhoAndWhen<T>>,

    pub owner: T::AccountId,

    // Can be updated by the owner:
    pub parent_id: Option<SpaceId>,
    pub handle: Option<Vec<u8>>,
    pub content: Content,
    pub hidden: bool,

    pub posts_count: u32,
    pub hidden_posts_count: u32,
    pub followers_count: u32,

    pub score: i32,

    /// Allows to override the default permissions for this space.
    pub permissions: Option<SpacePermissions>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
#[allow(clippy::option_option)]
pub struct SpaceUpdate {
    pub parent_id: Option<Option<SpaceId>>,
    pub handle: Option<Option<Vec<u8>>>,
    pub content: Option<Content>,
    pub hidden: Option<bool>,
    pub permissions: Option<Option<SpacePermissions>>,
}

type BalanceOf<T> = <<T as pallet_utils::Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

/// The pallet's configuration trait.
pub trait Trait: system::Trait
    + pallet_utils::Trait
    + pallet_permissions::Trait
{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    type Roles: PermissionChecker<AccountId=Self::AccountId>;

    type SpaceFollows: SpaceFollowsProvider<AccountId=Self::AccountId>;

    type BeforeSpaceCreated: BeforeSpaceCreated<Self>;

    type AfterSpaceUpdated: AfterSpaceUpdated<Self>;

    type IsAccountBlocked: IsAccountBlocked<AccountId=Self::AccountId>;

    type IsContentBlocked: IsContentBlocked;

    type SpaceCreationFee: Get<BalanceOf<Self>>;

    type DefaultRPCLimit: Get<u64>;
}

decl_error! {
  pub enum Error for Module<T: Trait> {
    /// Space was not found by id.
    SpaceNotFound,
    /// Space handle is not unique.
    SpaceHandleIsNotUnique,
    /// Nothing to update in space.
    NoUpdatesForSpace,
    /// Only space owner can manage their space.
    NotASpaceOwner,
    /// User has no permission to update this space.
    NoPermissionToUpdateSpace,
    /// User has no permission to create subspaces in this space
    NoPermissionToCreateSubspaces,
    /// Space is at root level, no parent_id specified
    SpaceIsAtRoot,
  }
}

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as SpacesModule {

        pub NextSpaceId get(fn next_space_id): SpaceId = 1001;

        pub SpaceById get(fn space_by_id) build(|config: &GenesisConfig<T>| {
          let mut spaces: Vec<(SpaceId, Space<T>)> = Vec::new();
          let endowed_account = config.endowed_account.clone();
          for id in 1..=1000 {
            spaces.push((id, Space::<T>::new(id, None, endowed_account.clone(), Content::None, None)));
          }
          spaces
        }):
            map hasher(twox_64_concat) SpaceId => Option<Space<T>>;

        pub SpaceIdByHandle get(fn space_id_by_handle):
            map hasher(blake2_128_concat) Vec<u8> => Option<SpaceId>;

        pub SpaceIdsByOwner get(fn space_ids_by_owner):
            map hasher(twox_64_concat) T::AccountId => Vec<SpaceId>;
    }
    add_extra_genesis {
      config(endowed_account): T::AccountId;
    }
}

decl_event!(
    pub enum Event<T> where
        <T as system::Trait>::AccountId,
    {
        SpaceCreated(AccountId, SpaceId),
        SpaceUpdated(AccountId, SpaceId),
        SpaceDeleted(AccountId, SpaceId),
    }
);

// The pallet's dispatchable functions.
decl_module! {
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {

    const SpaceCreationFee: BalanceOf<T> = T::SpaceCreationFee::get();

    const DefaultRPCLimit: u64 = T::DefaultRPCLimit::get();

    // Initializing errors
    type Error = Error<T>;

    // Initializing events
    fn deposit_event() = default;

    #[weight = 500_000 + T::DbWeight::get().reads_writes(4, 4)]
    pub fn create_space(
      origin,
      parent_id_opt: Option<SpaceId>,
      handle_opt: Option<Vec<u8>>,
      content: Content
    ) -> DispatchResult {
      let owner = ensure_signed(origin)?;

      Utils::<T>::is_valid_content(content.clone())?;

      let mut handle_in_lowercase: Vec<u8> = Vec::new();
      if let Some(original_handle) = handle_opt.clone() {
        handle_in_lowercase = Self::lowercase_and_validate_space_handle(original_handle)?;
      }

      // TODO: add tests for this case
      if let Some(parent_id) = parent_id_opt {
        let parent_space = Self::require_space(parent_id)?;

        ensure!(!T::IsAccountBlocked::is_account_blocked(owner.clone(), parent_id), UtilsError::<T>::AccountIsBlocked);
        ensure!(!T::IsContentBlocked::is_content_blocked(content.clone(), parent_id), UtilsError::<T>::ContentIsBlocked);

        Self::ensure_account_has_space_permission(
          owner.clone(),
          &parent_space,
          SpacePermission::CreateSubspaces,
          Error::<T>::NoPermissionToCreateSubspaces.into()
        )?;
      }

      <T as pallet_utils::Trait>::Currency::transfer(
        &owner,
        &Utils::<T>::treasury_account(),
        T::SpaceCreationFee::get(),
        ExistenceRequirement::KeepAlive
      )?;

      let space_id = Self::next_space_id();
      let new_space = &mut Space::new(space_id, parent_id_opt, owner.clone(), content, handle_opt);

      T::BeforeSpaceCreated::before_space_created(owner.clone(), new_space)?;

      <SpaceById<T>>::insert(space_id, new_space);
      <SpaceIdsByOwner<T>>::mutate(owner.clone(), |ids| ids.push(space_id));
      NextSpaceId::mutate(|n| { *n += 1; });

      if !handle_in_lowercase.is_empty() {
        SpaceIdByHandle::insert(handle_in_lowercase, space_id);
      }

      Self::deposit_event(RawEvent::SpaceCreated(owner, space_id));
      Ok(())
    }

    #[weight = 500_000 + T::DbWeight::get().reads_writes(2, 3)]
    pub fn update_space(origin, space_id: SpaceId, update: SpaceUpdate) -> DispatchResult {
      let owner = ensure_signed(origin)?;

      let has_updates =
        update.parent_id.is_some() ||
        update.handle.is_some() ||
        update.content.is_some() ||
        update.hidden.is_some() ||
        update.permissions.is_some();

      ensure!(has_updates, Error::<T>::NoUpdatesForSpace);

      let mut space = Self::require_space(space_id)?;

      ensure!(!T::IsAccountBlocked::is_account_blocked(owner.clone(), space.id), UtilsError::<T>::AccountIsBlocked);

      Self::ensure_account_has_space_permission(
        owner.clone(),
        &space,
        SpacePermission::UpdateSpace,
        Error::<T>::NoPermissionToUpdateSpace.into()
      )?;

      let mut is_update_applied = false;
      let mut old_data = SpaceUpdate::default();

      // TODO: add tests for this case
      if let Some(parent_id_opt) = update.parent_id {
        if parent_id_opt != space.parent_id {

          if let Some(parent_id) = parent_id_opt {
            let parent_space = Self::require_space(parent_id)?;

            Self::ensure_account_has_space_permission(
              owner.clone(),
              &parent_space,
              SpacePermission::CreateSubspaces,
              Error::<T>::NoPermissionToCreateSubspaces.into()
            )?;
          }

          old_data.parent_id = Some(space.parent_id);
          space.parent_id = parent_id_opt;
          is_update_applied = true;
        }
      }

      if let Some(content) = update.content {
        if content != space.content {
          Utils::<T>::is_valid_content(content.clone())?;

          ensure!(!T::IsContentBlocked::is_content_blocked(content.clone(), space.id), UtilsError::<T>::ContentIsBlocked);
          if let Some(parent_id) = space.parent_id {
            ensure!(!T::IsContentBlocked::is_content_blocked(content.clone(), parent_id), UtilsError::<T>::ContentIsBlocked);
          }

          old_data.content = Some(space.content);
          space.content = content;
          is_update_applied = true;
        }
      }

      if let Some(hidden) = update.hidden {
        if hidden != space.hidden {
          old_data.hidden = Some(space.hidden);
          space.hidden = hidden;
          is_update_applied = true;
        }
      }

      if let Some(overrides_opt) = update.permissions {
        if space.permissions != overrides_opt {
          old_data.permissions = Some(space.permissions);

          if let Some(mut overrides) = overrides_opt.clone() {
            overrides.none = overrides.none.map(
              |mut none_permissions_set| {
                none_permissions_set.extend(T::DefaultSpacePermissions::get().none.unwrap_or_default());
                none_permissions_set
              }
            );

            space.permissions = Some(overrides);
          } else {
            space.permissions = overrides_opt;
          }

          is_update_applied = true;
        }
      }

      if let Some(handle_opt) = update.handle {
        if handle_opt != space.handle {
          if let Some(new_handle) = handle_opt.clone() {
            let handle_in_lowercase = Self::lowercase_and_validate_space_handle(new_handle)?;
            SpaceIdByHandle::insert(handle_in_lowercase, space_id);
          }
          if let Some(old_handle) = space.handle.clone() {
            SpaceIdByHandle::remove(old_handle);
          }
          old_data.handle = Some(space.handle);
          space.handle = handle_opt;
          is_update_applied = true;
        }
      }

      // Update this space only if at least one field should be updated:
      if is_update_applied {
        space.updated = Some(WhoAndWhen::<T>::new(owner.clone()));

        <SpaceById<T>>::insert(space_id, space.clone());
        T::AfterSpaceUpdated::after_space_updated(owner.clone(), &space, old_data);

        Self::deposit_event(RawEvent::SpaceUpdated(owner, space_id));
      }
      Ok(())
    }
  }
}

impl<T: Trait> Space<T> {
    pub fn new(
        id: SpaceId,
        parent_id: Option<SpaceId>,
        created_by: T::AccountId,
        content: Content,
        handle: Option<Vec<u8>>,
    ) -> Self {
        Space {
            id,
            created: WhoAndWhen::<T>::new(created_by.clone()),
            updated: None,
            owner: created_by,
            parent_id,
            handle,
            content,
            hidden: false,
            posts_count: 0,
            hidden_posts_count: 0,
            followers_count: 0,
            score: 0,
            permissions: None,
        }
    }

    pub fn is_owner(&self, account: &T::AccountId) -> bool {
        self.owner == *account
    }

    pub fn is_follower(&self, account: &T::AccountId) -> bool {
        T::SpaceFollows::is_space_follower(account.clone(), self.id)
    }

    pub fn ensure_space_owner(&self, account: T::AccountId) -> DispatchResult {
        ensure!(self.is_owner(&account), Error::<T>::NotASpaceOwner);
        Ok(())
    }

    pub fn inc_posts(&mut self) {
        self.posts_count = self.posts_count.saturating_add(1);
    }

    pub fn dec_posts(&mut self) {
        self.posts_count = self.posts_count.saturating_sub(1);
    }

    pub fn inc_hidden_posts(&mut self) {
        self.hidden_posts_count = self.hidden_posts_count.saturating_add(1);
    }

    pub fn dec_hidden_posts(&mut self) {
        self.hidden_posts_count = self.hidden_posts_count.saturating_sub(1);
    }

    pub fn inc_followers(&mut self) {
        self.followers_count = self.followers_count.saturating_add(1);
    }

    pub fn dec_followers(&mut self) {
        self.followers_count = self.followers_count.saturating_sub(1);
    }

    #[allow(clippy::comparison_chain)]
    pub fn change_score(&mut self, diff: i16) {
        if diff > 0 {
            self.score = self.score.saturating_add(diff.abs() as i32);
        } else if diff < 0 {
            self.score = self.score.saturating_sub(diff.abs() as i32);
        }
    }

    pub fn try_get_parent(&self) -> Result<SpaceId, DispatchError> {
        self.parent_id.ok_or_else(|| Error::<T>::SpaceIsAtRoot.into())
    }
}

impl Default for SpaceUpdate {
    fn default() -> Self {
        SpaceUpdate {
            parent_id: None,
            handle: None,
            content: None,
            hidden: None,
            permissions: None,
        }
    }
}

impl<T: Trait> Module<T> {

    /// Check that there is a `Space` with such `space_id` in the storage
    /// or return`SpaceNotFound` error.
    pub fn ensure_space_exists(space_id: SpaceId) -> DispatchResult {
        ensure!(<SpaceById<T>>::contains_key(space_id), Error::<T>::SpaceNotFound);
        Ok(())
    }

    /// Get `Space` by id from the storage or return `SpaceNotFound` error.
    pub fn require_space(space_id: SpaceId) -> Result<Space<T>, DispatchError> {
        Ok(Self::space_by_id(space_id).ok_or(Error::<T>::SpaceNotFound)?)
    }

    pub fn lowercase_and_validate_space_handle(handle: Vec<u8>) -> Result<Vec<u8>, DispatchError> {
        let handle_in_lowercase = Utils::<T>::lowercase_and_validate_a_handle(handle)?;

        // Check if a handle is unique across all spaces' handles:
        ensure!(Self::space_id_by_handle(handle_in_lowercase.clone()).is_none(), Error::<T>::SpaceHandleIsNotUnique);

        Ok(handle_in_lowercase)
    }

    pub fn ensure_account_has_space_permission(
        account: T::AccountId,
        space: &Space<T>,
        permission: SpacePermission,
        error: DispatchError,
    ) -> DispatchResult {
        let is_owner = space.is_owner(&account);
        let is_follower = space.is_follower(&account);

        let ctx = SpacePermissionsContext {
            space_id: space.id,
            is_space_owner: is_owner,
            is_space_follower: is_follower,
            space_perms: space.permissions.clone(),
        };

        T::Roles::ensure_account_has_space_permission(
            account,
            ctx,
            permission,
            error,
        )
    }

    pub fn try_move_space_to_root(space_id: SpaceId) -> DispatchResult {
        let mut space = Self::require_space(space_id)?;
        space.parent_id = None;

        SpaceById::<T>::insert(space_id, space);
        Ok(())
    }

    pub fn mutate_space_by_id<F: FnOnce(&mut Space<T>)> (
        space_id: SpaceId,
        f: F
    ) -> Result<Space<T>, DispatchError> {
        <SpaceById<T>>::mutate(space_id, |space_opt| {
            if let Some(ref mut space) = space_opt.clone() {
                f(space);
                *space_opt = Some(space.clone());

                return Ok(space.clone());
            }

            Err(Error::<T>::SpaceNotFound.into())
        })
    }

    pub fn get_last_space_id() -> SpaceId {
        Self::next_space_id().saturating_sub(1)
    }

    pub fn get_last_space() -> Option<Space<T>> {
        let last_space_id = Self::next_space_id().saturating_sub(1);

        let limit_space_id = last_space_id.saturating_sub(T::DefaultRPCLimit::get());

        for space_id in last_space_id..limit_space_id {
            let space_opt = Self::space_by_id(space_id);
            if space_opt.is_some() {
                return space_opt
            }
        }

        None
    }

    pub fn get_hidden_space_ids(limit_opt: Option<u64>, offset_opt: Option<u64>) -> Vec<SpaceId> {
        let mut last_space_id = Self::next_space_id();

        if let Some(offset) = offset_opt {
            last_space_id = last_space_id.saturating_sub(offset);
        }

        let first_space_id: u64;
        if let Some(limit) = limit_opt {
            first_space_id = last_space_id.saturating_sub(limit);
        } else {
            first_space_id = last_space_id.saturating_sub(T::DefaultRPCLimit::get());
        }

        let mut hidden_space_ids: Vec<SpaceId> = Vec::new();

        for space_id in first_space_id..last_space_id {
            let space_opt = Self::space_by_id(space_id);

            if let Some(space) = space_opt.clone() {
                if space.hidden {
                    hidden_space_ids.push(space.id);
                }
            }
        }

        hidden_space_ids
    }

    pub fn get_public_space_ids(limit: u64, offset: u64) -> Vec<SpaceId> {
        let mut last_space_id = Self::next_space_id();
        last_space_id = last_space_id.saturating_sub(offset);

        let first_space_id: u64;
        first_space_id = last_space_id.saturating_sub(limit);

        let mut public_space_ids: Vec<SpaceId> = Vec::new();
        for space_id in first_space_id..last_space_id {
            let space_opt = Self::space_by_id(space_id);

            if let Some(space) = space_opt.clone() {
                if !space.hidden && !space.content.is_none() {
                    public_space_ids.push(space.id);
                }
            }
        }

        public_space_ids
    }

    pub fn get_unlisted_space_ids(limit: u64, offset: u64) -> Vec<SpaceId> {
        let mut last_space_id = Self::next_space_id();

        last_space_id = last_space_id.saturating_sub(offset);

        let first_space_id: u64;
        first_space_id = last_space_id.saturating_sub(limit);


        let mut unlisted_space_ids: Vec<SpaceId> = Vec::new();

        for space_id in first_space_id..last_space_id {
            let space_opt = Self::space_by_id(space_id);

            if let Some(space) = space_opt.clone() {
                if space.hidden && space.content.is_none() {
                    unlisted_space_ids.push(space.id);
                }
            }
        }

        unlisted_space_ids
    }

}

impl<T: Trait> SpaceForRolesProvider for Module<T> {
    type AccountId = T::AccountId;

    fn get_space(id: SpaceId) -> Result<SpaceForRoles<Self::AccountId>, DispatchError> {
        let space = Module::<T>::require_space(id)?;

        Ok(SpaceForRoles {
            owner: space.owner,
            permissions: space.permissions,
        })
    }
}

pub trait BeforeSpaceCreated<T: Trait> {
    fn before_space_created(follower: T::AccountId, space: &mut Space<T>) -> DispatchResult;
}

impl<T: Trait> BeforeSpaceCreated<T> for () {
    fn before_space_created(_follower: T::AccountId, _space: &mut Space<T>) -> DispatchResult {
        Ok(())
    }
}

#[impl_trait_for_tuples::impl_for_tuples(10)]
pub trait AfterSpaceUpdated<T: Trait> {
    fn after_space_updated(sender: T::AccountId, space: &Space<T>, old_data: SpaceUpdate);
}
