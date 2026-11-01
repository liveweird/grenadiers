//! Deterministic hint construction for the inactive continuation selector.

use crate::cpu::forecast::{HazardForecast, TickWindow};
use crate::cpu::{MovementIntent, Observation, WorkBudget};
use crate::game::types::{HALF_FOOTPRINT, UNITS_PER_CELL};
use crate::game::{CellId, Direction, Position, Tick};

use super::RouteFailure;
use super::continuation_types::{
    ContinuationRequest, MAX_CONTINUATION_LEGS, MAX_CONTINUATION_MOVES, NominalLeg,
};
use super::motion::{Motion, advance};
use super::validation;

const MAX_TICK_HINT_WORK: usize = 9;

pub(crate) fn continuation_candidates(
    observation: &Observation,
    forecast: &HazardForecast,
    request: ContinuationRequest,
    refuge: CellId,
    first_axis: Direction,
    work: &mut WorkBudget,
) -> Result<Vec<Vec<NominalLeg>>, RouteFailure> {
    let context = validation::validate(
        observation,
        forecast,
        TickWindow {
            start: request.first_update,
            end: Tick(request.first_update.0 + 1),
        },
        request.survive_until,
    )?;
    let vertical_first = matches!(first_axis, Direction::North | Direction::South);
    let mut candidates = Vec::with_capacity(2);
    for first_vertical in [vertical_first, !vertical_first] {
        match build_candidate(&context, request, refuge, first_vertical, work) {
            Ok(legs) if !candidates.contains(&legs) => candidates.push(legs),
            Ok(_) | Err(RouteFailure::NoRoute | RouteFailure::Capacity) => {}
            Err(failure) => return Err(failure),
        }
    }
    Ok(candidates)
}

fn build_candidate(
    context: &validation::Context<'_>,
    request: ContinuationRequest,
    refuge: CellId,
    first_vertical: bool,
    work: &mut WorkBudget,
) -> Result<Vec<NominalLeg>, RouteFailure> {
    let mut motion = Motion::stopped(context, request.first_update);
    let target = Position::center(refuge);
    let mut tick = request.first_update;
    let mut legs = Vec::with_capacity(3);
    let axes = if first_vertical {
        [true, false]
    } else {
        [false, true]
    };
    for (index, vertical) in axes.into_iter().enumerate() {
        let direction = direction_toward(motion.position, target, vertical);
        let final_axis =
            index == 1 || direction_toward(motion.position, target, !vertical).is_none();
        let Some(direction) = direction else {
            continue;
        };
        let updates = advance_axis(
            context,
            &mut motion,
            direction,
            refuge,
            final_axis,
            &mut tick,
            work,
        )?;
        legs.push(NominalLeg::Move { direction, updates });
        if !final_axis {
            legs.push(NominalLeg::Stop { updates: 3 });
            for _ in 0..3 {
                charge_hint_tick(context, work)?;
                advance(context, &mut motion, MovementIntent::Stop, tick)?;
                tick = next_tick(tick)?;
            }
        }
    }
    if legs.is_empty()
        || legs.len() > MAX_CONTINUATION_LEGS
        || moving_updates(&legs)? > MAX_CONTINUATION_MOVES
        || !inside_refuge(motion.position, refuge)
    {
        return Err(RouteFailure::NoRoute);
    }
    Ok(legs)
}

#[allow(clippy::too_many_arguments)]
fn advance_axis(
    context: &validation::Context<'_>,
    motion: &mut Motion,
    direction: Direction,
    refuge: CellId,
    final_axis: bool,
    tick: &mut Tick,
    work: &mut WorkBudget,
) -> Result<u8, RouteFailure> {
    let target = Position::center(refuge);
    for updates in 1..=MAX_CONTINUATION_MOVES as u8 {
        charge_hint_tick(context, work)?;
        let before = motion.position;
        advance(context, motion, MovementIntent::Direction(direction), *tick)?;
        if motion.position == before {
            return Err(RouteFailure::NoRoute);
        }
        *tick = next_tick(*tick)?;
        let reached = if final_axis {
            inside_refuge(motion.position, refuge)
        } else if matches!(direction, Direction::North | Direction::South) {
            motion.position.y == target.y
        } else {
            motion.position.x == target.x
        };
        if reached {
            return Ok(updates);
        }
    }
    Err(RouteFailure::Capacity)
}

fn charge_hint_tick(
    context: &validation::Context<'_>,
    work: &mut WorkBudget,
) -> Result<(), RouteFailure> {
    let units = context
        .observation
        .parts()
        .actors
        .len()
        .checked_add(MAX_TICK_HINT_WORK)
        .and_then(|value| u32::try_from(value).ok())
        .ok_or(RouteFailure::Capacity)?;
    work.try_take(units)
        .map_err(|_| RouteFailure::WorkExhausted)?;
    super::storm::charge_tick(context.storm, work)
}

fn direction_toward(from: Position, to: Position, vertical: bool) -> Option<Direction> {
    if vertical {
        (from.y != to.y).then_some(if from.y < to.y {
            Direction::South
        } else {
            Direction::North
        })
    } else {
        (from.x != to.x).then_some(if from.x < to.x {
            Direction::East
        } else {
            Direction::West
        })
    }
}

fn inside_refuge(position: Position, refuge: CellId) -> bool {
    let min_x = i32::from(refuge.column) * UNITS_PER_CELL;
    let min_y = i32::from(refuge.row) * UNITS_PER_CELL;
    position.x - HALF_FOOTPRINT >= min_x
        && position.x + HALF_FOOTPRINT <= min_x + UNITS_PER_CELL
        && position.y - HALF_FOOTPRINT >= min_y
        && position.y + HALF_FOOTPRINT <= min_y + UNITS_PER_CELL
}

fn moving_updates(legs: &[NominalLeg]) -> Result<u16, RouteFailure> {
    legs.iter().try_fold(0_u16, |total, leg| match leg {
        NominalLeg::Move { updates, .. } => total
            .checked_add(u16::from(*updates))
            .ok_or(RouteFailure::Capacity),
        NominalLeg::Stop { .. } => Ok(total),
    })
}

fn next_tick(tick: Tick) -> Result<Tick, RouteFailure> {
    tick.0
        .checked_add(1)
        .map(Tick)
        .ok_or(RouteFailure::InvalidRequest)
}
