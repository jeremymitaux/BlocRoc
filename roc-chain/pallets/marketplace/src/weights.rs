//! Placeholder weights for `pallet_marketplace`. Replace with benchmarked values before mainnet.

use frame_support::weights::Weight;

pub trait WeightInfo {
    fn list() -> Weight;
    fn delist() -> Weight;
    fn buy() -> Weight;
}

pub struct SubstrateWeight<T>(sp_std::marker::PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn list() -> Weight {
        Weight::from_parts(10_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    fn delist() -> Weight {
        Weight::from_parts(10_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn buy() -> Weight {
        Weight::from_parts(10_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(2))
    }
}

impl WeightInfo for () {
    fn list() -> Weight { Weight::zero() }
    fn delist() -> Weight { Weight::zero() }
    fn buy() -> Weight { Weight::zero() }
}
