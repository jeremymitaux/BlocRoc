use crate::{mock::*, AuthorizedScanners, Error, ScanRecord};
use frame_support::{assert_noop, assert_ok};

// ── authorize_scanner ─────────────────────────────────────────────────────────

#[test]
fn authorize_scanner_sets_flag() {
    new_test_ext().execute_with(|| {
        assert_ok!(Scanner::authorize_scanner(RuntimeOrigin::signed(1), 42, 10));
        assert!(AuthorizedScanners::<Test>::get(42u64, 10u64));
    });
}

// ── revoke_scanner ────────────────────────────────────────────────────────────

#[test]
fn revoke_scanner_clears_flag() {
    new_test_ext().execute_with(|| {
        assert_ok!(Scanner::authorize_scanner(RuntimeOrigin::signed(1), 42, 10));
        assert_ok!(Scanner::revoke_scanner(RuntimeOrigin::signed(1), 42, 10));
        assert!(!AuthorizedScanners::<Test>::get(42u64, 10u64));
    });
}

// ── validate_entry ────────────────────────────────────────────────────────────

#[test]
fn validate_entry_records_scan() {
    new_test_ext().execute_with(|| {
        assert_ok!(Scanner::authorize_scanner(RuntimeOrigin::signed(1), 42, 10));
        assert_ok!(Scanner::validate_entry(RuntimeOrigin::signed(10), 42, 99));
        assert!(ScanRecord::<Test>::contains_key(99u64));
    });
}

#[test]
fn validate_entry_fails_if_unauthorized() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Scanner::validate_entry(RuntimeOrigin::signed(99), 42, 0),
            Error::<Test>::UnauthorizedScanner
        );
    });
}

#[test]
fn validate_entry_fails_if_already_scanned() {
    new_test_ext().execute_with(|| {
        assert_ok!(Scanner::authorize_scanner(RuntimeOrigin::signed(1), 42, 10));
        assert_ok!(Scanner::validate_entry(RuntimeOrigin::signed(10), 42, 99));
        assert_noop!(
            Scanner::validate_entry(RuntimeOrigin::signed(10), 42, 99),
            Error::<Test>::AlreadyScanned
        );
    });
}
