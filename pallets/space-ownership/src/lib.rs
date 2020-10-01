#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    ensure,
    dispatch::DispatchResult,
    traits::Get
};
use sp_std::prelude::*;
use frame_system::{self as system, ensure_signed};

use df_traits::moderation::IsAccountBlocked;
use pallet_spaces::{Module as Spaces, SpaceById, SpaceIdsByOwner};
use pallet_utils::{Error as UtilsError, SpaceId, vec_remove_on};

/// The pallet's configuration trait.
pub trait Trait: system::Trait
    + pallet_utils::Trait
    + pallet_spaces::Trait
{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_error! {
  pub enum Error for Module<T: Trait> {
    /// The current space owner cannot transfer ownership to themself.
    CannotTranferToCurrentOwner,
    /// Account is already an owner of a space.
    AlreadyASpaceOwner,
    /// There is no pending ownership transfer for a given space.
    NoPendingTransferOnSpace,
    /// Account is not allowed to accept ownership transfer.
    NotAllowedToAcceptOwnershipTransfer,
    /// Account is not allowed to reject ownership transfer.
    NotAllowedToRejectOwnershipTransfer,
  }
}

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as SpaceOwnershipModule {
        pub PendingSpaceOwner get(fn pending_space_owner):
            map hasher(twox_64_concat) SpaceId => Option<T::AccountId>;
    }
}

decl_event!(
    pub enum Event<T> where
        <T as system::Trait>::AccountId,
    {
        SpaceOwnershipTransferCreated(/* current owner */ AccountId, SpaceId, /* new owner */ AccountId),
        SpaceOwnershipTransferAccepted(AccountId, SpaceId),
        SpaceOwnershipTransferRejected(AccountId, SpaceId),
    }
);

// The pallet's dispatchable functions.
decl_module! {
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {

    // Initializing errors
    type Error = Error<T>;

    // Initializing events
    fn deposit_event() = default;

    #[weight = 10_000 + T::DbWeight::get().reads_writes(1, 1)]
    pub fn transfer_space_ownership(origin, space_id: SpaceId, transfer_to: T::AccountId) -> DispatchResult {
      let who = ensure_signed(origin)?;

      let space = Spaces::<T>::require_space(space_id)?;
      space.ensure_space_owner(who.clone())?;

      ensure!(who != transfer_to, Error::<T>::CannotTranferToCurrentOwner);
      ensure!(!T::IsAccountBlocked::is_account_blocked(transfer_to.clone(), space_id), UtilsError::<T>::AccountIsBlocked);

      <PendingSpaceOwner<T>>::insert(space_id, transfer_to.clone());

      Self::deposit_event(RawEvent::SpaceOwnershipTransferCreated(who, space_id, transfer_to));
      Ok(())
    }

    #[weight = 10_000 + T::DbWeight::get().reads_writes(2, 2)]
    pub fn accept_pending_ownership(origin, space_id: SpaceId) -> DispatchResult {
      let new_owner = ensure_signed(origin)?;

      let mut space = Spaces::require_space(space_id)?;
      ensure!(!space.is_owner(&new_owner), Error::<T>::AlreadyASpaceOwner);

      let transfer_to = Self::pending_space_owner(space_id).ok_or(Error::<T>::NoPendingTransferOnSpace)?;
      ensure!(new_owner == transfer_to, Error::<T>::NotAllowedToAcceptOwnershipTransfer);

      // Here we know that the origin is eligible to become a new owner of this space.
      <PendingSpaceOwner<T>>::remove(space_id);

      let old_owner = space.owner;
      space.owner = new_owner.clone();
      <SpaceById<T>>::insert(space_id, space);

      // Remove space id from the list of spaces by old owner
      <SpaceIdsByOwner<T>>::mutate(old_owner.clone(), |space_ids| vec_remove_on(space_ids, space_id));

      // Add space id to the list of spaces by new owner
      <SpaceIdsByOwner<T>>::mutate(new_owner.clone(), |ids| ids.push(space_id));

      // TODO add a new owner as a space follower? See T::BeforeSpaceCreated::before_space_created(new_owner.clone(), space)?;

      Self::deposit_event(RawEvent::SpaceOwnershipTransferAccepted(new_owner, space_id));
      Ok(())
    }

    #[weight = 10_000 + T::DbWeight::get().reads_writes(2, 1)]
    pub fn reject_pending_ownership(origin, space_id: SpaceId) -> DispatchResult {
      let who = ensure_signed(origin)?;

      let space = Spaces::<T>::require_space(space_id)?;
      let transfer_to = Self::pending_space_owner(space_id).ok_or(Error::<T>::NoPendingTransferOnSpace)?;
      ensure!(who == transfer_to || who == space.owner, Error::<T>::NotAllowedToRejectOwnershipTransfer);

      <PendingSpaceOwner<T>>::remove(space_id);

      Self::deposit_event(RawEvent::SpaceOwnershipTransferRejected(who, space_id));
      Ok(())
    }
  }
}
