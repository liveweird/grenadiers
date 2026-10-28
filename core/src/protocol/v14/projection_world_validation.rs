use std::collections::BTreeSet;

use super::projection_facts::{PrivatePickupPhase, WorldFacts};
use super::{DrawReason, ProjectionError, RoundResult, Terrain};

pub(crate) fn validate(world: &WorldFacts) -> Result<(), ProjectionError> {
    validate_clock(world)?;
    validate_arena(world)?;
    validate_flames(world)?;
    validate_pickups(world)?;
    validate_result(world)
}

fn validate_clock(world: &WorldFacts) -> Result<(), ProjectionError> {
    (world.running_updates <= crate::protocol::MAX_RUNNING_UPDATES
        && world.remaining_ticks <= crate::protocol::MAX_RUNNING_UPDATES
        && world.running_updates.checked_add(world.remaining_ticks)
            == Some(crate::protocol::MAX_RUNNING_UPDATES))
    .then_some(())
    .ok_or(ProjectionError::InvalidClock)
}

fn validate_arena(world: &WorldFacts) -> Result<(), ProjectionError> {
    let arena = &world.arena;
    super::projection_geometry::validate_dimensions(world)?;
    if arena.blocks.len() > crate::protocol::MAX_BLOCKS
        || !strict_cells(&arena.blocks)
        || arena
            .blocks
            .iter()
            .any(|&cell| terrain_at(world, cell) != Some(Terrain::Floor))
    {
        return Err(ProjectionError::InvalidArena);
    }
    Ok(())
}

fn validate_flames(world: &WorldFacts) -> Result<(), ProjectionError> {
    if world.flames.len() > crate::protocol::MAX_FLAMES
        || !world
            .flames
            .windows(2)
            .all(|pair| cell_key(pair[0].cell) < cell_key(pair[1].cell))
        || world.flames.iter().any(|flame| {
            terrain_at(world, flame.cell) != Some(Terrain::Floor)
                || world.result.is_none() && flame.expires_at <= world.physical_tick
        })
    {
        return Err(ProjectionError::InvalidFlames);
    }
    Ok(())
}

fn validate_pickups(world: &WorldFacts) -> Result<(), ProjectionError> {
    let blocks = cell_set(&world.arena.blocks);
    let flames = world
        .flames
        .iter()
        .map(|flame| cell_key(flame.cell))
        .collect::<BTreeSet<_>>();
    let exposed = world
        .private_pickups
        .iter()
        .filter(|facts| facts.phase == PrivatePickupPhase::Exposed)
        .map(|facts| (facts.cell, facts.kind))
        .collect::<Vec<_>>();
    if world.pickups.len() > crate::protocol::MAX_PICKUPS
        || world.private_pickups.len() > crate::protocol::MAX_PICKUPS
        || !world
            .pickups
            .windows(2)
            .all(|pair| cell_key(pair[0].cell) < cell_key(pair[1].cell))
        || !strict_cells(&world.private_pickup_cells)
        || !world
            .private_pickups
            .windows(2)
            .all(|pair| cell_key(pair[0].cell) < cell_key(pair[1].cell))
        || world.private_pickup_cells
            != world
                .private_pickups
                .iter()
                .map(|facts| facts.cell)
                .collect::<Vec<_>>()
        || exposed
            != world
                .pickups
                .iter()
                .map(|pickup| (pickup.cell, pickup.kind))
                .collect::<Vec<_>>()
        || world.private_pickups.iter().any(|facts| {
            let cell = cell_key(facts.cell);
            terrain_at(world, facts.cell) != Some(Terrain::Floor)
                || facts.kind == super::PickupKind::OilBarrel
                || match facts.phase {
                    PrivatePickupPhase::Concealed => !blocks.contains(&cell),
                    PrivatePickupPhase::Pending => {
                        blocks.contains(&cell) || !flames.contains(&cell)
                    }
                    PrivatePickupPhase::Exposed => blocks.contains(&cell) || flames.contains(&cell),
                }
        })
        || world.pickups.iter().any(|pickup| {
            terrain_at(world, pickup.cell) != Some(Terrain::Floor)
                || blocks.contains(&cell_key(pickup.cell))
                || flames.contains(&cell_key(pickup.cell))
                || pickup.kind == super::PickupKind::OilBarrel
                || !world.private_pickup_cells.contains(&pickup.cell)
        })
    {
        return Err(ProjectionError::InvalidPickups);
    }
    Ok(())
}

fn validate_result(world: &WorldFacts) -> Result<(), ProjectionError> {
    let survivors = world.actors.iter().filter(|actor| actor.view.alive).count();
    let valid = match world.result {
        None => survivors >= 2 && world.running_updates < crate::protocol::MAX_RUNNING_UPDATES,
        Some(RoundResult::Winner { seat }) => {
            survivors == 1
                && world
                    .actors
                    .get(seat.index())
                    .is_some_and(|actor| actor.view.alive)
        }
        Some(RoundResult::Draw {
            reason: DrawReason::MutualElimination,
        }) => survivors == 0,
        Some(RoundResult::Draw {
            reason: DrawReason::Timeout,
        }) => survivors >= 2 && world.running_updates == crate::protocol::MAX_RUNNING_UPDATES,
    };
    valid.then_some(()).ok_or(ProjectionError::InvalidResult)
}

pub(super) fn terrain_at(world: &WorldFacts, cell: crate::protocol::Cell) -> Option<Terrain> {
    super::projection_geometry::terrain_at(world, cell)
}

pub(super) fn cell_key(cell: crate::protocol::Cell) -> (u8, u8) {
    (cell.row(), cell.column())
}

fn cell_set(cells: &[crate::protocol::Cell]) -> BTreeSet<(u8, u8)> {
    cells.iter().copied().map(cell_key).collect()
}

fn strict_cells(cells: &[crate::protocol::Cell]) -> bool {
    cells
        .windows(2)
        .all(|pair| cell_key(pair[0]) < cell_key(pair[1]))
}
