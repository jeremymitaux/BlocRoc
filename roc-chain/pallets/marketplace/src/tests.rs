use crate::{mock::*, Error, Listings, TicketListing};
use frame_support::{assert_noop, assert_ok};

// ── list ──────────────────────────────────────────────────────────────────────

#[test]
fn list_creates_listing() {
    new_test_ext().execute_with(|| {
        assert_ok!(Marketplace::list(RuntimeOrigin::signed(1), 42, 1_000));

        let listing = Listings::<Test>::get(0).expect("listing should exist");
        assert_eq!(listing.seller, 1);
        assert_eq!(listing.ticket_id, 42);
        assert_eq!(listing.price, 1_000);
        assert_eq!(TicketListing::<Test>::get(42), Some(0));
    });
}

#[test]
fn list_fails_if_already_listed() {
    new_test_ext().execute_with(|| {
        assert_ok!(Marketplace::list(RuntimeOrigin::signed(1), 42, 1_000));
        assert_noop!(
            Marketplace::list(RuntimeOrigin::signed(1), 42, 2_000),
            Error::<Test>::AlreadyListed
        );
    });
}

// ── delist ────────────────────────────────────────────────────────────────────

#[test]
fn delist_removes_listing() {
    new_test_ext().execute_with(|| {
        assert_ok!(Marketplace::list(RuntimeOrigin::signed(1), 42, 1_000));
        assert_ok!(Marketplace::delist(RuntimeOrigin::signed(1), 0));

        assert!(Listings::<Test>::get(0).is_none());
        assert!(TicketListing::<Test>::get(42).is_none());
    });
}

#[test]
fn delist_fails_if_not_seller() {
    new_test_ext().execute_with(|| {
        assert_ok!(Marketplace::list(RuntimeOrigin::signed(1), 42, 1_000));
        assert_noop!(
            Marketplace::delist(RuntimeOrigin::signed(99), 0),
            Error::<Test>::NotSeller
        );
    });
}

#[test]
fn delist_fails_if_listing_not_found() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Marketplace::delist(RuntimeOrigin::signed(1), 999),
            Error::<Test>::ListingNotFound
        );
    });
}

// ── buy ───────────────────────────────────────────────────────────────────────

#[test]
fn buy_removes_listing() {
    new_test_ext().execute_with(|| {
        assert_ok!(Marketplace::list(RuntimeOrigin::signed(1), 42, 1_000));
        assert_ok!(Marketplace::buy(RuntimeOrigin::signed(2), 0));

        assert!(Listings::<Test>::get(0).is_none());
        assert!(TicketListing::<Test>::get(42).is_none());
    });
}

#[test]
fn buy_fails_if_listing_not_found() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Marketplace::buy(RuntimeOrigin::signed(2), 999),
            Error::<Test>::ListingNotFound
        );
    });
}
