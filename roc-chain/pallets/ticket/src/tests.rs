//! Data-structure and storage tests for `pallet_ticket`.
//!
//! No extrinsic logic exists yet — these tests verify:
//! 1. Structs can be constructed, SCALE-encoded, and decoded correctly.
//! 2. Storage items can be inserted, read, and removed.
//! 3. Reverse index storage (DoubleMap) behaves as expected.
//! 4. Counter storage values start at 0 and can be incremented.
//!
//! Extrinsic tests (create_event, mint_tickets, etc.) will be added when
//! the call logic is implemented.

use crate::{
    mock::*,
    pallet::{
        BalanceOf, EventDetails, Events, NextEventId, NextTicketId, TicketDetails,
        TicketsByEvent, TicketsByOwner, Tickets, TransferRejectedReason,
    },
    EventId, TicketId,
};
use codec::{Decode, Encode};
use frame_support::BoundedVec;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn bounded(s: &[u8]) -> BoundedVec<u8, <Test as crate::Config>::MaxStringLength> {
    BoundedVec::try_from(s.to_vec()).expect("test string within MaxStringLength")
}

fn make_event_details(
    name: &[u8],
    capacity: u32,
    price: u64,
) -> EventDetails<Test> {
    EventDetails {
        name: bounded(name),
        venue_name: bounded(b"Test Venue"),
        date: 1_700_000_000,
        capacity,
        ticket_price: price,
        resale_cap_percent: 110,
        tickets_sold: 0,
        creator: ALICE,
        is_active: true,
    }
}

fn make_ticket_details(
    event_id: EventId,
    owner: u64,
    price: u64,
) -> TicketDetails<Test> {
    TicketDetails {
        event_id,
        tier: bounded(b"General Admission"),
        seat: None,
        original_price: price,
        current_price: price,
        owner,
        is_used: false,
        is_listed_for_resale: false,
    }
}

// ── Counter storage tests ──────────────────────────────────────────────────────

#[test]
fn next_event_id_starts_at_zero() {
    new_test_ext().execute_with(|| {
        assert_eq!(NextEventId::<Test>::get(), 0);
    });
}

#[test]
fn next_ticket_id_starts_at_zero() {
    new_test_ext().execute_with(|| {
        assert_eq!(NextTicketId::<Test>::get(), 0);
    });
}

#[test]
fn counters_can_be_incremented() {
    new_test_ext().execute_with(|| {
        NextEventId::<Test>::put(5u64);
        NextTicketId::<Test>::put(42u64);
        assert_eq!(NextEventId::<Test>::get(), 5);
        assert_eq!(NextTicketId::<Test>::get(), 42);
    });
}

// ── EventDetails storage tests ────────────────────────────────────────────────

#[test]
fn event_can_be_inserted_and_retrieved() {
    new_test_ext().execute_with(|| {
        let event = make_event_details(b"Rock Night", 500, 10_000);
        Events::<Test>::insert(0u64, event.clone());

        let stored = Events::<Test>::get(0u64).expect("event should exist");
        assert_eq!(stored.name, event.name);
        assert_eq!(stored.capacity, 500);
        assert_eq!(stored.ticket_price, 10_000);
        assert_eq!(stored.creator, ALICE);
        assert!(stored.is_active);
        assert_eq!(stored.tickets_sold, 0);
    });
}

#[test]
fn event_returns_none_when_missing() {
    new_test_ext().execute_with(|| {
        assert!(Events::<Test>::get(999u64).is_none());
    });
}

#[test]
fn event_can_be_mutated() {
    new_test_ext().execute_with(|| {
        Events::<Test>::insert(0u64, make_event_details(b"Festival", 100, 5_000));

        Events::<Test>::mutate(0u64, |maybe| {
            if let Some(e) = maybe {
                e.tickets_sold = 10;
                e.is_active = false;
            }
        });

        let stored = Events::<Test>::get(0u64).unwrap();
        assert_eq!(stored.tickets_sold, 10);
        assert!(!stored.is_active);
    });
}

#[test]
fn multiple_events_are_stored_independently() {
    new_test_ext().execute_with(|| {
        Events::<Test>::insert(0u64, make_event_details(b"Event A", 100, 1_000));
        Events::<Test>::insert(1u64, make_event_details(b"Event B", 200, 2_000));

        assert_eq!(Events::<Test>::get(0u64).unwrap().capacity, 100);
        assert_eq!(Events::<Test>::get(1u64).unwrap().capacity, 200);
    });
}

// ── TicketDetails storage tests ───────────────────────────────────────────────

#[test]
fn ticket_can_be_inserted_and_retrieved() {
    new_test_ext().execute_with(|| {
        let ticket = make_ticket_details(0, ALICE, 10_000);
        Tickets::<Test>::insert(0u64, ticket.clone());

        let stored = Tickets::<Test>::get(0u64).expect("ticket should exist");
        assert_eq!(stored.event_id, 0);
        assert_eq!(stored.owner, ALICE);
        assert_eq!(stored.original_price, 10_000);
        assert!(!stored.is_used);
        assert!(!stored.is_listed_for_resale);
    });
}

#[test]
fn ticket_returns_none_when_missing() {
    new_test_ext().execute_with(|| {
        assert!(Tickets::<Test>::get(999u64).is_none());
    });
}

#[test]
fn ticket_with_seat_stores_seat() {
    new_test_ext().execute_with(|| {
        let mut ticket = make_ticket_details(0, BOB, 5_000);
        ticket.seat = Some(bounded(b"Block A Row 3 Seat 12"));
        Tickets::<Test>::insert(0u64, ticket);

        let stored = Tickets::<Test>::get(0u64).unwrap();
        assert_eq!(stored.seat, Some(bounded(b"Block A Row 3 Seat 12")));
    });
}

