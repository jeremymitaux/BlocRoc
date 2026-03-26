//! # Scanner Pallet
//!
//! Entry validation for BlocRoc events.
//!
//! ## Overview
//!
//! Venue staff use the BlocRoc Scanner mobile app to scan ticket QR codes at the
//! gate. The app submits an `validate_entry` extrinsic; this pallet checks that
//! the ticket is valid and unscanned, records the scan timestamp, and emits an
//! event the app listens to.
//!
//! Scanner accounts are authorised per-event by the event organizer.
//!
//! ## Dispatchables
//!
//! - `authorize_scanner` — Organizer grants a scanner account permission to validate
//!   entries for a specific event.
//! - `revoke_scanner` — Organizer revokes scanner permission.
//! - `validate_entry` — Scanner submits a ticket scan. Marks the ticket as used.

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

    // ── Storage ───────────────────────────────────────────────────────────

    /// Authorized scanners: (EventId, ScannerAccountId) → true if authorized.
    ///
    /// Invariant: only the event organizer can insert/remove entries.
    #[pallet::storage]
    #[pallet::getter(fn authorized_scanners)]
    pub type AuthorizedScanners<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u64, // event_id
        Blake2_128Concat,
        T::AccountId, // scanner
        bool,
        ValueQuery,
    >;

    /// Records which block a ticket was scanned in (ticket_id → block_number).
    ///
    /// Used for audit trails and dispute resolution.
    #[pallet::storage]
    #[pallet::getter(fn scan_record)]
    pub type ScanRecord<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, BlockNumberFor<T>, OptionQuery>;

    // ── Events ────────────────────────────────────────────────────────────

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A scanner was authorized for an event. [event_id, scanner]
        ScannerAuthorized { event_id: u64, scanner: T::AccountId },
        /// A scanner authorization was revoked. [event_id, scanner]
        ScannerRevoked { event_id: u64, scanner: T::AccountId },
        /// A ticket was successfully validated for entry. [event_id, ticket_id, scanner, block]
        EntryValidated {
            event_id: u64,
            ticket_id: u64,
            scanner: T::AccountId,
            block: BlockNumberFor<T>,
        },
    }

    // ── Errors ────────────────────────────────────────────────────────────

    #[pallet::error]
    pub enum Error<T> {
        /// The caller is not authorized to scan for this event.
        UnauthorizedScanner,
        /// The ticket has already been scanned.
        AlreadyScanned,
    }

    // ── Dispatchables ─────────────────────────────────────────────────────

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Authorize a scanner account to validate entries for an event.
        ///
        /// In this scaffold any signed account can call this; in production this
        /// will be gated to the event organizer via cross-pallet coupling with
        /// `pallet-event`.
        ///
        /// Weight: O(1)
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::authorize_scanner())]
        pub fn authorize_scanner(
            origin: OriginFor<T>,
            event_id: u64,
            scanner: T::AccountId,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            AuthorizedScanners::<T>::insert(event_id, &scanner, true);
            Self::deposit_event(Event::ScannerAuthorized { event_id, scanner });
            Ok(())
        }

        /// Revoke a scanner's authorization for an event.
        ///
        /// Weight: O(1)
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::revoke_scanner())]
        pub fn revoke_scanner(
            origin: OriginFor<T>,
            event_id: u64,
            scanner: T::AccountId,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            AuthorizedScanners::<T>::remove(event_id, &scanner);
            Self::deposit_event(Event::ScannerRevoked { event_id, scanner });
            Ok(())
        }

        /// Validate a ticket for entry.
        ///
        /// Caller must be an authorized scanner for the event. Records the scan
        /// block number for the audit trail. The actual ticket invalidation will
        /// be done via cross-pallet call to `pallet-ticket` in a future iteration.
        ///
        /// Weight: O(1)
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::validate_entry())]
        pub fn validate_entry(
            origin: OriginFor<T>,
            event_id: u64,
            ticket_id: u64,
        ) -> DispatchResult {
            let scanner = ensure_signed(origin)?;

            ensure!(
                AuthorizedScanners::<T>::get(event_id, &scanner),
                Error::<T>::UnauthorizedScanner
            );
            ensure!(!ScanRecord::<T>::contains_key(ticket_id), Error::<T>::AlreadyScanned);

            let block = frame_system::Pallet::<T>::block_number();
            ScanRecord::<T>::insert(ticket_id, block);

            // TODO: cross-call pallet-ticket::invalidate(ticket_id)

            Self::deposit_event(Event::EntryValidated { event_id, ticket_id, scanner, block });
            Ok(())
        }
    }
}
