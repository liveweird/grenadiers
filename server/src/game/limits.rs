use std::time::Duration;

use tokio::time::Instant;

const INBOUND_BURST: f64 = 30.0;
const INBOUND_REFILL_PER_SECOND: f64 = 60.0;
const INACTIVITY_TIMEOUT: Duration = Duration::from_secs(30);
const UNSEATED_LIFETIME: Duration = Duration::from_secs(300);

pub(crate) struct InboundLimit {
    available: f64,
    updated_at: Instant,
}

impl InboundLimit {
    pub(crate) fn new(now: Instant) -> Self {
        Self {
            available: INBOUND_BURST,
            updated_at: now,
        }
    }

    pub(crate) fn take(&mut self, now: Instant) -> bool {
        let elapsed = now.saturating_duration_since(self.updated_at);
        self.available =
            (self.available + elapsed.as_secs_f64() * INBOUND_REFILL_PER_SECOND).min(INBOUND_BURST);
        self.updated_at = self.updated_at.max(now);
        if self.available < 1.0 {
            false
        } else {
            self.available -= 1.0;
            true
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum SessionDeadline {
    Active,
    Inactive,
    Expired,
}

pub(super) struct SessionDeadlines {
    expires_at: Instant,
    last_inbound: Instant,
}

impl SessionDeadlines {
    pub(super) fn new(now: Instant) -> Self {
        Self {
            expires_at: now + UNSEATED_LIFETIME,
            last_inbound: now,
        }
    }

    pub(super) fn record_inbound(&mut self, now: Instant) {
        self.last_inbound = self.last_inbound.max(now);
    }

    pub(super) fn restart_unseated(&mut self, now: Instant) {
        self.expires_at = now + UNSEATED_LIFETIME;
    }

    pub(super) fn inactivity_at(&self) -> Instant {
        self.last_inbound + INACTIVITY_TIMEOUT
    }

    pub(super) fn expires_at(&self) -> Instant {
        self.expires_at
    }

    pub(super) fn classify(&self, now: Instant) -> SessionDeadline {
        if now >= self.expires_at {
            SessionDeadline::Expired
        } else if now >= self.inactivity_at() {
            SessionDeadline::Inactive
        } else {
            SessionDeadline::Active
        }
    }

    pub(super) fn classify_member(&self, now: Instant) -> SessionDeadline {
        if now >= self.inactivity_at() {
            SessionDeadline::Inactive
        } else {
            SessionDeadline::Active
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inbound_limit_counts_controls_and_refills_at_sixty_per_second() {
        let now = Instant::now();
        let mut limit = InboundLimit::new(now);
        for _ in 0..30 {
            assert!(limit.take(now));
        }
        assert!(!limit.take(now));
        assert!(!limit.take(now + Duration::from_millis(16)));
        assert!(limit.take(now + Duration::from_millis(17)));
    }

    #[test]
    fn activity_refreshes_inactivity_but_not_absolute_expiry() {
        let started = Instant::now();
        let mut deadlines = SessionDeadlines::new(started);

        deadlines.record_inbound(started + Duration::from_secs(29));
        assert_eq!(
            deadlines.classify(started + Duration::from_secs(58)),
            SessionDeadline::Active
        );
        assert_eq!(
            deadlines.classify(started + Duration::from_secs(59)),
            SessionDeadline::Inactive
        );

        deadlines.record_inbound(started + Duration::from_secs(299));
        assert_eq!(
            deadlines.classify(started + Duration::from_secs(300)),
            SessionDeadline::Expired
        );
    }

    #[test]
    fn membership_disables_old_expiry_and_leave_starts_a_fresh_window() {
        let started = Instant::now();
        let mut deadlines = SessionDeadlines::new(started);
        let later = started + Duration::from_secs(299);
        deadlines.record_inbound(later);
        assert_eq!(
            deadlines.classify_member(started + UNSEATED_LIFETIME),
            SessionDeadline::Active
        );

        deadlines.restart_unseated(later);
        assert_eq!(
            deadlines.classify(later + UNSEATED_LIFETIME - Duration::from_nanos(1)),
            SessionDeadline::Inactive
        );
        deadlines.record_inbound(later + Duration::from_secs(299));
        assert_eq!(
            deadlines.classify(later + UNSEATED_LIFETIME),
            SessionDeadline::Expired
        );
    }
}
