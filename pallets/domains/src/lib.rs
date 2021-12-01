//! # Module for storing purchased domains.
//!
//! Pallet that allows a trusted bridge account to store the user's purchased domains.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;


#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        dispatch::DispatchResult, pallet_prelude::*,
        traits::{Currency, ReservableCurrency},
    };
    use frame_system::Pallet as System;
    use frame_system::pallet_prelude::*;
    use scale_info::TypeInfo;
    use sp_runtime::traits::Saturating;
    use sp_std::vec::Vec;

    use df_traits::SpacesProvider;
    use pallet_utils::{Content, PostId, SpaceId, WhoAndWhen};
    use pallet_utils::Pallet as Utils;

    type DomainsVec = Vec<Vec<u8>>;

    const MIN_TLD_LENGTH: usize = 2;
    const MIN_DOMAIN_LENGTH: usize = 3;
    const MAX_DOMAIN_LENGTH: usize = 63;

    type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
    pub enum EntityId<AccountId> {
        Account(AccountId),
        Space(SpaceId),
        Post(PostId),
    }

    #[derive(Encode, Decode, TypeInfo)]
    #[scale_info(skip_type_params(T))]
    pub struct DomainMeta<T: Config> {
        created: WhoAndWhen<T>,
        updated: Option<WhoAndWhen<T>>,

        expires_at: T::BlockNumber,
        owner: T::AccountId,

        content: Content,
        inner_value: Option<EntityId<T::AccountId>>,
        outer_value: Option<Vec<u8>>,
    }

    impl<T: Config> DomainMeta<T> {
        fn new(
            expires_at: T::BlockNumber,
            owner: T::AccountId,
            content: Content,
            inner_value: Option<EntityId<T::AccountId>>,
            outer_value: Option<Vec<u8>>,
        ) -> Self {
            Self {
                created: WhoAndWhen::new(owner.clone()),
                updated: None,
                expires_at,
                owner,
                content,
                inner_value,
                outer_value,
            }
        }
    }

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_utils::Config {
        /// The overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The currency trait.
        type Currency: ReservableCurrency<Self::AccountId>;

        /// The loose coupled provider to get a space.
        type SpacesProvider: SpacesProvider;

        /// The maximum amount of time the domain may be held for.
        #[pallet::constant]
        type ReservationPeriodLimit: Get<Self::BlockNumber>;

        /// The length limit for the domains meta outer value.
        #[pallet::constant]
        type OuterValueLimit: Get<u16>;

        /// The tokens amount to deposit for the outer value.
        #[pallet::constant]
        type OuterValueDeposit: Get<BalanceOf<Self>>;

        // TODO: add price coefficients for different domains lengths
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn reserved_domain)]
    pub(super) type ReservedDomains<T> = StorageMap<_, Twox64Concat, Vec<u8>, bool, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn top_level_domain_allowed)]
    pub(super) type AllowedTopLevelDomains<T> =
        StorageMap<_, Twox64Concat, Vec<u8>, bool, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn purchased_domain)]
    pub(super) type PurchasedDomains<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, Vec<u8>, Blake2_128Concat, Vec<u8>, DomainMeta<T>>;

    #[pallet::storage]
    #[pallet::getter(fn trusted_account)]
    pub(super) type TrustedAccount<T: Config> = StorageValue<_, T::AccountId>;

    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        // TODO: add payment amount to the event
        /// The domain name was successfully stored.
        DomainStored(T::AccountId, Vec<u8>, Vec<u8>),
        /// The list of top level domains was successfully added.
        TopLevelDomainsAdded,
        /// The trusted bridge account was successfully set.
        TrustedAccountSet(T::AccountId),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// This account is not a trusted bridge account, and not allowed to store domains.
        AccountNotAllowedToStoreDomains,
        /// This domain cannot be purchased yet, because it is reserved.
        // TODO
        DomainReserved,
        /// This domain is already held by another account.
        DomainAlreadyStored,
        /// Lower than Second level domains are not allowed.
        // TODO
        LowerLevelDomainsNotAllowed,
        /// Outer value exceeds its length limit.
        OuterValueOffLengthLimit,
        /// Cannot store a domain for that long period of time.
        TooBigReservationPeriod,
        /// The top level domain may contain only A-Z, 0-9 and hyphen characters.
        TopLevelDomainContainsInvalidChar,
        /// The top level domain length must be between 3 and 63 characters, inclusive.
        TopLevelDomainIsOffLengthLimits,
        /// This top level domain is not allowed.
        TopLevelDomainNotAllowed,
        /// Pallet is inactive due to trusted bridge account account is not set.
        TrustedAccountNotSet,
        /// This inner value is not supported yet.
        InnerValueNotSupported,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(3, 1))]
        pub fn store(
            origin: OriginFor<T>,
            owner: T::AccountId,
            tld: Vec<u8>,
            user_domain: Vec<u8>,
            expires_in: T::BlockNumber,
            content: Content,
            inner_value: Option<EntityId<T::AccountId>>,
            outer_value: Option<Vec<u8>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let trusted_account = Self::require_trusted_account()?;
            ensure!(trusted_account == who, Error::<T>::AccountNotAllowedToStoreDomains);

            ensure!(
                expires_in <= T::ReservationPeriodLimit::get(),
                Error::<T>::TooBigReservationPeriod,
            );

            Utils::<T>::is_valid_content(content.clone())?;

            Self::ensure_tld_allowed(&tld)?;
            Self::ensure_valid_domain(&user_domain)?;

            // Note that while upper and lower case letters are allowed in domain
            // names, no significance is attached to the case. That is, two names with
            // the same spelling but different case are to be treated as if identical.
            let tld_lowered = tld.to_ascii_lowercase();
            let user_domain_lowered = user_domain.to_ascii_lowercase();

            ensure!(
                Self::purchased_domain(&tld_lowered, &user_domain_lowered).is_none(),
                Error::<T>::DomainAlreadyStored,
            );

            Self::ensure_valid_inner_value(&inner_value)?;
            Self::ensure_valid_outer_value(&outer_value)?;

            let expires_at = expires_in.saturating_add(System::<T>::block_number());
            let domain_meta = DomainMeta::new(
                expires_at,
                owner.clone(),
                content,
                inner_value,
                outer_value,
            );

            PurchasedDomains::<T>::insert(&tld_lowered, &user_domain_lowered, domain_meta);

            // TODO: calculate the payment amount and store it

            Self::deposit_event(Event::DomainStored(owner, tld, user_domain));
            Ok(Default::default())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(domains.len() as u64))]
        pub fn add_top_level_domains(
            origin: OriginFor<T>,
            domains: DomainsVec,
        ) -> DispatchResult {
            // TODO: refactor this
            ensure_root(origin)?;

            for domain in &domains {
                Self::ensure_valid_tld(domain)?;
            }

            domains.iter().for_each(|domain| AllowedTopLevelDomains::<T>::insert(domain, true));

            Self::deposit_event(Event::TopLevelDomainsAdded);
            Ok(Default::default())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn set_trusted_account(
            origin: OriginFor<T>,
            target: T::AccountId,
        ) -> DispatchResult {
            ensure_root(origin)?;

            TrustedAccount::<T>::put(&target);

            Self::deposit_event(Event::TrustedAccountSet(target));
            Ok(Default::default())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Checks the length of the provided u8 slice.
        fn ensure_domain_has_valid_length(
            string: &[u8],
            min: usize,
            max: usize,
            error: Error<T>,
        ) -> DispatchResult {
            let length = string.len();
            ensure!(length >= min && length <= max, error);

            Ok(Default::default())
        }

        /// Throws an error if domain contains invalid character.
        fn ensure_domain_contains_valid_chars(domain: &[u8], error: Error<T>) -> DispatchResult {
            let first_char_alpha = domain.first()
                .filter(|c| (**c).is_ascii_alphabetic());

            let last_char_not_hyphen = domain.last().filter(|c| **c != b'-');

            ensure!(
                first_char_alpha.is_some() && last_char_not_hyphen.is_some() &&
                domain.iter().all(|c| c.is_ascii_alphanumeric() || *c == b'-'),
                error
            );

            Ok(Default::default())
        }

        /// The domain must match the recommended IETF conventions:
        /// https://datatracker.ietf.org/doc/html/rfc1035#section-2.3.1
        ///
        /// The domains must must start with a letter, end with a letter or digit,
        /// and have as interior characters only letters, digits, and hyphen.
        /// There are also some restrictions on the length:
        /// Domains length must be between 3 and 63 characters.
        pub fn ensure_valid_domain(domain: &[u8]) -> DispatchResult {
            Self::ensure_domain_has_valid_length(
                domain,
                MIN_DOMAIN_LENGTH,
                MAX_DOMAIN_LENGTH,
                Error::<T>::TopLevelDomainIsOffLengthLimits,
            )?;

            Self::ensure_domain_contains_valid_chars(
                domain, Error::<T>::TopLevelDomainContainsInvalidChar
            )?;

            Ok(Default::default())
        }

        /// Top level domain must match the IETF convention:
        /// https://tools.ietf.org/id/draft-liman-tld-names-00.html#rfc.section.2
        pub fn ensure_valid_tld(domain: &[u8]) -> DispatchResult {
            // The TLD label MUST be at least 2 characters long and MAY be as long as 63 characters
            // - not counting any leading or trailing periods (.).
            Self::ensure_domain_has_valid_length(
                domain,
                MIN_TLD_LENGTH,
                MAX_DOMAIN_LENGTH,
                Error::<T>::TopLevelDomainIsOffLengthLimits,
            )?;

            // The TLD consist of only ASCII characters from the groups "letters" (A-Z),
            // "digits" (0-9) and "hyphen" (-).
            // It MUST start with an ASCII "letter", and it MUST NOT end with a "hyphen".
            // Upper and lower case MAY be mixed at random, since DNS lookups are case-insensitive.
            Self::ensure_domain_contains_valid_chars(
                domain, Error::<T>::TopLevelDomainContainsInvalidChar
            )?;

            Ok(Default::default())
        }

        /// Fails if the top level domain is not listed as allowed.
        pub fn ensure_tld_allowed(domain: &[u8]) -> DispatchResult {
            let domain_lc = domain.to_ascii_lowercase();
            ensure!(Self::top_level_domain_allowed(&domain_lc), Error::<T>::TopLevelDomainNotAllowed);

            Ok(Default::default())
        }

        /// Fails if `TrustedAccount` is not set.
        pub fn require_trusted_account() -> Result<T::AccountId, DispatchError> {
            Ok(Self::trusted_account().ok_or(Error::<T>::TrustedAccountNotSet)?)
        }

        pub fn ensure_valid_inner_value(
            inner_value: &Option<EntityId<T::AccountId>>
        ) -> DispatchResult {
            if inner_value.is_none() { return Ok(()) }

            return match inner_value.clone().unwrap() {
                EntityId::Space(space_id) => T::SpacesProvider::ensure_space_exists(space_id),
                // TODO: support all inner values
                _ => Err(Error::<T>::InnerValueNotSupported.into()),
            }
        }

        pub fn ensure_valid_outer_value(outer_value: &Option<Vec<u8>>) -> DispatchResult {
            if let Some(outer) = &outer_value {
                ensure!(
                    outer.len() < T::OuterValueLimit::get().into(),
                    Error::<T>::OuterValueOffLengthLimit
                );
            }
            Ok(())
        }
    }
}
