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

use sp_runtime::traits::Bounded;
use sp_std::{vec, vec::Vec};

use pallet_utils::mock_functions::valid_content_ipfs;

fn account_with_balance<T: Config>() -> T::AccountId {
	let owner: T::AccountId = whitelisted_caller();
	<T as Config>::Currency::make_free_balance_be(&owner, BalanceOf::<T>::max_value());

	owner
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

fn add_domain<T: Config>(owner: T::AccountId) -> Result<Domain, DispatchErrorWithPostInfo> {
	let domain = mock_domain::<T>();

	add_tld::<T>(domain.tld.clone())?;

	let expires_in = T::ReservationPeriodLimit::get();
	let sold_for = BalanceOf::<T>::max_value();

	Pallet::<T>::register_domain(
		RawOrigin::Root.into(), owner, domain.clone(), valid_content_ipfs(), expires_in, sold_for,
	)?;

	Ok(domain)
}

benchmarks! {
	register_domain {
		let owner = account_with_balance::<T>();

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
		let full_domain = add_domain::<T>(owner.clone())?;

		let value = Some(DomainInnerLink::Account(owner.clone()));
	}: _(RawOrigin::Signed(owner), full_domain.clone(), value.clone())
	verify {
		let Domain { tld, domain } = Pallet::<T>::lower_domain(&full_domain);
		let DomainMeta { inner_value, .. } = RegisteredDomains::<T>::get(&tld, &domain).unwrap();
		ensure!(value == inner_value, "Inner value was not set.")
	}

	set_outer_value {
		let s in 1 .. T::OuterValueLimit::get().into();

		let owner = account_with_balance::<T>();

		let full_domain = add_domain::<T>(owner.clone())?;

		let value = Some(vec![b'A'; s as usize]);
	}: _(RawOrigin::Signed(owner), full_domain.clone(), value.clone())
	verify {
		let Domain { tld, domain } = Pallet::<T>::lower_domain(&full_domain);
		let DomainMeta { outer_value, .. } = RegisteredDomains::<T>::get(&tld, &domain).unwrap();
		ensure!(value == outer_value, "Outer value was not set.")
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
