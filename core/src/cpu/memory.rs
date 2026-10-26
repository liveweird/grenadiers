use std::collections::VecDeque;

use crate::game::SeatId;

use super::{DecisionId, Intent, WorldId};

pub const MAX_MEMORY_HISTORY: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MemoryKey {
    pub world: WorldId,
    pub cpu_seat: SeatId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MemoryEntry {
    pub decision: DecisionId,
    pub intent: Intent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemoryUpdate {
    Added,
    Evicted(MemoryEntry),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemoryError {
    InvalidCapacity,
    WrongWorld,
    DecisionNotAdmitted,
    AlreadyRemembered,
}

/// Bounded facts retained by a CPU across command reconnects.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CpuMemory {
    key: MemoryKey,
    capacity: usize,
    history: VecDeque<MemoryEntry>,
    last_admitted: Option<DecisionId>,
}

impl CpuMemory {
    pub fn new(key: MemoryKey, capacity: usize) -> Result<Self, MemoryError> {
        if capacity == 0 || capacity > MAX_MEMORY_HISTORY {
            return Err(MemoryError::InvalidCapacity);
        }
        Ok(Self {
            key,
            capacity,
            history: VecDeque::with_capacity(capacity),
            last_admitted: None,
        })
    }

    #[must_use]
    pub const fn key(&self) -> MemoryKey {
        self.key
    }

    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.capacity
    }

    #[must_use]
    pub fn history(&self) -> &VecDeque<MemoryEntry> {
        &self.history
    }

    #[must_use]
    pub const fn last_admitted(&self) -> Option<DecisionId> {
        self.last_admitted
    }

    pub fn remember(
        &mut self,
        decision: DecisionId,
        intent: Intent,
    ) -> Result<MemoryUpdate, MemoryError> {
        if decision.world != self.key.world {
            return Err(MemoryError::WrongWorld);
        }
        if self.last_admitted != Some(decision) {
            return Err(MemoryError::DecisionNotAdmitted);
        }
        if self.history.iter().any(|entry| entry.decision == decision) {
            return Err(MemoryError::AlreadyRemembered);
        }
        let evicted = (self.history.len() == self.capacity)
            .then(|| self.history.pop_front())
            .flatten();
        self.history.push_back(MemoryEntry { decision, intent });
        Ok(match evicted {
            Some(entry) => MemoryUpdate::Evicted(entry),
            None => MemoryUpdate::Added,
        })
    }

    /// Resets all round-bound knowledge while retaining the configured bound.
    pub fn reset_for(&mut self, key: MemoryKey) {
        self.key = key;
        self.history.clear();
        self.last_admitted = None;
    }

    pub(crate) fn set_last_admitted(&mut self, decision: DecisionId) {
        self.last_admitted = Some(decision);
    }
}
