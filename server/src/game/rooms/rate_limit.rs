use std::time::Instant;

const OPERATION_BURST: f64 = 32.0;
const OPERATIONS_PER_SECOND: f64 = 8.0;

pub(super) struct OperationBudgets {
    create: Bucket,
    join: Bucket,
    resume: Bucket,
}

impl OperationBudgets {
    pub(super) fn new(now: Instant) -> Self {
        Self {
            create: Bucket::new(now),
            join: Bucket::new(now),
            resume: Bucket::new(now),
        }
    }

    pub(super) fn take_create(&mut self, now: Instant) -> bool {
        self.create.take(now)
    }

    pub(super) fn take_join(&mut self, now: Instant) -> bool {
        self.join.take(now)
    }

    pub(super) fn take_resume(&mut self, now: Instant) -> bool {
        self.resume.take(now)
    }
}

pub(super) struct Bucket {
    available: f64,
    updated_at: Instant,
}

#[cfg(test)]
#[path = "rate_limit_tests.rs"]
mod tests;

impl Bucket {
    pub(super) fn new(now: Instant) -> Self {
        Self {
            available: OPERATION_BURST,
            updated_at: now,
        }
    }

    pub(super) fn take(&mut self, now: Instant) -> bool {
        let elapsed = now.saturating_duration_since(self.updated_at);
        self.available =
            (self.available + elapsed.as_secs_f64() * OPERATIONS_PER_SECOND).min(OPERATION_BURST);
        self.updated_at = self.updated_at.max(now);
        if self.available < 1.0 {
            return false;
        }
        self.available -= 1.0;
        true
    }
}
