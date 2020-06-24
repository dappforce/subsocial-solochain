#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::string_lit_as_bytes)]

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    ensure,
};
use sp_std::prelude::*;
use system::ensure_signed;

use pallet_spaces::{Module as Spaces, SpaceById};
use pallet_utils::SpaceId;

// mod tests;

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
    trait Store for Module<T: Trait> as TemplateModule {
        pub PendingSpaceOwner get(fn pending_space_owner): map SpaceId => Option<T::AccountId>;
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

    // Initializing events
    fn deposit_event() = default;

    pub fn transfer_space_ownership(origin, space_id: SpaceId, transfer_to: T::AccountId) {
      let who = ensure_signed(origin)?;

      let space = Spaces::<T>::require_space(space_id)?;
      space.ensure_space_owner(who.clone())?;

      ensure!(who != transfer_to, Error::<T>::CannotTranferToCurrentOwner);
      Spaces::<T>::ensure_space_exists(space_id)?;

      <PendingSpaceOwner<T>>::insert(space_id, transfer_to.clone());
      Self::deposit_event(RawEvent::SpaceOwnershipTransferCreated(who, space_id, transfer_to));
    }

    pub fn accept_pending_ownership(origin, space_id: SpaceId) {
      let who = ensure_signed(origin)?;

      let mut space = Spaces::require_space(space_id)?;
      let transfer_to = Self::pending_space_owner(space_id).ok_or(Error::<T>::NoPendingTransferOnSpace)?;
      ensure!(who == transfer_to, Error::<T>::NotAllowedToAcceptOwnershipTransfer);

      // Here we know that the origin is eligible to become a new owner of this space.
      <PendingSpaceOwner<T>>::remove(space_id);

      space.owner = who.clone();
      <SpaceById<T>>::insert(space_id, space);
      Self::deposit_event(RawEvent::SpaceOwnershipTransferAccepted(who, space_id));
    }

    pub fn reject_pending_ownership(origin, space_id: SpaceId) {
      let who = ensure_signed(origin)?;

      let space = Spaces::<T>::require_space(space_id)?;
      let transfer_to = Self::pending_space_owner(space_id).ok_or(Error::<T>::NoPendingTransferOnSpace)?;
      ensure!(who == transfer_to || who == space.owner, Error::<T>::NotAllowedToRejectOwnershipTransfer);

      <PendingSpaceOwner<T>>::remove(space_id);
      Self::deposit_event(RawEvent::SpaceOwnershipTransferRejected(who, space_id));
    }
  }
}
