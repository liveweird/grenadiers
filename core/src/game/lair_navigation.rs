use std::collections::{BTreeSet, VecDeque};

use super::Mob;
use crate::game::geometry::{MovementSegment, MovementTrace};
use crate::game::input::Direction;
use crate::game::simulation::Actor;
use crate::game::types::{CellId, Position, TICKS_PER_SECOND, Tick};
use crate::game::world::World;

type Distances = Vec<Option<u16>>;

pub(super) fn target_searches(
    world: &World,
    actors: &[Actor],
    target_order: &[usize],
    tick: Tick,
    danger: &BTreeSet<CellId>,
) -> Vec<(usize, Distances)> {
    target_order
        .iter()
        .enumerate()
        .filter_map(|(rank, &index)| {
            let actor = &actors[index];
            (actor.alive() && !actor.effects().hidden_from_opponents(tick)).then_some((rank, actor))
        })
        .filter_map(|(rank, actor)| {
            actor.position().cell().and_then(|cell| {
                super::traversable(cell, world, danger, tick)
                    .then(|| (rank, bfs(world, cell, danger, tick)))
            })
        })
        .collect()
}

fn bfs(world: &World, start: CellId, danger: &BTreeSet<CellId>, tick: Tick) -> Distances {
    let mut distances = vec![None; world.arena.cell_count()];
    let mut pending = VecDeque::from([start]);
    distances[cell_index(world, start)] = Some(0);
    while let Some(cell) = pending.pop_front() {
        let distance = distances[cell_index(world, cell)].expect("queued cells are visited");
        for direction in 0..4 {
            let next = super::adjacent(cell, direction);
            let index = cell_index(world, next);
            if distances[index].is_none() && super::traversable(next, world, danger, tick) {
                distances[index] = Some(distance + 1);
                pending.push_back(next);
            }
        }
    }
    distances
}

fn cell_index(world: &World, cell: CellId) -> usize {
    usize::from(cell.row) * usize::from(world.arena.width()) + usize::from(cell.column)
}

pub(super) fn advance_mob(
    mob: &mut Mob,
    world: &World,
    danger: &BTreeSet<CellId>,
    searches: &[(usize, Distances)],
    frozen: bool,
    tick: Tick,
) -> MovementTrace {
    let mut trace = MovementTrace::new(mob.position);
    if frozen {
        mob.residual = 0;
        return trace;
    }
    let Some(cell) = mob.position.cell() else {
        return trace;
    };
    let center = Position::center(cell);
    if mob.position == center {
        mob.escaping = false;
    }
    let direction = choose_direction(mob, world, danger, searches, tick, cell, center);
    let Some(direction) = direction else {
        mob.residual = 0;
        return trace;
    };
    let direction = directions()[usize::from(direction)];
    let numerator = mob.residual + super::MOB_UNITS_PER_SECOND;
    let mut distance = i32::from(numerator / TICKS_PER_SECOND);
    mob.residual = numerator % TICKS_PER_SECOND;
    if mob.escaping {
        distance = distance.min(distance_to_center(mob.position, center, direction));
    }
    let start = mob.position;
    match direction {
        Direction::North => mob.position.y -= distance,
        Direction::East => mob.position.x += distance,
        Direction::South => mob.position.y += distance,
        Direction::West => mob.position.x -= distance,
    }
    mob.heading = direction;
    trace.push(MovementSegment::new(start, mob.position));
    trace
}

fn choose_direction(
    mob: &mut Mob,
    world: &World,
    danger: &BTreeSet<CellId>,
    searches: &[(usize, Distances)],
    tick: Tick,
    cell: CellId,
    center: Position,
) -> Option<u8> {
    if mob.position != center && mob.escaping {
        return Some(direction_index(mob.heading));
    }
    if mob.position != center && danger.contains(&cell) {
        mob.escaping = true;
        return Some(direction_index(opposite(mob.heading)));
    }
    if mob.position != center {
        return Some(direction_index(mob.heading));
    }
    if danger.contains(&cell) {
        return (0..4)
            .find(|&dir| super::traversable(super::adjacent(cell, dir), world, danger, tick));
    }
    searches
        .iter()
        .filter_map(|(actor, distances)| {
            distances[cell_index(world, cell)].map(|distance| (distance, *actor, distances))
        })
        .min_by_key(|(distance, actor, _)| (*distance, *actor))
        .and_then(|(_, _, distances)| {
            let current = distances[cell_index(world, cell)]?;
            (0..4).find(|&dir| {
                let next = super::adjacent(cell, dir);
                distances[cell_index(world, next)] == current.checked_sub(1)
            })
        })
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

fn distance_to_center(position: Position, center: Position, direction: Direction) -> i32 {
    match direction {
        Direction::North => position.y.saturating_sub(center.y),
        Direction::East => center.x.saturating_sub(position.x),
        Direction::South => center.y.saturating_sub(position.y),
        Direction::West => position.x.saturating_sub(center.x),
    }
    .max(0)
}

pub(crate) fn traces_contact_actor(traces: &[MovementTrace], actor: Position) -> bool {
    traces.iter().any(|trace| {
        trace
            .segments()
            .any(|segment| crate::game::stationary_contact_entry(segment, actor))
    })
}

pub(crate) fn actor_trace_contacts_mobs(trace: &MovementTrace, mobs: &[Mob]) -> bool {
    mobs.iter().any(|mob| {
        trace
            .segments()
            .any(|segment| crate::game::stationary_contact_entry(segment, mob.position))
    })
}

pub(crate) fn actor_position_contacts_mobs(position: Position, mobs: &[Mob]) -> bool {
    mobs.iter().any(|mob| {
        (i64::from(position.x) - i64::from(mob.position.x)).abs()
            < i64::from(crate::game::types::HALF_FOOTPRINT) * 2
            && (i64::from(position.y) - i64::from(mob.position.y)).abs()
                < i64::from(crate::game::types::HALF_FOOTPRINT) * 2
    })
}
