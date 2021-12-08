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
	register_domain {
		let owner: T::AccountId = whitelisted_caller();

		let max_length_domain = vec![b'A'; T::MaxDomainLength::get().into()];
		let full_domain = Domain {
			tld: max_length_domain.clone(),
			domain: max_length_domain,
		};

		Pallet::<T>::add_top_level_domains(
			RawOrigin::Root.into(),
			vec![full_domain.tld.clone()],
		)?;

		let expires_in = T::ReservationPeriodLimit::get();
		let sold_for = BalanceOf::<T>::max_value();

	}: _(RawOrigin::Root, owner, full_domain.clone(), valid_content_ipfs(), expires_in, sold_for)
	verify {
		let Domain { tld, domain } = Pallet::<T>::lower_domain(&full_domain);
		assert!(RegisteredDomains::<T>::get(&tld, &domain).is_some());
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
