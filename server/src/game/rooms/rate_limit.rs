use std::time::Instant;

use super::{OPERATION_BURST, OPERATIONS_PER_SECOND};

pub(super) struct Bucket {
    available: f64,
    updated_at: Instant,
}

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
