use std::collections::BTreeSet;

use super::Mob;
use crate::game::geometry::{distance_to_blocking_cell, position_overlaps_cell};
use crate::game::input::Direction;
use crate::game::types::{CellId, Position, Tick};
use crate::game::world::World;

pub(super) fn overlaps(position: Position, pits: &BTreeSet<CellId>) -> bool {
    pits.iter()
        .any(|&pit| position_overlaps_cell(position, pit))
}

pub(super) fn traversable(
    cell: CellId,
    world: &World,
    danger: &BTreeSet<CellId>,
    pits: Option<&BTreeSet<CellId>>,
    tick: Tick,
) -> bool {
    !pits.is_some_and(|pits| pits.contains(&cell)) && super::traversable(cell, world, danger, tick)
}

/// Handles the exceptional part of pit navigation before ordinary target routing.
///
/// A mob born on a pit keeps moving outward until its whole footprint is clear.
/// A mob that approached from safe floor retreats to its known-safe leg origin.
pub(super) fn escape_direction(
    mob: &mut Mob,
    world: &World,
    danger: &BTreeSet<CellId>,
    pits: &BTreeSet<CellId>,
    tick: Tick,
) -> Option<Option<u8>> {
    if overlaps(mob.position, pits) {
        let origin_is_dangerous = overlaps(mob.leg_origin, pits)
            || mob
                .leg_origin
                .cell()
                .is_none_or(|cell| danger.contains(&cell));
        if origin_is_dangerous {
            let direction = outward_direction(mob, world, danger, pits, tick)?;
            mob.heading = direction;
            mob.leg_origin = mob.position;
            mob.retreat_anchor = None;
            return Some(Some(direction_index(direction)));
        }
        if mob.retreat_anchor == Some(mob.leg_origin) {
            return Some(Some(direction_index(mob.heading)));
        }
        mob.retreat_anchor = Some(mob.leg_origin);
        let direction = opposite(mob.heading);
        mob.heading = direction;
        return Some(Some(direction_index(direction)));
    }

    if mob.position != mob.leg_origin
        && destination_cell(mob).is_some_and(|cell| pits.contains(&cell))
    {
        mob.retreat_anchor = Some(mob.leg_origin);
        let direction = opposite(mob.heading);
        mob.heading = direction;
        return Some(Some(direction_index(direction)));
    }
    None
}

fn outward_direction(
    mob: &Mob,
    world: &World,
    danger: &BTreeSet<CellId>,
    pits: &BTreeSet<CellId>,
    tick: Tick,
) -> Option<Direction> {
    let cell = mob.position.cell()?;
    let preferred = direction_index(mob.heading);
    (0..4)
        .map(|offset| (preferred + offset) % 4)
        .find(|&direction| {
            let next = super::adjacent(cell, direction);
            traversable(next, world, danger, Some(pits), tick)
        })
        .map(|direction| directions()[usize::from(direction)])
}

fn destination_cell(mob: &Mob) -> Option<CellId> {
    let origin = mob.leg_origin.cell()?;
    Some(super::adjacent(origin, direction_index(mob.heading)))
}

pub(super) fn allowed_distance(
    position: Position,
    direction: Direction,
    requested: i32,
    pits: Option<&BTreeSet<CellId>>,
) -> i32 {
    let Some(pits) = pits else { return requested };
    if overlaps(position, pits) {
        return requested;
    }
    pits.iter()
        .filter_map(|&pit| distance_to_blocking_cell(position, direction, pit))
        .fold(requested, i32::min)
}

const fn directions() -> [Direction; 4] {
    [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ]
}

const fn direction_index(direction: Direction) -> u8 {
    match direction {
        Direction::North => 0,
        Direction::East => 1,
        Direction::South => 2,
        Direction::West => 3,
    }
}

const fn opposite(direction: Direction) -> Direction {
    match direction {
        Direction::North => Direction::South,
        Direction::East => Direction::West,
        Direction::South => Direction::North,
        Direction::West => Direction::East,
    }
}
