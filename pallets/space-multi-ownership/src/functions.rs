use super::*;

use sp_std::collections::btree_set::BTreeSet;
use frame_support::{dispatch::DispatchResult};

impl<T: Config> Module<T> {

  pub fn update_space_owners(who: T::AccountId, mut space_owners: SpaceOwners<T>, change: Change<T>) -> DispatchResult {
    let space_id = space_owners.space_id;
    let change_id = change.id;

    ensure!(change.confirmed_by.len() >= space_owners.threshold as usize, Error::<T>::NotEnoughConfirms);
    Self::move_change_from_pending_state_to_executed(space_id, change_id)?;

    space_owners.changes_count = space_owners.changes_count.checked_add(1).ok_or(Error::<T>::ChangesCountOverflow)?;
    if !change.add_owners.is_empty() || !change.remove_owners.is_empty() {
      space_owners.owners = Self::transform_new_owners_to_vec(
        space_owners.owners.clone(), change.add_owners.clone(), change.remove_owners.clone());
    }

    if let Some(threshold) = change.new_threshold {
      space_owners.threshold = threshold;
    }

    for account in &change.add_owners {
      <SpaceIdsOwnedByAccountId<T>>::mutate(account, |ids| ids.insert(space_id));
    }
    for account in &change.remove_owners {
      <SpaceIdsOwnedByAccountId<T>>::mutate(account, |ids| ids.remove(&space_id));
    }

    <SpaceOwnersBySpaceById<T>>::insert(space_id, space_owners);
    <ChangeById<T>>::insert(change_id, change);
    Self::deposit_event(RawEvent::SpaceOwnersUpdated(who, space_id, change_id));

    Ok(())
  }

  pub fn move_change_from_pending_state_to_executed(space_id: SpaceId, change_id: ChangeId) -> DispatchResult {
    ensure!(Self::space_owners_by_space_id(space_id).is_some(), Error::<T>::SpaceOwnersNotFound);
    ensure!(Self::change_by_id(change_id).is_some(), Error::<T>::ChangeNotFound);
    ensure!(!Self::executed_change_ids_by_space_id(space_id).iter().any(|&x| x == change_id), Error::<T>::ChangeAlreadyExecuted);

    PendingChangeIdBySpaceId::remove(&space_id);
    PendingChangeIds::mutate(|set| set.remove(&change_id));
    ExecutedChangeIdsBySpaceId::mutate(space_id, |ids| ids.push(change_id));

    Ok(())
  }

  pub fn transform_new_owners_to_vec(current_owners: Vec<T::AccountId>, add_owners: Vec<T::AccountId>, remove_owners: Vec<T::AccountId>) -> Vec<T::AccountId> {
    let mut owners_set: BTreeSet<T::AccountId> = BTreeSet::new();
    let mut new_owners_set: BTreeSet<T::AccountId> = BTreeSet::new();

    // Extract current space owners
    current_owners.iter().for_each(|x| { owners_set.insert(x.clone()); });
    // Extract owners that should be added
    add_owners.iter().for_each(|x| { new_owners_set.insert(x.clone()); });
    // Unite both sets
    owners_set = owners_set.union(&new_owners_set).cloned().collect();
    // Remove accounts that exist in remove_owners from set
    remove_owners.iter().for_each(|x| { owners_set.remove(x); });

    owners_set.iter().cloned().collect()
  }

  pub fn delete_expired_changes(block_number: T::BlockNumber) {
    if (block_number % T::DeleteExpiredChangesPeriod::get()).is_zero() {
      for change_id in Self::pending_change_ids() {
        if let Some(change) = Self::change_by_id(change_id) {
          if block_number >= change.expires_at {
            PendingChangeIdBySpaceId::remove(&change.space_id);
            <ChangeById<T>>::remove(&change_id);
            PendingChangeIds::mutate(|set| set.remove(&change_id));
          }
        }
      }
    }
  }
}
