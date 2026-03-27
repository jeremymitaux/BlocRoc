//! Weight stubs for `pallet_ticket`.
//!
//! These are placeholder values. Before mainnet, run `frame-benchmarking`:
//!
//! ```bash
//! cargo build --features runtime-benchmarks --release
//! ./target/release/roc-node benchmark pallet \
//!   --chain dev --pallet pallet_ticket --extrinsic "*" \
//!   --steps 50 --repeat 20 \
//!   --output roc-chain/pallets/ticket/src/weights.rs
//! ```

use frame_support::weights::Weight;

/// Weight functions for each dispatchable in `pallet_ticket`.
/// Methods are named after the extrinsics that will be added in the next prompt.
pub trait WeightInfo {
    fn create_event() -> Weight;
    fn mint_tickets(count: u32) -> Weight;
    fn purchase_ticket() -> Weight;
    fn transfer_ticket() -> Weight;
    fn validate_ticket() -> Weight;
}

/// Placeholder weights used in the production runtime.
/// Every value is a conservative over-estimate; replace with benchmarks before mainnet.
pub struct SubstrateWeight<T>(sp_std::marker::PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn create_event() -> Weight {
        // Reads: NextEventId (1)
        // Writes: NextEventId, Events (2)
        Weight::from_parts(20_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    fn mint_tickets(count: u32) -> Weight {
        // Reads: NextTicketId, Events (2)
        // Writes: NextTicketId, Events.tickets_sold, Tickets×count, TicketsByOwner×count, TicketsByEvent×count
        Weight::from_parts(20_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2 + 3 * count as u64))
    }

    fn purchase_ticket() -> Weight {
        // Reads: Tickets, Events (2)
        // Writes: Tickets, TicketsByOwner×2, Events.tickets_sold (4)
        Weight::from_parts(30_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(4))
    }

    fn transfer_ticket() -> Weight {
        // Reads: Tickets, Events (2)
        // Writes: Tickets, TicketsByOwner×2 (3)
        Weight::from_parts(25_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    fn validate_ticket() -> Weight {
        // Reads: Tickets (1)
        // Writes: Tickets (1)
        Weight::from_parts(15_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
}

/// No-op weights used in unit tests so tests don't need a full runtime.
impl WeightInfo for () {
    fn create_event() -> Weight { Weight::zero() }
    fn mint_tickets(_: u32) -> Weight { Weight::zero() }
    fn purchase_ticket() -> Weight { Weight::zero() }
    fn transfer_ticket() -> Weight { Weight::zero() }
    fn validate_ticket() -> Weight { Weight::zero() }
}
