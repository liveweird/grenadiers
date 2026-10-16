use std::{
    sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use super::*;

fn deterministic_sessions(now: Instant) -> GameSessions {
    let next = AtomicU8::new(0);
    GameSessions::with_random_fill(now, move |bytes| {
        let value = next.fetch_add(1, Ordering::Relaxed);
        bytes.fill(value);
        Ok(())
    })
}

#[test]
fn issues_unique_canonical_tokens_and_refills_at_eight_per_second() {
    let start = Instant::now();
    let sessions = deterministic_sessions(start);
    let mut tokens = Vec::new();
    for index in 0..40 {
        let response = sessions
            .issue(start + Duration::from_millis(index * 125))
            .unwrap();
        assert_eq!(response.token.as_str().len(), 64);
        assert!(
            response
                .token
                .as_str()
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        );
        tokens.push(response.token);
    }
    tokens.sort();
    tokens.dedup();
    assert_eq!(tokens.len(), 40);
}

#[test]
fn rate_limit_has_burst_of_32_and_refills_fractionally() {
    let start = Instant::now();
    let sessions = deterministic_sessions(start);
    for _ in 0..32 {
        sessions.issue(start).unwrap();
    }
    assert!(matches!(
        sessions.issue(start),
        Err(SessionIssueError::RateLimited)
    ));
    assert!(matches!(
        sessions.issue(start + Duration::from_millis(124)),
        Err(SessionIssueError::RateLimited)
    ));
    sessions.issue(start + Duration::from_millis(125)).unwrap();
}

#[test]
fn capacity_is_128_and_expiry_is_inclusive() {
    let start = Instant::now();
    let sessions = deterministic_sessions(start);
    for index in 0..128 {
        sessions
            .issue(start + Duration::from_millis(index * 125))
            .unwrap();
    }
    assert!(matches!(
        sessions.issue(start + Duration::from_secs(16)),
        Err(SessionIssueError::CapacityReached)
    ));
    sessions
        .issue(start + Duration::from_secs(300))
        .expect("sessions expiring exactly now are pruned");
}

#[test]
fn live_collisions_do_not_overwrite_or_extend_the_existing_session() {
    let start = Instant::now();
    let sessions = GameSessions::with_random_fill(start, |bytes| {
        bytes.fill(7);
        Ok(())
    });
    let original = sessions.issue(start).unwrap();
    assert!(matches!(
        sessions.issue(start + Duration::from_secs(1)),
        Err(SessionIssueError::CreationUnavailable)
    ));
    let replacement = sessions
        .issue(start + SESSION_LIFETIME)
        .expect("the original expiry was preserved");
    assert_eq!(replacement.token.as_str(), original.token.as_str());
}

#[test]
fn concurrent_issuance_never_exceeds_capacity() {
    let start = Instant::now();
    let sessions = Arc::new(deterministic_sessions(start));
    for index in 0..120 {
        sessions
            .issue(start + Duration::from_millis(index * 125))
            .unwrap();
    }

    let issue_at = start + Duration::from_secs(100);
    let handles = (0..32)
        .map(|_| {
            let sessions = Arc::clone(&sessions);
            thread::spawn(move || sessions.issue(issue_at))
        })
        .collect::<Vec<_>>();
    let issued = handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .filter(Result::is_ok)
        .count();
    assert_eq!(issued, 8);
    assert!(matches!(
        sessions.issue(issue_at),
        Err(SessionIssueError::CapacityReached | SessionIssueError::RateLimited)
    ));
}

#[test]
fn random_failure_is_an_unavailable_error() {
    let now = Instant::now();
    let sessions = GameSessions::with_random_fill(now, |_| Err(()));
    assert!(matches!(
        sessions.issue(now),
        Err(SessionIssueError::CreationUnavailable)
    ));
}
