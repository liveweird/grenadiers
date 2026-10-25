use std::{net::Ipv4Addr, time::Duration};

use super::super::{BucketSpec, MAX_TRACKED_PEERS};
use super::*;

fn key(last_octet: u8) -> PeerKey {
    PeerKey::Ipv4(Ipv4Addr::new(192, 0, 2, last_octet))
}

fn gameplay_limits(
    max_keys: usize,
    burst: f64,
    sockets: usize,
    admissions: usize,
) -> GameplayLimits {
    GameplayLimits {
        max_keys,
        buckets: [BucketSpec {
            burst,
            refill_per_second: 1.0,
        }; GameplayOperation::COUNT],
        socket_limit: sockets,
        room_admission_limit: admissions,
    }
}

fn diagnostic_limits(max_keys: usize, burst: f64, sockets: usize) -> DiagnosticLimits {
    DiagnosticLimits {
        max_keys,
        bucket: BucketSpec {
            burst,
            refill_per_second: 1.0,
        },
        socket_limit: sockets,
    }
}

#[test]
fn one_peer_exhaustion_does_not_consume_another_peers_budget() {
    let now = Instant::now();
    let admission = GameplayAdmission::with_limits(now, gameplay_limits(8, 1.0, 8, 4));
    assert!(
        admission
            .check(key(1), GameplayOperation::Mint, now)
            .is_ok()
    );
    assert_eq!(
        admission.check(key(1), GameplayOperation::Mint, now),
        Err(PolicyDenial {
            operation: DenialOperation::Mint,
            reason: DenialReason::PeerRate
        })
    );
    assert!(
        admission
            .check(key(2), GameplayOperation::Mint, now)
            .is_ok()
    );
}

#[test]
fn create_join_resume_and_upgrade_budgets_are_independent() {
    let now = Instant::now();
    let admission = GameplayAdmission::with_limits(now, gameplay_limits(8, 1.0, 8, 4));
    for operation in [
        GameplayOperation::Mint,
        GameplayOperation::Upgrade,
        GameplayOperation::Create,
        GameplayOperation::Join,
        GameplayOperation::Resume,
    ] {
        assert!(
            admission.check(key(1), operation, now).is_ok(),
            "{operation:?}"
        );
        assert_eq!(
            admission
                .check(key(1), operation, now)
                .expect_err("second attempt is denied")
                .reason,
            DenialReason::PeerRate
        );
    }
}

#[test]
fn diagnostic_budgets_and_tracking_are_isolated_from_gameplay() {
    let now = Instant::now();
    let gameplay = GameplayAdmission::with_limits(now, gameplay_limits(1, 1.0, 1, 1));
    let diagnostic = DiagnosticAdmission::with_limits(now, diagnostic_limits(1, 1.0, 1));
    assert!(
        gameplay
            .check(key(1), GameplayOperation::Upgrade, now)
            .is_ok()
    );
    assert!(diagnostic.check_upgrade(key(1), now).is_ok());
    assert_eq!(
        diagnostic
            .check_upgrade(key(1), now)
            .expect_err("diagnostic exhausted")
            .reason,
        DenialReason::PeerRate
    );
    assert!(gameplay.acquire_socket(key(1), now).is_ok());
    assert!(diagnostic.acquire_socket(key(1), now).is_ok());
    assert!(
        gameplay
            .check(key(2), GameplayOperation::Mint, now)
            .is_err()
    );
    assert!(diagnostic.check_upgrade(key(2), now).is_err());
}

#[test]
fn typed_permits_enforce_limits_and_release_on_drop() {
    let now = Instant::now();
    let gameplay = GameplayAdmission::with_limits(now, gameplay_limits(4, 8.0, 1, 1));
    let socket = gameplay.acquire_socket(key(1), now).expect("first socket");
    assert_eq!(gameplay.counts(key(1)), Some((1, 0)));
    assert_eq!(
        gameplay
            .acquire_socket(key(1), now)
            .err()
            .expect("socket limit")
            .reason,
        DenialReason::PeerCapacity
    );
    drop(socket);
    assert_eq!(gameplay.counts(key(1)), Some((0, 0)));
    let replacement = gameplay
        .acquire_socket(key(1), now)
        .expect("released socket capacity");

    let attempt = gameplay
        .acquire_room_admission(key(1), now)
        .expect("first room attempt");
    assert_eq!(gameplay.counts(key(1)), Some((1, 1)));
    assert_eq!(
        gameplay
            .acquire_room_admission(key(1), now)
            .err()
            .expect("admission limit")
            .reason,
        DenialReason::PeerCapacity
    );
    drop(attempt);
    assert!(gameplay.acquire_room_admission(key(1), now).is_ok());
    drop(replacement);
}

#[test]
fn diagnostic_permit_releases_when_its_owner_is_cancelled() {
    let now = Instant::now();
    let diagnostic = DiagnosticAdmission::with_limits(now, diagnostic_limits(4, 4.0, 1));
    let permit = diagnostic
        .acquire_socket(key(1), now)
        .expect("first diagnostic socket");
    assert_eq!(
        diagnostic
            .acquire_socket(key(1), now)
            .err()
            .expect("at limit")
            .reason,
        DenialReason::PeerCapacity
    );
    drop(permit);
    assert!(diagnostic.acquire_socket(key(1), now).is_ok());
}

