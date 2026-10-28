use std::{
    fmt::Write,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::game::sessions::ConnectionId;

use super::super::types::BindingKey;
use super::{BindingFence, ResumeCredential, ResumeCredentialError};

fn binding(connection_id: u64, generation: u64) -> BindingKey {
    BindingKey {
        connection_id: ConnectionId(connection_id),
        generation,
    }
}

fn fixed(byte: u8) -> impl Fn(&mut [u8]) -> Result<(), ()> {
    move |output| {
        output.fill(byte);
        Ok(())
    }
}

#[test]
fn entropy_failure_is_atomic_and_distinct() {
    let existing = ResumeCredential::mint(&fixed(1), None).unwrap();
    let before = existing.token();
    let calls = AtomicUsize::new(0);
    let failed = ResumeCredential::mint(
        &|_| {
            calls.fetch_add(1, Ordering::Relaxed);
            Err(())
        },
        Some(&existing),
    );
    assert_eq!(failed, Err(ResumeCredentialError::EntropyUnavailable));
    assert_eq!(calls.load(Ordering::Relaxed), 1);
    assert!(existing.matches(&before));
}

#[test]
fn collision_attempts_are_bounded_and_success_can_follow_collisions() {
    let other = ResumeCredential::mint(&fixed(7), None).unwrap();
    let calls = AtomicUsize::new(0);
    let exhausted = ResumeCredential::mint(
        &|output| {
            calls.fetch_add(1, Ordering::Relaxed);
            output.fill(7);
            Ok(())
        },
        Some(&other),
    );
    assert_eq!(exhausted, Err(ResumeCredentialError::CollisionExhausted));
    assert_eq!(calls.load(Ordering::Relaxed), 3);

    let calls = AtomicUsize::new(0);
    let minted = ResumeCredential::mint(
        &|output| {
            let call = calls.fetch_add(1, Ordering::Relaxed) + 1;
            output.fill(if call < 3 { 7 } else { 8 });
            Ok(())
        },
        Some(&other),
    )
    .unwrap();
    assert_eq!(calls.load(Ordering::Relaxed), 3);
    assert_ne!(minted, other);
}

#[test]
fn unique_mint_checks_every_retained_credential_not_only_an_adjacent_seat() {
    let first = ResumeCredential::mint(&fixed(1), None).unwrap();
    let middle = ResumeCredential::mint(&fixed(2), Some(&first)).unwrap();
    let last = ResumeCredential::mint(&fixed(3), Some(&middle)).unwrap();
    let calls = AtomicUsize::new(0);
    let minted = ResumeCredential::mint_unique(
        &|output| {
            let call = calls.fetch_add(1, Ordering::Relaxed);
            output.fill([1, 3, 4][call]);
            Ok(())
        },
        &[&first, &middle, &last],
    )
    .unwrap();
    assert_eq!(calls.load(Ordering::Relaxed), 3);
    assert_ne!(minted, first);
    assert_ne!(minted, middle);
    assert_ne!(minted, last);
}

#[test]
fn capabilities_are_independent_stable_matchable_and_redacted() {
    let first = ResumeCredential::mint(&fixed(3), None).unwrap();
    let second = ResumeCredential::mint(&fixed(4), Some(&first)).unwrap();
    let retained = first.clone();
    let first_token = first.token();
    let second_token = second.token();

    assert_ne!(first, second);
    assert_eq!(first, retained);
    assert!(first.matches(&first_token));
    assert!(!first.matches(&second_token));

    let mut debug = String::new();
    write!(&mut debug, "{first:?} {first_token:?}").unwrap();
    assert!(!debug.contains(first_token.as_str()));
    assert!(!debug.contains(second_token.as_str()));
    assert_eq!(format!("{first:?}"), "ResumeCredential([REDACTED])");
}

#[test]
fn binding_fence_matches_and_replaces_exact_connection_generation_pair() {
    let original = binding(10, 4);
    let replacement = binding(20, 1);
    let mut fence = BindingFence::new(original);

    assert!(fence.matches(original));
    assert!(!fence.matches(binding(10, 3)));
    assert!(!fence.matches(binding(11, 4)));
    assert!(!fence.replace_if_current(binding(10, 3), replacement));
    assert_eq!(fence.current(), original);
    assert!(fence.replace_if_current(original, replacement));
    assert_eq!(fence.current(), replacement);
    assert!(!fence.matches(original));
}
