use std::collections::BTreeSet;

use super::arena::{ARENA_HEIGHT, ARENA_WIDTH, Terrain};
use super::geometry::{MovementSegment, MovementTrace};
use super::input::Direction;
use super::simulation::{Actor, TurnAssist};
use super::types::{CellId, EntityId, HALF_FOOTPRINT, Position, TICKS_PER_SECOND, UNITS_PER_CELL};
use super::world::World;

pub(crate) fn advance_actor(
    world: &World,
    actor: &mut Actor,
    turn_assist: i32,
    tick: super::types::Tick,
) -> MovementTrace {
    let mut trace = MovementTrace::new(actor.position);
    let Some(raw_direction) = actor.held.active() else {
        actor.residual = 0;
        actor.pending_turn = None;
        return trace;
    };
    let direction = if actor.effects().reverses_controls(tick) {
        opposite(raw_direction)
    } else {
        raw_direction
    };

    if actor
        .pending_turn
        .is_some_and(|turn| turn.direction != direction)
    {
        actor.pending_turn = None;
    }
    let numerator = actor.residual + actor.speed.units_per_second();
    let mut budget = i32::from(numerator / TICKS_PER_SECOND);
    actor.residual = numerator % TICKS_PER_SECOND;

    if let Some(turn) = actor.pending_turn {
        continue_turn(world, actor, turn, &mut budget, &mut trace);
        return trace;
    }
    if same_axis(actor.heading, direction) {
        actor.heading = direction;
        move_actor(world, actor, direction, budget, &mut trace);
        return trace;
    }

    let coordinate = alignment_coordinate(direction, actor.position);
    let target = nearest_center(coordinate);
    if (coordinate - target).abs() > turn_assist {
        return trace;
    }
    let correction = target - coordinate;
    if correction != 0 {
        let correction_direction = correction_direction(direction, correction);
        if allowed_distance(
            world,
            actor.position,
            correction_direction,
            correction.abs(),
            &actor.bomb_grace,
        ) != correction.abs()
        {
            return trace;
        }
        let turn = TurnAssist { direction, target };
        actor.pending_turn = Some(turn);
        continue_turn(world, actor, turn, &mut budget, &mut trace);
    } else {
        actor.heading = direction;
        move_actor(world, actor, direction, budget, &mut trace);
    }
    trace
}

const fn opposite(direction: Direction) -> Direction {
    match direction {
        Direction::North => Direction::South,
        Direction::East => Direction::West,
        Direction::South => Direction::North,
        Direction::West => Direction::East,
    }
}

fn continue_turn(
    world: &World,
    actor: &mut Actor,
    turn: TurnAssist,
    budget: &mut i32,
    trace: &mut MovementTrace,
) {
    let coordinate = alignment_coordinate(turn.direction, actor.position);
    let correction = turn.target - coordinate;
    if correction != 0 {
        let direction = correction_direction(turn.direction, correction);
        let distance = correction.abs().min(*budget);
        let moved = move_actor(world, actor, direction, distance, trace);
        *budget -= moved;
        if moved != distance || alignment_coordinate(turn.direction, actor.position) != turn.target
        {
            return;
        }
    }
    actor.pending_turn = None;
    actor.heading = turn.direction;
    move_actor(world, actor, turn.direction, *budget, trace);
}

fn move_actor(
    world: &World,
    actor: &mut Actor,
    direction: Direction,
    requested: i32,
    trace: &mut MovementTrace,
) -> i32 {
    let distance = allowed_distance(
        world,
        actor.position,
        direction,
        requested,
        &actor.bomb_grace,
    );
    if distance == 0 {
        return 0;
    }
    let start = actor.position;
    actor.position = translated(start, direction, distance);
    trace.push(MovementSegment::new(start, actor.position));
    prune_bomb_grace(world, actor);
    distance
}

pub(crate) fn position_is_free(
    world: &World,
    position: Position,
    bomb_grace: &BTreeSet<EntityId>,
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
    blocking_cells(world).all(|(cell, bomb_id)| {
        !position_overlaps_cell(position, cell)
            || bomb_id.is_some_and(|id| bomb_grace.contains(&id))
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

fn allowed_distance(
    world: &World,
    position: Position,
    direction: Direction,
    requested: i32,
    bomb_grace: &BTreeSet<EntityId>,
) -> i32 {
    let mut allowed = requested.max(0);
    let arena_right = i32::from(ARENA_WIDTH) * UNITS_PER_CELL;
    let arena_bottom = i32::from(ARENA_HEIGHT) * UNITS_PER_CELL;
    allowed = match direction {
        Direction::East => allowed.min(arena_right - HALF_FOOTPRINT - position.x),
        Direction::West => allowed.min(position.x - HALF_FOOTPRINT),
        Direction::South => allowed.min(arena_bottom - HALF_FOOTPRINT - position.y),
        Direction::North => allowed.min(position.y - HALF_FOOTPRINT),
    };
    for (cell, bomb_id) in blocking_cells(world) {
        if bomb_id.is_some_and(|id| bomb_grace.contains(&id)) {
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
        allowed = allowed.min(candidate);
    }
    allowed.max(0)
}

fn blocking_cells(world: &World) -> impl Iterator<Item = (CellId, Option<EntityId>)> + '_ {
    let terrain = (0..ARENA_HEIGHT).flat_map(move |row| {
        (0..ARENA_WIDTH).filter_map(move |column| {
            let cell = CellId::new(column, row);
            (world.arena.terrain(cell) == Some(Terrain::Wall)
                || world.arena.blocks().contains(&cell))
            .then_some((cell, None))
        })
    });
    terrain.chain(
        world
            .bombs
            .values()
            .map(|bomb| (bomb.cell(), Some(bomb.id()))),
    )
}

fn prune_bomb_grace(world: &World, actor: &mut Actor) {
    actor.bomb_grace.retain(|id| {
        world
            .bombs
            .get(id)
            .is_some_and(|bomb| position_overlaps_cell(actor.position, bomb.cell()))
    });
}

fn translated(position: Position, direction: Direction, distance: i32) -> Position {
    match direction {
        Direction::North => Position {
            y: position.y - distance,
            ..position
        },
        Direction::East => Position {
            x: position.x + distance,
            ..position
        },
        Direction::South => Position {
            y: position.y + distance,
            ..position
        },
        Direction::West => Position {
            x: position.x - distance,
            ..position
        },
    }
}

fn same_axis(left: Direction, right: Direction) -> bool {
    matches!(
        (left, right),
        (
            Direction::North | Direction::South,
            Direction::North | Direction::South
        ) | (
            Direction::East | Direction::West,
            Direction::East | Direction::West
        )
    )
}

fn alignment_coordinate(direction: Direction, position: Position) -> i32 {
    match direction {
        Direction::East | Direction::West => position.y,
        Direction::North | Direction::South => position.x,
    }
}

fn nearest_center(coordinate: i32) -> i32 {
    coordinate.div_euclid(UNITS_PER_CELL) * UNITS_PER_CELL + UNITS_PER_CELL / 2
}

fn correction_direction(requested: Direction, correction: i32) -> Direction {
    match (requested, correction.is_positive()) {
        (Direction::East | Direction::West, true) => Direction::South,
        (Direction::East | Direction::West, false) => Direction::North,
        (Direction::North | Direction::South, true) => Direction::East,
        (Direction::North | Direction::South, false) => Direction::West,
    }
}

#[cfg(test)]
#[path = "movement_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "forgiving_turn_tests.rs"]
mod forgiving_turn_tests;
