use super::{MatchRuntime, MatchRuntimeError};

impl MatchRuntime {
    pub(super) fn ensure_revision(&self) -> Result<(), MatchRuntimeError> {
        self.revision
            .checked_add(1)
            .and_then(|revision| grenadiers_core::protocol::Counter::new(revision).ok())
            .map(|_| ())
            .ok_or(MatchRuntimeError::CapacityReached)
    }

    pub(super) fn next_revision(&self) -> Result<u32, MatchRuntimeError> {
        self.revision
            .checked_add(1)
            .and_then(|revision| grenadiers_core::protocol::Counter::new(revision).ok())
            .map(|counter| counter.get())
            .ok_or(MatchRuntimeError::CapacityReached)
    }

    pub(super) fn clear_round(&mut self) {
        self.sim = None;
        self.reports.clear();
        self.inputs.reset_round();
    }
}
