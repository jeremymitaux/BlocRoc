use crate::{mock::*, Error, Event, TicketOwner, TicketUsed};
use frame_support::{assert_noop, assert_ok};

// ── mint ──────────────────────────────────────────────────────────────────────

#[test]
fn mint_assigns_owner_and_increments_id() {
    new_test_ext().execute_with(|| {
        assert_ok!(Ticket::mint(RuntimeOrigin::signed(1), 42, 2));

        assert_eq!(TicketOwner::<Test>::get(0), Some(2));
        assert_eq!(crate::NextTicketId::<Test>::get(), 1);
    });
}

#[test]
fn mint_emits_event() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        assert_ok!(Ticket::mint(RuntimeOrigin::signed(1), 42, 2));

        System::assert_last_event(
            Event::TicketMinted { ticket_id: 0, event_id: 42, owner: 2 }.into(),
        );
    });
}

// ── transfer ──────────────────────────────────────────────────────────────────

#[test]
fn transfer_changes_owner() {
    new_test_ext().execute_with(|| {
        assert_ok!(Ticket::mint(RuntimeOrigin::signed(1), 42, 2));
        assert_ok!(Ticket::transfer(RuntimeOrigin::signed(2), 0, 3));

        assert_eq!(TicketOwner::<Test>::get(0), Some(3));
    });
}

#[test]
fn transfer_fails_if_not_owner() {
    new_test_ext().execute_with(|| {
        assert_ok!(Ticket::mint(RuntimeOrigin::signed(1), 42, 2));
        assert_noop!(
            Ticket::transfer(RuntimeOrigin::signed(99), 0, 3),
            Error::<Test>::NotTicketOwner
        );
    });
}

#[test]
fn transfer_fails_if_ticket_not_found() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Ticket::transfer(RuntimeOrigin::signed(1), 999, 2),
            Error::<Test>::TicketNotFound
        );
    });
}

#[test]
fn transfer_fails_if_already_used() {
    new_test_ext().execute_with(|| {
        assert_ok!(Ticket::mint(RuntimeOrigin::signed(1), 42, 2));
        assert_ok!(Ticket::invalidate(RuntimeOrigin::signed(1), 0));
        assert_noop!(
            Ticket::transfer(RuntimeOrigin::signed(2), 0, 3),
            Error::<Test>::TicketAlreadyUsed
        );
    });
}

// ── burn ─────────────────────────────────────────────────────────────────────

#[test]
fn burn_removes_ticket() {
    new_test_ext().execute_with(|| {
        assert_ok!(Ticket::mint(RuntimeOrigin::signed(1), 42, 2));
        assert_ok!(Ticket::burn(RuntimeOrigin::signed(2), 0));

        assert_eq!(TicketOwner::<Test>::get(0), None);
    });
}

#[test]
fn burn_fails_if_not_owner() {
    new_test_ext().execute_with(|| {
        assert_ok!(Ticket::mint(RuntimeOrigin::signed(1), 42, 2));
        assert_noop!(
            Ticket::burn(RuntimeOrigin::signed(99), 0),
            Error::<Test>::NotTicketOwner
        );
    });
}

// ── invalidate ────────────────────────────────────────────────────────────────

#[test]
fn invalidate_marks_ticket_used() {
    new_test_ext().execute_with(|| {
        assert_ok!(Ticket::mint(RuntimeOrigin::signed(1), 42, 2));
        assert_ok!(Ticket::invalidate(RuntimeOrigin::signed(1), 0));

        assert!(TicketUsed::<Test>::get(0));
    });
}

#[test]
fn invalidate_fails_if_already_used() {
    new_test_ext().execute_with(|| {
        assert_ok!(Ticket::mint(RuntimeOrigin::signed(1), 42, 2));
        assert_ok!(Ticket::invalidate(RuntimeOrigin::signed(1), 0));
        assert_noop!(
            Ticket::invalidate(RuntimeOrigin::signed(1), 0),
            Error::<Test>::TicketAlreadyUsed
        );
    });
}

#[test]
fn invalidate_fails_if_ticket_not_found() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Ticket::invalidate(RuntimeOrigin::signed(1), 999),
            Error::<Test>::TicketNotFound
        );
    });
}
