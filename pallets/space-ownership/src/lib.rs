#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    ensure,
    dispatch::DispatchResult,
    traits::Get
};
use sp_std::prelude::*;
use frame_system::{self as system, ensure_signed};

use pallet_spaces::{Module as Spaces, SpaceById};
use pallet_utils::SpaceId;

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
    /// The current space owner cannot transfer ownership to himself.
    CannotTranferToCurrentOwner,
    /// There is no transfer ownership by space that is provided.
    NoPendingTransferOnSpace,
    /// The account is not allowed to accept transfer ownership.
    NotAllowedToAcceptOwnershipTransfer,
    /// The account is not allowed to reject transfer ownership.
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
      Spaces::<T>::ensure_space_exists(space_id)?;

      <PendingSpaceOwner<T>>::insert(space_id, transfer_to.clone());

      Self::deposit_event(RawEvent::SpaceOwnershipTransferCreated(who, space_id, transfer_to));
      Ok(())
    }

    #[weight = 10_000 + T::DbWeight::get().reads_writes(2, 2)]
    pub fn accept_pending_ownership(origin, space_id: SpaceId) -> DispatchResult {
      let who = ensure_signed(origin)?;

      let mut space = Spaces::require_space(space_id)?;
      let transfer_to = Self::pending_space_owner(space_id).ok_or(Error::<T>::NoPendingTransferOnSpace)?;
      ensure!(who == transfer_to, Error::<T>::NotAllowedToAcceptOwnershipTransfer);

      // Here we know that the origin is eligible to become a new owner of this space.
      <PendingSpaceOwner<T>>::remove(space_id);

      space.owner = who.clone();
      <SpaceById<T>>::insert(space_id, space);

      Self::deposit_event(RawEvent::SpaceOwnershipTransferAccepted(who, space_id));
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
