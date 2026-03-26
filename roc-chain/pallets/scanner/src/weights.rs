//! Placeholder weights for `pallet_scanner`. Replace with benchmarked values before mainnet.

use frame_support::weights::Weight;

pub trait WeightInfo {
    fn authorize_scanner() -> Weight;
    fn revoke_scanner() -> Weight;
    fn validate_entry() -> Weight;
}

pub struct SubstrateWeight<T>(sp_std::marker::PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn authorize_scanner() -> Weight {
        Weight::from_parts(10_000, 0)
            .saturating_add(T::DbWeight::get().writes(1))
    }
    fn revoke_scanner() -> Weight {
        Weight::from_parts(10_000, 0)
            .saturating_add(T::DbWeight::get().writes(1))
    }
    fn validate_entry() -> Weight {
        Weight::from_parts(10_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }
}

impl WeightInfo for () {
    fn authorize_scanner() -> Weight { Weight::zero() }
    fn revoke_scanner() -> Weight { Weight::zero() }
    fn validate_entry() -> Weight { Weight::zero() }
}
