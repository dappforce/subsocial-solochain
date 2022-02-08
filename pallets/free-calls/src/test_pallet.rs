//! Just a dummy pallet to use during the tests
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_support::weights::PostDispatchInfo;
    use frame_system::pallet_prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);


    #[pallet::storage]
    #[pallet::getter(fn something)]
    pub type Something<T> = StorageValue<_, u32>;


    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ValueStored(u32, T::AccountId),
    }

    #[pallet::error]
    pub enum Error<T> {
        DoNotCallMe,
        DoNotSendZero,
    }


    #[pallet::call]
    impl<T: Config> Pallet<T> {

        #[pallet::weight(10_000)]
        pub fn store_value(origin: OriginFor<T>, something: u32) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            ensure!(something != 0, Error::<T>::DoNotSendZero);

            // Update storage.
            <Something<T>>::put(something);

            // Emit an event.
            Self::deposit_event(Event::ValueStored(something, who));

            Ok(PostDispatchInfo {
                actual_weight: Some(50_000),
                pays_fee: Pays::Yes,
            })
        }

        #[pallet::weight(10_000)]
        pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            ensure!(false, Error::<T>::DoNotCallMe);

            Ok(())
        }
    }
}