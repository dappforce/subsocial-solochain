#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
    traits::Get,
};
use frame_system::{self as system, ensure_signed};
use sp_std::prelude::*;

use df_traits::moderation::IsAccountBlocked;
use pallet_spaces::{Module as Spaces, SpaceById, SpaceIdsByOwner};
use pallet_utils::{remove_from_vec, Error as UtilsError, SpaceId};

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_utils::Config + pallet_spaces::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_runtime_upgrade() -> Weight {
            let old_pallet_prefix = "SpaceOwnershipModule";
            let new_pallet_prefix = Self::name();
            frame_support::log::info!(
                "Move Storage from {} to {}",
                old_pallet_prefix,
                new_pallet_prefix
            );
            frame_support::migration::move_pallet(
                old_pallet_prefix.as_bytes(),
                new_pallet_prefix.as_bytes(),
            );
            T::BlockWeights::get().max_block
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
        pub fn transfer_space_ownership(
            origin: OriginFor<T>,
            space_id: SpaceId,
            transfer_to: T::AccountId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let space = Spaces::<T>::require_space(space_id)?;
            space.ensure_space_owner(who.clone())?;

            ensure!(who != transfer_to, Error::<T>::CannotTranferToCurrentOwner);
            ensure!(
                T::IsAccountBlocked::is_allowed_account(transfer_to.clone(), space_id),
                UtilsError::<T>::AccountIsBlocked
            );

            PendingSpaceOwner::<T>::insert(space_id, transfer_to.clone());

            Self::deposit_event(Event::SpaceOwnershipTransferCreated(
                who,
                space_id,
                transfer_to,
            ));
            Ok(())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2, 2))]
        pub fn accept_pending_ownership(origin: OriginFor<T>, space_id: SpaceId) -> DispatchResult {
            use frame_support::StorageMap;

            let new_owner = ensure_signed(origin)?;

            let mut space = Spaces::require_space(space_id)?;
            ensure!(!space.is_owner(&new_owner), Error::<T>::AlreadyASpaceOwner);

            let transfer_to =
                Self::pending_space_owner(space_id).ok_or(Error::<T>::NoPendingTransferOnSpace)?;
            ensure!(
                new_owner == transfer_to,
                Error::<T>::NotAllowedToAcceptOwnershipTransfer
            );

            // Here we know that the origin is eligible to become a new owner of this space.
            PendingSpaceOwner::<T>::remove(space_id);

            Spaces::maybe_transfer_handle_deposit_to_new_space_owner(&space, &new_owner)?;

            let old_owner = space.owner;
            space.owner = new_owner.clone();
            SpaceById::<T>::insert(space_id, space);

            // Remove space id from the list of spaces by old owner
            SpaceIdsByOwner::<T>::mutate(old_owner, |space_ids| {
                remove_from_vec(space_ids, space_id)
            });

            // Add space id to the list of spaces by new owner
            SpaceIdsByOwner::<T>::mutate(new_owner.clone(), |ids| ids.push(space_id));

            // TODO add a new owner as a space follower? See T::BeforeSpaceCreated::before_space_created(new_owner.clone(), space)?;

            Self::deposit_event(Event::SpaceOwnershipTransferAccepted(new_owner, space_id));
            Ok(())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2, 1))]
        pub fn reject_pending_ownership(origin: OriginFor<T>, space_id: SpaceId) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let space = Spaces::<T>::require_space(space_id)?;
            let transfer_to =
                Self::pending_space_owner(space_id).ok_or(Error::<T>::NoPendingTransferOnSpace)?;
            ensure!(
                who == transfer_to || who == space.owner,
                Error::<T>::NotAllowedToRejectOwnershipTransfer
            );

            PendingSpaceOwner::<T>::remove(space_id);

            Self::deposit_event(Event::SpaceOwnershipTransferRejected(who, space_id));
            Ok(())
        }
    }

    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        SpaceOwnershipTransferCreated(
            /* current owner */ T::AccountId,
            SpaceId,
            /* new owner */ T::AccountId,
        ),
        SpaceOwnershipTransferAccepted(T::AccountId, SpaceId),
        SpaceOwnershipTransferRejected(T::AccountId, SpaceId),
    }

    /// Old name generated by `decl_event`.
    #[deprecated(note = "use `Event` instead")]
    pub type RawEvent<T> = Event<T>;

    #[pallet::error]
    pub enum Error<T> {
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

    #[pallet::storage]
    #[pallet::getter(fn pending_space_owner)]
    pub type PendingSpaceOwner<T: Config> = StorageMap<_, Twox64Concat, SpaceId, T::AccountId>;
}
