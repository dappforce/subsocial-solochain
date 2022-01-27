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

    pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    /// Information about the locked tokens on the parachain.
    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug)]
    pub struct LockedInfo<T: Config> {
        /// How many tokens are locked.
        pub locked_amount: BalanceOf<T>,

        /// At what block the tokens will be unlocked.
        pub unlocks_at: T::BlockNumber,

        /// How many blocks the tokens will be locked for.
        pub lock_period: T::BlockNumber,
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
        // TODO Maybe rename to Oracle or OracleOrigin?
        type ManagerOrigin: EnsureOrigin<Self::Origin>;

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
        LockedInfo<T>,
        OptionQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Locked information changed for an account. [who]
        LockedInfoSet(T::AccountId),

        /// Locked information is cleared for an account. [who]
        LockedInfoCleared(T::AccountId),
    }
    
    #[pallet::call]
    impl<T: Config> Pallet<T> {

        /// Sets the locked information for an account.
        #[pallet::weight((
            <T as Config>::WeightInfo::set_locked_info(),
            DispatchClass::Operational,
            Pays::Yes,
        ))]
        pub fn set_locked_info(
            origin: OriginFor<T>,
            account: T::AccountId,
            locked_amount: BalanceOf<T>,
            lock_period: T::BlockNumber,
            unlocks_at: T::BlockNumber,
        ) -> DispatchResultWithPostInfo {
            let _ = T::ManagerOrigin::ensure_origin(origin)?;

            let locked_info = LockedInfo {
                locked_amount,
                lock_period,
                unlocks_at,
            };
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
            let _ = T::ManagerOrigin::ensure_origin(origin)?;

            <LockedInfoByAccount<T>>::remove(account.clone());

            Self::deposit_event(Event::LockedInfoCleared(account));

            // If the call did succeed, don't charge the caller
            Ok(Pays::No.into())
        }
    }
}
