//! # Locker Mirror Pallet
//!
//! Pallet that mirrors locked tokens and period from the parachain.

#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod mock;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
mod weights;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{pallet_prelude::*};
    use frame_support::traits::{Currency};
    use frame_system::pallet_prelude::*;
    use crate::weights::WeightInfo;

    /// The type used to represent block numbers on the parachain.
    type ParachainBlockNumber = u32;

    /// The type used to represent event index on the parachain.
    type ParachainEventIndex = u32;

    pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    pub type LockedInfoOf<T> = LockedInfo<<T as frame_system::Config>::BlockNumber, BalanceOf<T>>;

    /// Information about the locked tokens on the parachain.
    #[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
    pub struct LockedInfo<BlockNumber, Balance> {
        /// How many tokens are locked.
        pub locked_amount: Balance,

        /// At what block that tokens did get locked.
        pub locked_at: BlockNumber,

        /// At what block the locked info will be expired. or None if it doesn't expires.
        pub expires_at: Option<BlockNumber>,
    }

    /// Information about a parachain event.
    #[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
    pub struct ParachainEvent {
        /// The parachain block number at which the event was found.
        pub block_number: ParachainBlockNumber,

        /// The index of the parachain event.
        pub event_index: ParachainEventIndex,
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The Currency handler.
        type Currency: Currency<Self::AccountId>;

        /// The oracle origin which can mirror the locked tokens.
        type OracleOrigin: EnsureOrigin<Self::Origin>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
    }

    /// Stores information about locked tokens and period for each account.
    #[pallet::storage]
    #[pallet::getter(fn locked_info_by_account)]
    pub type LockedInfoByAccount<T: Config> = StorageMap<
        _,
        Twox64Concat,
        T::AccountId,
        LockedInfoOf<T>,
        OptionQuery,
    >;

    /// Stores information about last processed event on the parachain.
    #[pallet::storage]
    #[pallet::getter(fn last_processed_parachain_event)]
    pub type LastProcessedParachainEvent<T: Config> = StorageValue<_, ParachainEvent>;

    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Locked information changed for an account. [who]
        LockedInfoSet(T::AccountId),

        /// Locked information is cleared for an account. [who]
        LockedInfoCleared(T::AccountId),

        /// Last processed event have been set.
        LastProcessedEventSet(ParachainEvent),
    }
    
    #[pallet::call]
    impl<T: Config> Pallet<T> {

        #[pallet::weight((
            <T as Config>::WeightInfo::set_last_processed_parachain_event(),
            DispatchClass::Operational,
            Pays::Yes,
        ))]
        pub fn set_last_processed_parachain_event(
            origin: OriginFor<T>,
            last_processed_event_info: ParachainEvent,
        ) -> DispatchResultWithPostInfo {
            let _ = T::OracleOrigin::ensure_origin(origin)?;

            <LastProcessedParachainEvent<T>>::put(last_processed_event_info.clone());

            Self::deposit_event(Event::LastProcessedEventSet(last_processed_event_info));

            Ok(Pays::No.into())
        }

        /// Sets the locked information for an account.
        #[pallet::weight((
            <T as Config>::WeightInfo::set_locked_info(),
            DispatchClass::Operational,
            Pays::Yes,
        ))]
        pub fn set_locked_info(
            origin: OriginFor<T>,
            account: T::AccountId,
            locked_info: LockedInfoOf<T>,
        ) -> DispatchResultWithPostInfo {
            let _ = T::OracleOrigin::ensure_origin(origin)?;

            <LockedInfoByAccount<T>>::insert(account.clone(), locked_info);

            Self::deposit_event(Event::LockedInfoSet(account));

            // If the call did succeed, don't charge the caller
            Ok(Pays::No.into())
        }

        /// Clears the locked information for an account.
        #[pallet::weight((
            <T as Config>::WeightInfo::clear_locked_info(),
            DispatchClass::Operational,
            Pays::Yes,
        ))]
        pub fn clear_locked_info(
            origin: OriginFor<T>,
            account: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let _ = T::OracleOrigin::ensure_origin(origin)?;

            <LockedInfoByAccount<T>>::remove(account.clone());

            Self::deposit_event(Event::LockedInfoCleared(account));

            // If the call did succeed, don't charge the caller
            Ok(Pays::No.into())
        }
    }
}
