//! Replay of a bounded, deterministic refuge hint through the full route proof.

use crate::cpu::forecast::HazardForecast;
use crate::cpu::{MovementIntent, Observation, WorkBudget};
use crate::game::{CellId, Direction};

use super::route::{SearchNode, initial_lanes, validate_request};
use super::route_expand::{apply_move, apply_wait, finish, terminal};
use super::validation;
use super::{RouteCertificate, RouteFailure, RouteRequest, RouteStep};

pub(super) fn plan_route_to_refuge(
    observation: &Observation,
    forecasts: &[&HazardForecast],
    request: RouteRequest,
    first_step: RouteStep,
    origin: CellId,
    refuge: CellId,
    work: &mut WorkBudget,
) -> Result<RouteCertificate, RouteFailure> {
    let forecast = *forecasts.first().ok_or(RouteFailure::InvalidRequest)?;
    if forecasts.iter().any(|candidate| {
        candidate.key() != forecast.key()
            || candidate.basis_tick() != forecast.basis_tick()
            || candidate.horizon_end() != forecast.horizon_end()
            || candidate.dimensions() != forecast.dimensions()
    }) {
        return Err(RouteFailure::IdentityMismatch);
    }
    let context = validation::validate(
        observation,
        forecast,
        request.start_window,
        request.survive_until,
    )?;
    validate_request(&context, forecast, request)?;
    let (lanes, assumptions) =
        initial_lanes(&context, forecasts, request, Some(origin), None, work)?;
    let mut node = SearchNode {
        steps: Vec::new(),
        lanes,
        assumptions,
    };
    let primary = match first_step {
        RouteStep::Move { direction, .. } => direction,
        RouteStep::Wait { .. } => return Err(RouteFailure::InvalidRequest),
    };
    apply_step(&context, forecasts, request, first_step, &mut node, work)?;

    while node.steps.len() < usize::from(request.limits.max_steps) {
        if at_refuge(&node, refuge) && terminal(&context, forecasts, request, &mut node, work)? {
            return finish(&context, forecast, request, node, work);
        }
        let direction =
            next_direction(&context, &node, refuge, primary).ok_or(RouteFailure::NoRoute)?;
        let updates = updates_toward_refuge(&context, &node, direction, refuge, primary, work)?;
        apply_step(
            &context,
            forecasts,
            request,
            RouteStep::Move { direction, updates },
            &mut node,
            work,
        )?;
    }
    if at_refuge(&node, refuge) && terminal(&context, forecasts, request, &mut node, work)? {
        return finish(&context, forecast, request, node, work);
    }
    Err(RouteFailure::NoRoute)
}

fn apply_step(
    context: &validation::Context<'_>,
    forecasts: &[&HazardForecast],
    request: RouteRequest,
    step: RouteStep,
    node: &mut SearchNode,
    work: &mut WorkBudget,
) -> Result<(), RouteFailure> {
    match step {
        RouteStep::Move { direction, updates } => apply_move(
            context,
            forecasts,
            request.mode,
            direction,
            updates,
            false,
            node,
            work,
        )?,
        RouteStep::Wait { updates } => {
            apply_wait(context, forecasts, request.mode, updates, node, work)?
        }
    }
    node.steps.push(step);
    Ok(())
}

fn at_refuge(node: &SearchNode, refuge: CellId) -> bool {
    node.lanes
        .iter()
        .all(|lane| lane.motion.position.cell() == Some(refuge))
}

