//! # Ticket Pallet
//!
//! Core data model for the BlocRoc ticketing platform.
//!
//! ## Overview
//!
//! This pallet owns two first-class on-chain entities:
//!
//! - **`EventDetails`** — an event record created by an organiser. Tracks capacity,
//!   pricing, resale rules, and how many tickets have been sold.
//! - **`TicketDetails`** — an individual ticket tied to an event. Tracks ownership,
//!   tier, optional seat assignment, pricing, and lifecycle status.
//!
//! Six storage items are provided:
//!
//! | Item | Type | Purpose |
//! |---|---|---|
//! | `NextEventId` | `StorageValue<u64>` | Auto-increment counter |
//! | `NextTicketId` | `StorageValue<u64>` | Auto-increment counter |
//! | `Events` | `StorageMap<EventId, EventDetails>` | Event records |
//! | `Tickets` | `StorageMap<TicketId, TicketDetails>` | Ticket records |
//! | `TicketsByOwner` | `StorageDoubleMap<AccountId, TicketId, ()>` | Reverse index |
//! | `TicketsByEvent` | `StorageDoubleMap<EventId, TicketId, ()>` | Reverse index |
//!
//! ## Extrinsics
//!
//! ## Extrinsics
//!
//! Five dispatchable calls:
//!
//! 1. **`create_event`** — organiser creates an event, stores `EventDetails`.
//! 2. **`mint_tickets`** — event creator mints a batch of tickets (up to remaining capacity).
//! 3. **`purchase_ticket`** — buyer pays the creator the ticket price, ownership transfers.
//! 4. **`transfer_ticket`** — secondary market transfer with resale-cap enforcement.
//! 5. **`validate_ticket`** — scanner marks a ticket as used for venue entry.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;

/// Newtype wrapper for event IDs. Using a named type catches accidental
/// confusion between event IDs and ticket IDs at compile time.
pub type EventId = u64;

/// Newtype wrapper for ticket IDs.
pub type TicketId = u64;

#[frame_support::pallet]
pub mod pallet {
    use super::{EventId, TicketId};
    use crate::weights::WeightInfo;
    use frame_support::{
        pallet_prelude::*,
        traits::{Currency, ExistenceRequirement},
    };
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::Zero;
    use sp_runtime::SaturatedConversion;

    /// Shorthand for the balance type derived from the pallet's `Currency`
    /// associated type. Avoids repeating the long path everywhere.
    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    // ── Pallet declaration ────────────────────────────────────────────────────

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    // ── Config ────────────────────────────────────────────────────────────────

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching runtime event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The currency used for ticket prices and transfers.
        type Currency: Currency<Self::AccountId>;

        /// Maximum byte length for string fields (name, venue, tier, seat).
        /// Bounds all `BoundedVec<u8, _>` fields in the structs stored on-chain.
        #[pallet::constant]
        type MaxStringLength: Get<u32>;

