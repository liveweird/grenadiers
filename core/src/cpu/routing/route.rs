use std::collections::{BTreeMap, BTreeSet};

use crate::cpu::forecast::{HazardForecast, TickWindow};
use crate::cpu::{MovementIntent, Observation, WorkBudget};
use crate::game::types::UNITS_PER_CELL;
use crate::game::{Direction, EntityId, Tick};

use super::motion::{Motion, advance};
use super::route_checks::{CollisionProofs, check_transition_all, wait_boundaries, wait_lengths};
use super::route_expand::{apply_move, apply_wait, finish, terminal};
use super::route_priority::{dominated, risk, state_key};
use super::route_time::add_tick;
use super::safety::RecipientBarrier;
use super::safety::stationary_segment;
use super::validation::{self, Context};
use super::{
    CERTIFIED_MOVE_UPDATES, MAX_ROUTE_EXPANSIONS, MAX_ROUTE_STEPS, RouteCertificate, RouteFailure,
    RouteMode, RouteRequest, RouteStep,
};

pub(super) const CONDITIONAL_MOVE_UPDATES: u8 = 3;

#[derive(Clone)]
pub(super) struct LaneState {
    pub(super) motion: Motion,
    pub(super) ready: Tick,
    pub(super) last_stop: Option<Tick>,
    pub(super) origin_grace: Option<crate::game::CellId>,
    pub(super) recipient_barrier: Option<RecipientBarrier>,
}

#[derive(Clone)]
pub(super) struct SearchNode {
    pub(super) steps: Vec<RouteStep>,
    pub(super) lanes: Vec<LaneState>,
    pub(super) assumptions: BTreeSet<EntityId>,
}

pub fn plan_route(
    observation: &Observation,
    forecast: &HazardForecast,
    request: RouteRequest,
    work: &mut WorkBudget,
) -> Result<RouteCertificate, RouteFailure> {
    if observation.parts().rules.version >= 8 {
        return Err(RouteFailure::InvalidRequest);
    }
    plan_route_seeded(observation, &[forecast], request, None, None, None, work)
}

pub(crate) fn plan_respawn_route(
    observation: &Observation,
    forecast: &HazardForecast,
    request: RouteRequest,
    work: &mut WorkBudget,
) -> Result<RouteCertificate, RouteFailure> {
    if observation.parts().rules.version != 9 {
        return Err(RouteFailure::InvalidRequest);
    }
    plan_route_seeded(observation, &[forecast], request, None, None, None, work)
}