fn next_direction(
    context: &validation::Context<'_>,
    node: &SearchNode,
    refuge: CellId,
    primary: Direction,
) -> Option<Direction> {
    let position = node.lanes.first()?.motion.position;
    let cell = position.cell()?;
    if matches!(primary, Direction::North | Direction::South) && cell.row != refuge.row {
        return Some(if cell.row < refuge.row {
            Direction::South
        } else {
            Direction::North
        });
    }
    let center = crate::game::Position::center(refuge);
    if matches!(primary, Direction::North | Direction::South)
        && cell.column != refuge.column
        && (position.y - center.y).abs()
            > context
                .observation
                .parts()
                .rules
                .settings
                .turn_assist_units()
    {
        return Some(if position.y < center.y {
            Direction::South
        } else {
            Direction::North
        });
    }
    if matches!(primary, Direction::East | Direction::West) && cell.column != refuge.column {
        return Some(if cell.column < refuge.column {
            Direction::East
        } else {
            Direction::West
        });
    }
    if matches!(primary, Direction::East | Direction::West)
        && cell.row != refuge.row
        && (position.x - center.x).abs()
            > context
                .observation
                .parts()
                .rules
                .settings
                .turn_assist_units()
    {
        return Some(if position.x < center.x {
            Direction::East
        } else {
            Direction::West
        });
    }
    if cell.row != refuge.row {
        return Some(if cell.row < refuge.row {
            Direction::South
        } else {
            Direction::North
        });
    }
    (cell.column != refuge.column).then_some({
        if cell.column < refuge.column {
            Direction::East
        } else {
            Direction::West
        }
    })
}

fn updates_toward_refuge(
    context: &validation::Context<'_>,
    node: &SearchNode,
    direction: Direction,
    refuge: CellId,
    primary: Direction,
    work: &mut WorkBudget,
) -> Result<u8, RouteFailure> {
    let mut simulations = node.lanes.clone();
    let turning_after = direction == primary
        && simulations
            .first()
            .and_then(|lane| lane.motion.position.cell())
            .is_some_and(|cell| match primary {
                Direction::North | Direction::South => cell.column != refuge.column,
                Direction::East | Direction::West => cell.row != refuge.row,
            });
    let target_center = crate::game::Position::center(refuge);
    let mut best = None;
    let mut turn_center = None;
    for completed in 1..=super::CERTIFIED_MOVE_UPDATES {
        for lane in &mut simulations {
            work.try_take(1).map_err(|_| RouteFailure::WorkExhausted)?;
            super::storm::charge_tick(context.storm, work)?;
            let _ = super::motion::advance(
                context,
                &mut lane.motion,
                MovementIntent::Direction(direction),
                super::route_time::add_tick(lane.ready, u64::from(completed - 1))?,
            )?;
        }
        if simulations.iter().all(|lane| {
            let position = lane.motion.position;
            match direction {
                Direction::North | Direction::South => {
                    position.cell().map(|cell| cell.row) == Some(refuge.row)
                }
                Direction::East | Direction::West => {
                    position.cell().map(|cell| cell.column) == Some(refuge.column)
                }
            }
        }) {
            let distance = simulations
                .iter()
                .map(|lane| match direction {
                    Direction::North | Direction::South => {
                        (lane.motion.position.y - target_center.y).unsigned_abs()
                    }
                    Direction::East | Direction::West => {
                        (lane.motion.position.x - target_center.x).unsigned_abs()
                    }
                })
                .max()
                .ok_or(RouteFailure::NoRoute)?;
            if turning_after && turn_center.is_none() {
                let crossed_center = simulations.iter().all(|lane| match direction {
                    Direction::South => lane.motion.position.y > target_center.y,
                    Direction::North => lane.motion.position.y < target_center.y,
                    Direction::East => lane.motion.position.x > target_center.x,
                    Direction::West => lane.motion.position.x < target_center.x,
                });
                if crossed_center {
                    // Leave one update of interior turn-assist margin before
                    // the center crossing; the following perpendicular input
                    // consumes that margin while aligning to the center line.
                    turn_center = Some(completed.saturating_sub(2).max(1));
                }
            }
            if best.is_none_or(|(_, old)| distance < old) {
                best = Some((completed, distance));
            }
        }
    }
    Ok(turn_center
        .or(best.map(|(updates, _)| updates))
        .unwrap_or(super::CERTIFIED_MOVE_UPDATES))
}
