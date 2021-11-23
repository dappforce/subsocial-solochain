//! Benchmarks for Template Pallet

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use crate::{Module as Pallet};
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;
use sp_std::vec;
use sp_std::{vec::Vec, boxed::Box};

benchmarks! {
    set_email_verified {
        let caller: T::AccountId = whitelisted_caller();
    }: _(RawOrigin::Root, caller.clone())
    verify {
        let is_caller_email_verified = Pallet::<T>::account_trust_levels_contains(&caller,TrustLevels::EMAIL_VERIFIED);
        assert_eq!(is_caller_email_verified, true);
    }

    set_phone_number_verified {
        let caller: T::AccountId = whitelisted_caller();
    }: _(RawOrigin::Root, caller.clone())
    verify {
        let is_caller_phone_number_verified = Pallet::<T>::account_trust_levels_contains(&caller,TrustLevels::PHONE_NUMBER_VERIFIED);
        assert_eq!(is_caller_phone_number_verified, true);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::mock::{Test, ExtBuilder};
    use frame_support::assert_ok;

    #[test]
    fn test_benchmarks() {
        ExtBuilder::build().execute_with(|| {
            assert_ok!(test_benchmark_set_email_verified::<Test>());
        });
    }
}
