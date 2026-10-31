use crate::cpu::MovementIntent;
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
    storm_deadlines: Option<super::storm::DeadlineBounds>,
}

impl Motion {
    pub(super) fn stopped(context: &Context<'_>, tick: Tick) -> Self {
        Self {
            position: context.actor.position,
            heading: context.actor.heading,
            residual: 0,
            pending: None,
            reversed: reverse_active(context, tick),
            storm_deadlines: context
                .storm
                .map(super::storm::StormContext::initial_deadline),
        }
    }

    pub(super) fn stop(&mut self) {
        self.residual = 0;
        self.pending = None;
    }

    pub(super) const fn storm_deadlines(&self) -> Option<super::storm::DeadlineBounds> {
        self.storm_deadlines
    }
}

pub(super) fn advance(
    context: &Context<'_>,
    motion: &mut Motion,
    movement: MovementIntent,
    tick: Tick,
) -> Result<Vec<MovementSegment>, RouteFailure> {
    let actor_start = motion.position;
    let MovementIntent::Direction(input) = movement else {
        motion.stop();
        let segments = vec![MovementSegment::new(motion.position, motion.position)];
        apply_storm(context, motion, tick, actor_start, &segments)?;
        return Ok(segments);
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
    let storm_active = match motion.storm_deadlines {
        Some(bounds) if tick < bounds.lower => true,
        Some(bounds) if tick >= bounds.upper => false,
        Some(_) => return Err(RouteFailure::NoRoute),
        None => false,
    };
    let (mut budget, residual) =
        EffectiveSpeed::from_modifiers(context.actor.speed, stacks, storm_active)
            .advance(motion.residual);
    motion.residual = residual;
    let mut segments = Vec::with_capacity(2);
    if same_axis(motion.heading, desired) {
        motion.heading = desired;
        move_axis(context, motion, desired, budget, &mut segments);
        let segments = stationary_if_empty(motion, segments);
        apply_storm(context, motion, tick, actor_start, &segments)?;
        return Ok(segments);
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
        let segments = stationary_if_empty(motion, segments);
        apply_storm(context, motion, tick, actor_start, &segments)?;
        return Ok(segments);
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
            let segments = stationary_if_empty(motion, segments);
            apply_storm(context, motion, tick, actor_start, &segments)?;
            return Ok(segments);
        }
    }
    motion.pending = None;
    motion.heading = desired;
    move_axis(context, motion, desired, budget, &mut segments);
    let segments = stationary_if_empty(motion, segments);
    apply_storm(context, motion, tick, actor_start, &segments)?;
    Ok(segments)
}

fn apply_storm(
    context: &Context<'_>,
    motion: &mut Motion,
    tick: Tick,
    actor_start: Position,
    segments: &[MovementSegment],
) -> Result<(), RouteFailure> {
    if let (Some(storm), Some(deadlines)) = (context.storm, motion.storm_deadlines.as_mut()) {
        storm.apply_tick(tick, actor_start, segments, deadlines)?;
    }
    Ok(())
}

fn move_axis(
    context: &Context<'_>,
    motion: &mut Motion,
    direction: Direction,
    requested: i32,
    segments: &mut Vec<MovementSegment>,
) -> i32 {
    let allowed = clipped_players(
        context,
        direction,
        clipped_static(context, motion.position, direction, requested),
        motion.position,
    );
    if allowed == 0 {
        return 0;
    }
    let start = motion.position;
    motion.position = translated(start, direction, allowed);
    segments.push(MovementSegment::new(start, motion.position));
    allowed
}

fn clipped_players(
    context: &Context<'_>,
    direction: Direction,
    requested: i32,
    start: Position,
) -> i32 {
    if !matches!(context.observation.parts().rules.version, 9 | 10) {
        return requested;
    }
    let obstacles = context
        .observation
        .parts()
        .actors
        .iter()
        .filter_map(|actor| match actor {
            crate::cpu::ObservedActor::Visible(actor)
                if actor.alive && actor.seat != context.observation.parts().recipient =>
            {
                Some(crate::game::player_collision::PlayerObstacle::stationary(
                    actor.seat,
                    actor.position,
                ))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    crate::game::player_collision::clip_player_segment(
        context.observation.parts().recipient,
        start,
        direction,
        requested,
        &obstacles,
    )
    .distance
}

fn clipped_static(
    context: &Context<'_>,
    position: Position,
    direction: Direction,
    requested: i32,
) -> i32 {
    let arena = &context.observation.parts().arena;
    let mut allowed = boundary_distance(position, direction, requested, arena.width, arena.height);
    let end = translated(position, direction, allowed);
    for cell in footprint_cells(
        &[MovementSegment::new(position, end)],
        (arena.width, arena.height),
    ) {
        let Some(index) = arena.index(cell) else {
            continue;
        };
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
