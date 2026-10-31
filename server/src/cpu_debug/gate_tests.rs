use std::{net::Ipv4Addr, time::Duration};

use super::*;

#[test]
fn peer_budget_sustains_two_hertz_at_the_exact_refill_boundary() {
    let now = Instant::now();
    let gate = CpuDebugGate::new(now);
    let peer = PeerKey::Ipv4(Ipv4Addr::new(192, 0, 2, 90));

    drop(gate.acquire(peer, now).unwrap());
    drop(gate.acquire(peer, now).unwrap());
    assert!(matches!(
        gate.acquire(peer, now),
        Err(GateError::RateLimited)
    ));
    assert!(matches!(
        gate.acquire(peer, now + Duration::from_millis(499)),
        Err(GateError::RateLimited)
    ));
    drop(
        gate.acquire(peer, now + Duration::from_millis(500))
            .unwrap(),
    );
    assert!(matches!(
        gate.acquire(peer, now + Duration::from_millis(999)),
        Err(GateError::RateLimited)
    ));
    drop(gate.acquire(peer, now + Duration::from_secs(1)).unwrap());
}
