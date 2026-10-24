use std::collections::BTreeSet;

use super::arena::{ARENA_HEIGHT, ARENA_WIDTH, Terrain};
use super::input::Direction;
use super::types::{CellId, EntityId, HALF_FOOTPRINT, Position, UNITS_PER_CELL};
use super::world::World;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Collision {
    pub(crate) distance: i32,
    pub(crate) bomb: Option<EntityId>,
    pub(crate) ambiguous: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ObstacleAccess<'a> {
    pub(crate) can_pass: bool,
    pub(crate) soft_grace: &'a BTreeSet<CellId>,
    pub(crate) oil_grace: &'a BTreeSet<CellId>,
}

pub(crate) fn position_is_free(
    world: &World,
    position: Position,
    bomb_grace: &BTreeSet<EntityId>,
    can_pass_soft_walls: bool,
    soft_wall_grace: &BTreeSet<CellId>,
    unlit_oil_grace: &BTreeSet<CellId>,
) -> bool {
    let x = i64::from(position.x);
    let y = i64::from(position.y);
    let half = i64::from(HALF_FOOTPRINT);
    if x - half < 0
        || y - half < 0
        || x + half > i64::from(ARENA_WIDTH) * i64::from(UNITS_PER_CELL)
        || y + half > i64::from(ARENA_HEIGHT) * i64::from(UNITS_PER_CELL)
    {
        return false;
    }
    blocking_cells(world).all(|(cell, bomb_id, soft, oil)| {
        !position_overlaps_cell(position, cell)
            || bomb_id.is_some_and(|id| bomb_grace.contains(&id))
            || soft && (can_pass_soft_walls || soft_wall_grace.contains(&cell))
            || oil && (can_pass_soft_walls || unlit_oil_grace.contains(&cell))
    })
}

pub(crate) fn position_overlaps_cell(position: Position, cell: CellId) -> bool {
    let left = i32::from(cell.column) * UNITS_PER_CELL;
    let top = i32::from(cell.row) * UNITS_PER_CELL;
    position.x - HALF_FOOTPRINT < left + UNITS_PER_CELL
        && position.x + HALF_FOOTPRINT > left
        && position.y - HALF_FOOTPRINT < top + UNITS_PER_CELL
        && position.y + HALF_FOOTPRINT > top
}

pub(crate) fn allowed_distance(
    world: &World,
    position: Position,
    direction: Direction,
    requested: i32,
    bomb_grace: &BTreeSet<EntityId>,
    access: ObstacleAccess<'_>,
) -> i32 {
    allowed_distance_with_blocker(world, position, direction, requested, bomb_grace, access)
        .distance
}

pub(crate) fn allowed_distance_with_blocker(
    world: &World,
    position: Position,
    direction: Direction,
    requested: i32,
    bomb_grace: &BTreeSet<EntityId>,
    access: ObstacleAccess<'_>,
) -> Collision {
    let mut allowed = requested.max(0);
    let arena_right = i32::from(ARENA_WIDTH) * UNITS_PER_CELL;
    let arena_bottom = i32::from(ARENA_HEIGHT) * UNITS_PER_CELL;
    let boundary = match direction {
        Direction::East => allowed.min(arena_right - HALF_FOOTPRINT - position.x),
        Direction::West => allowed.min(position.x - HALF_FOOTPRINT),
        Direction::South => allowed.min(arena_bottom - HALF_FOOTPRINT - position.y),
        Direction::North => allowed.min(position.y - HALF_FOOTPRINT),
    };
    let mut bomb = None;
    let mut ambiguous = boundary < allowed;
    allowed = allowed.min(boundary);
    for (cell, bomb_id, soft, oil) in blocking_cells(world) {
        if bomb_id.is_some_and(|id| bomb_grace.contains(&id))
            || soft && (access.can_pass || access.soft_grace.contains(&cell))
            || oil && (access.can_pass || access.oil_grace.contains(&cell))
        {
            continue;
        }
        let left = i32::from(cell.column) * UNITS_PER_CELL;
        let top = i32::from(cell.row) * UNITS_PER_CELL;
        let right = left + UNITS_PER_CELL;
        let bottom = top + UNITS_PER_CELL;
        let transverse_overlap = match direction {
            Direction::East | Direction::West => {
                position.y - HALF_FOOTPRINT < bottom && position.y + HALF_FOOTPRINT > top
            }
            Direction::North | Direction::South => {
                position.x - HALF_FOOTPRINT < right && position.x + HALF_FOOTPRINT > left
            }
        };
        if !transverse_overlap {
            continue;
        }
        let candidate = match direction {
            Direction::East if position.x + HALF_FOOTPRINT <= left => {
                left - HALF_FOOTPRINT - position.x
            }
            Direction::West if position.x - HALF_FOOTPRINT >= right => {
                position.x - HALF_FOOTPRINT - right
            }
            Direction::South if position.y + HALF_FOOTPRINT <= top => {
                top - HALF_FOOTPRINT - position.y
            }
            Direction::North if position.y - HALF_FOOTPRINT >= bottom => {
                position.y - HALF_FOOTPRINT - bottom
            }
            _ => continue,
        };
        if candidate < allowed {
            allowed = candidate;
            bomb = bomb_id;
            ambiguous = false;
        } else if candidate == allowed && candidate < requested.max(0) {
            ambiguous = true;
        }
    }
    Collision {
        distance: allowed.max(0),
        bomb,
        ambiguous,
    }
}

fn blocking_cells(
    world: &World,
) -> impl Iterator<Item = (CellId, Option<EntityId>, bool, bool)> + '_ {
    let terrain = (0..ARENA_HEIGHT).flat_map(move |row| {
        (0..ARENA_WIDTH).filter_map(move |column| {
            let cell = CellId::new(column, row);
            if world.arena.terrain(cell) == Some(Terrain::Wall) {
                Some((cell, None, false, false))
            } else if world.arena.blocks().contains(&cell) {
                Some((cell, None, true, false))
            } else {
                None
            }
        })
    });
    terrain
        .chain(world.oil.iter().filter_map(|(cell, state)| {
            (state == crate::game::OilState::Unlit).then_some((cell, None, false, true))
        }))
        .chain(
            world
                .bombs
                .values()
                .filter(|bomb| !matches!(bomb.trigger(), super::BombTrigger::Contact))
                .map(|bomb| (bomb.cell(), Some(bomb.id()), false, false)),
        )
}
