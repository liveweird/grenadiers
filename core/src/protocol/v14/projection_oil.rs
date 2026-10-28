use std::collections::BTreeSet;

use super::projection_facts::{OilPhase, WorldFacts};
use super::{EquipmentId, ItemId, ProjectionError, Terrain};

pub(crate) fn validate(world: &WorldFacts) -> Result<(), ProjectionError> {
    let has_private = !world.oil.is_empty()
        || world
            .actors
            .iter()
            .any(|actor| !actor.unlit_oil_grace.is_empty());
    if has_private && !world.enabled_items.contains(ItemId::OilBarrel)
        || world.oil.len() > crate::game::MAX_OIL_BARRELS
        || !strict_oil(&world.oil)
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
        if terrain_at(world, oil.cell) != Some(Terrain::Floor)
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
    if flames.union(&burning).count() > world.arena.terrain.len() {
        return Err(ProjectionError::InvalidFlames);
    }
    validate_grace(world)
}

fn validate_grace(world: &WorldFacts) -> Result<(), ProjectionError> {
    for actor in &world.actors {
        let holder = actor.view.inventory.equipment == Some(EquipmentId::WallPass);
        if actor.soft_wall_grace.len() + actor.unlit_oil_grace.len() > 4
            || !strict_cells(&actor.soft_wall_grace)
            || !strict_cells(&actor.unlit_oil_grace)
            || !actor.soft_wall_grace.is_empty() && !world.enabled_items.contains(ItemId::WallPass)
            || (!actor.view.alive
                && (!actor.soft_wall_grace.is_empty() || !actor.unlit_oil_grace.is_empty()))
            || (holder && (!actor.soft_wall_grace.is_empty() || !actor.unlit_oil_grace.is_empty()))
            || actor.soft_wall_grace.iter().any(|cell| {
                !world.arena.blocks.contains(cell) || !position_overlaps(actor.view.position, *cell)
            })
            || actor.unlit_oil_grace.iter().any(|cell| {
                !world
                    .oil
                    .iter()
                    .any(|oil| oil.cell == *cell && oil.phase == OilPhase::Unlit)
                    || !position_overlaps(actor.view.position, *cell)
            })
        {
            return Err(ProjectionError::InvalidActors);
        }
        if actor.view.alive
            && !holder
            && (world.arena.blocks.iter().any(|cell| {
                position_overlaps(actor.view.position, *cell)
                    && !actor.soft_wall_grace.contains(cell)
            }) || world.oil.iter().any(|oil| {
                oil.phase == OilPhase::Unlit
                    && position_overlaps(actor.view.position, oil.cell)
                    && !actor.unlit_oil_grace.contains(&oil.cell)
            }))
        {
            return Err(ProjectionError::InvalidActors);
        }
    }
    Ok(())
}

fn terrain_at(world: &WorldFacts, cell: crate::protocol::Cell) -> Option<Terrain> {
    super::projection_geometry::terrain_at(world, cell)
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
fn strict_oil(oil: &[super::projection_facts::OilFacts]) -> bool {
    oil.windows(2)
        .all(|pair| cell_key(pair[0].cell) < cell_key(pair[1].cell))
}
