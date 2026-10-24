use std::collections::BTreeSet;

use super::projection_facts::{OilPhase, ProjectionError, WorldFacts};
use super::{EquipmentId, ItemId, ItemSet, MAX_FLAMES, Terrain};

pub(super) fn validate_world(
    world: &WorldFacts,
    supported_items: ItemSet,
) -> Result<(), ProjectionError> {
    let supported = supported_items.contains(ItemId::OilBarrel)
        && world.available_items.contains(ItemId::OilBarrel);
    let enabled = world.enabled_items.contains(ItemId::OilBarrel);
    let has_private =
        !world.oil.is_empty() || world.unlit_oil_grace.iter().any(|grace| !grace.is_empty());
    if has_private && (!supported || !enabled)
        || world.oil.len() > crate::game::MAX_OIL_BARRELS
        || !world
            .oil
            .windows(2)
            .all(|pair| cell_key(pair[0].cell) < cell_key(pair[1].cell))
    {
        return Err(ProjectionError::InvalidItems);
    }

    let blocks = cell_set(&world.arena.blocks);
    let bombs = world
        .bombs
        .iter()
        .map(|bomb| cell_key(bomb.cell))
        .collect::<BTreeSet<_>>();
    let flames = world
        .flames
        .iter()
        .map(|flame| cell_key(flame.cell))
        .collect::<BTreeSet<_>>();
    let pickups = cell_set(&world.private_pickup_cells);
    let mut burning = BTreeSet::new();
    for oil in &world.oil {
        let cell = cell_key(oil.cell);
        let blocked = blocks.contains(&cell);
        if super::projection_validation::terrain_at(world, oil.cell) != Some(Terrain::Floor)
            || pickups.contains(&cell)
            || match oil.phase {
                OilPhase::Concealed => !blocked || flames.contains(&cell),
                OilPhase::Pending => blocked,
                OilPhase::Unlit => blocked || bombs.contains(&cell) || flames.contains(&cell),
                OilPhase::Burning => {
                    !burning.insert(cell)
                        || blocked
                        || bombs.contains(&cell)
                        || flames.contains(&cell)
                }
            }
        {
            return Err(ProjectionError::InvalidItems);
        }
    }
    if flames.union(&burning).count() > MAX_FLAMES {
        return Err(ProjectionError::InvalidFlames);
    }
    validate_grace(world)
}

fn validate_grace(world: &WorldFacts) -> Result<(), ProjectionError> {
    for ((actor, soft_grace), oil_grace) in world
        .actors
        .iter()
        .zip(&world.soft_wall_grace)
        .zip(&world.unlit_oil_grace)
    {
        let holder = actor.inventory.equipment == Some(EquipmentId::WallPass);
        if soft_grace.len() + oil_grace.len() > 4
            || !strict_cells(oil_grace)
            || (!actor.alive && !oil_grace.is_empty())
            || (holder && !oil_grace.is_empty())
            || oil_grace.iter().any(|cell| {
                !world
                    .oil
                    .iter()
                    .any(|oil| oil.cell == *cell && oil.phase == OilPhase::Unlit)
                    || !position_overlaps(actor.position, *cell)
            })
        {
            return Err(ProjectionError::InvalidActors);
        }
        if actor.alive && !holder {
            for oil in &world.oil {
                if oil.phase == OilPhase::Unlit
                    && position_overlaps(actor.position, oil.cell)
                    && !oil_grace.contains(&oil.cell)
                {
                    return Err(ProjectionError::InvalidActors);
                }
            }
        }
    }
    Ok(())
}

fn position_overlaps(position: crate::protocol::Position, cell: crate::protocol::Cell) -> bool {
    const HALF_FOOTPRINT: i32 = 320;
    const UNITS_PER_CELL: i32 = 1_024;
    let left = i32::from(cell.column()) * UNITS_PER_CELL;
    let top = i32::from(cell.row()) * UNITS_PER_CELL;
    i32::from(position.x()) - HALF_FOOTPRINT < left + UNITS_PER_CELL
        && i32::from(position.x()) + HALF_FOOTPRINT > left
        && i32::from(position.y()) - HALF_FOOTPRINT < top + UNITS_PER_CELL
        && i32::from(position.y()) + HALF_FOOTPRINT > top
}

fn cell_key(cell: crate::protocol::Cell) -> (u8, u8) {
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
