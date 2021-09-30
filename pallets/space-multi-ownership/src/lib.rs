#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use sp_std::prelude::*;
use sp_std::collections::{btree_map::BTreeMap, btree_set::BTreeSet};
use sp_runtime::{RuntimeDebug, traits::Zero};

use frame_support::{
  decl_error, decl_event, decl_module, decl_storage, ensure,
  traits::Get
};
use frame_system::{self as system, ensure_signed};

use pallet_utils::{SpaceId, WhoAndWhen};

pub mod functions;

#[cfg(test)]
mod tests;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct SpaceOwners<T: Config> {
  pub created: WhoAndWhen<T>,
  pub space_id: SpaceId,
  pub owners: Vec<T::AccountId>,
  pub threshold: u16,
  pub changes_count: u16,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Change<T: Config> {
  pub created: WhoAndWhen<T>,
  pub id: ChangeId,
  pub space_id: SpaceId,
  pub add_owners: Vec<T::AccountId>,
  pub remove_owners: Vec<T::AccountId>,
  pub new_threshold: Option<u16>,
  pub notes: Vec<u8>,
  pub confirmed_by: Vec<T::AccountId>,
  pub expires_at: T::BlockNumber,
}

type ChangeId = u64;

/// The pallet's configuration trait.
pub trait Config: system::Config
  + pallet_timestamp::Config
  + pallet_utils::Config
{
  /// The overarching event type.
  type Event: From<Event<Self>> + Into<<Self as system::Config>::Event>;

  /// Minimum space owners allowed.
  type MinSpaceOwners: Get<u16>;

  /// Maximum space owners allowed.
  type MaxSpaceOwners: Get<u16>;

  /// Maximum length of change notes.
  type MaxChangeNotesLength: Get<u16>;

  /// Expiration time for change proposal.
  type BlocksToLive: Get<Self::BlockNumber>;

  /// Period in blocks for which change proposal is can remain in a pending state until deleted.
  type DeleteExpiredChangesPeriod: Get<Self::BlockNumber>;
}

decl_error! {
  pub enum Error for Module<T: Config> {
    /// Space owners was not found by id
    SpaceOwnersNotFound,
    /// Change was not found by id
    ChangeNotFound,
    /// Space owners already exist on this space
    SpaceOwnersAlreadyExist,

    /// There can not be less owners than allowed
    NotEnoughOwners,
    /// There can not be more owners than allowed
    TooManyOwners,
    /// Account is not a space owner
    NotASpaceOwner,

    /// The threshold can not be less than 1
    ZeroThershold,
    /// The required confirmation count can not be greater than owners count"
    TooBigThreshold,
    /// Change notes are too long
    ChangeNotesOversize,
    /// No space owners will left in result of change
    NoSpaceOwnersLeft,
    /// No updates proposed with this change
    NoUpdatesProposed,
    /// No fields update in result of change proposal
    NoFieldsUpdatedOnProposal,

    /// Account has already confirmed this change
    ChangeAlreadyConfirmed,
    /// There are not enough confirmations for this change
    NotEnoughConfirms,
    /// Change is already executed
    ChangeAlreadyExecuted,
    /// Change is not related to this space
    ChangeNotRelatedToSpace,
    /// Pending change already exists
    PendingChangeAlreadyExists,
    /// Pending change doesn't exist
    PendingChangeDoesNotExist,

    /// Account is not a proposal creator
    NotAChangeCreator,

    /// Overflow when incrementing a counter of executed changes
    ChangesCountOverflow,
  }
}

// This pallet's storage items.
decl_storage! {
  trait Store for Module<T: Config> as SpaceMultiOwnershipModule {
    SpaceOwnersBySpaceById get(fn space_owners_by_space_id):
      map hasher(twox_64_concat) SpaceId => Option<SpaceOwners<T>>;

    SpaceIdsOwnedByAccountId get(fn space_ids_owned_by_account_id):
      map hasher(twox_64_concat) T::AccountId => BTreeSet<SpaceId> = BTreeSet::new();

    NextChangeId get(fn next_change_id): ChangeId = 1;

    ChangeById get(fn change_by_id):
      map hasher(twox_64_concat) ChangeId => Option<Change<T>>;

    PendingChangeIdBySpaceId get(fn pending_change_id_by_space_id):
      map hasher(twox_64_concat) SpaceId => Option<ChangeId>;

    PendingChangeIds get(fn pending_change_ids): BTreeSet<ChangeId> = BTreeSet::new();

    ExecutedChangeIdsBySpaceId get(fn executed_change_ids_by_space_id):
      map hasher(twox_64_concat) SpaceId => Vec<ChangeId>;
  }
}

// The pallet's dispatchable functions.
decl_module! {
  pub struct Module<T: Config> for enum Call where origin: T::Origin {
    /// Minimum space owners allowed.
    const MinSpaceOwners: u16 = T::MinSpaceOwners::get();

    /// Maximum space owners allowed.
    const MaxSpaceOwners: u16 = T::MaxSpaceOwners::get();

    /// Maximum length of change notes.
    const MaxChangeNotesLength: u16 = T::MaxChangeNotesLength::get();

    /// Period in blocks for which change proposal is can remain in a pending state until deleted.
    const BlocksToLive: T::BlockNumber = T::BlocksToLive::get();

    /// Period in blocks to initialize deleting of pending changes that are outdated.
    const DeleteExpiredChangesPeriod: T::BlockNumber = T::DeleteExpiredChangesPeriod::get();

    // Initializing events
    fn deposit_event() = default;

    fn on_finalize(n: T::BlockNumber) {
      Self::delete_expired_changes(n);
    }

    #[weight = T::DbWeight::get().reads_writes(2, 2) + 10_000]
    pub fn create_space_owners(
      origin,
      space_id: SpaceId,
      owners: Vec<T::AccountId>,
      threshold: u16
    ) {
      let who = ensure_signed(origin)?;

      ensure!(Self::space_owners_by_space_id(space_id).is_none(), Error::<T>::SpaceOwnersAlreadyExist);

      let mut owners_map: BTreeMap<T::AccountId, bool> = BTreeMap::new();
      let mut unique_owners: Vec<T::AccountId> = Vec::new();

      for owner in owners.iter() {
        if !owners_map.contains_key(&owner) {
          owners_map.insert(owner.clone(), true);
          unique_owners.push(owner.clone());
        }
      }

      let owners_count = unique_owners.len() as u16;
      ensure!(owners_count >= T::MinSpaceOwners::get(), Error::<T>::NotEnoughOwners);
      ensure!(owners_count <= T::MaxSpaceOwners::get(), Error::<T>::TooManyOwners);

      ensure!(threshold <= owners_count, Error::<T>::TooBigThreshold);
      ensure!(threshold > 0, Error::<T>::ZeroThershold);

      let new_space_owners = SpaceOwners {
        created: WhoAndWhen::<T>::new(who.clone()),
        space_id,
        owners: unique_owners.clone(),
        threshold,
        changes_count: 0
      };

      <SpaceOwnersBySpaceById<T>>::insert(space_id, new_space_owners);

      for owner in unique_owners.iter() {
        <SpaceIdsOwnedByAccountId<T>>::mutate(owner.clone(), |ids| ids.insert(space_id));
      }

      Self::deposit_event(RawEvent::SpaceOwnersCreated(who, space_id));
    }

    #[weight = T::DbWeight::get().reads_writes(5, 4) + 10_000]
    pub fn propose_change(
      origin,
      space_id: SpaceId,
      add_owners: Vec<T::AccountId>,
      remove_owners: Vec<T::AccountId>,
      new_threshold: Option<u16>,
      notes: Vec<u8>
    ) {
      let who = ensure_signed(origin)?;

      let has_updates =
        !add_owners.is_empty() ||
        !remove_owners.is_empty() ||
        new_threshold.is_some();

      ensure!(has_updates, Error::<T>::NoUpdatesProposed);
      ensure!(notes.len() <= T::MaxChangeNotesLength::get() as usize, Error::<T>::ChangeNotesOversize);

      let space_owners = Self::space_owners_by_space_id(space_id).ok_or(Error::<T>::SpaceOwnersNotFound)?;
      ensure!(Self::pending_change_id_by_space_id(space_id).is_none(), Error::<T>::PendingChangeAlreadyExists);

      let is_space_owner = space_owners.owners.iter().any(|owner| *owner == who.clone());
      ensure!(is_space_owner, Error::<T>::NotASpaceOwner);

      let mut fields_updated : u16 = 0;

      let result_owners = Self::transform_new_owners_to_vec(space_owners.owners.clone(), add_owners.clone(), remove_owners.clone());
      ensure!(!result_owners.is_empty(), Error::<T>::NoSpaceOwnersLeft);
      if result_owners != space_owners.owners {
        fields_updated += 1;
      }

      if let Some(threshold) = new_threshold {
        if space_owners.threshold != threshold {
          ensure!(threshold as usize <= result_owners.len(), Error::<T>::TooBigThreshold);
          ensure!(threshold > 0, Error::<T>::ZeroThershold);
          fields_updated += 1;
        }
      }

      let change_id = Self::next_change_id();
      let mut new_change = Change {
        created: WhoAndWhen::<T>::new(who.clone()),
        id: change_id,
        space_id,
        add_owners,
        remove_owners,
        new_threshold,
        notes,
        confirmed_by: Vec::new(),
        expires_at: <system::Module<T>>::block_number() + T::BlocksToLive::get()
      };

      if fields_updated > 0 {
        new_change.confirmed_by.push(who.clone());
        <ChangeById<T>>::insert(change_id, new_change);
        PendingChangeIdBySpaceId::insert(space_id, change_id);
        PendingChangeIds::mutate(|set| set.insert(change_id));
        NextChangeId::mutate(|n| { *n += 1; });

        Self::deposit_event(RawEvent::ChangeProposed(who, space_id, change_id));
      } else {
        return Err(Error::<T>::NoFieldsUpdatedOnProposal.into());
      }
    }

    #[weight = T::DbWeight::get().reads_writes(3, 1) + 10_000]
    pub fn confirm_change(
      origin,
      space_id: SpaceId,
      change_id: ChangeId
    ) {
      let who = ensure_signed(origin)?;

      let space_owners = Self::space_owners_by_space_id(space_id).ok_or(Error::<T>::SpaceOwnersNotFound)?;

      let is_space_owner = space_owners.owners.iter().any(|owner| *owner == who.clone());
      ensure!(is_space_owner, Error::<T>::NotASpaceOwner);

      let mut change = Self::change_by_id(change_id).ok_or(Error::<T>::ChangeNotFound)?;

      let pending_change_id = Self::pending_change_id_by_space_id(space_id).ok_or(Error::<T>::PendingChangeDoesNotExist)?;
      ensure!(pending_change_id == change_id, Error::<T>::ChangeNotRelatedToSpace);

      // Check whether sender confirmed change or not
      ensure!(!change.confirmed_by.iter().any(|account| *account == who.clone()), Error::<T>::ChangeAlreadyConfirmed);

      change.confirmed_by.push(who.clone());

      if change.confirmed_by.len() == space_owners.threshold as usize {
        Self::update_space_owners(who.clone(), space_owners, change)?;
      } else {
        <ChangeById<T>>::insert(change_id, change);
      }

      Self::deposit_event(RawEvent::ChangeConfirmed(who, space_id, change_id));
    }

    #[weight = T::DbWeight::get().reads_writes(4, 3) + 10_000]
    pub fn cancel_change(
      origin,
      space_id: SpaceId,
      change_id: ChangeId
    ) {
      let who = ensure_signed(origin)?;

      let space_owners = Self::space_owners_by_space_id(space_id).ok_or(Error::<T>::SpaceOwnersNotFound)?;

      let is_space_owner = space_owners.owners.iter().any(|owner| *owner == who.clone());
      ensure!(is_space_owner, Error::<T>::NotASpaceOwner);

      let pending_change_id = Self::pending_change_id_by_space_id(space_id).ok_or(Error::<T>::PendingChangeDoesNotExist)?;
      ensure!(pending_change_id == change_id, Error::<T>::ChangeNotRelatedToSpace);

      let change = Self::change_by_id(change_id).ok_or(Error::<T>::ChangeNotFound)?;
      ensure!(change.created.account == who, Error::<T>::NotAChangeCreator);

      <ChangeById<T>>::remove(change_id);
      PendingChangeIdBySpaceId::remove(space_id);
      PendingChangeIds::mutate(|set| set.remove(&change_id));

      Self::deposit_event(RawEvent::ProposalCanceled(who, space_id));
    }
  }
}

decl_event!(
  pub enum Event<T> where
    <T as system::Config>::AccountId,
   {
    SpaceOwnersCreated(AccountId, SpaceId),
    ChangeProposed(AccountId, SpaceId, ChangeId),
    ProposalCanceled(AccountId, SpaceId),
    ChangeConfirmed(AccountId, SpaceId, ChangeId),
    SpaceOwnersUpdated(AccountId, SpaceId, ChangeId),
  }
);
