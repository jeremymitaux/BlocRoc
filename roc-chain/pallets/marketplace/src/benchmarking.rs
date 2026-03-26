//! Benchmarks for `pallet_marketplace`.

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Pallet as Marketplace;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn list() {
        let seller: T::AccountId = whitelisted_caller();
        #[extrinsic_call]
        _(RawOrigin::Signed(seller.clone()), 0u64, 1_000u128);
        assert!(crate::Listings::<T>::contains_key(0u64));
    }

    #[benchmark]
    fn delist() {
        let seller: T::AccountId = whitelisted_caller();
        Marketplace::<T>::list(RawOrigin::Signed(seller.clone()).into(), 0u64, 1_000u128)
            .expect("setup");
        #[extrinsic_call]
        _(RawOrigin::Signed(seller), 0u64);
        assert!(!crate::Listings::<T>::contains_key(0u64));
    }

    #[benchmark]
    fn buy() {
        let seller: T::AccountId = account("seller", 0, 0);
        let buyer: T::AccountId = whitelisted_caller();
        Marketplace::<T>::list(RawOrigin::Signed(seller).into(), 0u64, 1_000u128)
            .expect("setup");
        #[extrinsic_call]
        _(RawOrigin::Signed(buyer), 0u64);
        assert!(!crate::Listings::<T>::contains_key(0u64));
    }

    impl_benchmark_test_suite!(Marketplace, crate::mock::new_test_ext(), crate::mock::Test);
}