#[test]
fn ticket_is_used_flag_can_be_set() {
    new_test_ext().execute_with(|| {
        Tickets::<Test>::insert(0u64, make_ticket_details(0, ALICE, 10_000));
        Tickets::<Test>::mutate(0u64, |maybe| {
            if let Some(t) = maybe { t.is_used = true; }
        });
        assert!(Tickets::<Test>::get(0u64).unwrap().is_used);
    });
}

// ── TicketsByOwner (DoubleMap) tests ──────────────────────────────────────────

#[test]
fn tickets_by_owner_insert_and_contains() {
    new_test_ext().execute_with(|| {
        TicketsByOwner::<Test>::insert(ALICE, 0u64, ());
        TicketsByOwner::<Test>::insert(ALICE, 1u64, ());
        TicketsByOwner::<Test>::insert(BOB, 2u64, ());

        assert!(TicketsByOwner::<Test>::contains_key(ALICE, 0u64));
        assert!(TicketsByOwner::<Test>::contains_key(ALICE, 1u64));
        assert!(TicketsByOwner::<Test>::contains_key(BOB, 2u64));
        // BOB does not own ticket 0
        assert!(!TicketsByOwner::<Test>::contains_key(BOB, 0u64));
    });
}

#[test]
fn tickets_by_owner_remove_clears_entry() {
    new_test_ext().execute_with(|| {
        TicketsByOwner::<Test>::insert(ALICE, 0u64, ());
        TicketsByOwner::<Test>::remove(ALICE, 0u64);
        assert!(!TicketsByOwner::<Test>::contains_key(ALICE, 0u64));
    });
}

#[test]
fn tickets_by_owner_prefix_can_be_cleared() {
    new_test_ext().execute_with(|| {
        TicketsByOwner::<Test>::insert(ALICE, 0u64, ());
        TicketsByOwner::<Test>::insert(ALICE, 1u64, ());
        TicketsByOwner::<Test>::insert(ALICE, 2u64, ());

        // Remove all of Alice's tickets at once (prefix removal)
        let _ = TicketsByOwner::<Test>::clear_prefix(ALICE, u32::MAX, None);

        assert!(!TicketsByOwner::<Test>::contains_key(ALICE, 0u64));
        assert!(!TicketsByOwner::<Test>::contains_key(ALICE, 1u64));
        assert!(!TicketsByOwner::<Test>::contains_key(ALICE, 2u64));
    });
}

// ── TicketsByEvent (DoubleMap) tests ──────────────────────────────────────────

#[test]
fn tickets_by_event_insert_and_contains() {
    new_test_ext().execute_with(|| {
        TicketsByEvent::<Test>::insert(0u64, 0u64, ()); // event 0, ticket 0
        TicketsByEvent::<Test>::insert(0u64, 1u64, ()); // event 0, ticket 1
        TicketsByEvent::<Test>::insert(1u64, 2u64, ()); // event 1, ticket 2

        assert!(TicketsByEvent::<Test>::contains_key(0u64, 0u64));
        assert!(TicketsByEvent::<Test>::contains_key(0u64, 1u64));
        assert!(TicketsByEvent::<Test>::contains_key(1u64, 2u64));
        assert!(!TicketsByEvent::<Test>::contains_key(1u64, 0u64));
    });
}

// ── SCALE encoding / decoding round-trip tests ────────────────────────────────

#[test]
fn event_details_encodes_and_decodes() {
    new_test_ext().execute_with(|| {
        let original = make_event_details(b"Encode Test Event", 100, 9_999);
        let encoded = original.encode();
        let decoded =
            EventDetails::<Test>::decode(&mut &encoded[..]).expect("decode should succeed");
        assert_eq!(original, decoded);
    });
}

#[test]
fn ticket_details_encodes_and_decodes() {
    new_test_ext().execute_with(|| {
        let original = make_ticket_details(7, CHARLIE, 888);
        let encoded = original.encode();
        let decoded =
            TicketDetails::<Test>::decode(&mut &encoded[..]).expect("decode should succeed");
        assert_eq!(original, decoded);
    });
}

#[test]
fn transfer_rejected_reason_encodes_and_decodes() {
    let reasons = [
        TransferRejectedReason::TicketAlreadyUsed,
        TransferRejectedReason::ResaleCapExceeded,
        TransferRejectedReason::NotTicketOwner,
        TransferRejectedReason::EventNotActive,
    ];
    for reason in &reasons {
        let encoded = reason.encode();
        let decoded =
            TransferRejectedReason::decode(&mut &encoded[..]).expect("decode should succeed");
        assert_eq!(*reason, decoded);
    }
}

// ── BoundedVec length enforcement ────────────────────────────────────────────

#[test]
fn bounded_vec_rejects_strings_over_max_length() {
    // MaxStringLength is 64 in the test config.
    let too_long = vec![b'x'; 65];
    let result =
        BoundedVec::<u8, <Test as crate::Config>::MaxStringLength>::try_from(too_long);
    assert!(result.is_err(), "Should reject strings longer than MaxStringLength");
}

#[test]
fn bounded_vec_accepts_strings_at_max_length() {
    let exactly_max = vec![b'x'; 64];
    let result =
        BoundedVec::<u8, <Test as crate::Config>::MaxStringLength>::try_from(exactly_max);
    assert!(result.is_ok());
}