        /// Weight information for dispatchables in this pallet.
        type WeightInfo: WeightInfo;
    }

    // ── Domain types ──────────────────────────────────────────────────────────

    /// All the reasons a ticket transfer can be rejected.
    ///
    /// Carried in the `TransferRejected` event so the frontend and indexer can
    /// display a specific message rather than a generic error.
    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum TransferRejectedReason {
        /// The ticket has already been scanned for entry.
        TicketAlreadyUsed,
        /// The requested resale price exceeds the organiser's resale cap.
        ResaleCapExceeded,
        /// The caller does not own the ticket.
        NotTicketOwner,
        /// The event has been deactivated.
        EventNotActive,
    }

    /// On-chain record for an event created by an organiser.
    ///
    /// Invariants:
    /// - `tickets_sold <= capacity` at all times
    /// - Minting is only allowed while `is_active == true`
    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct EventDetails<T: Config> {
        /// Human-readable event name (e.g. "Glastonbury 2025 — Main Stage").
        pub name: BoundedVec<u8, T::MaxStringLength>,
        /// Venue name (e.g. "O2 Arena").
        pub venue_name: BoundedVec<u8, T::MaxStringLength>,
        /// Unix timestamp of the event start time.
        pub date: u64,
        /// Maximum number of tickets that can be minted for this event.
        pub capacity: u32,
        /// Primary sale price per ticket in the chain's native token.
        pub ticket_price: BalanceOf<T>,
        /// Maximum resale price as a percentage of the original price.
        /// E.g. `110` means tickets can be resold for at most 110% of face value.
        /// Must be >= 100 (no below-face-value cap enforced).
        pub resale_cap_percent: u8,
        /// Number of tickets minted so far. Must never exceed `capacity`.
        pub tickets_sold: u32,
        /// The account that created the event and can manage it.
        pub creator: T::AccountId,
        /// Whether this event is accepting ticket mints and transfers.
        /// Set to `false` to cancel or pause the event.
        pub is_active: bool,
    }

    /// On-chain record for an individual ticket.
    ///
    /// Invariants:
    /// - `current_price <= original_price * resale_cap_percent / 100`
    ///   (enforced at transfer time, not stored here)
    /// - `is_used` is monotonically set to `true` — it can never go back
    /// - `is_listed_for_resale` must be `false` if `is_used` is `true`
    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct TicketDetails<T: Config> {
        /// The event this ticket grants access to.
        pub event_id: EventId,
        /// Ticket tier (e.g. "General Admission", "VIP", "Backstage").
        pub tier: BoundedVec<u8, T::MaxStringLength>,
        /// Optional specific seat reference (e.g. "Block A, Row 3, Seat 12").
        /// `None` for general admission.
        pub seat: Option<BoundedVec<u8, T::MaxStringLength>>,
        /// Price paid at primary sale. Used to enforce the resale cap.
        pub original_price: BalanceOf<T>,
        /// Current asking price if listed for resale, otherwise equal to
        /// `original_price`. Updated each time the ticket is transferred.
        pub current_price: BalanceOf<T>,
        /// Current owner of this ticket.
        pub owner: T::AccountId,
        /// `true` after the ticket has been scanned for venue entry.
        /// Cannot be reversed.
        pub is_used: bool,
        /// `true` if the owner has listed this ticket on the secondary market.
        pub is_listed_for_resale: bool,
    }

    // ── Storage ───────────────────────────────────────────────────────────────

    /// Monotonically increasing counter used to assign unique EventIds.
    ///
    /// Starts at 0. Incremented atomically when a new event is created.
    #[pallet::storage]
    #[pallet::getter(fn next_event_id)]
    pub type NextEventId<T: Config> = StorageValue<_, EventId, ValueQuery>;

    /// Monotonically increasing counter used to assign unique TicketIds.
    ///
    /// Starts at 0. Incremented atomically each time a ticket is minted.
    #[pallet::storage]
    #[pallet::getter(fn next_ticket_id)]
    pub type NextTicketId<T: Config> = StorageValue<_, TicketId, ValueQuery>;

    /// Primary event record store.
    ///
    /// Key: EventId (Blake2_128Concat — supports prefix iteration)
    /// Value: EventDetails<T>
    ///
    /// Invariant: an entry is present here for the lifetime of an event.
    /// Events are never deleted — they can only be deactivated (`is_active = false`).
    #[pallet::storage]
    #[pallet::getter(fn events)]
    pub type Events<T: Config> =
        StorageMap<_, Blake2_128Concat, EventId, EventDetails<T>, OptionQuery>;

    /// Primary ticket record store.
    ///
    /// Key: TicketId (Blake2_128Concat)
    /// Value: TicketDetails<T>
    ///
    /// Invariant: a ticket is present here from the moment it is minted until
    /// it is burned (if a burn mechanic is added). `is_used` flags used tickets
    /// but the record is retained for auditability.
    #[pallet::storage]
    #[pallet::getter(fn tickets)]
    pub type Tickets<T: Config> =
        StorageMap<_, Blake2_128Concat, TicketId, TicketDetails<T>, OptionQuery>;

    /// Reverse index: owner → set of owned ticket IDs.
    ///
    /// Used to answer "show me all tickets owned by address X" without a full
    /// scan of the `Tickets` map. The value is `()` — the fact of membership
    /// is all that matters.
    ///
    /// Must be kept in sync with `Tickets::owner` at every ownership change.
    #[pallet::storage]
    #[pallet::getter(fn tickets_by_owner)]
    pub type TicketsByOwner<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId, // first key: owner
        Blake2_128Concat,
        TicketId, // second key: ticket_id
        (),
        OptionQuery,
    >;

    /// Reverse index: event → set of ticket IDs belonging to that event.
    ///
    /// Used to answer "list all tickets for event X" and to enforce capacity
    /// checks efficiently. Must be kept in sync with `Tickets::event_id`.
    #[pallet::storage]
    #[pallet::getter(fn tickets_by_event)]
    pub type TicketsByEvent<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        EventId, // first key: event_id
        Blake2_128Concat,
        TicketId, // second key: ticket_id
        (),
        OptionQuery,
    >;

    // ── Events (Substrate events, not music events) ───────────────────────────

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new event was created on-chain.
        ///
        /// Emitted by: `create_event`
        EventCreated {
            event_id: EventId,
            creator: T::AccountId,
            /// Included so the indexer can store the name without a storage
            /// read on every `EventCreated` event.
            name: BoundedVec<u8, T::MaxStringLength>,
        },

        /// A batch of tickets was minted for an event.
        ///
        /// Ticket IDs are contiguous: `[start_id, start_id + count)`.
        /// Emitted by: `mint_tickets`
        TicketsMinted {
            event_id: EventId,
            count: u32,
            start_id: TicketId,
        },

        /// A ticket was purchased from the primary sale.
        ///
        /// Emitted by: `purchase_ticket`
        TicketPurchased {
            ticket_id: TicketId,
            buyer: T::AccountId,
            price: BalanceOf<T>,
        },

        /// A ticket was transferred on the secondary market.
        ///
        /// `price` is 0 for free (gifted) transfers.
        /// Emitted by: `transfer_ticket`
        TicketTransferred {
            ticket_id: TicketId,
            from: T::AccountId,
            to: T::AccountId,
            price: BalanceOf<T>,
        },

        /// A ticket was scanned and validated for venue entry.
        ///
        /// After this event, `is_used` is permanently `true`.
        /// Emitted by: `validate_ticket`
        TicketValidated {
            ticket_id: TicketId,
            event_id: EventId,
        },

        /// A transfer was attempted but rejected.
        ///
        /// Emitted instead of returning an error so the scanner app and
        /// frontend can distinguish rejection reasons without parsing error
        /// codes. The extrinsic itself still returns `Ok(())` in this case.
        TransferRejected {
            ticket_id: TicketId,
            reason: TransferRejectedReason,
        },
    }

    // ── Errors ────────────────────────────────────────────────────────────────

    #[pallet::error]
    pub enum Error<T> {
        // ── Event errors ──────────────────────────────────────────────────
        /// No event exists with the given EventId.
        EventNotFound,
        /// The event exists but `is_active` is `false` — it has been
        /// cancelled or paused and no new tickets can be minted.
        EventNotActive,
        /// All capacity has been sold. `tickets_sold == capacity`.
        EventSoldOut,

        // ── Ticket errors ─────────────────────────────────────────────────
        /// No ticket exists with the given TicketId.
        TicketNotFound,
        /// The ticket has already been scanned for entry (`is_used == true`).
        /// It cannot be transferred, resold, or validated again.
        TicketAlreadyUsed,
        /// The caller does not match the ticket's `owner` field.
        NotTicketOwner,

        // ── Transfer / resale errors ──────────────────────────────────────
        /// The requested resale price exceeds `original_price * resale_cap_percent / 100`.
        ResaleCapExceeded,
        /// The buyer does not have enough balance to complete the purchase.
        InsufficientFunds,

        // ── Authorization errors ──────────────────────────────────────────
        /// The caller is not the `creator` of this event.
        NotEventCreator,

        // ── Arithmetic errors ─────────────────────────────────────────────
        /// EventId counter overflowed `u64::MAX`.
        EventIdOverflow,
        /// TicketId counter overflowed `u64::MAX`.
        TicketIdOverflow,
        /// `tickets_sold` counter overflowed. Should be unreachable in practice
        /// since `tickets_sold <= capacity <= u32::MAX`.
        TicketsSoldOverflow,
        /// Resale cap arithmetic overflowed. Occurs if `original_price *
        /// resale_cap_percent` overflows the Balance type.
        ResaleCapArithmetic,
    }

    // ── Dispatchables ────────────────────────────────────────────────────────

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a new event.
        ///
        /// Any signed account can be an organiser. Stores an `EventDetails`
        /// record, increments `NextEventId`, and emits `EventCreated`.
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::create_event())]
        pub fn create_event(
            origin: OriginFor<T>,
            name: BoundedVec<u8, T::MaxStringLength>,
            venue_name: BoundedVec<u8, T::MaxStringLength>,
            date: u64,
            capacity: u32,
            ticket_price: BalanceOf<T>,
            resale_cap_percent: u8,
        ) -> DispatchResult {
            let creator = ensure_signed(origin)?;

            let event_id = NextEventId::<T>::get();
            let next_id = event_id.checked_add(1).ok_or(Error::<T>::EventIdOverflow)?;

            let event = EventDetails {
                name: name.clone(),
                venue_name,
                date,
                capacity,
                ticket_price,
                resale_cap_percent,
                tickets_sold: 0,
                creator: creator.clone(),
                is_active: true,
            };

            Events::<T>::insert(event_id, event);
            NextEventId::<T>::put(next_id);

            Self::deposit_event(Event::EventCreated { event_id, creator, name });
            Ok(())
        }

        /// Mint a batch of tickets for an event.
        ///
        /// Only the event creator may call this. `count` tickets are created
        /// with contiguous IDs starting from `NextTicketId`. Tickets are
        /// initially owned by the creator (primary inventory). The event's
        /// `tickets_sold` is **not** incremented here — it is incremented when
        /// a fan calls `purchase_ticket`.
        ///
        /// Constraints:
        /// - Event must exist and be active.
        /// - `tickets_sold + count <= capacity` (cannot mint beyond capacity).
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::mint_tickets(*count))]
        pub fn mint_tickets(
            origin: OriginFor<T>,
            event_id: EventId,
            count: u32,
        ) -> DispatchResult {
            let caller = ensure_signed(origin)?;

            let event = Events::<T>::get(event_id).ok_or(Error::<T>::EventNotFound)?;
            ensure!(event.is_active, Error::<T>::EventNotActive);
            ensure!(event.creator == caller, Error::<T>::NotEventCreator);

            let remaining = event
                .capacity
                .checked_sub(event.tickets_sold)
                .unwrap_or(0);
            ensure!(count <= remaining, Error::<T>::EventSoldOut);

            let start_id = NextTicketId::<T>::get();
            let end_id = start_id
                .checked_add(count as u64)
                .ok_or(Error::<T>::TicketIdOverflow)?;

            for ticket_id in start_id..end_id {
                let ticket = TicketDetails {
                    event_id,
                    tier: BoundedVec::default(),
                    seat: None,
                    original_price: event.ticket_price,
                    current_price: event.ticket_price,
                    owner: caller.clone(),
                    is_used: false,
                    is_listed_for_resale: false,
                };

                Tickets::<T>::insert(ticket_id, ticket);
                TicketsByOwner::<T>::insert(&caller, ticket_id, ());
                TicketsByEvent::<T>::insert(event_id, ticket_id, ());
            }

            NextTicketId::<T>::put(end_id);

            Self::deposit_event(Event::TicketsMinted { event_id, count, start_id });
            Ok(())
        }

        /// Purchase a ticket from the primary sale.
        ///
        /// The buyer pays `ticket_price` to the event creator. Ownership
        /// transfers from the creator to the buyer, and `tickets_sold` on the
        /// event is incremented.
        ///
        /// Constraints:
        /// - Ticket must exist and not already be used.
        /// - The event must be active.
        /// - The current owner must be the event creator (primary sale only;
        ///   use `transfer_ticket` for secondary market).
        /// - The buyer must have enough balance to pay `ticket_price`.
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::purchase_ticket())]
        pub fn purchase_ticket(
            origin: OriginFor<T>,
            ticket_id: TicketId,
        ) -> DispatchResult {
            let buyer = ensure_signed(origin)?;

            Tickets::<T>::try_mutate(ticket_id, |maybe_ticket| -> DispatchResult {
                let ticket = maybe_ticket.as_mut().ok_or(Error::<T>::TicketNotFound)?;
                ensure!(!ticket.is_used, Error::<T>::TicketAlreadyUsed);

                let event = Events::<T>::get(ticket.event_id)
                    .ok_or(Error::<T>::EventNotFound)?;
                ensure!(event.is_active, Error::<T>::EventNotActive);
                ensure!(ticket.owner == event.creator, Error::<T>::NotEventCreator);

                // Transfer ticket_price from buyer to creator.
                T::Currency::transfer(
                    &buyer,
                    &event.creator,
                    event.ticket_price,
                    ExistenceRequirement::KeepAlive,
                )
                .map_err(|_| Error::<T>::InsufficientFunds)?;

                // Update reverse indexes.
                TicketsByOwner::<T>::remove(&ticket.owner, ticket_id);
                TicketsByOwner::<T>::insert(&buyer, ticket_id, ());

                // Update ticket ownership.
                let price = event.ticket_price;
                ticket.owner = buyer.clone();

                // Increment tickets_sold on the event.
                Events::<T>::try_mutate(ticket.event_id, |maybe_event| -> DispatchResult {
                    let evt = maybe_event.as_mut().ok_or(Error::<T>::EventNotFound)?;
                    evt.tickets_sold = evt
                        .tickets_sold
                        .checked_add(1)
                        .ok_or(Error::<T>::TicketsSoldOverflow)?;
                    Ok(())
                })?;

                Self::deposit_event(Event::TicketPurchased {
                    ticket_id,
                    buyer,
                    price,
                });

                Ok(())
            })
        }

        /// Transfer a ticket on the secondary market.
        ///
        /// The caller must be the current ticket owner. If `price > 0`, funds
        /// are transferred from `to` to the caller. The price is checked
        /// against the event's resale cap: it must satisfy
        /// `price <= original_price * resale_cap_percent / 100`.
        ///
        /// A failed transfer (ticket used, resale cap exceeded, event not
        /// active) returns a `DispatchError` so the extrinsic is reverted.
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::transfer_ticket())]
        pub fn transfer_ticket(
            origin: OriginFor<T>,
            ticket_id: TicketId,
            to: T::AccountId,
            price: BalanceOf<T>,
        ) -> DispatchResult {
            let caller = ensure_signed(origin)?;

            Tickets::<T>::try_mutate(ticket_id, |maybe_ticket| -> DispatchResult {
                let ticket = maybe_ticket.as_mut().ok_or(Error::<T>::TicketNotFound)?;
                ensure!(ticket.owner == caller, Error::<T>::NotTicketOwner);
                ensure!(!ticket.is_used, Error::<T>::TicketAlreadyUsed);

                let event = Events::<T>::get(ticket.event_id)
                    .ok_or(Error::<T>::EventNotFound)?;
                ensure!(event.is_active, Error::<T>::EventNotActive);

                // Enforce resale cap: price ≤ original_price × cap% / 100.
                // We do the arithmetic in u128 to avoid needing From<u64> on BalanceOf<T>.
                if !price.is_zero() {
                    let orig_u128: u128 = ticket.original_price.saturated_into();
                    let max_u128 = orig_u128
                        .checked_mul(event.resale_cap_percent as u128)
                        .ok_or(Error::<T>::ResaleCapArithmetic)?
                        / 100u128;
                    let price_u128: u128 = price.saturated_into();
                    ensure!(price_u128 <= max_u128, Error::<T>::ResaleCapExceeded);

                    // Transfer funds from buyer to seller.
                    T::Currency::transfer(
                        &to,
                        &caller,
                        price,
                        ExistenceRequirement::KeepAlive,
                    )
                    .map_err(|_| Error::<T>::InsufficientFunds)?;
                }

                // Update reverse indexes.
                TicketsByOwner::<T>::remove(&caller, ticket_id);
                TicketsByOwner::<T>::insert(&to, ticket_id, ());

                // Update ticket record.
                ticket.owner = to.clone();
                ticket.current_price = price;
                ticket.is_listed_for_resale = false;

                Self::deposit_event(Event::TicketTransferred {
                    ticket_id,
                    from: caller,
                    to,
                    price,
                });

                Ok(())
            })
        }

        /// Validate a ticket for venue entry.
        ///
        /// Any signed account can call this (in production, restrict via an
        /// `EnsureOrigin` or couple with the scanner pallet). Sets `is_used`
        /// to `true`, which is permanent — the ticket can never be transferred
        /// or validated again.
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::validate_ticket())]
        pub fn validate_ticket(
            origin: OriginFor<T>,
            ticket_id: TicketId,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            Tickets::<T>::try_mutate(ticket_id, |maybe_ticket| -> DispatchResult {
                let ticket = maybe_ticket.as_mut().ok_or(Error::<T>::TicketNotFound)?;
                ensure!(!ticket.is_used, Error::<T>::TicketAlreadyUsed);

                let event_id = ticket.event_id;
                ticket.is_used = true;
                ticket.is_listed_for_resale = false;

                Self::deposit_event(Event::TicketValidated { ticket_id, event_id });
                Ok(())
            })
        }
    }
}
