use std::time::Instant;

pub(crate) struct TokenBucket {
    available: f64,
    capacity: f64,
    refill_per_second: f64,
    updated_at: Instant,
}

impl TokenBucket {
    pub(crate) fn new(now: Instant, capacity: f64, refill_per_second: f64) -> Self {
        Self {
            available: capacity,
            capacity,
            refill_per_second,
            updated_at: now,
        }
    }

    pub(crate) fn take(&mut self, now: Instant) -> bool {
        let elapsed = now.saturating_duration_since(self.updated_at);
        self.available =
            (self.available + elapsed.as_secs_f64() * self.refill_per_second).min(self.capacity);
        self.updated_at = self.updated_at.max(now);
        if self.available < 1.0 {
            return false;
        }
        self.available -= 1.0;
        true
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn fractional_refill_reaches_a_whole_token_at_the_configured_rate() {
        let now = Instant::now();
        let mut bucket = TokenBucket::new(now, 1.0, 2.0);
        assert!(bucket.take(now));
        assert!(!bucket.take(now + Duration::from_millis(499)));
        assert!(bucket.take(now + Duration::from_millis(500)));
    }

    #[test]
    fn refill_saturates_at_the_configured_capacity() {
        let now = Instant::now();
        let later = now + Duration::from_secs(10);
        let mut bucket = TokenBucket::new(now, 2.0, 8.0);
        assert!(bucket.take(now));
        assert!(bucket.take(later));
        assert!(bucket.take(later));
        assert!(!bucket.take(later));
    }

    #[test]
    fn backwards_captured_time_does_not_refill_or_rewind_the_timeline() {
        let now = Instant::now();
        let later = now + Duration::from_secs(1);
        let mut bucket = TokenBucket::new(now, 1.0, 1.0);
        assert!(bucket.take(now));
        assert!(bucket.take(later));
        assert!(!bucket.take(now));
        assert!(!bucket.take(later));
    }

    #[test]
    fn bucket_instances_have_independent_budgets() {
        let now = Instant::now();
        let mut first = TokenBucket::new(now, 1.0, 1.0);
        let mut second = TokenBucket::new(now, 1.0, 1.0);
        assert!(first.take(now));
        assert!(!first.take(now));
        assert!(second.take(now));
    }
}
