#![cfg_attr(not(feature = "std"), no_std)]

// use bitflags::bitflags;
use codec::{Encode, Decode};
use frame_support::{
	decl_module, decl_storage, decl_event,
	dispatch::DispatchResultWithPostInfo,
	traits::{EnsureOrigin, Get},
	weights::Pays,
};
// use sp_runtime::RuntimeDebug;
use sp_std::{
	vec::Vec, collections::btree_set::BTreeSet,
};
use df_traits::TrustHandler;
/*
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;
*/
pub trait Trait: frame_system::Trait {
	/// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

	/// Required origin to change someone's trust level.
	type SetTrustLevel: EnsureOrigin<Self::Origin>;
}

/*bitflags! {
	#[derive(Encode, Decode, Default)]
	pub struct TrustLevels: i8 {
		const EMAIL_CONFIRMED = 0b00000001;
		const PHONE_NUMBER_CONFIRMED = 0b00000010;
	}
}*/

#[derive(Encode, Decode, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum TrustLevels {
	None,
	EmailConfirmed,
	PhoneNumberConfirmed,
}

impl Default for TrustLevels {
	fn default() -> Self {
		Self::None
	}
}

/*impl TrustLevels {
	/// Choose all variants except for `one`.
	pub fn except(one: TrustLevels) -> TrustLevels {
		let mut mask = Self::all();
		mask.toggle(one);
		mask
	}
}*/

decl_storage! {
	trait Store for Module<T: Trait> as TrustModule {
		TrustLevelsByAccount get(fn trust_levels_by_account): map
			hasher(blake2_128_concat) T::AccountId
			=> Vec<TrustLevels>;
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId {
		TrustLevelChanged(AccountId, Vec<TrustLevels>),
	}
);

impl<T: Trait> Module<T> {
	fn account_trust_levels_contains(who: &T::AccountId, trust_level: TrustLevels) -> bool {
		let trust_levels = Self::trust_levels_by_account(who);
		trust_levels.contains(&trust_level)
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn set_trust_level(origin, who: T::AccountId, new_trust_levels: Vec<TrustLevels>) -> DispatchResultWithPostInfo {
			T::SetTrustLevel::ensure_origin(origin)?;

			let unique_trust_levels: BTreeSet<TrustLevels> = new_trust_levels.into_iter().collect();
			let trust_levels: Vec<TrustLevels> = unique_trust_levels.into_iter().collect();

			TrustLevelsByAccount::<T>::insert(&who, trust_levels.clone());

			Self::deposit_event(RawEvent::TrustLevelChanged(who, trust_levels));
			Ok(Pays::No.into())
		}
	}
}

impl<T: Trait> TrustHandler<T::AccountId> for Module<T> {
	fn is_trusted_account(who: &T::AccountId) -> bool {
		let trust_levels = Self::trust_levels_by_account(who);
		!trust_levels.is_empty()
	}

	fn is_email_confirmed(who: &T::AccountId) -> bool {
		Self::account_trust_levels_contains(who, TrustLevels::EmailConfirmed)
	}

	fn is_phone_number_confirmed(who: &T::AccountId) -> bool {
		Self::account_trust_levels_contains(who, TrustLevels::PhoneNumberConfirmed)
	}
}
