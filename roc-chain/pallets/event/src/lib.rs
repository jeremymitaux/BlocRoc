//! # Event Pallet
//!
//! Event creation and capacity management for BlocRoc.
//!
//! ## Overview
//!
//! Organizers create on-chain event records that define the parameters for
//! ticket minting: capacity, pricing tiers, sale window, and IPFS metadata CID.
//! The pallet tracks sold ticket counts against capacity to prevent overselling.
//!
//! ## Dispatchables
//!
//! - `create_event` — Organizer creates a new event with capacity and metadata.
//! - `update_metadata` — Organizer updates the IPFS metadata CID.
//! - `cancel_event` — Organizer cancels the event.
//! - `increment_sold` — Called internally when a ticket is minted (via coupling or XCM).

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;

#[frame_support::pallet]
pub mod pallet {
    use crate::weights::WeightInfo;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    // ── Pallet declaration ────────────────────────────────────────────────

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    // ── Config ────────────────────────────────────────────────────────────

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type WeightInfo: WeightInfo;
    }

    // ── Types ─────────────────────────────────────────────────────────────

    /// On-chain event record.
    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct EventRecord<T: Config> {
        /// Account that created the event.
        pub organizer: T::AccountId,
        /// Maximum number of tickets that can be sold.
        pub capacity: u32,
        /// Number of tickets sold so far.
        pub sold: u32,
        /// IPFS CID pointing to off-chain event metadata (name, venue, date, etc.).
        pub metadata_cid: BoundedVec<u8, ConstU32<64>>,
        /// Whether this event has been cancelled.
        pub cancelled: bool,
    }

    // ── Storage ───────────────────────────────────────────────────────────

    /// Auto-incrementing event ID counter.
    #[pallet::storage]
    #[pallet::getter(fn next_event_id)]
    pub type NextEventId<T: Config> = StorageValue<_, u64, ValueQuery>;

    /// Maps an EventId to its EventRecord.
    ///
    /// Invariant: only non-cancelled events with sold < capacity can have new tickets minted.
    #[pallet::storage]
    #[pallet::getter(fn events)]
    pub type Events<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, EventRecord<T>, OptionQuery>;

    // ── Events ────────────────────────────────────────────────────────────

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new event was created. [event_id, organizer, capacity]
        EventCreated { event_id: u64, organizer: T::AccountId, capacity: u32 },
        /// Event metadata was updated. [event_id]
        MetadataUpdated { event_id: u64 },
        /// An event was cancelled. [event_id]
        EventCancelled { event_id: u64 },
        /// Sold count was incremented. [event_id, new_sold_count]
        SoldIncremented { event_id: u64, sold: u32 },
    }

    // ── Errors ────────────────────────────────────────────────────────────

    #[pallet::error]
    pub enum Error<T> {
        /// The event does not exist.
        EventNotFound,
        /// The caller is not the event organizer.
        NotOrganizer,
        /// The event has been cancelled.
        EventCancelled,
        /// The event is sold out.
        SoldOut,
        /// Arithmetic overflow on sold count.
        SoldCountOverflow,
        /// Arithmetic overflow on event ID.
        EventIdOverflow,
    }

    // ── Dispatchables ─────────────────────────────────────────────────────

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a new event.
        ///
        /// Weight: O(1)
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::create_event())]
        pub fn create_event(
            origin: OriginFor<T>,
            capacity: u32,
            metadata_cid: BoundedVec<u8, ConstU32<64>>,
        ) -> DispatchResult {
            let organizer = ensure_signed(origin)?;

            let event_id = Self::next_event_id();
            let next = event_id.checked_add(1).ok_or(Error::<T>::EventIdOverflow)?;

            NextEventId::<T>::put(next);
            Events::<T>::insert(
                event_id,
                EventRecord {
                    organizer: organizer.clone(),
                    capacity,
                    sold: 0,
                    metadata_cid,
                    cancelled: false,
                },
            );

            Self::deposit_event(Event::EventCreated { event_id, organizer, capacity });
            Ok(())
        }

        /// Update the IPFS metadata CID for an event.
        ///
        /// Caller must be the event organizer.
        ///
        /// Weight: O(1)
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::update_metadata())]
        pub fn update_metadata(
            origin: OriginFor<T>,
            event_id: u64,
            new_cid: BoundedVec<u8, ConstU32<64>>,
        ) -> DispatchResult {
            let caller = ensure_signed(origin)?;

            Events::<T>::try_mutate(event_id, |maybe_record| -> DispatchResult {
                let record = maybe_record.as_mut().ok_or(Error::<T>::EventNotFound)?;
                ensure!(record.organizer == caller, Error::<T>::NotOrganizer);
                ensure!(!record.cancelled, Error::<T>::EventCancelled);
                record.metadata_cid = new_cid;
                Ok(())
            })?;

            Self::deposit_event(Event::MetadataUpdated { event_id });
            Ok(())
        }

        /// Cancel an event.
        ///
        /// Caller must be the event organizer.
        ///
        /// Weight: O(1)
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::cancel_event())]
        pub fn cancel_event(origin: OriginFor<T>, event_id: u64) -> DispatchResult {
            let caller = ensure_signed(origin)?;

            Events::<T>::try_mutate(event_id, |maybe_record| -> DispatchResult {
                let record = maybe_record.as_mut().ok_or(Error::<T>::EventNotFound)?;
                ensure!(record.organizer == caller, Error::<T>::NotOrganizer);
                ensure!(!record.cancelled, Error::<T>::EventCancelled);
                record.cancelled = true;
                Ok(())
            })?;

            Self::deposit_event(Event::EventCancelled { event_id });
            Ok(())
        }

        /// Increment the sold-ticket count for an event.
        ///
        /// Intended to be called by the ticket pallet or via governance — not directly
        /// by end users. Access control will be tightened in a future iteration.
        ///
        /// Weight: O(1)
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::increment_sold())]
        pub fn increment_sold(origin: OriginFor<T>, event_id: u64) -> DispatchResult {
            ensure_signed(origin)?;

            Events::<T>::try_mutate(event_id, |maybe_record| -> DispatchResult {
                let record = maybe_record.as_mut().ok_or(Error::<T>::EventNotFound)?;
                ensure!(!record.cancelled, Error::<T>::EventCancelled);
                ensure!(record.sold < record.capacity, Error::<T>::SoldOut);
                record.sold = record.sold.checked_add(1).ok_or(Error::<T>::SoldCountOverflow)?;
                Self::deposit_event(Event::SoldIncremented { event_id, sold: record.sold });
                Ok(())
            })
        }
    }
}
