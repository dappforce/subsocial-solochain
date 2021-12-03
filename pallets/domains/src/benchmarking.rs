//! Benchmarking for pallet-domains

use super::*;

#[allow(unused)]
use crate::{Pallet as Pallet, BalanceOf};
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_support::traits::Get;
use frame_system::RawOrigin;

use sp_runtime::traits::Bounded;
use sp_std::vec;

use pallet_utils::mock_functions::valid_content_ipfs;

benchmarks! {
	purchase_domain {
		let owner: T::AccountId = whitelisted_caller();

		let max_length_domain = vec![b'A'; MAX_DOMAIN_LENGTH];
		let domain = Domain {
			tld: max_length_domain.clone(),
			nested: max_length_domain,
		};

		Pallet::<T>::add_top_level_domains(
			RawOrigin::Root.into(),
			vec![domain.tld.clone()],
		)?;

		let inner_value = Some(EntityId::Account(owner.clone()));
		let outer_value = Some(vec![b'a'; T::OuterValueLimit::get().into()]);

		let expires_in = T::ReservationPeriodLimit::get();
		let sold_for = BalanceOf::<T>::max_value();

	}: _(RawOrigin::Root, owner, domain.clone(), valid_content_ipfs(), inner_value, outer_value, expires_in, sold_for)
	verify {
		let Domain { tld, nested } = domain;
		assert!(PurchasedDomains::<T>::get(&tld, &nested).is_some());
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
