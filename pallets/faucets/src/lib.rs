//! # Faucets Module
//!
//! The Faucets pallet allows a root key (sudo) to add accounts (faucets) that are eligible
//! to drip free tokens to other accounts (recipients).
//!
//! Currently, only sudo account can add, update and remove faucets.
//! But this can be changed in the future to allow anyone else
//! to set up new faucets for their needs.
//!
//! This would allow each space to create its own faucet(s) and distribute its tokens to its
//! members based on a set of conditions the space decides suits the needs of its community.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use scale_info::TypeInfo;
use frame_support::{
    dispatch::{DispatchError, DispatchResult, DispatchResultWithPostInfo},
    ensure,
    traits::{Currency, ExistenceRequirement},
    weights::Pays,
};
use frame_system::{self as system, ensure_root, ensure_signed};
use sp_runtime::RuntimeDebug;
use sp_runtime::traits::{Saturating, Zero};
use sp_std::{
    collections::btree_set::BTreeSet,
    prelude::*,
};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub use pallet::*;

type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as system::Config>::AccountId>>::Balance;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct Faucet<T: Config> {
    // Settings
    pub enabled: bool,
    pub period: T::BlockNumber,
    pub period_limit: BalanceOf<T>,
    pub drip_limit: BalanceOf<T>,

    // State
    pub next_period_at: T::BlockNumber,
    pub dripped_in_current_period: BalanceOf<T>,
}

