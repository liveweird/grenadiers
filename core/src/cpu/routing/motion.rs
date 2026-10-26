use crate::cpu::MovementIntent;
use crate::game::arena::{ARENA_HEIGHT, ARENA_WIDTH};
use crate::game::effects::EffectiveSpeed;
use crate::game::geometry::{MovementSegment, boundary_distance, distance_to_blocking_cell};
use crate::game::{Direction, Position, Tick};

use super::RouteFailure;
use super::pulse_geometry::{
    alignment_coordinate, correction_direction, nearest_center, opposite, same_axis, translated,
};
use super::safety::footprint_cells;
use super::validation::Context;

#[derive(Clone)]
pub(super) struct Motion {
    pub(super) position: Position,
    pub(super) heading: Direction,
    residual: u16,
    pending: Option<(Direction, i32)>,
    reversed: bool,
}

impl Motion {
    pub(super) fn stopped(context: &Context<'_>, tick: Tick) -> Self {
        Self {
            position: context.actor.position,
            heading: context.actor.heading,
            residual: 0,
            pending: None,
            reversed: reverse_active(context, tick),
        }
    }

    pub(super) fn stop(&mut self) {
        self.residual = 0;
        self.pending = None;
    }
}

pub(super) fn advance(
    context: &Context<'_>,
    motion: &mut Motion,
    movement: MovementIntent,
    tick: Tick,
) -> Result<Vec<MovementSegment>, RouteFailure> {
    let MovementIntent::Direction(input) = movement else {
        motion.stop();
        return Ok(vec![MovementSegment::new(motion.position, motion.position)]);
    };
    let reversed = reverse_active(context, tick);
    if reversed != motion.reversed {
        motion.pending = None;
        motion.reversed = reversed;
    }
    let desired = if reversed { opposite(input) } else { input };
    if motion.pending.is_some_and(|pending| pending.0 != desired) {
        motion.pending = None;
    }
    let stacks = context
        .slow_expiries
        .iter()
        .filter(|expiry| **expiry > tick)
        .count();
    let (mut budget, residual) =
        EffectiveSpeed::from_slow_foot_stacks(context.actor.speed, stacks).advance(motion.residual);
    motion.residual = residual;
    let mut segments = Vec::with_capacity(2);
    if same_axis(motion.heading, desired) {
        motion.heading = desired;
        move_axis(context, motion, desired, budget, &mut segments);
        return Ok(stationary_if_empty(motion, segments));
    }
    let target = motion.pending.map_or_else(
        || nearest_center(alignment_coordinate(desired, motion.position)),
        |pending| pending.1,
    );
    let correction = target - alignment_coordinate(desired, motion.position);
    if correction.unsigned_abs()
        > context
            .observation
            .parts()
            .rules
            .settings
            .turn_assist_units() as u32
    {
        return Ok(stationary_if_empty(motion, segments));
    }
    if correction != 0 {
        let correction_direction = correction_direction(desired, correction);
        let requested = budget.min(correction.abs());
        let moved = move_axis(
            context,
            motion,
            correction_direction,
            requested,
            &mut segments,
        );
        budget -= moved;
        if moved != requested || alignment_coordinate(desired, motion.position) != target {
            motion.pending = Some((desired, target));
            return Ok(stationary_if_empty(motion, segments));
        }
    }
    motion.pending = None;
    motion.heading = desired;
    move_axis(context, motion, desired, budget, &mut segments);
    Ok(stationary_if_empty(motion, segments))
}

fn move_axis(
    context: &Context<'_>,
    motion: &mut Motion,
    direction: Direction,
    requested: i32,
    segments: &mut Vec<MovementSegment>,
) -> i32 {
    let allowed = clipped_static(context, motion.position, direction, requested);
    if allowed == 0 {
        return 0;
    }
    let start = motion.position;
    motion.position = translated(start, direction, allowed);
    segments.push(MovementSegment::new(start, motion.position));
    allowed
}

fn clipped_static(
    context: &Context<'_>,
    position: Position,
    direction: Direction,
    requested: i32,
) -> i32 {
    let mut allowed = boundary_distance(position, direction, requested, ARENA_WIDTH, ARENA_HEIGHT);
    let end = translated(position, direction, allowed);
    for cell in footprint_cells(&[MovementSegment::new(position, end)]) {
        let index = usize::from(cell.row) * usize::from(ARENA_WIDTH) + usize::from(cell.column);
        if context.observation.parts().arena.terrain[index] == crate::game::Terrain::Wall
            && let Some(distance) = distance_to_blocking_cell(position, direction, cell)
        {
            allowed = allowed.min(distance);
        }
    }
    allowed.max(0)
}

fn stationary_if_empty(
    motion: &Motion,
    mut segments: Vec<MovementSegment>,
) -> Vec<MovementSegment> {
    if segments.is_empty() {
        segments.push(MovementSegment::new(motion.position, motion.position));
    }
    segments
}

fn reverse_active(context: &Context<'_>, tick: Tick) -> bool {
    context.reverse_expiry.is_some_and(|expiry| tick < expiry)
}
