//! # Donations Module
//!
//! The Donations module allows accounts to make donations (aka tips) to other accounts, spaces,
//! or posts.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};
use sp_std::prelude::*;
use sp_runtime::RuntimeDebug;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, ensure, traits::Get,
    dispatch::{DispatchResult, DispatchError},
    traits::{Currency, ExistenceRequirement}
};
use frame_system::{self as system, ensure_signed};

use pallet_posts::{Module as Posts, PostId};
use pallet_spaces::{Module as Spaces};
use pallet_utils::{Content, WhoAndWhen, SpaceId};

type BalanceOf<T> = <<T as pallet_utils::Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

pub type DonationId = u64;

// TODO find a better name. Maybe DonationReason?
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum DonationRecipient<AccountId> {
    Account(AccountId),
    Space(SpaceId),
    Post(PostId),
}

/// A struct that describes a single donation made by an account.
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Donation<T: Trait> {
    pub id: DonationId,
    pub created: WhoAndWhen<T>,
    pub recipient: DonationRecipient<T::AccountId>, // TODO rename to 'reason'?
    pub donation_wallet: T::AccountId, // TODO rename to 'recipient_wallet' or 'recipient'?
    pub amount: BalanceOf<T>,
    pub comment_id: Option<PostId>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct DonationSettings<BalanceOf> {
    pub donations_allowed: bool, // TODO rename to donations_enabled ?
    pub min_amount: Option<BalanceOf>,
    pub max_amount: Option<BalanceOf>,

    // TODO think about 'post owner can receive donations' or 'who receives donations on post'?
    // TODO % of post donations that space takes. 0% by default.
    // TODO donation_posts_allowed: bool
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct DonationSettingsUpdate<BalanceOf> {
    pub donations_allowed: Option<bool>,
    pub min_amount: Option<Option<BalanceOf>>,
    pub max_amount: Option<Option<BalanceOf>>,
}

pub trait Trait: system::Trait
    + pallet_posts::Trait
    + pallet_spaces::Trait
    + pallet_utils::Trait
{
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
    trait Store for Module<T: Trait> as DonationsModule {

        /// An id for the next donation.
        pub NextDonationId get(fn next_donation_id):
            DonationId = 1;

        /// Donation details by its id (key).
        pub DonationById get(fn donation_by_id):
            map hasher(twox_64_concat) DonationId
            => Option<Donation<T>>;

        /// Ids of all donations made by a supporter account (key).
        pub DonationIdsBySupporter get(fn donations_by_supporter):
            map hasher(blake2_128_concat) T::AccountId
            => Vec<DonationId>;

        /// Ids of all donations received by this recipient (key).
        /// A recipient can be either account, space or post.
        pub DonationIdsByRecipient get(fn donation_ids_by_recipient):
            map hasher(blake2_128_concat) DonationRecipient<T::AccountId>
            => Vec<DonationId>;

        /// A custom wallet for a certain recipient (key).
        /// This means that any account, space or post can set up a custom wallet address
        /// that will be used for future donations to this recipient.
        pub DonationWalletByRecipient get(fn donation_wallet_by_recipient):
            map hasher(blake2_128_concat) DonationRecipient<T::AccountId>
            => Option<T::AccountId>;

        /// A custom donation settings for a certain recipient (key).
        /// This means that any account, space or post can set up a custom donation settings
        /// that will be applied to future donations to this recipient.å
        pub DonationSettingsByRecipient get(fn donation_settings_by_recipient):
            map hasher(blake2_128_concat) DonationRecipient<T::AccountId>
            => Option<DonationSettings<BalanceOf<T>>>;
    }
}

decl_event!(
    pub enum Event<T> where
        AccountId = <T as system::Trait>::AccountId,
        DonationRecipient = DonationRecipient<<T as system::Trait>::AccountId>,
        BalanceOf = BalanceOf<T>
    {
        Donated(
            // Supporter - from whom if was donated.
            AccountId,
            // To which recipient it was donated.
            DonationRecipient,
            // Amount of donated tokens.
            BalanceOf
        ),
        DonationWalletUpdated(
            // Origin - who set a new wallet.
            AccountId,
            // For which recipient a new wallet was set.
            DonationRecipient,
            // An address of a new wallet.
            AccountId
        ),
        DonationWalletRemoved(
            // Origin - who removed a donation wallet.
            AccountId,
            // From which recipient a wallet was removed.
            DonationRecipient
        ),
        DonationSettingsUpdated(
            // Origin - who updated the donation settings.
            AccountId,
            // For which recipient the donation settings have been updated.
            DonationRecipient
        ),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Thrown if an origin is not allowed to change a donation wallet,
        /// because their are not an owner of this recipient (e.g. space or post owner).
        NotRecipientManager,
        /// Nothing to update in the donation settings.
        NoUpdatesForDonationSettings,
        /// Donation amount is less than a `min_amount` in donation settings.
        TooSmallDonation,
        /// Donation amount is greater than a `max_amount` in donation settings.
        TooLargeDonation,
        /// A space, a post or an account doesn't accept donations based on its donation settings.
        DonationsAreDisabled,
    }
}

decl_module! {
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {

    type Error = Error<T>;

    fn deposit_event() = default;

    /// Donate a certain `amount` of tokens to a `recipient` with an optional comment message.
    #[weight = 10_000 + T::DbWeight::get().reads_writes(5, 4)]
    pub fn donate(
        origin,
        recipient: DonationRecipient<T::AccountId>,
        amount: BalanceOf<T>,
        comment_content: Content
    ) -> DispatchResult {
        let supporter = ensure_signed(origin)?;

        let settings = Self::resolve_donation_settings(recipient.clone())?;

        ensure!(settings.donations_allowed, Error::<T>::DonationsAreDisabled);

        if let Some(min_amount) = settings.min_amount {
            ensure!(amount >= min_amount, Error::<T>::TooSmallDonation);
        }
        
        if let Some(max_amount) = settings.max_amount {
            ensure!(amount <= max_amount, Error::<T>::TooLargeDonation);
        }
            
        let donation_wallet = Self::get_recipient_wallet(recipient.clone())?;
        let donation_id = Self::next_donation_id();

        // TODO check settings.donation_posts_allowed to post as a root post to space.
        // TODO create a comment under the post or a new post in DonationSpace

        let donation = Donation {
            id: donation_id,
            created: WhoAndWhen::<T>::new(supporter.clone()),
            recipient: recipient.clone(),
            donation_wallet: donation_wallet.clone(),
            amount,
            comment_id: None // TODO put id of created comment
        };

        // Transfer donated tokens from a supporter to a donation wallet of this reason.
        T::Currency::transfer(&supporter, &donation_wallet, amount, ExistenceRequirement::KeepAlive)?;

        DonationById::<T>::insert(donation_id, donation);
        DonationIdsBySupporter::<T>::mutate(supporter.clone(), |ids| ids.push(donation_id));
        DonationIdsByRecipient::<T>::mutate(recipient.clone(), |ids| ids.push(donation_id));
        NextDonationId::mutate(|n| { *n += 1; });

        Self::deposit_event(RawEvent::Donated(supporter, recipient, amount));
        Ok(())
    }

    /// Set a new wallet address for a `recipient`.
    #[weight = 10_000 + T::DbWeight::get().reads_writes(1, 1)]
    pub fn set_donation_wallet(
        origin,
        recipient: DonationRecipient<T::AccountId>,
        new_wallet: T::AccountId,
    ) -> DispatchResult {
        let who = ensure_signed(origin)?;
        Self::ensure_recipient_manager(who.clone(), recipient.clone())?;

        DonationWalletByRecipient::<T>::insert(recipient.clone(), new_wallet.clone());
        Self::deposit_event(RawEvent::DonationWalletUpdated(who, recipient, new_wallet));
        Ok(())
    }

    /// Remove a custom wallet address from a `recipient`.
    /// When donations wallet is removed, an address of `recipient` manager (owner)
    /// will be used as a wallet address for donations.
    #[weight = 10_000 + T::DbWeight::get().reads_writes(1, 1)]
    pub fn remove_donation_wallet(
        origin,
        recipient: DonationRecipient<T::AccountId>,
    ) -> DispatchResult {
        let who = ensure_signed(origin)?;
        Self::ensure_recipient_manager(who.clone(), recipient.clone())?;

        DonationWalletByRecipient::<T>::remove(recipient.clone());
        Self::deposit_event(RawEvent::DonationWalletRemoved(who, recipient));
        Ok(())
    }

    /// Override the default donation settings of this pallet with the settings
    /// specific to this `recipient`.
    #[weight = 10_000 + T::DbWeight::get().reads_writes(3, 1)]
    pub fn update_settings(
        origin,
        recipient: DonationRecipient<T::AccountId>,
        update: DonationSettingsUpdate<BalanceOf<T>>
    ) -> DispatchResult {
        let who = ensure_signed(origin)?;

        let has_updates =
            update.donations_allowed.is_some() ||
            update.min_amount.is_some() ||
            update.max_amount.is_some();

        ensure!(has_updates, Error::<T>::NoUpdatesForDonationSettings);

        Self::ensure_recipient_manager(who.clone(), recipient.clone())?;

        // `true` if there is at least one updated field.
        let mut should_update = false;

        let mut settings = Self::resolve_donation_settings(recipient.clone())?;

        if let Some(donations_allowed) = update.donations_allowed {
            if donations_allowed != settings.donations_allowed {
                settings.donations_allowed = donations_allowed;
                should_update = true;
            }
        }

        if let Some(min_amount) = update.min_amount {
            if min_amount != settings.min_amount {
                settings.min_amount = min_amount;
                should_update = true;
            }
        }

        if let Some(max_amount) = update.max_amount {
            if max_amount != settings.max_amount {
                settings.max_amount = max_amount;
                should_update = true;
            }
        }

        if should_update {
            DonationSettingsByRecipient::<T>::insert(recipient.clone(), settings);
            Self::deposit_event(RawEvent::DonationSettingsUpdated(who, recipient));
        }
        Ok(())
    }
  }
}

impl<BalanceOf> Default for DonationSettings<BalanceOf> {

    /// Returns a default value for donation settings.
    fn default() -> Self {
        DonationSettings {
            donations_allowed: true,
            min_amount: None,
            max_amount: None,
        }
    }
}

impl<T: Trait> Module<T> {

    /// Get a space owner and wrap it into `DonationRecipient`.
    pub fn resolve_space_owner_as_recipient(space_id: SpaceId) -> Result<DonationRecipient<T::AccountId>, DispatchError> {
        let space = Spaces::<T>::require_space(space_id)?;
        Ok(DonationRecipient::Account(space.owner))
    }

    /// Get a post owner and wrap it into `DonationRecipient`.
    pub fn resolve_post_owner_as_recipient(post_id: PostId) -> Result<DonationRecipient<T::AccountId>, DispatchError> {
        let post = Posts::<T>::require_post(post_id)?;
        Ok(DonationRecipient::Account(post.owner))
    }

    /// Resolve the donation settings by a specified reason or its owner's reason
    /// if it's a space or a post.
    pub fn resolve_donation_settings(
        recipient: DonationRecipient<T::AccountId>
    ) -> Result<DonationSettings<BalanceOf<T>>, DispatchError> {
        let maybe_settings = Self::donation_settings_by_recipient(recipient.clone());
        if let Some(settings) = maybe_settings {
            return Ok(settings)
        }
        
        // If a donation wallet is not defined for this recipient:
        match recipient {
            DonationRecipient::Account(_) => {
                Ok(DonationSettings::default())
            }
            DonationRecipient::Space(space_id) => {
                let space_owner = Self::resolve_space_owner_as_recipient(space_id)?;
                Self::resolve_donation_settings(space_owner)
            },
            DonationRecipient::Post(post_id) => {
                let post_owner = Self::resolve_space_owner_as_recipient(post_id)?;
                Self::resolve_donation_settings(post_owner)
            },
        }
    }

    /// Returns an account that should be used as a donation wallet for this recipient.
    pub fn get_recipient_wallet(
        recipient: DonationRecipient<T::AccountId>
    ) -> Result<T::AccountId, DispatchError> {
        let wallet = DonationWalletByRecipient::<T>::get(recipient.clone());
        if let Some(account) = wallet {
            return Ok(account)
        }

        // If a donation wallet is not defined for this recipient:
        match recipient {
            DonationRecipient::Account(account) => {
                Ok(account)
            }
            DonationRecipient::Space(space_id) => {
                let owner = Self::resolve_space_owner_as_recipient(space_id)?;
                Self::get_recipient_wallet(owner)
            },
            DonationRecipient::Post(post_id) => {
                let owner = Self::resolve_space_owner_as_recipient(post_id)?;
                Self::get_recipient_wallet(owner)
            },
        }
    }

    /// Checks if `maybe_owner` can manage / is an owner of a `recipient`.
    pub fn ensure_recipient_manager(
        maybe_owner: T::AccountId,
        recipient: DonationRecipient<T::AccountId>,
    ) -> DispatchResult {
        let is_owner = match recipient {
            DonationRecipient::Account(account) => {
                account == maybe_owner
            },
            DonationRecipient::Space(space_id) => {
                let space = Spaces::<T>::require_space(space_id)?;
                space.is_owner(&maybe_owner)
            },
            DonationRecipient::Post(post_id) => {
                let post = Posts::<T>::require_post(post_id)?;
                post.is_owner(&maybe_owner)
            },
        };
        ensure!(is_owner, Error::<T>::NotRecipientManager);
        Ok(())
    }
}