#[test]
fn full_table_keeps_existing_keys_and_rejects_unseen_recent_keys() {
    let now = Instant::now();
    let admission = GameplayAdmission::with_limits(now, gameplay_limits(2, 2.0, 1, 1));
    assert!(
        admission
            .check(key(1), GameplayOperation::Mint, now)
            .is_ok()
    );
    assert!(
        admission
            .check(key(2), GameplayOperation::Mint, now)
            .is_ok()
    );
    assert_eq!(admission.len(), 2);
    assert_eq!(
        admission
            .check(
                key(3),
                GameplayOperation::Mint,
                now + Duration::from_secs(59)
            )
            .expect_err("full table")
            .reason,
        DenialReason::TrackingCapacity
    );
    assert!(
        admission
            .check(key(1), GameplayOperation::Create, now)
            .is_ok()
    );
    assert_eq!(admission.len(), 2);
}

#[test]
fn idle_entries_are_evicted_but_live_permits_are_not() {
    let now = Instant::now();
    let admission = GameplayAdmission::with_limits(now, gameplay_limits(2, 2.0, 1, 1));
    let live = admission.acquire_socket(key(1), now).expect("live socket");
    assert!(
        admission
            .check(key(2), GameplayOperation::Mint, now)
            .is_ok()
    );
    assert!(
        admission
            .check(
                key(3),
                GameplayOperation::Mint,
                now + Duration::from_secs(60)
            )
            .is_ok()
    );
    assert_eq!(admission.len(), 2);
    assert_eq!(admission.counts(key(1)), Some((1, 0)));
    assert!(admission.counts(key(2)).is_none());
    drop(live);
}

#[test]
fn rate_denial_refreshes_activity_before_an_idle_sweep() {
    let now = Instant::now();
    let admission = GameplayAdmission::with_limits(now, gameplay_limits(2, 1.0, 1, 1));
    assert!(
        admission
            .check(key(1), GameplayOperation::Mint, now)
            .is_ok()
    );
    assert!(
        admission
            .check(key(2), GameplayOperation::Mint, now)
            .is_ok()
    );
    let denied_at = now + Duration::from_secs(59);
    assert!(
        admission
            .check(key(1), GameplayOperation::Mint, denied_at)
            .is_ok()
    );
    assert_eq!(
        admission
            .check(key(1), GameplayOperation::Mint, denied_at)
            .expect_err("same-time retry is denied")
            .reason,
        DenialReason::PeerRate
    );
    assert!(
        admission
            .check(
                key(3),
                GameplayOperation::Mint,
                now + Duration::from_secs(60)
            )
            .is_ok()
    );
    assert!(admission.counts(key(1)).is_some());
    assert!(admission.counts(key(2)).is_none());
}

#[test]
fn permit_release_starts_a_fresh_idle_window() {
    let released_at = Instant::now();
    let initial = released_at
        .checked_sub(Duration::from_secs(120))
        .expect("test instant supports subtraction");
    let admission = GameplayAdmission::with_limits(initial, gameplay_limits(2, 1.0, 1, 1));
    let live = admission
        .acquire_socket(key(1), initial)
        .expect("live socket");
    assert!(
        admission
            .check(key(2), GameplayOperation::Mint, initial)
            .is_ok()
    );
    drop(live);

    assert!(
        admission
            .check(
                key(3),
                GameplayOperation::Mint,
                released_at + Duration::from_secs(59)
            )
            .is_ok()
    );
    assert!(admission.counts(key(1)).is_some());
    assert!(admission.counts(key(2)).is_none());
    assert!(
        admission
            .check(
                key(4),
                GameplayOperation::Mint,
                released_at + Duration::from_secs(61)
            )
            .is_ok()
    );
    assert!(admission.counts(key(1)).is_none());
}

#[test]
fn backwards_timestamps_neither_evict_entries_nor_refill_budgets() {
    let now = Instant::now();
    let later = now + Duration::from_secs(100);
    let admission = GameplayAdmission::with_limits(later, gameplay_limits(1, 1.0, 1, 1));
    assert!(
        admission
            .check(key(1), GameplayOperation::Mint, later)
            .is_ok()
    );
    assert_eq!(
        admission
            .check(key(1), GameplayOperation::Mint, now)
            .expect_err("no backwards refill")
            .reason,
        DenialReason::PeerRate
    );
    assert_eq!(
        admission
            .check(key(2), GameplayOperation::Mint, now)
            .expect_err("no backwards eviction")
            .reason,
        DenialReason::TrackingCapacity
    );
    assert_eq!(admission.len(), 1);
}

#[test]
fn production_defaults_match_the_reviewed_initial_limits() {
    let now = Instant::now();
    let gameplay = GameplayAdmission::new(now);
    for _ in 0..8 {
        assert!(gameplay.check(key(1), GameplayOperation::Mint, now).is_ok());
    }
    assert!(
        gameplay
            .check(key(1), GameplayOperation::Mint, now)
            .is_err()
    );
    let diagnostic = DiagnosticAdmission::new(now);
    for _ in 0..4 {
        assert!(diagnostic.check_upgrade(key(1), now).is_ok());
    }
    assert!(diagnostic.check_upgrade(key(1), now).is_err());
}

#[test]
fn production_tracking_table_has_an_exact_4096_key_cap() {
    let now = Instant::now();
    let gameplay = GameplayAdmission::new(now);
    for index in 0..MAX_TRACKED_PEERS {
        let peer = PeerKey::Ipv4(Ipv4Addr::from(index as u32));
        assert!(gameplay.check(peer, GameplayOperation::Mint, now).is_ok());
    }
    let unseen = PeerKey::Ipv4(Ipv4Addr::from(MAX_TRACKED_PEERS as u32));
    assert_eq!(
        gameplay
            .check(unseen, GameplayOperation::Mint, now)
            .expect_err("tracking capacity is fixed")
            .reason,
        DenialReason::TrackingCapacity
    );
}