pub(super) fn plan_route_seeded(
    observation: &Observation,
    forecasts: &[&HazardForecast],
    request: RouteRequest,
    first_step: Option<RouteStep>,
    origin_grace: Option<crate::game::CellId>,
    recipient_barrier: Option<RecipientBarrier>,
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
    let (lanes, assumptions) = initial_lanes(
        &context,
        forecasts,
        request,
        origin_grace,
        recipient_barrier,
        work,
    )?;
    let mut boundaries = Vec::new();
    for candidate in forecasts {
        boundaries.extend(wait_boundaries(&context, candidate, work)?);
    }
    boundaries.sort_unstable();
    boundaries.dedup();
    let mut root = SearchNode {
        steps: Vec::new(),
        lanes,
        assumptions,
    };
    if let Some(step) = first_step {
        match step {
            RouteStep::Move { direction, updates } => apply_move(
                &context,
                forecasts,
                request.mode,
                direction,
                updates,
                false,
                &mut root,
                work,
            )?,
            RouteStep::Wait { updates } => {
                apply_wait(&context, forecasts, request.mode, updates, &mut root, work)?
            }
        }
        root.steps.push(step);
        if terminal(&context, forecasts, request, &mut root, work)? {
            return finish(&context, forecast, request, root, work);
        }
    }
    if terminal(&context, forecasts, request, &mut root, work)? {
        let mut stopped = root.clone();
        apply_wait(&context, forecasts, request.mode, 1, &mut stopped, work)?;
        stopped.steps.push(RouteStep::Wait { updates: 1 });
        if terminal(&context, forecasts, request, &mut stopped, work)? {
            return finish(&context, forecast, request, stopped, work);
        }
    }
    let root_risk = risk(forecasts, request, &root, origin_grace, work)?;
    let mut queue = vec![(root_risk, 0_u32, root)];
    let mut visited = BTreeMap::new();
    let mut expansions = 0_u16;
    let mut order = 1_u32;
    while !queue.is_empty() {
        work.try_take(u32::try_from(queue.len()).map_err(|_| RouteFailure::Capacity)?)
            .map_err(|_| RouteFailure::WorkExhausted)?;
        let index = queue
            .iter()
            .enumerate()
            .min_by_key(|(_, (risk, order, node))| (*risk, usize::MAX - node.steps.len(), *order))
            .map(|(index, _)| index)
            .ok_or(RouteFailure::Capacity)?;
        let (_, _, node) = queue.swap_remove(index);
        let key = state_key(&node);
        if dominated(&mut visited, key, node.steps.len()) {
            continue;
        }
        if expansions >= request.limits.max_expansions {
            return Err(RouteFailure::Capacity);
        }
        work.try_take(1).map_err(|_| RouteFailure::WorkExhausted)?;
        expansions += 1;
        if node.steps.len() >= usize::from(request.limits.max_steps) {
            continue;
        }
        for direction in [
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
        ] {
            for updates in move_lengths(&context, &node, request.mode, direction, work)? {
                let mut next = node.clone();
                match apply_move(
                    &context,
                    forecasts,
                    request.mode,
                    direction,
                    updates,
                    true,
                    &mut next,
                    work,
                ) {
                    Ok(()) => {}
                    Err(RouteFailure::NoRoute) => continue,
                    Err(error) => return Err(error),
                }
                next.steps.push(RouteStep::Move { direction, updates });
                if terminal(&context, forecasts, request, &mut next, work)? {
                    return finish(&context, forecast, request, next, work);
                }
                let candidate_risk = risk(forecasts, request, &next, origin_grace, work)?;
                queue.push((candidate_risk, order, next));
                order = order.checked_add(1).ok_or(RouteFailure::Capacity)?;
            }
        }
        let ready_ticks = node.lanes.iter().map(|lane| lane.ready).collect::<Vec<_>>();
        for updates in wait_lengths(&boundaries, request, &ready_ticks)? {
            let mut next = node.clone();
            match apply_wait(&context, forecasts, request.mode, updates, &mut next, work) {
                Ok(()) => {}
                Err(RouteFailure::NoRoute) => continue,
                Err(error) => return Err(error),
            }
            next.steps.push(RouteStep::Wait { updates });
            if terminal(&context, forecasts, request, &mut next, work)? {
                return finish(&context, forecast, request, next, work);
            }
            let candidate_risk = risk(forecasts, request, &next, origin_grace, work)?;
            queue.push((candidate_risk, order, next));
            order = order.checked_add(1).ok_or(RouteFailure::Capacity)?;
        }
    }
    Err(RouteFailure::NoRoute)
}

fn validate_request(
    context: &Context<'_>,
    forecast: &HazardForecast,
    request: RouteRequest,
) -> Result<(), RouteFailure> {
    let basis = forecast.basis_tick().0;
    let earliest_start = basis.checked_add(1).ok_or(RouteFailure::InvalidRequest)?;
    let latest_end = basis.checked_add(8).ok_or(RouteFailure::InvalidRequest)?;
    let width = request
        .start_window
        .end
        .0
        .checked_sub(request.start_window.start.0)
        .ok_or(RouteFailure::InvalidRequest)?;
    if request.limits.max_steps == 0
        || request.limits.max_steps > MAX_ROUTE_STEPS
        || request.limits.max_expansions == 0
        || request.limits.max_expansions > MAX_ROUTE_EXPANSIONS
        || request.start_window.start.0 < earliest_start
        || request.start_window.end.0 > latest_end
        || width == 0
        || width > 7
        || request.survive_until <= request.start_window.end
        || request.survive_until > forecast.horizon_end()
        || request.settled_stop_tick > forecast.basis_tick()
        || request.settled_stop_tick > context.observation.parts().tick
    {
        return Err(RouteFailure::InvalidRequest);
    }
    Ok(())
}

