//! Placeholder weights for `pallet_event`. Replace with benchmarked values before mainnet.

use frame_support::weights::Weight;

pub trait WeightInfo {
    fn create_event() -> Weight;
    fn update_metadata() -> Weight;
    fn cancel_event() -> Weight;
    fn increment_sold() -> Weight;
}

pub struct SubstrateWeight<T>(sp_std::marker::PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn create_event() -> Weight {
        Weight::from_parts(10_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn update_metadata() -> Weight {
        Weight::from_parts(10_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    fn cancel_event() -> Weight {
        Weight::from_parts(10_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    fn increment_sold() -> Weight {
        Weight::from_parts(10_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
}

impl WeightInfo for () {
    fn create_event() -> Weight { Weight::zero() }
    fn update_metadata() -> Weight { Weight::zero() }
    fn cancel_event() -> Weight { Weight::zero() }
    fn increment_sold() -> Weight { Weight::zero() }
}
