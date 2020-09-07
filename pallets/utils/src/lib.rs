#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_module, decl_storage, decl_event,
    dispatch::{DispatchError, DispatchResult}, ensure,
    traits::{
        Currency, Get,
        Imbalance, OnUnbalanced,
    },
};
use sp_runtime::RuntimeDebug;
use sp_std::{
    collections::btree_set::BTreeSet,
    prelude::*,
};
use frame_system::{self as system};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub type SpaceId = u64;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct WhoAndWhen<T: Trait> {
    pub account: T::AccountId,
    pub block: T::BlockNumber,
    pub time: T::Moment,
}

impl<T: Trait> WhoAndWhen<T> {
    pub fn new(account: T::AccountId) -> Self {
        WhoAndWhen {
            account,
            block: <system::Module<T>>::block_number(),
            time: <pallet_timestamp::Module<T>>::now(),
        }
    }
}

#[derive(Encode, Decode, Ord, PartialOrd, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum User<AccountId> {
    Account(AccountId),
    Space(SpaceId),
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum Content {
    /// No content.
    None,
    /// A raw vector of bytes.
    Raw(Vec<u8>),
    /// IPFS CID v0 of content.
    IPFS(Vec<u8>),
    /// Hypercore protocol (former DAT) id of content.
    Hyper(Vec<u8>),
}

impl Content {
    pub fn is_none(&self) -> bool {
        self == &Self::None
    }

    pub fn is_some(&self) -> bool {
        !self.is_none()
    }
}

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;
type NegativeImbalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::NegativeImbalance;

pub trait Trait: system::Trait + pallet_timestamp::Trait
{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    /// The currency mechanism.
    type Currency: Currency<Self::AccountId>;

    /// A valid length of IPFS CID in bytes.
    type IpfsCidLen: Get<u32>;

    /// Min length of a space handle.
    type MinHandleLen: Get<u32>;

    /// Max length of a space handle.
    type MaxHandleLen: Get<u32>;
}

decl_storage! {
    trait Store for Module<T: Trait> as UtilsModule {
        TreasuryAccount build(|config| config.treasury_account.clone()): T::AccountId;
    }
    add_extra_genesis {
        config(treasury_account): T::AccountId;
        build(|config| {
			// Create Treasury account
			let _ = T::Currency::make_free_balance_be(
				&config.treasury_account,
				T::Currency::minimum_balance(),
			);
		});
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        const IpfsCidLen: u32 = T::IpfsCidLen::get();

        const MinHandleLen: u32 = T::MinHandleLen::get();

        const MaxHandleLen: u32 = T::MaxHandleLen::get();

        // Initializing errors
        type Error = Error<T>;

        // Initializing events
        fn deposit_event() = default;
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Account is blocked in a given space.
        AccountIsBlocked,
        /// Content is blocked in a given space.
        ContentIsBlocked,
        /// IPFS CID is invalid.
        InvalidIpfsCid,
        /// `Raw` content type is not yet supported.
        RawContentTypeNotSupported,
        /// `Hyper` content type is not yet supported.
        HypercoreContentTypeNotSupported,
        /// Space handle is too short.
        HandleIsTooShort,
        /// Space handle is too long.
        HandleIsTooLong,
        /// Space handle contains invalid characters.
        HandleContainsInvalidChars,
        /// Content type is `None`.
        ContentIsEmpty,
    }
}

decl_event!(
    pub enum Event<T> where Balance = BalanceOf<T>
    {
		Deposit(Balance),
    }
);

fn num_bits<P>() -> usize {
    sp_std::mem::size_of::<P>() * 8
}

/// Returns `None` for `x == 0`.
pub fn log_2(x: u32) -> Option<u32> {
    if x > 0 {
        Some(
            num_bits::<u32>() as u32
            - x.leading_zeros()
            - 1
        )
    } else { None }
}

pub fn vec_remove_on<F: PartialEq>(vector: &mut Vec<F>, element: F) {
    if let Some(index) = vector.iter().position(|x| *x == element) {
        // TODO fix: swap_remove doesn't remove tha last element.
        vector.swap_remove(index);
    }
}

impl<T: Trait> Module<T> {

    pub fn is_valid_content(content: Content) -> DispatchResult {
        match content {
            Content::None => Ok(()),
            Content::Raw(_) => Err(Error::<T>::RawContentTypeNotSupported.into()),
            Content::IPFS(ipfs_cid) => {

                // TODO write tests for IPFS CID v0 and v1.

                ensure!(ipfs_cid.len() == T::IpfsCidLen::get() as usize, Error::<T>::InvalidIpfsCid);
                Ok(())
            },
            Content::Hyper(_) => Err(Error::<T>::HypercoreContentTypeNotSupported.into())
        }
    }

    pub fn convert_users_vec_to_btree_set(
        users_vec: Vec<User<T::AccountId>>
    ) -> Result<BTreeSet<User<T::AccountId>>, DispatchError> {
        let mut users_set: BTreeSet<User<T::AccountId>> = BTreeSet::new();

        for user in users_vec.iter() {
            users_set.insert(user.clone());
        }

        Ok(users_set)
    }

    /// Check if a handle contains only valid chars: 0-9, a-z, _.
    /// An example of a valid handle: `good_handle_123`.
    fn is_valid_handle_char(c: u8) -> bool {
        matches!(c, b'0'..=b'9' | b'a'..=b'z' | b'_')
    }

    /// Check if a handle length fits into min/max length requirements.
    /// Lowercase a handle.
    /// Check if a handle contains only valid chars: 0-9, a-z, _.
    /// Check if a handle is unique across all spaces' handles (requires one storage read).
    pub fn lowercase_and_validate_a_handle(handle: Vec<u8>) -> Result<Vec<u8>, DispatchError> {
        // Check min and max lengths of a handle:
        ensure!(handle.len() >= T::MinHandleLen::get() as usize, Error::<T>::HandleIsTooShort);
        ensure!(handle.len() <= T::MaxHandleLen::get() as usize, Error::<T>::HandleIsTooLong);

        let handle_in_lowercase = handle.to_ascii_lowercase();

        // Check if a handle contains only valid chars: 0-9, a-z, _.
        let is_only_valid_chars = handle_in_lowercase.iter().all(|&x| Self::is_valid_handle_char(x));
        ensure!(is_only_valid_chars, Error::<T>::HandleContainsInvalidChars);

        // Return a lower-cased version of a handle.
        Ok(handle_in_lowercase)
    }

    /// Ensure that a given content is not `None`.
    pub fn ensure_content_is_some(content: &Content) -> DispatchResult {
        ensure!(content.is_some(), Error::<T>::ContentIsEmpty);
        Ok(())
    }
}

impl<T: Trait> OnUnbalanced<NegativeImbalanceOf<T>> for Module<T> {
    fn on_nonzero_unbalanced(amount: NegativeImbalanceOf<T>) {
        let numeric_amount = amount.peek();
        let treasury_account = TreasuryAccount::<T>::get();

        // Must resolve into existing but better to be safe.
        let _ = T::Currency::resolve_creating(&treasury_account, amount);

        Self::deposit_event(RawEvent::Deposit(numeric_amount));
    }
}
