use std::collections::BTreeMap;

use super::{
    arena::Arena,
    bombs::Bomb,
    generation::{GeneratedArena, GenerationRecord},
    pickups::{Pickup, PickupKind, PickupLayers, PickupPlacementError},
    types::{CellId, EntityId, Tick},
};

/// The independent spatial layers used by deterministic simulation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct World {
    pub(crate) arena: Arena,
    pub(crate) bombs: BTreeMap<EntityId, Bomb>,
    pub(crate) pickups: PickupLayers,
    pub(crate) hazards: BTreeMap<CellId, Tick>,
}

impl World {
    #[must_use]
    pub fn new(arena: Arena) -> Self {
        Self {
            arena,
            bombs: BTreeMap::new(),
            pickups: PickupLayers::default(),
            hazards: BTreeMap::new(),
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

    /// Builds a world from generated layers while returning replay metadata to the caller.
    pub fn from_generated(
        generated: GeneratedArena,
    ) -> Result<(Self, GenerationRecord), PickupPlacementError> {
        let (arena, concealed, record) = generated.into_parts();
        Self::with_pickups(arena, concealed, BTreeMap::new(), BTreeMap::new())
            .map(|world| (world, record))
    }

    /// Builds a world after validating all pickup layers against the arena.
    pub fn with_pickups(
        arena: Arena,
        concealed: BTreeMap<CellId, PickupKind>,
        pending: BTreeMap<CellId, PickupKind>,
        exposed: BTreeMap<CellId, PickupKind>,
    ) -> Result<Self, PickupPlacementError> {
        let pickups = PickupLayers::try_new(&arena, concealed, pending, exposed)?;
        Ok(Self {
            arena,
            bombs: BTreeMap::new(),
            pickups,
            hazards: BTreeMap::new(),
        })
    }

    /// Returns only pickups currently visible to players.
    #[must_use]
    pub const fn exposed_pickups(&self) -> &BTreeMap<CellId, Pickup> {
        &self.pickups.exposed
    }

    #[must_use]
    pub const fn hazards(&self) -> &BTreeMap<CellId, Tick> {
        &self.hazards
    }

    pub(crate) fn expose_flame_free_pickups(&mut self) {
        self.pickups.expose_flame_free(&self.hazards);
    }
}

#[cfg(test)]
#[path = "world_tests.rs"]
mod tests;
