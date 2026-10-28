use crate::protocol::{Cell, Position};
use crate::room::BoardSize;

use super::projection_facts::WorldFacts;
use super::{ProjectionError, Terrain};

pub(super) fn validate_dimensions(world: &WorldFacts) -> Result<(), ProjectionError> {
    let board = BoardSize::try_from((u16::from(world.arena.width), u16::from(world.arena.height)))
        .map_err(|()| ProjectionError::InvalidArena)?;
    if world.arena.terrain.len() != board.cell_count()
        || (world.rules_version.get() < 6 && board != BoardSize::FifteenByThirteen)
    {
        return Err(ProjectionError::InvalidArena);
    }
    Ok(())
}

pub(super) fn terrain_at(world: &WorldFacts, cell: Cell) -> Option<Terrain> {
    if cell.column() >= world.arena.width || cell.row() >= world.arena.height {
        return None;
    }
    world
        .arena
        .terrain
        .get(usize::from(cell.row()) * usize::from(world.arena.width) + usize::from(cell.column()))
        .copied()
}

pub(super) fn position_in_bounds(world: &WorldFacts, position: Position) -> bool {
    let half = crate::game::types::HALF_FOOTPRINT;
    let units = crate::game::types::UNITS_PER_CELL;
    i32::from(position.x()) >= half
        && i32::from(position.y()) >= half
        && i32::from(position.x()) + half <= i32::from(world.arena.width) * units
        && i32::from(position.y()) + half <= i32::from(world.arena.height) * units
}