fn initial_lanes(
    context: &Context<'_>,
    forecasts: &[&HazardForecast],
    request: RouteRequest,
    origin_grace: Option<crate::game::CellId>,
    recipient_barrier: Option<RecipientBarrier>,
    work: &mut WorkBudget,
) -> Result<(Vec<LaneState>, BTreeSet<EntityId>), RouteFailure> {
    let mut lanes = Vec::new();
    let mut assumptions = BTreeSet::new();
    for value in request.start_window.start.0..request.start_window.end.0 {
        let first = Tick(value);
        let mut motion = Motion::stopped(context, first);
        if context.storm.is_none() {
            let segment = stationary_segment(context.actor.position);
            assumptions.extend(check_transition_all(
                context,
                forecasts,
                &segment,
                TickWindow {
                    start: add_tick(context.observation.parts().tick, 1)?,
                    end: first,
                },
                request.mode,
                CollisionProofs {
                    placement_origin: None,
                    recipient_barrier,
                },
                work,
            )?);
        }
        if context.storm.is_some() {
            for tick in add_tick(context.observation.parts().tick, 1)?.0..first.0 {
                super::storm::charge_tick(context.storm, work)?;
                let tick = Tick(tick);
                let segment = advance(context, &mut motion, MovementIntent::Stop, tick)?;
                assumptions.extend(check_transition_all(
                    context,
                    forecasts,
                    &segment,
                    TickWindow {
                        start: tick,
                        end: add_tick(tick, 1)?,
                    },
                    request.mode,
                    CollisionProofs {
                        placement_origin: None,
                        recipient_barrier,
                    },
                    work,
                )?);
            }
        }
        lanes.push(LaneState {
            motion,
            ready: first,
            last_stop: None,
            origin_grace,
            recipient_barrier,
        });
    }
    Ok((lanes, assumptions))
}

fn move_lengths(
    context: &Context<'_>,
    node: &SearchNode,
    mode: RouteMode,
    direction: Direction,
    work: &mut WorkBudget,
) -> Result<Vec<u8>, RouteFailure> {
    let cap = if node.assumptions.is_empty() {
        CERTIFIED_MOVE_UPDATES
    } else {
        CONDITIONAL_MOVE_UPDATES
    };
    let mut simulations = node.lanes.clone();
    let mut centered = None;
    for completed in 1..=cap {
        for lane in &mut simulations {
            work.try_take(1).map_err(|_| RouteFailure::WorkExhausted)?;
            super::storm::charge_tick(context.storm, work)?;
            let _ = advance(
                context,
                &mut lane.motion,
                MovementIntent::Direction(direction),
                add_tick(lane.ready, u64::from(completed - 1))?,
            )?;
        }
        if completed < cap
            && simulations
                .iter()
                .all(|lane| in_turn_band(direction, lane.motion.position, context))
        {
            centered = Some(completed);
            break;
        }
    }
    let mut lengths = centered.into_iter().collect::<Vec<_>>();
    if mode == RouteMode::AllowStationaryAssumptions && cap > CONDITIONAL_MOVE_UPDATES {
        lengths.push(CONDITIONAL_MOVE_UPDATES);
    }
    lengths.push(cap);
    lengths.sort_unstable();
    lengths.dedup();
    Ok(lengths)
}

fn in_turn_band(
    direction: Direction,
    position: crate::game::Position,
    context: &Context<'_>,
) -> bool {
    let coordinate = match direction {
        Direction::North | Direction::South => position.y,
        Direction::East | Direction::West => position.x,
    };
    let center = coordinate.div_euclid(UNITS_PER_CELL) * UNITS_PER_CELL + UNITS_PER_CELL / 2;
    (coordinate - center).abs()
        <= context
            .observation
            .parts()
            .rules
            .settings
            .turn_assist_units()
}
