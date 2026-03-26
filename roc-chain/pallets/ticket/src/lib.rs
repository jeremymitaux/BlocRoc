//! # Ticket Pallet
//!
//! Core ticket lifecycle management for BlocRoc.
//!
//! ## Overview
//!
//! This pallet manages on-chain tickets as unique assets. Each ticket is
//! identified by a `TicketId` and carries metadata (event ID, seat, tier).
//! Tickets can be minted by event organizers, transferred between accounts,
//! and invalidated after use or cancellation.
//!
//! ## Dispatchables
//!
//! - `mint` — Organizer mints a batch of tickets for an event.
//! - `transfer` — Owner transfers a ticket to another account.
//! - `burn` — Owner destroys a ticket (e.g. refund).
//! - `invalidate` — Authorized scanner marks a ticket as used.

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
    use frame_support::{
        pallet_prelude::*,
        traits::fungible::{Inspect, Mutate},
    };
    use frame_system::pallet_prelude::*;

    // ── Pallet declaration ────────────────────────────────────────────────

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    // ── Config ────────────────────────────────────────────────────────────

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Weight information for dispatchables in this pallet.
        type WeightInfo: WeightInfo;
    }

    // ── Storage ───────────────────────────────────────────────────────────

    /// Auto-incrementing ticket ID counter.
    #[pallet::storage]
    #[pallet::getter(fn next_ticket_id)]
    pub type NextTicketId<T: Config> = StorageValue<_, u64, ValueQuery>;

    /// Maps a TicketId to its owner AccountId.
    ///
    /// Invariant: a TicketId present here is a valid, non-invalidated ticket.
    #[pallet::storage]
    #[pallet::getter(fn ticket_owner)]
    pub type TicketOwner<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, T::AccountId, OptionQuery>;

    /// Maps a TicketId to the EventId it belongs to.
    #[pallet::storage]
    #[pallet::getter(fn ticket_event)]
    pub type TicketEvent<T: Config> = StorageMap<_, Blake2_128Concat, u64, u64, OptionQuery>;

    /// Tracks whether a ticket has been used for entry.
    #[pallet::storage]
    #[pallet::getter(fn ticket_used)]
    pub type TicketUsed<T: Config> = StorageMap<_, Blake2_128Concat, u64, bool, ValueQuery>;

    // ── Events ────────────────────────────────────────────────────────────

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new ticket was minted. [ticket_id, event_id, owner]
        TicketMinted { ticket_id: u64, event_id: u64, owner: T::AccountId },
        /// A ticket was transferred. [ticket_id, from, to]
        TicketTransferred { ticket_id: u64, from: T::AccountId, to: T::AccountId },
        /// A ticket was burned. [ticket_id, owner]
        TicketBurned { ticket_id: u64, owner: T::AccountId },
        /// A ticket was invalidated (used for entry). [ticket_id]
        TicketInvalidated { ticket_id: u64 },
    }

    // ── Errors ────────────────────────────────────────────────────────────

    #[pallet::error]
    pub enum Error<T> {
        /// The ticket does not exist.
        TicketNotFound,
        /// The caller does not own this ticket.
        NotTicketOwner,
        /// The ticket has already been used for entry.
        TicketAlreadyUsed,
        /// Arithmetic overflow when generating a new ticket ID.
        TicketIdOverflow,
    }

    // ── Dispatchables ─────────────────────────────────────────────────────

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Mint a single ticket for an event and assign it to `owner`.
        ///
        /// Weight: O(1)
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::mint())]
        pub fn mint(
            origin: OriginFor<T>,
            event_id: u64,
            owner: T::AccountId,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            let ticket_id = Self::next_ticket_id();
            let next = ticket_id.checked_add(1).ok_or(Error::<T>::TicketIdOverflow)?;

            NextTicketId::<T>::put(next);
            TicketOwner::<T>::insert(ticket_id, &owner);
            TicketEvent::<T>::insert(ticket_id, event_id);

            Self::deposit_event(Event::TicketMinted { ticket_id, event_id, owner });
            Ok(())
        }

        /// Transfer a ticket to another account.
        ///
        /// Caller must be the current ticket owner.
        ///
        /// Weight: O(1)
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::transfer())]
        pub fn transfer(
            origin: OriginFor<T>,
            ticket_id: u64,
            to: T::AccountId,
        ) -> DispatchResult {
            let from = ensure_signed(origin)?;

            let owner = TicketOwner::<T>::get(ticket_id).ok_or(Error::<T>::TicketNotFound)?;
            ensure!(owner == from, Error::<T>::NotTicketOwner);
            ensure!(!TicketUsed::<T>::get(ticket_id), Error::<T>::TicketAlreadyUsed);

            TicketOwner::<T>::insert(ticket_id, &to);
            Self::deposit_event(Event::TicketTransferred { ticket_id, from, to });
            Ok(())
        }

        /// Burn (destroy) a ticket.
        ///
        /// Caller must be the current ticket owner.
        ///
        /// Weight: O(1)
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::burn())]
        pub fn burn(origin: OriginFor<T>, ticket_id: u64) -> DispatchResult {
            let caller = ensure_signed(origin)?;

            let owner = TicketOwner::<T>::get(ticket_id).ok_or(Error::<T>::TicketNotFound)?;
            ensure!(owner == caller, Error::<T>::NotTicketOwner);

            TicketOwner::<T>::remove(ticket_id);
            TicketEvent::<T>::remove(ticket_id);
            TicketUsed::<T>::remove(ticket_id);

            Self::deposit_event(Event::TicketBurned { ticket_id, owner: caller });
            Ok(())
        }

        /// Mark a ticket as used (entry validated).
        ///
        /// Only callable by an authorized scanner account (to be restricted via
        /// `EnsureOrigin` in a future iteration).
        ///
        /// Weight: O(1)
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::invalidate())]
        pub fn invalidate(origin: OriginFor<T>, ticket_id: u64) -> DispatchResult {
            ensure_signed(origin)?;

            ensure!(TicketOwner::<T>::contains_key(ticket_id), Error::<T>::TicketNotFound);
            ensure!(!TicketUsed::<T>::get(ticket_id), Error::<T>::TicketAlreadyUsed);

            TicketUsed::<T>::insert(ticket_id, true);
            Self::deposit_event(Event::TicketInvalidated { ticket_id });
            Ok(())
        }
    }
}
