#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};

use frame_support::{decl_module, decl_storage, decl_event, decl_error, ensure, dispatch::DispatchResult};
use frame_system::{ensure_root, Module as System};
use sp_runtime::{
	RuntimeDebug,
	traits::Zero,
};

use pallet_spaces::Module as Spaces;
use pallet_utils::SpaceId;

// FIXME: uncomment when tests added
/*#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;*/

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Referrer<AccountId, BlockNumber> {
	referrer_id: Option<AccountId>,
	block_number: BlockNumber,
}

pub trait OnboardAccount<AccountId> {
	fn onboard_account(account: AccountId, referrer_space_id: Option<SpaceId>) -> DispatchResult;
}

impl<T: Trait> OnboardAccount<T::AccountId> for Module<T> {
	fn onboard_account(account: T::AccountId, referrer_space_id: Option<SpaceId>) -> DispatchResult {
		Self::ensure_account_not_onboarded(&account)?;

		let account_nonce = System::<T>::account_nonce(&account);
		let account_balance = pallet_balances::Module::<T>::free_balance(&account);

		let mut referrer_id = None;
		if account_nonce.is_zero() && account_balance.is_zero() {
			referrer_id = referrer_space_id
				.and_then(|space_id| Spaces::<T>::require_space(space_id)
					.map(|space| space.owner)
					.ok()
				);
		}

		ReferrerByAccount::<T>::insert(&account, Referrer {
			referrer_id,
			block_number: System::<T>::block_number()
		});

		Self::deposit_event(RawEvent::Onboarded(account));
		Ok(())
	}
}

pub trait Trait: frame_system::Trait + pallet_spaces::Trait + pallet_balances::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

decl_storage! {
	trait Store for Module<T: Trait> as OnboardModule {
		pub ReferrerByAccount get(fn referrer_by_account):
			map hasher(blake2_128_concat) T::AccountId => Option<Referrer<T::AccountId, T::BlockNumber>>;
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId {
		/// Account was successfully onboarded.
		Onboarded(AccountId),
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Account is already onboarded.
		AccountAlreadyOnboarded,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		#[weight = 10_000]
		pub fn onboard(origin, account_to_onboard: T::AccountId, referrer_space_id: Option<SpaceId>) -> DispatchResult {
			ensure_root(origin)?;

			Self::onboard_account(account_to_onboard, referrer_space_id)?;
			Ok(())
		}
	}
}

impl<T: Trait> Module<T> {
	fn ensure_account_not_onboarded(account: &T::AccountId) -> DispatchResult {
		ensure!(Self::referrer_by_account(&account).is_none(), Error::<T>::AccountAlreadyOnboarded);
		Ok(())
	}
}
