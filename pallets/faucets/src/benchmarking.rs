//! Faucets pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use sp_std::{vec};
use frame_system::{RawOrigin};
use frame_benchmarking::{benchmarks, account, whitelisted_caller};
use sp_runtime::traits::Bounded;
use frame_support::traits::Currency;

const SEED: u32 = 0;

fn faucet_with_free_balance<T: Config>() -> T::AccountId {
    let faucet: T::AccountId = account("user", 0, SEED);

    T::Currency::make_free_balance_be(&faucet, BalanceOf::<T>::max_value());

    faucet
}

fn add_faucet_with_balance<T: Config>() -> Result<T::AccountId, DispatchError> {
    let faucet: T::AccountId = faucet_with_free_balance::<T>();

    Module::<T>::add_faucet(
        RawOrigin::Root.into(),
        faucet.clone(),
        100u32.into(),
        BalanceOf::<T>::max_value(),
        BalanceOf::<T>::max_value()
    )?;

    Ok(faucet)
}

benchmarks! {
    add_faucet {
        let faucet: T::AccountId = faucet_with_free_balance::<T>();
    }: _(RawOrigin::Root, faucet.clone(), 100u32.into(), BalanceOf::<T>::max_value(), BalanceOf::<T>::max_value())
    verify {
        let faucet_account: Faucet<T> = FaucetByAccount::<T>::get(faucet).unwrap();

        assert_eq!(faucet_account.enabled, true);
        assert_eq!(faucet_account.period, 100u32.into());
        assert_eq!(faucet_account.period_limit, BalanceOf::<T>::max_value());
        assert_eq!(faucet_account.drip_limit, BalanceOf::<T>::max_value());
        assert_eq!(faucet_account.next_period_at, 0u32.into());
        assert_eq!(faucet_account.dripped_in_current_period, 0u32.into());
    }

    update_faucet {
        let faucet = add_faucet_with_balance::<T>()?;
    }: _(RawOrigin::Root, faucet.clone(), FaucetUpdate {
            enabled: None,
            period: Some(7_200u32.into()),
            period_limit: Some(90u32.into()),
            drip_limit: Some(40u32.into())
        })
    verify {
        let faucet_account: Faucet<T> = FaucetByAccount::<T>::get(faucet).unwrap();

        assert_eq!(faucet_account.period, 7_200u32.into());
        assert_eq!(faucet_account.period_limit, 90u32.into());
        assert_eq!(faucet_account.drip_limit, 40u32.into());
    }

    remove_faucets {
        let faucet = add_faucet_with_balance::<T>()?;
    }: _(RawOrigin::Root, vec![faucet.clone()])
    verify {
        assert!(FaucetByAccount::<T>::get(faucet).is_none())
    }

     drip {
        let faucet = add_faucet_with_balance::<T>()?;
        let faucet_origin = RawOrigin::Signed(faucet.clone());

        let caller: T::AccountId = whitelisted_caller();
        let origin = RawOrigin::Signed(caller.clone());

        let amount = T::Currency::minimum_balance();
    }: _(faucet_origin, caller.clone(), amount.clone())
    verify {
        assert_eq!(T::Currency::free_balance(&caller), amount);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::mock::{Test, ExtBuilder};
    use frame_support::assert_ok;

    #[test]
    fn test_benchmarks() {
        ExtBuilder::build_with_faucet().execute_with(|| {
            assert_ok!(test_benchmark_drip::<Test>());
            assert_ok!(test_benchmark_add_faucet::<Test>());
            assert_ok!(test_benchmark_remove_faucets::<Test>());
            assert_ok!(test_benchmark_update_faucet::<Test>());
        });
    }
}
