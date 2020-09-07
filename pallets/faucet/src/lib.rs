//! # Faucet Module
//!
//! The Faucet module allows a root key (sudo) to add accounts (faucets) that are eligible
//! to drip free tokens to other accounts (recipients).

// TODO rename pallet 'faucet' to 'faucets'? (y)
// TODO refactor sudo to generic account + add 'created' to FaucetSettings so we can check owner

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};
use sp_runtime::RuntimeDebug;
use sp_runtime::traits::{Saturating, Zero};
use sp_std::{
	prelude::*,
	iter::FromIterator,
	collections::btree_set::BTreeSet,
};

use frame_support::{
	decl_module, decl_storage, decl_event, decl_error, ensure,
	dispatch::{DispatchResult, DispatchError},
	traits::{Get, Currency, ExistenceRequirement},
	weights::Pays,
};
use frame_system::{self as system, ensure_signed, ensure_root};

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

type DropId = u64;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Drop<T: Trait> {
	id: DropId,
	first_drop_at: T::BlockNumber,
	total_dropped: BalanceOf<T>
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct FaucetSettings<BlockNumber, Balance> {
	period: Option<BlockNumber>, // TODO rename
	period_limit: Balance

	// TODO add: min_amount: Balance
	// TODO add: max_limit: Balance
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct FaucetSettingsUpdate<BlockNumber, Balance> {
	period: Option<Option<BlockNumber>>,
	period_limit: Option<Balance>
}

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

/// The pallet's configuration trait.
pub trait Trait: system::Trait {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

	type Currency: Currency<Self::AccountId>;
}

decl_storage! {
	trait Store for Module<T: Trait> as FaucetModule {
		pub NextDropId get(fn next_drop_id): DropId = 1;

		pub DropById get(fn drop_by_id):
			map hasher(twox_64_concat) DropId
			=> Option<Drop<T>>;

		pub DropIdByRecipient get(fn drop_id_by_recipient):
			map hasher(twox_64_concat) T::AccountId
			=> Option<DropId>;

		pub DropIdByAlias get(fn drop_id_by_alias):
			map hasher(blake2_128_concat) Vec<u8>
			=> Option<DropId>;

		// TODO rename to SettingsByFaucet ? (maybe)
		pub FaucetSettingsByAccount get(fn faucet_settings_by_account):
			map hasher(twox_64_concat) T::AccountId
			=> Option<FaucetSettings<T::BlockNumber, BalanceOf<T>>>;
	}
}

decl_event!(
	pub enum Event<T> where
		AccountId = <T as system::Trait>::AccountId,
		Balance = BalanceOf<T>
	{
		FaucetAdded(AccountId),
		FaucetUpdated(AccountId),
		FaucetsRemoved(Vec<AccountId>),
		Dropped(
			AccountId, // faucet
			AccountId, // recipient
			Balance // amount dropped
		),
	}
);

// The pallet's errors
decl_error! {
	pub enum Error for Module<T: Trait> {
		FaucetNotFound,
		FaucetAlreadyAdded,
		FaucetLimitReached,
		NoFreeBalanceOnAccount,
		NothingToUpdate,
		ZeroAmount,
	}
}

// The pallet's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing errors
		type Error = Error<T>;

		// Initializing events
		fn deposit_event() = default;

		#[weight = T::DbWeight::get().reads_writes(1, 1) + 10_000]
		pub fn add_faucet(
			origin,
			faucet: T::AccountId,
			settings: FaucetSettings<T::BlockNumber, BalanceOf<T>>
		) -> DispatchResult {
			ensure_root(origin)?;

			ensure!(
				Self::require_faucet_settings(&faucet).is_ok(),
				Error::<T>::FaucetAlreadyAdded
			);

			ensure!(
				T::Currency::free_balance(&faucet) >= T::Currency::minimum_balance(),
				Error::<T>::NoFreeBalanceOnAccount
			);

			FaucetSettingsByAccount::<T>::insert(faucet.clone(), settings);

			Self::deposit_event(RawEvent::FaucetAdded(faucet));
			Ok(())
		}

		#[weight = T::DbWeight::get().reads_writes(1, 1) + 20_000]
		pub fn update_faucet(
			origin,
			faucet: T::AccountId,
			update: FaucetSettingsUpdate<T::BlockNumber, BalanceOf<T>>
		) -> DispatchResult {
			ensure_root(origin)?;

			let has_updates =
				update.period.is_some() ||
				update.period_limit.is_some();

			ensure!(has_updates, Error::<T>::NothingToUpdate);

			let mut settings = Self::require_faucet_settings(&faucet)?;

			// `true` if there is at least one updated field.
			let mut should_update = false;

			if let Some(period) = update.period {
				if period != settings.period {
					settings.period = period;
					should_update = true;
				}
			}

			if let Some(period_limit) = update.period_limit {
				if period_limit != settings.period_limit {
					settings.period_limit = period_limit;
					should_update = true;
				}
			}

			if should_update {
				FaucetSettingsByAccount::<T>::insert(faucet.clone(), settings);
				Self::deposit_event(RawEvent::FaucetUpdated(faucet));
			}
			Ok(())
		}

		#[weight = T::DbWeight::get().reads_writes(0, 1) + 10_000
			+ 5_000 * faucets.len() as u64
		]
		pub fn remove_faucets(
			origin,
			faucets: Vec<T::AccountId>
		) -> DispatchResult {
			ensure_root(origin.clone())?;
			let root_key = ensure_signed(origin)?;

			let unique_faucets = BTreeSet::from_iter(faucets.iter());
			for faucet in unique_faucets.iter() {
				if Self::require_faucet_settings(faucet).is_ok() {
					T::Currency::transfer(
						faucet,
						&root_key,
						T::Currency::free_balance(faucet),
						ExistenceRequirement::AllowDeath
					)?;
				}

				// TODO move inside of if above? ^^
				FaucetSettingsByAccount::<T>::remove(faucet);
			}

			Self::deposit_event(RawEvent::FaucetsRemoved(faucets));
			Ok(())
		}

		#[weight = (
			T::DbWeight::get().reads_writes(6, 4) + 50_000, // TODO why 50k ?
			Pays::No
		)]
		pub fn drip(
			origin, // faucet
			amount: BalanceOf<T>,
			recipient: T::AccountId,
			recipient_aliases: Vec<u8> // TODO refactor to Vec<Vec<u8>> ? (yes)
		) -> DispatchResult {
			let faucet = ensure_signed(origin)?;

			ensure!(amount > Zero::zero(), Error::<T>::ZeroAmount);

			let settings = Self::require_faucet_settings(&faucet)?;

			// TODO check amount against settings.min_amount

			let maybe_drop = Self::drop_id_by_recipient(&recipient)
				.ok_or_else(|| Self::drop_id_by_alias(&recipient_aliases))
				.ok()
				.and_then(Self::drop_by_id);

			let mut is_new_drop = false;
			let mut drop = maybe_drop.unwrap_or_else(|| {
				is_new_drop = true;
				let drop_id = Self::next_drop_id();
				Drop::<T>::new(drop_id)
			});

			if !is_new_drop {
				let now = <system::Module<T>>::block_number();

				// TODO rename var 'past'
				let past = now.saturating_sub(settings.period.unwrap_or_else(Zero::zero));

				if past >= drop.first_drop_at {
					drop.first_drop_at = now;
					if settings.period.is_some() {
						drop.total_dropped = Zero::zero();
					}
				}
			}

			let amount_allowed = settings.period_limit.saturating_sub(drop.total_dropped);
			ensure!(amount_allowed >= amount, Error::<T>::FaucetLimitReached);

			T::Currency::transfer(
				&faucet,
				&recipient,
				amount,
				ExistenceRequirement::KeepAlive
			)?;

			drop.total_dropped = drop.total_dropped.saturating_add(amount);

			DropById::<T>::insert(drop.id, drop.clone());
			if is_new_drop {
				DropIdByRecipient::<T>::insert(&recipient, drop.id);
				DropIdByAlias::insert(recipient_aliases, drop.id);
				NextDropId::mutate(|x| *x += 1);
			}

			Self::deposit_event(RawEvent::Dropped(faucet, recipient, amount));
			Ok(())
		}
	}
}

impl<T: Trait> Module<T> {
	pub fn require_faucet_settings(
		faucet: &T::AccountId
	) -> Result<FaucetSettings<T::BlockNumber, BalanceOf<T>>, DispatchError> {
		Ok(Self::faucet_settings_by_account(faucet).ok_or(Error::<T>::FaucetNotFound)?)
	}
}

impl<T: Trait> Drop<T> {
	pub fn new(id: DropId) -> Self {
		Self {
			id,
			first_drop_at: <system::Module<T>>::block_number(),
			total_dropped: Zero::zero()
		}
	}
}
