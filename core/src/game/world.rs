use std::collections::BTreeMap;

use super::{
    arena::Arena,
    bombs::Bomb,
    generation::{GeneratedArena, GenerationRecord},
    oil::{OilBarrels, OilState},
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
    pub(crate) oil: OilBarrels,
}

impl World {
    #[must_use]
    pub fn new(arena: Arena) -> Self {
        Self {
            arena,
            bombs: BTreeMap::new(),
            pickups: PickupLayers::default(),
            hazards: BTreeMap::new(),
            oil: OilBarrels::default(),
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
        Self::with_pickups_and_oil(arena, concealed, pending, exposed, BTreeMap::new())
    }

    /// Builds a world with explicit Oil Barrel phases for replay and validation.
    pub(crate) fn with_pickups_and_oil(
        arena: Arena,
        mut concealed: BTreeMap<CellId, PickupKind>,
        mut pending: BTreeMap<CellId, PickupKind>,
        exposed: BTreeMap<CellId, PickupKind>,
        mut oil: BTreeMap<CellId, OilState>,
    ) -> Result<Self, PickupPlacementError> {
        if let Some((&cell, _)) = exposed
            .iter()
            .find(|(_, kind)| **kind == PickupKind::OilBarrel)
        {
            return Err(PickupPlacementError::ExposedOilBarrel(cell));
        }
        extract_oil(&mut concealed, OilState::Concealed, &mut oil)?;
        extract_oil(&mut pending, OilState::Pending, &mut oil)?;
        let oil = OilBarrels::new(oil).ok_or(PickupPlacementError::TooManyOilBarrels)?;
        validate_oil(&arena, &oil, &concealed, &pending, &exposed)?;
        let pickups = PickupLayers::try_new(&arena, concealed, pending, exposed)?;
        Ok(Self {
            arena,
            bombs: BTreeMap::new(),
            pickups,
            hazards: BTreeMap::new(),
            oil,
        })
    }

    /// Returns only pickups currently visible to players.
    #[must_use]
    pub const fn exposed_pickups(&self) -> &BTreeMap<CellId, Pickup> {
        &self.pickups.exposed
    }

    pub(crate) fn pickup_observations(
        &self,
    ) -> impl Iterator<Item = (CellId, PickupKind, super::PickupLayer)> + '_ {
        self.pickups.observations()
    }

    #[must_use]
    pub const fn hazards(&self) -> &BTreeMap<CellId, Tick> {
        &self.hazards
    }

    pub(crate) fn oil_barrels(&self) -> impl Iterator<Item = (CellId, OilState)> + '_ {
        self.oil.iter()
    }

    pub(crate) fn has_fire(&self, cell: CellId, tick: Tick) -> bool {
        self.oil.burning(cell)
            || self
                .hazards
                .get(&cell)
                .is_some_and(|expires| *expires > tick)
    }

    pub(crate) fn fire_cells(&self, tick: Tick) -> impl Iterator<Item = CellId> + '_ {
        self.hazards
            .iter()
            .filter_map(move |(&cell, &expires)| (expires > tick).then_some(cell))
            .chain(
                self.oil
                    .iter()
                    .filter_map(|(cell, state)| (state == OilState::Burning).then_some(cell)),
            )
    }

    pub(crate) fn committed_fire_cells(&self) -> impl Iterator<Item = CellId> + '_ {
        self.hazards.keys().copied().chain(
            self.oil
                .iter()
                .filter_map(|(cell, state)| (state == OilState::Burning).then_some(cell)),
        )
    }

    pub(crate) fn expose_flame_free_pickups(&mut self, tick: Tick) {
        let fire: BTreeMap<_, _> = self.fire_cells(tick).map(|cell| (cell, tick)).collect();
        self.pickups.expose_flame_free(&fire);
    }
}

fn extract_oil(
    pickups: &mut BTreeMap<CellId, PickupKind>,
    state: OilState,
    oil: &mut BTreeMap<CellId, OilState>,
) -> Result<(), PickupPlacementError> {
    let cells: Vec<_> = pickups
        .iter()
        .filter_map(|(&cell, &kind)| (kind == PickupKind::OilBarrel).then_some(cell))
        .collect();
    for cell in cells {
        pickups.remove(&cell);
        if oil.insert(cell, state).is_some() {
            return Err(PickupPlacementError::InvalidOilBarrel(cell));
        }
    }
    Ok(())
}

fn validate_oil(
    arena: &Arena,
    oil: &OilBarrels,
    concealed: &BTreeMap<CellId, PickupKind>,
    pending: &BTreeMap<CellId, PickupKind>,
    exposed: &BTreeMap<CellId, PickupKind>,
) -> Result<(), PickupPlacementError> {
    for (cell, state) in oil.iter() {
        let valid = arena.terrain(cell) == Some(super::Terrain::Floor)
            && match state {
                OilState::Concealed => arena.blocks().contains(&cell),
                OilState::Pending | OilState::Unlit | OilState::Burning => {
                    !arena.blocks().contains(&cell)
                }
            }
            && !concealed.contains_key(&cell)
            && !pending.contains_key(&cell)
            && !exposed.contains_key(&cell);
        if !valid {
            return Err(PickupPlacementError::InvalidOilBarrel(cell));
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "world_tests.rs"]
mod tests;
