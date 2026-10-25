use std::time::Instant;

use crate::rate_limit::TokenBucket;

const OPERATION_BURST: f64 = 32.0;
const OPERATIONS_PER_SECOND: f64 = 8.0;

pub(super) struct OperationBudgets {
    create: TokenBucket,
    join: TokenBucket,
    resume: TokenBucket,
}

impl OperationBudgets {
    pub(super) fn new(now: Instant) -> Self {
        Self {
            create: TokenBucket::new(now, OPERATION_BURST, OPERATIONS_PER_SECOND),
            join: TokenBucket::new(now, OPERATION_BURST, OPERATIONS_PER_SECOND),
            resume: TokenBucket::new(now, OPERATION_BURST, OPERATIONS_PER_SECOND),
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

#[cfg(test)]
#[path = "rate_limit_tests.rs"]
mod tests;
