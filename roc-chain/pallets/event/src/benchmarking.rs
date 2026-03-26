//! Benchmarks for `pallet_event`. See `pallet_ticket::benchmarking` for CLI usage.

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Pallet as PalletEvent;
use frame_benchmarking::v2::*;
use frame_support::BoundedVec;
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
    use super::*;

    fn test_cid() -> BoundedVec<u8, frame_support::traits::ConstU32<64>> {
        BoundedVec::try_from(b"QmTestCid".to_vec()).expect("within bounds")
    }

    #[benchmark]
    fn create_event() {
        let caller: T::AccountId = whitelisted_caller();
        #[extrinsic_call]
        _(RawOrigin::Signed(caller), 1000u32, test_cid());

        assert!(crate::Events::<T>::contains_key(0u64));
    }

    #[benchmark]
    fn update_metadata() {
        let caller: T::AccountId = whitelisted_caller();
        PalletEvent::<T>::create_event(RawOrigin::Signed(caller.clone()).into(), 100u32, test_cid())
            .expect("setup");
        #[extrinsic_call]
        _(RawOrigin::Signed(caller), 0u64, test_cid());
    }

    #[benchmark]
    fn cancel_event() {
        let caller: T::AccountId = whitelisted_caller();
        PalletEvent::<T>::create_event(RawOrigin::Signed(caller.clone()).into(), 100u32, test_cid())
            .expect("setup");
        #[extrinsic_call]
        _(RawOrigin::Signed(caller), 0u64);
        assert!(crate::Events::<T>::get(0u64).unwrap().cancelled);
    }

    #[benchmark]
    fn increment_sold() {
        let caller: T::AccountId = whitelisted_caller();
        PalletEvent::<T>::create_event(RawOrigin::Signed(caller.clone()).into(), 100u32, test_cid())
            .expect("setup");
        #[extrinsic_call]
        _(RawOrigin::Signed(caller), 0u64);
        assert_eq!(crate::Events::<T>::get(0u64).unwrap().sold, 1);
    }

    impl_benchmark_test_suite!(PalletEvent, crate::mock::new_test_ext(), crate::mock::Test);
}
