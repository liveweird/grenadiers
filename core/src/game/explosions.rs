use std::collections::{BTreeMap, BTreeSet};

use super::arena::Terrain;
use super::bombs::{BlastKind, Bomb, BombTrigger};
use super::input::Direction;
use super::types::{CellId, EntityId, Tick};
use super::world::World;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ExplosionPlan {
    pub(crate) detonated: BTreeSet<EntityId>,
    pub(crate) flames: BTreeSet<CellId>,
    pub(crate) destroyed_blocks: BTreeSet<CellId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExplosionError {
    UnsupportedBombTrigger(EntityId),
}

pub(crate) fn plan(world: &World, tick: Tick) -> Result<ExplosionPlan, ExplosionError> {
    for bomb in world.bombs.values() {
        if !matches!(bomb.trigger(), BombTrigger::Timed { .. }) {
            return Err(ExplosionError::UnsupportedBombTrigger(bomb.id()));
        }
    }
    let bombs_by_cell: BTreeMap<_, _> = world
        .bombs
        .values()
        .map(|bomb| (bomb.cell(), bomb.id()))
        .collect();
    let mut pending: BTreeSet<_> = world
        .bombs
        .values()
        .filter(|bomb| match bomb.trigger() {
            BombTrigger::Timed { detonates_at } => {
                detonates_at <= tick
                    || world
                        .hazards
                        .get(&bomb.cell())
                        .is_some_and(|expiry| *expiry > tick)
            }
            BombTrigger::Remote | BombTrigger::Contact => false,
        })
        .map(|bomb| bomb.id())
        .collect();
    let mut result = ExplosionPlan::default();

    while let Some(id) = pending.pop_first() {
        if !result.detonated.insert(id) {
            continue;
        }
        let Some(bomb) = world.bombs.get(&id) else {
            continue;
        };
        result.flames.insert(bomb.cell());
        for direction in [
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
        ] {
            trace_ray(
                world,
                &bombs_by_cell,
                *bomb,
                direction,
                &mut pending,
                &mut result,
            );
        }
    }
    Ok(result)
}

fn trace_ray(
    world: &World,
    bombs_by_cell: &BTreeMap<CellId, EntityId>,
    bomb: Bomb,
    direction: Direction,
    pending: &mut BTreeSet<EntityId>,
    result: &mut ExplosionPlan,
) {
    let mut cell = bomb.cell();
    for _ in 0..bomb.range() {
        let Some(next) = adjacent(cell, direction) else {
            break;
        };
        cell = next;
        if world.arena.terrain(cell) != Some(Terrain::Floor) {
            break;
        }
        result.flames.insert(cell);
        if world.arena.blocks().contains(&cell) {
            result.destroyed_blocks.insert(cell);
            if bomb.blast() == BlastKind::Standard {
                break;
            }
        }
        if let Some(id) = bombs_by_cell.get(&cell) {
            pending.insert(*id);
            break;
        }
    }
}

fn adjacent(cell: CellId, direction: Direction) -> Option<CellId> {
    match direction {
        Direction::North => cell
            .row
            .checked_sub(1)
            .map(|row| CellId::new(cell.column, row)),
        Direction::East => cell
            .column
            .checked_add(1)
            .map(|column| CellId::new(column, cell.row)),
        Direction::South => cell
            .row
            .checked_add(1)
            .map(|row| CellId::new(cell.column, row)),
        Direction::West => cell
            .column
            .checked_sub(1)
            .map(|column| CellId::new(column, cell.row)),
    }
}

#[cfg(test)]
#[path = "explosions_tests.rs"]
mod tests;
