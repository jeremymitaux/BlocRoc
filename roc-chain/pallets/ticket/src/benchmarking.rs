//! Benchmarks for `pallet_ticket`.
//!
//! Run with:
//! ```bash
//! cargo build --features runtime-benchmarks
//! ./target/release/roc-node benchmark pallet \
//!   --chain dev --pallet pallet_ticket --extrinsic "*" \
//!   --steps 50 --repeat 20 --output pallets/ticket/src/weights.rs
//! ```

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Pallet as Ticket;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn mint() {
        let caller: T::AccountId = whitelisted_caller();
        let owner: T::AccountId = account("owner", 0, 0);

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), 1u64, owner.clone());

        assert_eq!(crate::TicketOwner::<T>::get(0), Some(owner));
    }

    #[benchmark]
    fn transfer() {
        let owner: T::AccountId = whitelisted_caller();
        let recipient: T::AccountId = account("recipient", 0, 0);

        Ticket::<T>::mint(RawOrigin::Signed(owner.clone()).into(), 1u64, owner.clone())
            .expect("mint should succeed in benchmark setup");

        #[extrinsic_call]
        _(RawOrigin::Signed(owner), 0u64, recipient.clone());

        assert_eq!(crate::TicketOwner::<T>::get(0), Some(recipient));
    }

    #[benchmark]
    fn burn() {
        let owner: T::AccountId = whitelisted_caller();

        Ticket::<T>::mint(RawOrigin::Signed(owner.clone()).into(), 1u64, owner.clone())
            .expect("mint should succeed in benchmark setup");

        #[extrinsic_call]
        _(RawOrigin::Signed(owner), 0u64);

        assert_eq!(crate::TicketOwner::<T>::get(0u64), None);
    }

    #[benchmark]
    fn invalidate() {
        let caller: T::AccountId = whitelisted_caller();
        let owner: T::AccountId = account("owner", 0, 0);

        Ticket::<T>::mint(RawOrigin::Signed(caller.clone()).into(), 1u64, owner)
            .expect("mint should succeed in benchmark setup");

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), 0u64);

        assert!(crate::TicketUsed::<T>::get(0u64));
    }

    impl_benchmark_test_suite!(Ticket, crate::mock::new_test_ext(), crate::mock::Test);
}
