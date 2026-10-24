use std::collections::{BTreeMap, BTreeSet};

use super::arena::Terrain;
use super::bombs::{BlastKind, Bomb, BombTrigger};
use super::input::Direction;
use super::types::{CellId, EntityId, Tick};
use super::world::World;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ExplosionPlan {
    pub(crate) detonated: BTreeSet<EntityId>,
    pub(crate) destroyed_mines: BTreeSet<EntityId>,
    pub(crate) flames: BTreeSet<CellId>,
    pub(crate) destroyed_blocks: BTreeSet<CellId>,
    pub(crate) ignited_oil: BTreeSet<CellId>,
}

#[cfg(test)]
pub(crate) fn plan(
    world: &World,
    tick: Tick,
    remote_targets: [Option<EntityId>; 2],
) -> ExplosionPlan {
    plan_with_contacts(world, tick, remote_targets, &BTreeSet::new())
}

pub(crate) fn plan_with_contacts(
    world: &World,
    tick: Tick,
    remote_targets: [Option<EntityId>; 2],
    contact_targets: &BTreeSet<EntityId>,
) -> ExplosionPlan {
    let bombs_by_cell: BTreeMap<_, _> = world
        .bombs
        .values()
        .map(|bomb| (bomb.cell(), bomb.id()))
        .collect();
    let mut pending: BTreeSet<_> = world
        .bombs
        .values()
        .filter(|bomb| {
            let burning = world.has_fire(bomb.cell(), tick);
            (match bomb.trigger() {
                BombTrigger::Contact => contact_targets.contains(&bomb.id()),
                BombTrigger::Timed { detonates_at } => detonates_at <= tick,
                BombTrigger::Remote => remote_targets.contains(&Some(bomb.id())) || burning,
            }) || burning && !matches!(bomb.trigger(), BombTrigger::Contact)
        })
        .map(|bomb| bomb.id())
        .collect();
    let mut result = ExplosionPlan::default();
    result
        .ignited_oil
        .extend(world.oil.iter().filter_map(|(cell, state)| {
            (state == crate::game::OilState::Unlit && world.has_fire(cell, tick)).then_some(cell)
        }));
    result
        .destroyed_mines
        .extend(world.bombs.values().filter_map(|bomb| {
            (matches!(bomb.trigger(), BombTrigger::Contact)
                && !contact_targets.contains(&bomb.id())
                && world.has_fire(bomb.cell(), tick))
            .then_some(bomb.id())
        }));

    while let Some(id) = pending.pop_first() {
        if !result.detonated.insert(id) {
            continue;
        }
        let Some(bomb) = world.bombs.get(&id) else {
            continue;
        };
        if bomb.blast() == BlastKind::Power {
            plan_power_blast(world, *bomb, contact_targets, &mut pending, &mut result);
            continue;
        }
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
                contact_targets,
                &mut pending,
                &mut result,
            );
        }
    }
    result
}

fn plan_power_blast(
    world: &World,
    bomb: Bomb,
    contact_targets: &BTreeSet<EntityId>,
    pending: &mut BTreeSet<EntityId>,
    result: &mut ExplosionPlan,
) {
    for row in 0..super::arena::ARENA_HEIGHT {
        for column in 0..super::arena::ARENA_WIDTH {
            let cell = CellId::new(column, row);
            let distance = u32::from(column.abs_diff(bomb.cell().column))
                + u32::from(row.abs_diff(bomb.cell().row));
            if distance > u32::from(bomb.range())
                || world.arena.terrain(cell) != Some(Terrain::Floor)
            {
                continue;
            }
            result.flames.insert(cell);
            if world.oil.unlit(cell) {
                result.ignited_oil.insert(cell);
            }
            if world.arena.blocks().contains(&cell) {
                result.destroyed_blocks.insert(cell);
            }
        }
    }
    for other in world.bombs.values().filter(|other| {
        let cell = other.cell();
        let distance = u32::from(cell.column.abs_diff(bomb.cell().column))
            + u32::from(cell.row.abs_diff(bomb.cell().row));
        distance <= u32::from(bomb.range()) && world.arena.terrain(cell) == Some(Terrain::Floor)
    }) {
        if matches!(other.trigger(), BombTrigger::Contact) {
            if contact_targets.contains(&other.id()) {
                pending.insert(other.id());
            } else {
                result.destroyed_mines.insert(other.id());
            }
        } else {
            pending.insert(other.id());
        }
    }
}

fn trace_ray(
    world: &World,
    bombs_by_cell: &BTreeMap<CellId, EntityId>,
    bomb: Bomb,
    direction: Direction,
    contact_targets: &BTreeSet<EntityId>,
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
        if world.oil.unlit(cell) {
            result.ignited_oil.insert(cell);
            if bomb.blast() == BlastKind::Standard {
                break;
            }
        }
        if world.arena.blocks().contains(&cell) {
            result.destroyed_blocks.insert(cell);
            if bomb.blast() == BlastKind::Standard {
                break;
            }
        }
        if let Some(id) = bombs_by_cell.get(&cell) {
            let other = world
                .bombs
                .get(id)
                .expect("bomb-cell index references existing bombs");
            if matches!(other.trigger(), BombTrigger::Contact) {
                if contact_targets.contains(id) {
                    pending.insert(*id);
                } else {
                    result.destroyed_mines.insert(*id);
                }
            } else {
                pending.insert(*id);
                break;
            }
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

#[cfg(test)]
#[path = "remote_explosion_tests.rs"]
mod remote_tests;