//TODO: use better nomenclature for `period`, `period_limit`, `drip_limit` &
//`dripped_in_current_period`.

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct FaucetUpdate<BlockNumber, Balance> {
    pub enabled: Option<bool>,
    pub period: Option<BlockNumber>,
    pub period_limit: Option<Balance>,
    pub drip_limit: Option<Balance>,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {

        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type Currency: Currency<Self::AccountId>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        FaucetAdded { who: T::AccountId },
        FaucetUpdated { who: T::AccountId },
        FaucetsRemoved { removed: Vec<T::AccountId> },
        Dripped { issuer: T::AccountId, recipient: T::AccountId, amount: BalanceOf<T> },
    }

    #[pallet::error]
    pub enum Error<T> {
        FaucetNotFound,
        FaucetAlreadyAdded,
        NoFreeBalanceOnFaucet,
        NotEnoughFreeBalanceOnFaucet,
        NoFaucetsProvided,
        NoUpdatesProvided,
        NothingToUpdate,
        InvalidUpdate,
        FaucetDisabled,
        NotFaucetOwner,
        RecipientEqualsFaucet,
        DripLimitCannotExceedPeriodLimit,

        ZeroPeriodProvided,
        ZeroPeriodLimitProvided,
        ZeroDripLimitProvided,
        ZeroDripAmountProvided,

        PeriodLimitReached,
        DripLimitReached,
    }

    #[pallet::storage]
    #[pallet::getter(fn faucet_by_account)]
    pub(super) type FaucetByAccount<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, Faucet<T>>;
    

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(50_000 + T::DbWeight::get().reads_writes(2,1))]
        pub fn force_add_faucet(
            origin: OriginFor<T>,
            distro: T::AccountId,
            period: T::BlockNumber,
            period_limit: BalanceOf<T>,
            drip_limit: BalanceOf<T>,
        ) -> DispatchResult {
            ensure_root(origin)?;

            Self::new(distro, period, period_limit, drip_limit)
        }

        #[pallet::weight(50_000 + T::DbWeight::get().reads_writes(2, 1))]
        pub fn add_faucet(
            origin: OriginFor<T>,
            period: T::BlockNumber,
            period_limit: BalanceOf<T>,
            drip_limit: BalanceOf<T>,
        ) -> DispatchResult {
            let distro = ensure_signed(origin)?;

            Self::new(distro, period, period_limit, drip_limit)
        }

        #[pallet::weight(50_000 + T::DbWeight::get().reads_writes(1, 1))]
        pub fn update_faucet(
            origin: OriginFor<T>,
            faucet: T::AccountId,
            update: FaucetUpdate<T::BlockNumber, BalanceOf<T>>
        ) -> DispatchResult {
            ensure_root(origin)?;

            let has_updates =
                update.enabled.is_some() ||
                update.period.is_some() ||
                update.period_limit.is_some() ||
                update.drip_limit.is_some();
            ensure!(has_updates, Error::<T>::NoUpdatesProvided);
            let mut settings = Self::require_faucet(&faucet)?;
            let mut should_update = false;
            if let Some(enabled) = update.enabled {
                ensure!(enabled != settings.enabled, Error::<T>::InvalidUpdate);
                settings.enabled = enabled;
                should_update = true;
            }
            if let Some(period) = update.period {
                Self::ensure_period_not_zero(period)?;
                ensure!(period != settings.period, Error::<T>::InvalidUpdate);
                settings.period = period;
                should_update = true;
            }
            if let Some(period_limit) = update.period_limit {
                Self::ensure_period_limit_not_zero(period_limit)?;
                ensure!(period_limit != settings.period_limit, Error::<T>::InvalidUpdate);
                Self::ensure_drip_limit_lte_period_limit(settings.drip_limit, period_limit)?;
                settings.period_limit = period_limit;
                should_update = true;
            }
            if let Some(drip_limit) = update.drip_limit {
                Self::ensure_drip_limit_not_zero(drip_limit)?;
                ensure!(drip_limit != settings.drip_limit, Error::<T>::InvalidUpdate);
                Self::ensure_drip_limit_lte_period_limit(drip_limit, settings.period_limit)?;
                settings.drip_limit = drip_limit;
                should_update = true;
            }
            ensure!(should_update, Error::<T>::NothingToUpdate);

            FaucetByAccount::<T>::try_mutate(&faucet, |data| -> DispatchResult {
                if let Some(ref mut faucet) = data {
                    faucet.enabled = settings.enabled;
                    faucet.period = settings.period;
                    faucet.period_limit = settings.period_limit;
                    faucet.drip_limit = settings.drip_limit;
                }
                
                Ok(())
            })?;
            
            Self::deposit_event(Event::<T>::FaucetUpdated { who: faucet });
            Ok(())
        }

        #[pallet::weight(20_000 + T::DbWeight::get().reads_writes(0, 0) + 20_000 * faucets.len() as u64)]
        pub fn remove_faucets(
            origin: OriginFor<T>,
            faucets: Vec<T::AccountId>
        ) -> DispatchResult {
            ensure_root(origin)?;
            ensure!(!faucets.len().is_zero(), Error::<T>::NoFaucetsProvided);
            let unique_faucets = faucets.iter().collect::<BTreeSet<_>>();
            for faucet in unique_faucets.iter() {
                ensure!(FaucetByAccount::<T>::contains_key(faucet), Error::<T>::FaucetNotFound);
                FaucetByAccount::<T>::remove(faucet);
            }
            Self::deposit_event(Event::<T>::FaucetsRemoved { removed: faucets });
            Ok(())
        }

        #[pallet::weight(50_000 + T::DbWeight::get().reads_writes(3, 1))]
        pub fn drip(
            origin: OriginFor<T>, // Should be a faucet account, add a check
            recipient: T::AccountId,
            amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let faucet = ensure_signed(origin)?;
            ensure!(faucet != recipient, Error::<T>::RecipientEqualsFaucet);
            ensure!(amount > Zero::zero(), Error::<T>::ZeroDripAmountProvided);
            let mut settings = Self::require_faucet(&faucet)?;
            ensure!(settings.enabled, Error::<T>::FaucetDisabled);
            ensure!(amount <= settings.drip_limit, Error::<T>::DripLimitReached);
            let faucet_balance = T::Currency::free_balance(&faucet);
            ensure!(amount <= faucet_balance, Error::<T>::NotEnoughFreeBalanceOnFaucet);
            let current_block = <system::Pallet<T>>::block_number();
            if settings.next_period_at <= current_block {
                settings.next_period_at = current_block.saturating_add(settings.period);
                settings.dripped_in_current_period = Zero::zero();
            }
            let tokens_left_in_current_period = settings.period_limit
                .saturating_sub(settings.dripped_in_current_period);
            ensure!(amount <= tokens_left_in_current_period, Error::<T>::PeriodLimitReached);
            T::Currency::transfer(
                &faucet,
                &recipient,
                amount,
                ExistenceRequirement::KeepAlive
            )?;
            settings.dripped_in_current_period = amount
                .saturating_add(settings.dripped_in_current_period);
            FaucetByAccount::<T>::try_mutate(&faucet, |data| -> DispatchResult {
                if let Some(ref mut faucet) = data {
                    faucet.next_period_at = settings.next_period_at;
                    faucet.dripped_in_current_period = settings.dripped_in_current_period;
                }

                Ok(())
            })?;
            Self::deposit_event(Event::<T>::Dripped { issuer: faucet, recipient, amount });
            Ok(Pays::No.into())
        }
    }

    impl<T: Config> Pallet<T> {

        pub fn new(
            distro: T::AccountId,
            period: T::BlockNumber,
            period_limit: BalanceOf<T>,
            drip_limit: BalanceOf<T>,
        ) -> DispatchResult {
            Self::ensure_period_not_zero(period)?;
            Self::ensure_period_limit_not_zero(period_limit)?;
            Self::ensure_drip_limit_not_zero(drip_limit)?;
            Self::ensure_drip_limit_lte_period_limit(drip_limit, period_limit)?;

            ensure!(
                !FaucetByAccount::<T>::contains_key(&distro),
                Error::<T>::FaucetAlreadyAdded
            );

            ensure!(
                T::Currency::free_balance(&distro) >=
                T::Currency::minimum_balance(),
                Error::<T>::NoFreeBalanceOnFaucet
            );
            let faucet = Faucet {
                enabled: true,
                period,
                period_limit,
                drip_limit,
                next_period_at: Zero::zero(),
                dripped_in_current_period: Zero::zero(),
            };

            FaucetByAccount::<T>::insert(distro.clone(), faucet);
            Self::deposit_event(Event::<T>::FaucetAdded { who: distro });
            Ok(())
        }

        pub fn require_faucet(faucet: &T::AccountId) -> Result<Faucet<T>, DispatchError> {
            Ok(FaucetByAccount::<T>::get(faucet).ok_or(Error::<T>::FaucetNotFound)?)
        }

        fn ensure_period_not_zero(period: T::BlockNumber) -> DispatchResult {
            ensure!(period > Zero::zero(), Error::<T>::ZeroPeriodProvided);
            Ok(())
        }

        fn ensure_period_limit_not_zero(period_limit: BalanceOf<T>) -> DispatchResult {
            ensure!(period_limit > Zero::zero(), Error::<T>::ZeroPeriodLimitProvided);
            Ok(())
        }

        fn ensure_drip_limit_not_zero(drip_limit: BalanceOf<T>) -> DispatchResult {
            ensure!(drip_limit > Zero::zero(), Error::<T>::ZeroDripLimitProvided);
            Ok(())
        }   

        fn ensure_drip_limit_lte_period_limit(drip_limit: BalanceOf<T>, period_limit: BalanceOf<T>) -> DispatchResult {
            ensure!(drip_limit <= period_limit, Error::<T>::DripLimitCannotExceedPeriodLimit);
            Ok(())
        }
    }
}
