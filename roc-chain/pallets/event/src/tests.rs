use crate::{mock::*, Error, Events};
use frame_support::{assert_noop, assert_ok, BoundedVec};

fn cid(s: &[u8]) -> BoundedVec<u8, frame_support::traits::ConstU32<64>> {
    BoundedVec::try_from(s.to_vec()).expect("CID within bounds")
}

// ── create_event ──────────────────────────────────────────────────────────────

#[test]
fn create_event_stores_record() {
    new_test_ext().execute_with(|| {
        assert_ok!(Event::create_event(RuntimeOrigin::signed(1), 100, cid(b"QmTest")));

        let record = Events::<Test>::get(0).expect("event should exist");
        assert_eq!(record.organizer, 1);
        assert_eq!(record.capacity, 100);
        assert_eq!(record.sold, 0);
        assert!(!record.cancelled);
    });
}

#[test]
fn create_event_increments_id() {
    new_test_ext().execute_with(|| {
        assert_ok!(Event::create_event(RuntimeOrigin::signed(1), 50, cid(b"Qm1")));
        assert_ok!(Event::create_event(RuntimeOrigin::signed(1), 200, cid(b"Qm2")));
        assert_eq!(crate::NextEventId::<Test>::get(), 2);
    });
}

// ── cancel_event ──────────────────────────────────────────────────────────────

#[test]
fn cancel_event_marks_cancelled() {
    new_test_ext().execute_with(|| {
        assert_ok!(Event::create_event(RuntimeOrigin::signed(1), 100, cid(b"Qm")));
        assert_ok!(Event::cancel_event(RuntimeOrigin::signed(1), 0));

        assert!(Events::<Test>::get(0).unwrap().cancelled);
    });
}

#[test]
fn cancel_event_fails_if_not_organizer() {
    new_test_ext().execute_with(|| {
        assert_ok!(Event::create_event(RuntimeOrigin::signed(1), 100, cid(b"Qm")));
        assert_noop!(
            Event::cancel_event(RuntimeOrigin::signed(99), 0),
            Error::<Test>::NotOrganizer
        );
    });
}

#[test]
fn cancel_event_fails_if_already_cancelled() {
    new_test_ext().execute_with(|| {
        assert_ok!(Event::create_event(RuntimeOrigin::signed(1), 100, cid(b"Qm")));
        assert_ok!(Event::cancel_event(RuntimeOrigin::signed(1), 0));
        assert_noop!(
            Event::cancel_event(RuntimeOrigin::signed(1), 0),
            Error::<Test>::EventCancelled
        );
    });
}

// ── increment_sold ────────────────────────────────────────────────────────────

#[test]
fn increment_sold_increases_count() {
    new_test_ext().execute_with(|| {
        assert_ok!(Event::create_event(RuntimeOrigin::signed(1), 2, cid(b"Qm")));
        assert_ok!(Event::increment_sold(RuntimeOrigin::signed(1), 0));
        assert_eq!(Events::<Test>::get(0).unwrap().sold, 1);
    });
}

#[test]
fn increment_sold_fails_when_sold_out() {
    new_test_ext().execute_with(|| {
        assert_ok!(Event::create_event(RuntimeOrigin::signed(1), 1, cid(b"Qm")));
        assert_ok!(Event::increment_sold(RuntimeOrigin::signed(1), 0));
        assert_noop!(
            Event::increment_sold(RuntimeOrigin::signed(1), 0),
            Error::<Test>::SoldOut
        );
    });
}
