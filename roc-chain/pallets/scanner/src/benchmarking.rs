//! Benchmarks for `pallet_scanner`.

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Pallet as Scanner;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn authorize_scanner() {
        let organizer: T::AccountId = whitelisted_caller();
        let scanner: T::AccountId = account("scanner", 0, 0);
        #[extrinsic_call]
        _(RawOrigin::Signed(organizer), 0u64, scanner.clone());
        assert!(crate::AuthorizedScanners::<T>::get(0u64, scanner));
    }

    #[benchmark]
    fn revoke_scanner() {
        let organizer: T::AccountId = whitelisted_caller();
        let scanner: T::AccountId = account("scanner", 0, 0);
        Scanner::<T>::authorize_scanner(
            RawOrigin::Signed(organizer.clone()).into(),
            0u64,
            scanner.clone(),
        )
        .expect("setup");
        #[extrinsic_call]
        _(RawOrigin::Signed(organizer), 0u64, scanner.clone());
        assert!(!crate::AuthorizedScanners::<T>::get(0u64, scanner));
    }

    #[benchmark]
    fn validate_entry() {
        let organizer: T::AccountId = account("organizer", 0, 0);
        let scanner: T::AccountId = whitelisted_caller();
        Scanner::<T>::authorize_scanner(
            RawOrigin::Signed(organizer).into(),
            0u64,
            scanner.clone(),
        )
        .expect("setup");
        #[extrinsic_call]
        _(RawOrigin::Signed(scanner), 0u64, 0u64);
        assert!(crate::ScanRecord::<T>::contains_key(0u64));
    }

    impl_benchmark_test_suite!(Scanner, crate::mock::new_test_ext(), crate::mock::Test);
}
