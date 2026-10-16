use std::{
    sync::{
        Arc, Barrier,
        atomic::{AtomicU8, Ordering},
        mpsc,
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
fn out_of_order_captured_time_credits_each_refill_interval_once() {
    let start = Instant::now();
    let sessions = deterministic_sessions(start);
    for _ in 0..32 {
        sessions.issue(start).unwrap();
    }

    let later = start + Duration::from_millis(125);
    sessions.issue(later).unwrap();
    assert!(matches!(
        sessions.issue(start),
        Err(SessionIssueError::RateLimited)
    ));
    assert!(matches!(
        sessions.issue(later),
        Err(SessionIssueError::RateLimited)
    ));
}

#[test]
fn out_of_order_issuance_uses_effective_time_for_expiry() {
    let start = Instant::now();
    let later = start + Duration::from_secs(1);
    let sessions = deterministic_sessions(start);
    sessions.issue(later).unwrap();
    sessions.issue(start).unwrap();

    let registry = sessions.registry.lock().unwrap();
    assert_eq!(registry.sessions.len(), 2);
    assert!(registry.sessions.values().all(|state| matches!(
        state,
        SessionState::Issued { expires_at }
            if *expires_at == later + SESSION_LIFETIME
    )));
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
    for _ in 0..32 {
        assert!(matches!(
            sessions.issue(start + Duration::from_secs(16)),
            Err(SessionIssueError::CapacityReached)
        ));
    }
    assert!(matches!(
        sessions.issue(start + Duration::from_secs(16)),
        Err(SessionIssueError::RateLimited)
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
fn random_failures_consume_rate_limit_without_mutating_existing_session() {
    let now = Instant::now();
    let calls = Arc::new(AtomicU8::new(0));
    let observed_calls = Arc::clone(&calls);
    let sessions = GameSessions::with_random_fill(now, move |bytes| {
        if observed_calls.fetch_add(1, Ordering::Relaxed) == 0 {
            bytes.fill(9);
            Ok(())
        } else {
            Err(())
        }
    });
    let issued = sessions.issue(now).unwrap();
    for _ in 0..31 {
        assert!(matches!(
            sessions.issue(now),
            Err(SessionIssueError::CreationUnavailable)
        ));
    }
    assert_eq!(calls.load(Ordering::Relaxed), 32);
    assert!(matches!(
        sessions.issue(now),
        Err(SessionIssueError::RateLimited)
    ));
    assert_eq!(calls.load(Ordering::Relaxed), 32);

    let registry = sessions.registry.lock().unwrap();
    assert_eq!(registry.sessions.len(), 1);
    assert!(matches!(
        registry.sessions.get(&issued.token),
        Some(SessionState::Issued { expires_at }) if *expires_at == now + SESSION_LIFETIME
    ));
}

#[test]
fn claim_is_exclusive_and_lease_drop_permanently_consumes_token() {
    let now = Instant::now();
    let sessions = Arc::new(deterministic_sessions(now));
    let issued = sessions.issue(now).unwrap();
    let lease = sessions.claim(&issued.token, now).unwrap();
    assert!(matches!(
        sessions.claim(&issued.token, now),
        Err(SessionClaimError::ConnectionInUse)
    ));
    drop(lease);
    assert!(matches!(
        sessions.claim(&issued.token, now),
        Err(SessionClaimError::AuthenticationFailed)
    ));
}

#[test]
fn concurrent_claim_has_one_live_owner_and_cannot_be_revoked_by_losers() {
    const CLAIMANTS: usize = 16;

    let now = Instant::now();
    let sessions = Arc::new(deterministic_sessions(now));
    let token = sessions.issue(now).unwrap().token;
    let start = Arc::new(Barrier::new(CLAIMANTS + 1));
    let release_owner = Arc::new(Barrier::new(CLAIMANTS + 1));
    let (outcomes, received) = mpsc::channel();

    let handles = (0..CLAIMANTS)
        .map(|_| {
            let sessions = Arc::clone(&sessions);
            let token = token.clone();
            let start = Arc::clone(&start);
            let release_owner = Arc::clone(&release_owner);
            let outcomes = outcomes.clone();
            thread::spawn(move || {
                start.wait();
                let claim = sessions.claim(&token, now);
                outcomes
                    .send(claim.as_ref().map(|_| ()).map_err(|error| *error))
                    .unwrap();
                release_owner.wait();
                drop(claim);
            })
        })
        .collect::<Vec<_>>();
    drop(outcomes);

    start.wait();
    let outcomes = received.iter().take(CLAIMANTS).collect::<Vec<_>>();
    let winners = outcomes.iter().filter(|outcome| outcome.is_ok()).count();
    let in_use = outcomes
        .iter()
        .filter(|outcome| matches!(outcome, Err(SessionClaimError::ConnectionInUse)))
        .count();
    let owner_is_live = matches!(
        sessions.registry.lock().unwrap().sessions.get(&token),
        Some(SessionState::Claimed { .. })
    );

    release_owner.wait();
    for handle in handles {
        handle.join().unwrap();
    }
    assert_eq!(winners, 1);
    assert_eq!(in_use, CLAIMANTS - 1);
    assert!(owner_is_live);
    assert!(matches!(
        sessions.claim(&token, now),
        Err(SessionClaimError::AuthenticationFailed)
    ));
}

#[test]
fn claim_prunes_expired_tokens_and_fails_closed_on_id_exhaustion() {
    let now = Instant::now();
    let sessions = Arc::new(deterministic_sessions(now));
    let expired = sessions.issue(now).unwrap();
    assert!(matches!(
        sessions.claim(&expired.token, now + SESSION_LIFETIME),
        Err(SessionClaimError::AuthenticationFailed)
    ));

    let issued = sessions.issue(now + SESSION_LIFETIME).unwrap();
    sessions.registry.lock().unwrap().next_connection_id = u64::MAX;
    assert!(matches!(
        sessions.claim(&issued.token, now + SESSION_LIFETIME),
        Err(SessionClaimError::CapacityReached)
    ));
    assert!(matches!(
        sessions
            .registry
            .lock()
            .unwrap()
            .sessions
            .get(&issued.token),
        Some(SessionState::Issued { .. })
    ));
}

#[test]
fn upgrade_admission_has_independent_burst_and_refill() {
    let now = Instant::now();
    let sessions = GameSessions::new(now);
    for _ in 0..32 {
        assert!(sessions.allow_upgrade(now));
    }
    assert!(!sessions.allow_upgrade(now));
    assert!(!sessions.allow_upgrade(now + Duration::from_millis(124)));
    assert!(sessions.allow_upgrade(now + Duration::from_millis(125)));
}
