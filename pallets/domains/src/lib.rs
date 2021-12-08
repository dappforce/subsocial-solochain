//! # Module for storing registered domains.
//!
//! Pallet that allows a trusted bridge account to store the user's registered domains.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

// #[cfg(test)]
// mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;


#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        dispatch::DispatchResult, pallet_prelude::*,
        traits::{Currency, ReservableCurrency},
    };
    use frame_system::Pallet as System;
    use frame_system::pallet_prelude::*;
    use scale_info::TypeInfo;
    use sp_runtime::traits::{Saturating, Zero};
    use sp_std::vec::Vec;

    use df_traits::SpacesProvider;
    use pallet_utils::{Content, PostId, SpaceId, WhoAndWhen};
    use pallet_utils::Pallet as Utils;

    pub use crate::weights::WeightInfo;

    type DomainName = Vec<u8>;
    type DomainsVec = Vec<DomainName>;
    type InnerValue<T> = Option<EntityId<<T as frame_system::Config>::AccountId>>;
    type OuterValue = Option<Vec<u8>>;

    pub(crate) type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
    pub enum EntityId<AccountId> {
        Account(AccountId),
        Space(SpaceId),
        Post(PostId),
    }

    #[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
    pub struct Domain {
        pub tld: DomainName,
        pub domain: DomainName,
    }

    // A domain metadata.
    #[derive(Encode, Decode, TypeInfo)]
    #[scale_info(skip_type_params(T))]
    pub struct DomainMeta<T: Config> {
        // When the domain was created.
        created: WhoAndWhen<T>,
        // When the domain was updated.
        updated: Option<WhoAndWhen<T>>,

        // Specific block, when the domain will become unavailable.
        expires_at: T::BlockNumber,

        // The domain owner.
        owner: T::AccountId,

        // Some additional (custom) domain metadata.
        content: Content,

        // The inner domain link (some Subsocial entity).
        inner_value: InnerValue<T>,
        // The outer domain link (any string).
        outer_value: OuterValue,
        // The amount was held for storing outer value.
        outer_value_bond: BalanceOf<T>,
    }

    impl<T: Config> DomainMeta<T> {
        fn new(
            expires_at: T::BlockNumber,
            owner: T::AccountId,
            content: Content,
        ) -> Self {
            Self {
                created: WhoAndWhen::new(owner.clone()),
                updated: None,
                expires_at,
                owner,
                content,
                inner_value: None,
                outer_value: None,
                outer_value_bond: Zero::zero(),
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

        /// Top level domain minimum length.
        type MinTldLength: Get<u8>;

        /// Domains minimum length.
        type MinDomainLength: Get<u8>;

        /// Domains maximum length.
        type MaxDomainLength: Get<u8>;

        /// The maximum amount of time the domain may be held for.
        #[pallet::constant]
        type ReservationPeriodLimit: Get<Self::BlockNumber>;

        /// The length limit for the domains meta outer value.
        #[pallet::constant]
        type OuterValueLimit: Get<u16>;

        /// The amount held on deposit per byte within the domains outer value.
        #[pallet::constant]
        type OuterValueDepositPerByte: Get<BalanceOf<Self>>;

        /// The maximum domains amount can be inserted to a storage at once.
        #[pallet::constant]
        type DomainsInsertLimit: Get<u32>;

        // TODO: add price coefficients for different domains lengths

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn reserved_domain)]
    pub(super) type ReservedDomains<T> = StorageMap<_, Twox64Concat, DomainName, bool, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn top_level_domain_allowed)]
    pub(super) type AllowedTopLevelDomains<T> =
        StorageMap<_, Twox64Concat, DomainName, bool, ValueQuery>;

    // TODO: how to clean this when domain has expired?
    #[pallet::storage]
    #[pallet::getter(fn registered_domain)]
    pub(super) type RegisteredDomains<T: Config> =
        StorageDoubleMap<_,
            Blake2_128Concat,
            DomainName, /* TLD */
            Blake2_128Concat,
            DomainName, /* Domain */
            DomainMeta<T>
        >;

    #[pallet::storage]
    pub(super) type RegisteredDomainsByAccount<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, Vec<Domain>, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        // TODO: add payment amount to the event
        /// The domain name was successfully registered and stored.
        DomainRegistered(T::AccountId, Domain, BalanceOf<T>),
        /// The domain meta was successfully updated.
        DomainUpdated(T::AccountId, Domain),
        /// The domains list was successfully added to a reserved list.
        DomainsReserved(u16),
        /// The list of top level domains was successfully added to an allow list.
        TopLevelDomainsAllowed(u16),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The content stored in domain metadata was not changed.
        DomainContentWasNotChanged,
        /// Cannot insert that many domains to a storage at once.
        DomainsInsertLimitReached,
        /// The domain has expired.
        DomainHasExpired,
        /// Domain was not found by either custom domain name or top level domain.
        DomainNotFound,
        /// This domain cannot be registered yet, because it is reserved.
        DomainReserved,
        /// This domain is already held by another account.
        DomainAlreadyStored,
        /// A new inner value is the same as the old one.
        InnerValueNotChanged,
        /// Lower than Second level domains are not allowed.
        LowerLevelDomainsNotAllowed,
        /// This account is not allowed to update the domain metadata.
        NotADomainOwner,
        /// Outer value exceeds its length limit.
        OuterValueOffLengthLimit,
        /// A new outer value is the same as the old one.
        OuterValueNotChanged,
        /// Reservation period cannot be a zero value.
        ZeroReservationPeriod,
        /// Cannot store a domain for that long period of time.
        TooBigReservationPeriod,
        /// The top level domain may contain only A-Z, 0-9 and hyphen characters.
        TopLevelDomainContainsInvalidChar,
        /// The top level domain length must be between 3 and 63 characters, inclusive.
        TopLevelDomainIsOffLengthLimits,
        /// This top level domain is not allowed.
        TopLevelDomainNotAllowed,
        /// This inner value is not supported yet.
        InnerValueNotSupported,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(<T as Config>::WeightInfo::register_domain())]
        pub fn register_domain(
            origin: OriginFor<T>,
            owner: T::AccountId,
            full_domain: Domain,
            content: Content,
            expires_in: T::BlockNumber,
            #[pallet::compact] price: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            ensure!(!expires_in.is_zero(), Error::<T>::ZeroReservationPeriod);
            ensure!(
                expires_in <= T::ReservationPeriodLimit::get(),
                Error::<T>::TooBigReservationPeriod,
            );

            // Note that while upper and lower case letters are allowed in domain
            // names, no significance is attached to the case. That is, two names with
            // the same spelling but different case are to be treated as if identical.
            let Domain { tld, domain } = &full_domain;
            let full_domain_lc = Self::lower_domain(&full_domain);
            let Domain { tld: tld_lc, domain: domain_lc } = &full_domain_lc;

            ensure!(!Self::reserved_domain(tld_lc), Error::<T>::DomainReserved);

            Utils::<T>::is_valid_content(content.clone())?;

            Self::ensure_tld_allowed(tld)?;
            Self::ensure_valid_domain(domain)?;

            ensure!(
                Self::registered_domain(tld_lc, domain_lc).is_none(),
                Error::<T>::DomainAlreadyStored,
            );

            let expires_at = expires_in.saturating_add(System::<T>::block_number());
            // TODO: calculate the payment amount
            let domain_meta = DomainMeta::new(
                expires_at,
                owner.clone(),
                content,
            );

            RegisteredDomains::<T>::insert(tld_lc, domain_lc, domain_meta);
            RegisteredDomainsByAccount::<T>::mutate(&owner, |domains| domains.push(full_domain_lc));

            Self::deposit_event(Event::DomainRegistered(owner, full_domain, price));
            Ok(Pays::No.into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
        pub fn set_inner_value(
            origin: OriginFor<T>,
            domain: Domain,
            value: InnerValue<T>,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            let domain_lc = Self::lower_domain(&domain);
            let DomainMeta { owner, inner_value, expires_at, .. } =
                Self::require_domain(&domain_lc)?;

            ensure!(expires_at > System::<T>::block_number(), Error::<T>::DomainHasExpired);

            ensure!(sender == owner, Error::<T>::NotADomainOwner);
            ensure!(inner_value != value, Error::<T>::InnerValueNotChanged);

            Self::ensure_valid_inner_value(&value)?;

            Self::try_mutate_domain(&domain_lc, |meta| meta.inner_value = value)?;

            Self::deposit_event(Event::DomainUpdated(sender, domain));
            Ok(())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
        pub fn set_outer_value(
            origin: OriginFor<T>,
            domain: Domain,
            value_opt: OuterValue,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            let domain_lc = Self::lower_domain(&domain);
            let DomainMeta { owner, outer_value, expires_at, outer_value_bond, .. } =
                Self::require_domain(&domain_lc)?;

            ensure!(expires_at > System::<T>::block_number(), Error::<T>::DomainHasExpired);

            ensure!(sender == owner, Error::<T>::NotADomainOwner);
            ensure!(outer_value != value_opt, Error::<T>::OuterValueNotChanged);

            Self::ensure_valid_outer_value(&value_opt)?;

            let mut new_bond = Zero::zero();

            if let Some(value) = &value_opt {
                new_bond = T::OuterValueDepositPerByte::get().saturating_mul(
                    (value.len() as u32).into()
                );

                <T as Config>::Currency::reserve(&sender, new_bond)?;
            } else if !outer_value_bond.is_zero() {
                <T as Config>::Currency::unreserve(&sender, outer_value_bond);
            }

            Self::try_mutate_domain(&domain_lc, |meta| {
                meta.outer_value = value_opt;
                if outer_value_bond != new_bond {
                    meta.outer_value_bond = new_bond;
                }
            })?;

            Self::deposit_event(Event::DomainUpdated(sender, domain));
            Ok(())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
        pub fn set_domain_content(
            origin: OriginFor<T>,
            domain: Domain,
            new_content: Content,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            let domain_lc = Self::lower_domain(&domain);
            let DomainMeta{ owner, content, expires_at, .. } = Self::require_domain(&domain_lc)?;

            ensure!(expires_at > System::<T>::block_number(), Error::<T>::DomainHasExpired);

            ensure!(sender == owner, Error::<T>::NotADomainOwner);
            ensure!(content != new_content, Error::<T>::DomainContentWasNotChanged);

            Utils::<T>::is_valid_content(content.clone())?;

            Self::try_mutate_domain(&domain_lc, |meta| meta.content = content)?;

            Self::deposit_event(Event::DomainUpdated(sender, domain));
            Ok(())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(domains.len() as u64))]
        pub fn reserve(origin: OriginFor<T>, domains: DomainsVec) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            let domains_len = domains.len();
            Self::ensure_domains_insert_limit_not_reached(domains_len)?;

            Self::insert_domains(
                domains,
                Self::ensure_valid_domain,
                |domain| ReservedDomains::<T>::insert(domain, true),
            )?;

            Self::deposit_event(Event::DomainsReserved(domains_len as u16));
            Ok(Pays::No.into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(domains.len() as u64))]
        pub fn add_top_level_domains(
            origin: OriginFor<T>,
            domains: DomainsVec,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            let domains_len = domains.len();
            Self::ensure_domains_insert_limit_not_reached(domains_len)?;

            Self::insert_domains(
                domains,
                Self::ensure_valid_tld,
                |domain| AllowedTopLevelDomains::<T>::insert(domain.to_ascii_lowercase(), true),
            )?;

            Self::deposit_event(Event::TopLevelDomainsAllowed(domains_len as u16));
            Ok(Pays::No.into())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Checks the length of the provided u8 slice.
        fn ensure_domain_has_valid_length(
            string: &[u8],
            min: u8,
            max: u8,
            error: Error<T>,
        ) -> DispatchResult {
            let length = string.len();
            ensure!(length >= min.into() && length <= max.into(), error);

            Ok(())
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

            Ok(())
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
                T::MinDomainLength::get(),
                T::MaxDomainLength::get(),
                Error::<T>::TopLevelDomainIsOffLengthLimits,
            )?;

            ensure!(
                domain.iter().all(|c| *c != b'.'),
                Error::<T>::LowerLevelDomainsNotAllowed,
            );

            Self::ensure_domain_contains_valid_chars(
                domain, Error::<T>::TopLevelDomainContainsInvalidChar
            )?;

            Ok(())
        }

        /// Top level domain must match the IETF convention:
        /// https://tools.ietf.org/id/draft-liman-tld-names-00.html#rfc.section.2
        pub fn ensure_valid_tld(domain: &[u8]) -> DispatchResult {
            // The TLD label MUST be at least 2 characters long and MAY be as long as 63 characters
            // - not counting any leading or trailing periods (.).
            Self::ensure_domain_has_valid_length(
                domain,
                T::MinTldLength::get(),
                T::MaxDomainLength::get(),
                Error::<T>::TopLevelDomainIsOffLengthLimits,
            )?;

            // The TLD consist of only ASCII characters from the groups "letters" (A-Z),
            // "digits" (0-9) and "hyphen" (-).
            // It MUST start with an ASCII "letter", and it MUST NOT end with a "hyphen".
            // Upper and lower case MAY be mixed at random, since DNS lookups are case-insensitive.
            Self::ensure_domain_contains_valid_chars(
                domain, Error::<T>::TopLevelDomainContainsInvalidChar
            )?;

            Ok(())
        }

        /// Fails if the top level domain is not listed as allowed.
        pub fn ensure_tld_allowed(domain: &[u8]) -> DispatchResult {
            let domain_lc = domain.to_ascii_lowercase();
            ensure!(Self::top_level_domain_allowed(&domain_lc), Error::<T>::TopLevelDomainNotAllowed);

            Ok(())
        }

        pub fn ensure_valid_inner_value(inner_value: &InnerValue<T>) -> DispatchResult {
            if inner_value.is_none() { return Ok(()) }

            match inner_value.clone().unwrap() {
                EntityId::Space(space_id) => T::SpacesProvider::ensure_space_exists(space_id),
                EntityId::Account(_) => Ok(()),
                // TODO: support all inner values
                _ => Err(Error::<T>::InnerValueNotSupported.into()),
            }
        }

        pub fn ensure_valid_outer_value(outer_value: &OuterValue) -> DispatchResult {
            if let Some(outer) = &outer_value {
                ensure!(
                    outer.len() <= T::OuterValueLimit::get().into(),
                    Error::<T>::OuterValueOffLengthLimit
                );
            }
            Ok(())
        }

        pub fn insert_domains<F, S>(
            domains: DomainsVec,
            check_fn: F,
            insert_storage_fn: S,
        ) -> DispatchResult
            where
                F: Fn(&[u8]) -> DispatchResult,
                S: FnMut(&DomainName),
        {
            for domain in &domains {
                check_fn(domain)?;
            }

            domains.iter().for_each(insert_storage_fn);
            Ok(())
        }

        /// Try to get domain meta by it's custom and top level domain names.
        pub fn require_domain(domain: &Domain) -> Result<DomainMeta<T>, DispatchError> {
            Ok(Self::registered_domain(&domain.tld, &domain.domain).ok_or(Error::<T>::DomainNotFound)?)
        }

        pub fn lower_domain(domain: &Domain) -> Domain {
            Domain {
                tld: domain.tld.to_ascii_lowercase(),
                domain: domain.domain.to_ascii_lowercase(),
            }
        }

        pub fn try_mutate_domain<F>(full_domain_lc: &Domain, change_fn: F) -> DispatchResult
            where F: FnOnce(&mut DomainMeta<T>)
        {
            let Domain { tld, domain } = full_domain_lc;
            RegisteredDomains::<T>::try_mutate(&tld, &domain, |meta_opt| -> DispatchResult {
                if let Some(meta) = meta_opt {
                    change_fn(meta);
                    Ok(())
                } else {
                    Err(Error::<T>::DomainNotFound.into())
                }
            })
        }

        pub fn ensure_domains_insert_limit_not_reached(
            domains_len: usize,
        ) -> DispatchResultWithPostInfo {
            let domains_insert_limit = T::DomainsInsertLimit::get() as usize;
            ensure!(domains_len <= domains_insert_limit, Error::<T>::DomainsInsertLimitReached);

            Ok(Default::default())
        }
    }
}
