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
    use frame_support::{dispatch::DispatchResult, fail, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::StaticLookup;
    use sp_std::vec::Vec;
    use scale_info::TypeInfo;

    use pallet_utils::{Content, PostId, SpaceId, WhoAndWhen};
    use pallet_utils::Pallet as Utils;

    type DomainsVec = Vec<Vec<u8>>;

    const MIN_TLD_LENGTH: usize = 2;
    const MIN_DOMAIN_LENGTH: usize = 3;
    const MAX_DOMAIN_LENGTH: usize = 63;

    #[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
    pub enum EntityId<AccountId> {
        Space(SpaceId),
        Post(PostId),
        Account(AccountId),
    }

    #[derive(Encode, Decode, TypeInfo)]
    #[scale_info(skip_type_params(T))]
    // TODO: value: { created, updated, expired, owner, content, innerValue [OPTION](енам и показывать пока ток спейсы), outerValue [OPTION] }
    pub struct DomainMeta<T: Config> {
        created: WhoAndWhen<T>,
        updated: Option<WhoAndWhen<T>>,

        expired_at: T::BlockNumber,
        owner: T::AccountId,

        content: Content,
        inner_value: Option<EntityId<T::AccountId>>,
        outer_value: Content,
    }

    impl<T: Config> DomainMeta<T> {
        fn new(
            expired_at: T::BlockNumber,
            owner: T::AccountId,
            content: Content,
            inner_value: Option<EntityId<T::AccountId>>,
            outer_value: Content,
        ) -> Self {
            Self {
                created: WhoAndWhen::new(owner.clone()),
                updated: None,
                expired_at,
                owner,
                content,
                inner_value,
                outer_value,
            }
        }
    }

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_utils::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        #[pallet::constant]
        type ReservationPeriodLimit: Get<Self::BlockNumber>;

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
        TrustedAccountSet(T::AccountId),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// This account is not a trusted bridge account, and not allowed to store domains.
        AccountNotAllowedToStoreDomains,
        /// This domain cannot be purchased yet, because it is reserved.
        DomainReserved,
        /// This domain is already held by another account.
        DomainAlreadyStored,
        /// Both inner and outer value cannot be passed as domain metadata.
        // TODO: think on a shorter name
        DomainShouldHaveEitherInnerOrOuterValue,
        /// Lower than Second level domains are not allowed.
        LowerLevelDomainsNotAllowed,
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
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(3, 1))]
        pub fn store(
            origin: OriginFor<T>,
            tld: Vec<u8>,
            user_domain: Vec<u8>,
            expired_at: T::BlockNumber,
            content: Content,
            inner_value: Option<EntityId<T::AccountId>>,
            outer_value: Content,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let trusted_account = Self::require_trusted_account()?;
            ensure!(trusted_account == who, Error::<T>::AccountNotAllowedToStoreDomains);

            ensure!(
                expired_at > T::ReservationPeriodLimit::get(),
                Error::<T>::TooBigReservationPeriod,
            );

            Utils::<T>::is_valid_content(content.clone())?;

            Self::ensure_tld_allowed(&tld)?;
            Self::ensure_valid_domain(&user_domain)?;

            // TODO: refactor or move out to a separate function
            if outer_value.is_some() && inner_value.is_none() {
                Utils::<T>::is_valid_content(outer_value.clone())?;
            } else if let Some(value) = &inner_value {
                match value {
                    // TODO: implement via loose coupling preferably
                    EntityId::Space(_) => (),
                    EntityId::Post(_) => (),
                    // TODO: do we need some kind of lookup here?
                    EntityId::Account(_) => (),
                }
            } else {
                fail!(Error::<T>::DomainShouldHaveEitherInnerOrOuterValue);
            }

            // Note that while upper and lower case letters are allowed in domain
            // names, no significance is attached to the case. That is, two names with
            // the same spelling but different case are to be treated as if identical.
            let tld_lowered = tld.to_ascii_lowercase();
            let user_domain_lowered = user_domain.to_ascii_lowercase();

            ensure!(
                Self::purchased_domain(&tld_lowered, &user_domain_lowered).is_none(),
                Error::<T>::DomainAlreadyStored,
            );

            let domain_meta = DomainMeta::new(
                expired_at,
                who.clone(),
                content,
                inner_value,
                outer_value,
            );

            PurchasedDomains::<T>::insert(&tld_lowered, &user_domain_lowered, domain_meta);

            // TODO: calculate the payment amount and store it

            Self::deposit_event(Event::DomainStored(who, tld, user_domain));
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
            target: <T::Lookup as StaticLookup>::Source,
        ) -> DispatchResult {
            ensure_root(origin)?;
            let target = T::Lookup::lookup(target)?;

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
    }
}
