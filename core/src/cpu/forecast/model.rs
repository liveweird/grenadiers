use std::collections::{BTreeMap, BTreeSet};

use crate::cpu::{Observation, ObservationParts};
use crate::game::arena::{ARENA_WIDTH, Terrain};
use crate::game::blast_geometry::{BlastBomb, BlastField};
use crate::game::{BlastKind, CellId, EntityId, Tick};

#[derive(Clone, Copy)]
pub(super) enum Trigger {
    Timed(Tick),
    Paused(Tick),
    Remote,
    Contact,
}

#[derive(Clone, Copy)]
pub(super) struct ForecastBomb {
    pub(super) id: EntityId,
    pub(super) cell: CellId,
    pub(super) range: u16,
    pub(super) blast: BlastKind,
    pub(super) trigger: Trigger,
    pub(super) hypothetical: bool,
}

pub(super) struct ForecastField<'a> {
    pub(super) observation: &'a Observation,
    pub(super) blocks: &'a BTreeSet<CellId>,
    pub(super) oil: &'a BTreeSet<CellId>,
    pub(super) bombs: &'a BTreeMap<CellId, ForecastBomb>,
}

impl BlastField for ForecastField<'_> {
    fn is_floor(&self, cell: CellId) -> bool {
        terrain(self.observation.parts(), cell) == Some(Terrain::Floor)
    }

    fn has_soft_block(&self, cell: CellId) -> bool {
        self.blocks.contains(&cell)
    }

    fn has_unlit_oil(&self, cell: CellId) -> bool {
        self.oil.contains(&cell)
    }

    fn bomb_at(&self, cell: CellId) -> Option<BlastBomb> {
        let bomb = self.bombs.get(&cell)?;
        Some(BlastBomb {
            id: bomb.id,
            contact: matches!(bomb.trigger, Trigger::Contact),
        })
    }
}

pub(super) fn terrain(parts: &ObservationParts, cell: CellId) -> Option<Terrain> {
    if cell.column >= parts.arena.width || cell.row >= parts.arena.height {
        return None;
    }
    parts.arena.terrain.get(index(cell)).copied()
}

pub(super) fn index(cell: CellId) -> usize {
    usize::from(cell.row) * usize::from(ARENA_WIDTH) + usize::from(cell.column)
}

pub(super) fn unused_id(bombs: &[ForecastBomb]) -> EntityId {
    let mut value = u32::MAX;
    while bombs.iter().any(|bomb| bomb.id == EntityId(value)) {
        value -= 1;
    }
    EntityId(value)
}

pub(super) fn offset(cell: CellId, dx: i32, dy: i32) -> Option<CellId> {
    let column = i32::from(cell.column).checked_add(dx)?;
    let row = i32::from(cell.row).checked_add(dy)?;
    if column < 0 || row < 0 {
        return None;
    }
    Some(CellId::new(
        u16::try_from(column).ok()?,
        u16::try_from(row).ok()?,
    ))
}
