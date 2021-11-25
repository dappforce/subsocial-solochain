//! # Faucets Module
//!
//! The Faucets module allows a root key (sudo) to add accounts (faucets) that are eligible
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
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult, DispatchResultWithPostInfo},
    ensure,
    traits::{Currency, ExistenceRequirement, Get},
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

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct FaucetUpdate<BlockNumber, Balance> {
    pub enabled: Option<bool>,
    pub period: Option<BlockNumber>,
    pub period_limit: Option<Balance>,
    pub drip_limit: Option<Balance>,
}

type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as system::Config>::AccountId>>::Balance;

/// The pallet's configuration trait.
pub trait Config: system::Config {

    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Config>::Event>;

    type Currency: Currency<Self::AccountId>;
}

decl_storage! {
    trait Store for Module<T: Config> as FaucetsModule {

        /// Get a faucet data by its account id.
        pub FaucetByAccount get(fn faucet_by_account):
            map hasher(twox_64_concat) T::AccountId // Faucet account
            => Option<Faucet<T>>;
    }
}

decl_event!(
    pub enum Event<T> where
        AccountId = <T as system::Config>::AccountId,
        Balance = BalanceOf<T>
    {
        FaucetAdded(AccountId),
        FaucetUpdated(AccountId),
        FaucetsRemoved(Vec<AccountId>),
        Dripped(
            AccountId, // Faucet account
            AccountId, // Recipient account
            Balance    // Amount dripped
        ),
    }
);

decl_error! {
    pub enum Error for Module<T: Config> {
        FaucetNotFound,
        FaucetAlreadyAdded,
        NoFreeBalanceOnFaucet,
        NotEnoughFreeBalanceOnFaucet,
        NoFaucetsProvided,
        NoUpdatesProvided,
        NothingToUpdate,
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
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        // Initializing errors
        type Error = Error<T>;

        // Initializing events
        fn deposit_event() = default;

        #[weight = 50_000 + T::DbWeight::get().reads_writes(2, 1)]
        pub fn add_faucet(
            origin,
            faucet: T::AccountId,
            period: T::BlockNumber,
            period_limit: BalanceOf<T>,
            drip_limit: BalanceOf<T>,
        ) -> DispatchResult {

            ensure_root(origin)?;

            Self::ensure_period_not_zero(period)?;
            Self::ensure_period_limit_not_zero(period_limit)?;
            Self::ensure_drip_limit_not_zero(drip_limit)?;
            Self::ensure_drip_limit_lte_period_limit(drip_limit, period_limit)?;

            ensure!(
                Self::faucet_by_account(&faucet).is_none(),
                Error::<T>::FaucetAlreadyAdded
            );

            ensure!(
                T::Currency::free_balance(&faucet) >=
                T::Currency::minimum_balance(),
                Error::<T>::NoFreeBalanceOnFaucet
            );

            let new_faucet = Faucet::<T>::new(
                period,
                period_limit,
                drip_limit
            );

            FaucetByAccount::<T>::insert(faucet.clone(), new_faucet);
            Self::deposit_event(RawEvent::FaucetAdded(faucet));
            Ok(())
        }

        #[weight = 50_000 + T::DbWeight::get().reads_writes(1, 1)]
        pub fn update_faucet(
            origin,
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

            // `true` if there is at least one updated field.
            let mut should_update = false;

            if let Some(enabled) = update.enabled {
                if enabled != settings.enabled {
                    settings.enabled = enabled;
                    should_update = true;
                }
            }

            if let Some(period) = update.period {
                Self::ensure_period_not_zero(period)?;

                if period != settings.period {
                    settings.period = period;
                    should_update = true;
                }
            }

            if let Some(period_limit) = update.period_limit {
                Self::ensure_period_limit_not_zero(period_limit)?;

                if period_limit != settings.period_limit {
                    Self::ensure_drip_limit_lte_period_limit(settings.drip_limit, period_limit)?;

                    settings.period_limit = period_limit;
                    should_update = true;
                }
            }

            if let Some(drip_limit) = update.drip_limit {
                Self::ensure_drip_limit_not_zero(drip_limit)?;

                if drip_limit != settings.drip_limit {
                    Self::ensure_drip_limit_lte_period_limit(drip_limit, settings.period_limit)?;

                    settings.drip_limit = drip_limit;
                    should_update = true;
                }
            }

            ensure!(should_update, Error::<T>::NothingToUpdate);

            FaucetByAccount::<T>::insert(faucet.clone(), settings);
            Self::deposit_event(RawEvent::FaucetUpdated(faucet));
            Ok(())
        }

        #[weight = 20_000 + T::DbWeight::get().reads_writes(0, 0) + 20_000 * faucets.len() as u64]
        pub fn remove_faucets(
            origin,
            faucets: Vec<T::AccountId>
        ) -> DispatchResult {

            ensure_root(origin)?;

            ensure!(!faucets.len().is_zero(), Error::<T>::NoFaucetsProvided);

            let unique_faucets = faucets.iter().collect::<BTreeSet<_>>();
            for faucet in unique_faucets.iter() {
                FaucetByAccount::<T>::remove(faucet);
            }

            Self::deposit_event(RawEvent::FaucetsRemoved(faucets));
            Ok(())
        }

        #[weight = 50_000 + T::DbWeight::get().reads_writes(3, 1)]
        pub fn drip(
            origin, // Should be a faucet account
            recipient: T::AccountId,
            amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let faucet = ensure_signed(origin)?;

            // Validate input values
            ensure!(faucet != recipient, Error::<T>::RecipientEqualsFaucet);
            ensure!(amount > Zero::zero(), Error::<T>::ZeroDripAmountProvided);

            let mut settings = Self::require_faucet(&faucet)?;
            ensure!(settings.enabled, Error::<T>::FaucetDisabled);
            ensure!(amount <= settings.drip_limit, Error::<T>::DripLimitReached);

            let faucet_balance = T::Currency::free_balance(&faucet);
            ensure!(amount <= faucet_balance, Error::<T>::NotEnoughFreeBalanceOnFaucet);

            let current_block = <system::Pallet<T>>::block_number();

            if settings.next_period_at <= current_block {
                // Move to the next period and reset the period stats
                settings.next_period_at = current_block.saturating_add(settings.period);
                settings.dripped_in_current_period = Zero::zero();
            }

            // Calculate have many tokens still can be dripped in the current period
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

            FaucetByAccount::<T>::insert(&faucet, settings);

            Self::deposit_event(RawEvent::Dripped(faucet, recipient, amount));
            Ok(Pays::No.into())
        }
    }
}

impl<T: Config> Module<T> {

    pub fn require_faucet(faucet: &T::AccountId) -> Result<Faucet<T>, DispatchError> {
        Ok(Self::faucet_by_account(faucet).ok_or(Error::<T>::FaucetNotFound)?)
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

impl<T: Config> Faucet<T> {

    pub fn new(
        period: T::BlockNumber,
        period_limit: BalanceOf<T>,
        drip_limit: BalanceOf<T>,
    ) -> Self {
        Self {
            enabled: true,
            period,
            period_limit,
            drip_limit,

            next_period_at: Zero::zero(),
            dripped_in_current_period: Zero::zero(),
        }
    }
}
