use std::collections::BTreeSet;

use super::geometry::{MovementSegment, MovementTrace};
use super::input::Direction;
use super::movement_collision::{allowed_distance, allowed_distance_with_blocker};
pub(crate) use super::movement_collision::{position_is_free, position_overlaps_cell};
use super::simulation::{Actor, TurnAssist};
use super::types::{CellId, EntityId, Position, TICKS_PER_SECOND, UNITS_PER_CELL};
use super::world::World;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MovementOutcome {
    pub(crate) trace: MovementTrace,
    pub(crate) kick: Option<(EntityId, Direction)>,
}

pub(crate) struct AdvanceConfig<'a> {
    pub(crate) turn_assist: i32,
    pub(crate) tick: super::types::Tick,
    pub(crate) slow_foot_enabled: bool,
    pub(crate) wall_pass_enabled: bool,
    pub(crate) kick_enabled: bool,
    pub(crate) moving_at_start: &'a BTreeSet<EntityId>,
}

pub(crate) fn advance_actor(
    world: &World,
    actor: &mut Actor,
    config: &AdvanceConfig<'_>,
) -> MovementOutcome {
    let mut trace = MovementTrace::new(actor.position);
    let mut kick = None;
    let Some(raw_direction) = actor.held.active() else {
        actor.residual = 0;
        actor.pending_turn = None;
        return MovementOutcome { trace, kick };
    };
    let direction = if actor.effects().reverses_controls(config.tick) {
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
    // Every supported base rate is divisible by 64, so all six halvings remain exact while the
    // existing remainder continues to represent coordinate-unit sixtieths.
    let speed = if config.slow_foot_enabled {
        let effective = actor.effects().effective_speed(actor.speed, config.tick);
        effective.numerator() / effective.denominator()
    } else {
        actor.speed.units_per_second()
    };
    let numerator = actor.residual + speed;
    let mut budget = i32::from(numerator / TICKS_PER_SECOND);
    actor.residual = numerator % TICKS_PER_SECOND;

    if let Some(turn) = actor.pending_turn {
        continue_turn(
            world,
            actor,
            turn,
            &mut budget,
            &mut trace,
            config,
            &mut kick,
        );
        return MovementOutcome { trace, kick };
    }
    if same_axis(actor.heading, direction) {
        actor.heading = direction;
        kick = move_actor(world, actor, direction, budget, &mut trace, config, true);
        return MovementOutcome { trace, kick };
    }

    let coordinate = alignment_coordinate(direction, actor.position);
    let target = nearest_center(coordinate);
    if (coordinate - target).abs() > config.turn_assist {
        return MovementOutcome { trace, kick };
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
            config.wall_pass_enabled && actor.inventory().can_pass_soft_walls(),
            &actor.soft_wall_grace,
        ) != correction.abs()
        {
            return MovementOutcome { trace, kick };
        }
        let turn = TurnAssist { direction, target };
        actor.pending_turn = Some(turn);
        continue_turn(
            world,
            actor,
            turn,
            &mut budget,
            &mut trace,
            config,
            &mut kick,
        );
    } else {
        actor.heading = direction;
        kick = move_actor(world, actor, direction, budget, &mut trace, config, true);
    }
    MovementOutcome { trace, kick }
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
    config: &AdvanceConfig<'_>,
    kick: &mut Option<(EntityId, Direction)>,
) {
    let coordinate = alignment_coordinate(turn.direction, actor.position);
    let correction = turn.target - coordinate;
    if correction != 0 {
        let direction = correction_direction(turn.direction, correction);
        let distance = correction.abs().min(*budget);
        let before = actor.position;
        let _ = move_actor(world, actor, direction, distance, trace, config, false);
        let moved = match direction {
            Direction::East | Direction::West => (actor.position.x - before.x).abs(),
            Direction::North | Direction::South => (actor.position.y - before.y).abs(),
        };
        *budget -= moved;
        if moved != distance || alignment_coordinate(turn.direction, actor.position) != turn.target
        {
            return;
        }
    }
    actor.pending_turn = None;
    actor.heading = turn.direction;
    *kick = move_actor(world, actor, turn.direction, *budget, trace, config, true);
}

fn move_actor(
    world: &World,
    actor: &mut Actor,
    direction: Direction,
    requested: i32,
    trace: &mut MovementTrace,
    config: &AdvanceConfig<'_>,
    forward: bool,
) -> Option<(EntityId, Direction)> {
    let collision = allowed_distance_with_blocker(
        world,
        actor.position,
        direction,
        requested,
        &actor.bomb_grace,
        config.wall_pass_enabled && actor.inventory().can_pass_soft_walls(),
        &actor.soft_wall_grace,
    );
    let distance = collision.distance;
    let kick = if forward && config.kick_enabled && requested > 0 && !collision.ambiguous {
        collision
            .bomb
            .filter(|id| !config.moving_at_start.contains(id))
            .map(|id| (id, direction))
    } else {
        None
    };
    if distance == 0 {
        return kick;
    }
    let start = actor.position;
    actor.position = translated(start, direction, distance);
    trace.push(MovementSegment::new(start, actor.position));
    prune_bomb_grace(world, actor);
    prune_soft_wall_grace(world, actor);
    kick
}

pub(crate) fn overlapping_soft_blocks(world: &World, position: Position) -> BTreeSet<CellId> {
    world
        .arena
        .blocks()
        .iter()
        .filter(|cell| position_overlaps_cell(position, **cell))
        .copied()
        .collect()
}

pub(crate) fn prune_soft_wall_grace(world: &World, actor: &mut Actor) {
    actor.soft_wall_grace.retain(|cell| {
        world.arena.blocks().contains(cell) && position_overlaps_cell(actor.position, *cell)
    });
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
