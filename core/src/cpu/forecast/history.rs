use std::collections::BTreeMap;

use crate::cpu::{BombKnowledge, MemoryKey, Observation};
use crate::game::{CellId, EntityId, Tick};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct BombSnapshot {
    tick: Tick,
    cell: CellId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct BombHistory {
    previous: Option<BombSnapshot>,
    latest: BombSnapshot,
    displaced: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservedHazardMemory {
    key: MemoryKey,
    bombs: BTreeMap<EntityId, BombHistory>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HazardMemoryError {
    WrongWorld,
    BombKnowledgeUnavailable,
    StaleObservation,
    ConflictingObservation,
}

impl ObservedHazardMemory {
    #[must_use]
    pub const fn new(key: MemoryKey) -> Self {
        Self {
            key,
            bombs: BTreeMap::new(),
        }
    }

    #[must_use]
    pub const fn key(&self) -> MemoryKey {
        self.key
    }

    pub fn reset_for(&mut self, key: MemoryKey) {
        self.key = key;
        self.bombs.clear();
    }

    pub fn observe(&mut self, observation: &Observation) -> Result<(), HazardMemoryError> {
        let parts = observation.parts();
        if parts.world != self.key.world || parts.recipient != self.key.cpu_seat {
            return Err(HazardMemoryError::WrongWorld);
        }
        let BombKnowledge::Visible(observed) = &parts.bombs else {
            return Err(HazardMemoryError::BombKnowledgeUnavailable);
        };
        debug_assert!(observed.len() <= crate::cpu::observed_bomb_capacity(parts.rules.version));
        if self
            .bombs
            .values()
            .any(|history| history.latest.tick.0 > parts.tick.0)
        {
            return Err(HazardMemoryError::StaleObservation);
        }

        let mut next = BTreeMap::new();
        for bomb in observed {
            let snapshot = BombSnapshot {
                tick: parts.tick,
                cell: bomb.cell,
            };
            let history = match self.bombs.get(&bomb.id).copied() {
                Some(old) if old.latest.tick == parts.tick => {
                    if old.latest.cell != bomb.cell {
                        return Err(HazardMemoryError::ConflictingObservation);
                    }
                    old
                }
                Some(old) => BombHistory {
                    previous: Some(old.latest),
                    latest: snapshot,
                    displaced: old.displaced || old.latest.cell != bomb.cell,
                },
                None => BombHistory {
                    previous: None,
                    latest: snapshot,
                    displaced: false,
                },
            };
            next.insert(bomb.id, history);
        }
        self.bombs = next;
        Ok(())
    }

    pub(super) fn was_displaced(&self, id: EntityId) -> bool {
        self.bombs.get(&id).is_some_and(|history| history.displaced)
    }

    #[must_use]
    pub fn retained_samples(&self, id: EntityId) -> usize {
        self.bombs
            .get(&id)
            .map_or(0, |history| usize::from(history.previous.is_some()) + 1)
    }
}
