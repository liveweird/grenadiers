pub(crate) use super::geometry::position_overlaps_cell;
use super::geometry::{MovementSegment, MovementTrace};
pub(crate) use super::movement_collision::position_is_free;
use super::movement_collision::{ObstacleAccess, allowed_distance, allowed_distance_with_blocker};
use super::simulation::{Actor, TurnAssist};
use super::types::{CellId, EntityId, Position, TICKS_PER_SECOND, UNITS_PER_CELL};
use super::world::World;
use super::{effects::EffectiveSpeed, input::Direction};
use std::collections::BTreeSet;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MovementOutcome {
    pub(crate) trace: MovementTrace,
    pub(crate) kick: Option<(EntityId, Direction)>,
    pub(crate) original_budget: i32,
    pub(crate) forward_segment: Option<MovementSegment>,
}

#[derive(Default)]
struct MovementCapture {
    kick: Option<(EntityId, Direction)>,
    forward_segment: Option<MovementSegment>,
}

#[derive(Clone, Copy)]
struct MoveRequest {
    direction: Direction,
    distance: i32,
    forward: bool,
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
    let mut capture = MovementCapture::default();
    let Some(raw_direction) = actor.held.active() else {
        actor.residual = 0;
        actor.pending_turn = None;
        return MovementOutcome {
            trace,
            kick: capture.kick,
            original_budget: 0,
            forward_segment: capture.forward_segment,
        };
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
    let slow_stacks = if config.slow_foot_enabled {
        actor.effects().slow_foot_stacks(config.tick)
    } else {
        0
    };
    let storm_active = actor.storm_slow_active(config.tick);
    let speed =
        EffectiveSpeed::from_modifiers(actor.speed, slow_stacks, storm_active).units_per_second();
    let numerator = actor.residual + speed;
    let mut budget = i32::from(numerator / TICKS_PER_SECOND);
    let original_budget = budget;
    actor.residual = numerator % TICKS_PER_SECOND;

    if let Some(turn) = actor.pending_turn {
        continue_turn(
            world,
            actor,
            turn,
            &mut budget,
            &mut trace,
            config,
            &mut capture,
        );
        return MovementOutcome {
            trace,
            kick: capture.kick,
            original_budget,
            forward_segment: capture.forward_segment,
        };
    }
    if same_axis(actor.heading, direction) {
        actor.heading = direction;
        move_actor(
            world,
            actor,
            MoveRequest {
                direction,
                distance: budget,
                forward: true,
            },
            &mut trace,
            config,
            &mut capture,
        );
        return MovementOutcome {
            trace,
            kick: capture.kick,
            original_budget,
            forward_segment: capture.forward_segment,
        };
    }

    let coordinate = alignment_coordinate(direction, actor.position);
    let target = nearest_center(coordinate);
    if (coordinate - target).abs() > config.turn_assist {
        return MovementOutcome {
            trace,
            kick: capture.kick,
            original_budget,
            forward_segment: capture.forward_segment,
        };
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
            ObstacleAccess {
                can_pass: config.wall_pass_enabled && actor.inventory().can_pass_soft_walls(),
                soft_grace: &actor.soft_wall_grace,
                oil_grace: &actor.unlit_oil_grace,
            },
        ) != correction.abs()
        {
            return MovementOutcome {
                trace,
                kick: capture.kick,
                original_budget,
                forward_segment: capture.forward_segment,
            };
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
            &mut capture,
        );
    } else {
        actor.heading = direction;
        move_actor(
            world,
            actor,
            MoveRequest {
                direction,
                distance: budget,
                forward: true,
            },
            &mut trace,
            config,
            &mut capture,
        );
    }
    MovementOutcome {
        trace,
        kick: capture.kick,
        original_budget,
        forward_segment: capture.forward_segment,
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

fn continue_turn(
    world: &World,
    actor: &mut Actor,
    turn: TurnAssist,
    budget: &mut i32,
    trace: &mut MovementTrace,
    config: &AdvanceConfig<'_>,
    capture: &mut MovementCapture,
) {
    let coordinate = alignment_coordinate(turn.direction, actor.position);
    let correction = turn.target - coordinate;
    if correction != 0 {
        let direction = correction_direction(turn.direction, correction);
        let distance = correction.abs().min(*budget);
        let before = actor.position;
        move_actor(
            world,
            actor,
            MoveRequest {
                direction,
                distance,
                forward: false,
            },
            trace,
            config,
            capture,
        );
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
    move_actor(
        world,
        actor,
        MoveRequest {
            direction: turn.direction,
            distance: *budget,
            forward: true,
        },
        trace,
        config,
        capture,
    );
}

fn move_actor(
    world: &World,
    actor: &mut Actor,
    request: MoveRequest,
    trace: &mut MovementTrace,
    config: &AdvanceConfig<'_>,
    capture: &mut MovementCapture,
) {
    let collision = allowed_distance_with_blocker(
        world,
        actor.position,
        request.direction,
        request.distance,
        &actor.bomb_grace,
        ObstacleAccess {
            can_pass: config.wall_pass_enabled && actor.inventory().can_pass_soft_walls(),
            soft_grace: &actor.soft_wall_grace,
            oil_grace: &actor.unlit_oil_grace,
        },
    );
    let distance = collision.distance;
    let kick =
        if request.forward && config.kick_enabled && request.distance > 0 && !collision.ambiguous {
            collision
                .bomb
                .filter(|id| !config.moving_at_start.contains(id))
                .map(|id| (id, request.direction))
        } else {
            None
        };
    if request.forward {
        capture.kick = kick;
    }
    if distance == 0 {
        return;
    }
    let start = actor.position;
    actor.position = translated(start, request.direction, distance);
    let segment = MovementSegment::new(start, actor.position);
    trace.push(segment);
    if request.forward {
        capture.forward_segment = Some(segment);
    }
    prune_bomb_grace(world, actor);
    prune_soft_wall_grace(world, actor);
    prune_unlit_oil_grace(world, actor);
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

pub(crate) fn overlapping_unlit_oil(world: &World, position: Position) -> BTreeSet<CellId> {
    world
        .oil
        .iter()
        .filter_map(|(cell, state)| {
            (state == crate::game::OilState::Unlit && position_overlaps_cell(position, cell))
                .then_some(cell)
        })
        .collect()
}

pub(crate) fn prune_unlit_oil_grace(world: &World, actor: &mut Actor) {
    actor
        .unlit_oil_grace
        .retain(|cell| world.oil.unlit(*cell) && position_overlaps_cell(actor.position, *cell));
}

pub(crate) fn prune_bomb_grace(world: &World, actor: &mut Actor) {
    actor.bomb_grace.retain(|id| {
        world
            .bombs
            .get(id)
            .is_some_and(|bomb| position_overlaps_cell(actor.position, bomb.cell()))
    });
}

pub(crate) fn translated(position: Position, direction: Direction, distance: i32) -> Position {
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
