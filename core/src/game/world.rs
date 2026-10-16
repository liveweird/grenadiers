use std::collections::{BTreeMap, BTreeSet};

use super::{
    arena::Arena,
    bombs::Bomb,
    types::{CellId, EntityId},
};

/// The independent spatial layers used by deterministic simulation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct World {
    pub(crate) arena: Arena,
    pub(crate) bombs: BTreeMap<EntityId, Bomb>,
    pub(crate) pickups: BTreeSet<CellId>,
    pub(crate) hazards: BTreeSet<CellId>,
}

impl World {
    #[must_use]
    pub fn new(arena: Arena) -> Self {
        Self {
            arena,
            bombs: BTreeMap::new(),
            pickups: BTreeSet::new(),
            hazards: BTreeSet::new(),
        }
    }

    #[must_use]
    pub const fn arena(&self) -> &Arena {
        &self.arena
    }

    #[must_use]
    pub const fn bombs(&self) -> &BTreeMap<EntityId, Bomb> {
        &self.bombs
    }

    #[must_use]
    pub const fn pickups(&self) -> &BTreeSet<CellId> {
        &self.pickups
    }

    #[must_use]
    pub const fn hazards(&self) -> &BTreeSet<CellId> {
        &self.hazards
    }
}

#[cfg(test)]
#[path = "world_tests.rs"]
mod tests;
