//! Benchmarking for pallet-domains

use super::*;

#[allow(unused)]
use crate::{Pallet as Pallet, BalanceOf};
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_support::{
	ensure,
	dispatch::{DispatchErrorWithPostInfo, DispatchResultWithPostInfo},
	traits::{Currency, Get},
};
use frame_system::RawOrigin;

use sp_runtime::traits::{Bounded, StaticLookup};
use sp_std::{vec, vec::Vec};
use pallet_utils::Content;

use pallet_utils::mock_functions::valid_content_ipfs;

fn account_with_balance<T: Config>() -> T::AccountId {
	let owner: T::AccountId = whitelisted_caller();
	<T as Config>::Currency::make_free_balance_be(&owner, BalanceOf::<T>::max_value());

	owner
}

fn lookup_source_from_account<T: Config>(
	account: &T::AccountId,
) -> <T::Lookup as StaticLookup>::Source {
	T::Lookup::unlookup(account.clone())
}

fn create_domain_names<T: Config>(num: usize) -> DomainsVec {
	let mut domains: DomainsVec = Vec::new();

	let max_domain_length = T::MaxDomainLength::get() as usize;
	let mut domain = vec![b'A'; max_domain_length];

	for i in 0..num {
		let idx = i % max_domain_length;

		let next_char = (domain[idx] + 1).clamp(65, 90);
		let _ = sp_std::mem::replace(&mut domain[idx], next_char);
		domains.push(domain.clone());
	}

	assert_eq!(num, domains.len());

	domains
}

fn mock_domain<T: Config>() -> Domain {
	let max_length_domain = vec![b'A'; T::MaxDomainLength::get().into()];

	Domain {
		tld: max_length_domain.clone(),
		domain: max_length_domain,
	}
}

fn add_tld<T: Config>(tld: Vec<u8>) -> DispatchResultWithPostInfo {
	Pallet::<T>::add_tlds(
		RawOrigin::Root.into(),
		vec![tld],
	)
}

fn add_domain<T: Config>(owner: <T::Lookup as StaticLookup>::Source) -> Result<Domain, DispatchErrorWithPostInfo> {
	let domain = mock_domain::<T>();

	add_tld::<T>(domain.tld.clone())?;

	let expires_in = T::ReservationPeriodLimit::get();
	let sold_for = BalanceOf::<T>::max_value();

	Pallet::<T>::register_domain(
		RawOrigin::Root.into(), owner, domain.clone(), valid_content_ipfs(), expires_in, sold_for,
	)?;

	Ok(domain)
}

// TODO: replace with mock function when merged with other benchmarks.
fn valid_content_ipfs_2() -> Content {
	let mut new_content = Vec::new();
	if let Content::IPFS(mut content) = valid_content_ipfs() {
		content.swap_remove(0);
		content.push(b'a');
		new_content = content;
	}

	Content::IPFS(new_content)
}

benchmarks! {
	register_domain {
		let account_with_balance = account_with_balance::<T>();
		let owner = lookup_source_from_account::<T>(&account_with_balance);

		let full_domain = mock_domain::<T>();
		add_tld::<T>(full_domain.tld.clone())?;

		let expires_in = T::ReservationPeriodLimit::get();
		let price = BalanceOf::<T>::max_value();

	}: _(RawOrigin::Root, owner, full_domain.clone(), valid_content_ipfs(), expires_in, price)
	verify {
		let Domain { tld, domain } = Pallet::<T>::lower_domain(&full_domain);
		ensure!(RegisteredDomains::<T>::get(&tld, &domain).is_some(), "Domain was not purchased");
	}

	set_inner_value {
		let owner = account_with_balance::<T>();
		let full_domain = add_domain::<T>(lookup_source_from_account::<T>(&owner))?;

		let value = Some(DomainInnerLink::Account(owner.clone()));
	}: _(RawOrigin::Signed(owner), full_domain.clone(), value.clone())
	verify {
		let Domain { tld, domain } = Pallet::<T>::lower_domain(&full_domain);
		let DomainMeta { inner_value, .. } = RegisteredDomains::<T>::get(&tld, &domain).unwrap();
		ensure!(value == inner_value, "Inner value was not updated.")
	}

	set_outer_value {
		let owner = account_with_balance::<T>();
		let full_domain = add_domain::<T>(lookup_source_from_account::<T>(&owner))?;

		let value = Some(vec![b'A'; T::OuterValueLimit::get() as usize]);
	}: _(RawOrigin::Signed(owner), full_domain.clone(), value.clone())
	verify {
		let Domain { tld, domain } = Pallet::<T>::lower_domain(&full_domain);
		let DomainMeta { outer_value, .. } = RegisteredDomains::<T>::get(&tld, &domain).unwrap();
		ensure!(value == outer_value, "Outer value was not updated.")
	}

	set_domain_content {
		let owner = account_with_balance::<T>();

		let full_domain = add_domain::<T>(lookup_source_from_account::<T>(&owner))?;

		let new_content = valid_content_ipfs_2();
	}: _(RawOrigin::Signed(owner), full_domain.clone(), new_content.clone())
	verify {
		let Domain { tld, domain } = Pallet::<T>::lower_domain(&full_domain);
		let DomainMeta { content, .. } = RegisteredDomains::<T>::get(&tld, &domain).unwrap();
		ensure!(new_content == content, "Content was not updated.")
	}

	reserve_domains {
		let s in 1 .. T::DomainsInsertLimit::get() => ();
		let domains = create_domain_names::<T>(s as usize);
	}: _(RawOrigin::Root, domains)
	verify {
		ensure!(ReservedDomains::<T>::iter().count() as u32 == s, "Domains were not reserved.");
	}

	add_tlds {
		let s in 1 .. T::DomainsInsertLimit::get() => ();
		let domains = create_domain_names::<T>(s as usize);
	}: _(RawOrigin::Root, domains)
	verify {
		ensure!(SupportedTlds::<T>::iter().count() as u32 == s, "TLDs were not added.");
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
