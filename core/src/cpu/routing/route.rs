use std::collections::{BTreeMap, BTreeSet};

use crate::cpu::forecast::{HazardForecast, TickWindow};
use crate::cpu::{MovementIntent, Observation, WorkBudget};
use crate::game::types::UNITS_PER_CELL;
use crate::game::{Direction, EntityId, Tick};

use super::motion::{Motion, advance};
use super::route_checks::{check_transition, stationary_risk, wait_boundaries, wait_lengths};
use super::route_expand::{apply_move, apply_wait, finish, terminal};
use super::route_time::add_tick;
use super::safety::stationary_segment;
use super::validation::{self, Context};
use super::{
    MAX_ROUTE_EXPANSIONS, MAX_ROUTE_STEPS, RouteCertificate, RouteFailure, RouteMode, RouteRequest,
    RouteStep,
};

const CERTIFIED_MOVE_UPDATES: u8 = 12;
pub(super) const CONDITIONAL_MOVE_UPDATES: u8 = 3;

#[derive(Clone)]
pub(super) struct LaneState {
    pub(super) motion: Motion,
    pub(super) ready: Tick,
    pub(super) last_stop: Option<Tick>,
}

#[derive(Clone)]
pub(super) struct SearchNode {
    pub(super) steps: Vec<RouteStep>,
    pub(super) lanes: Vec<LaneState>,
    pub(super) assumptions: BTreeSet<EntityId>,
}

type StateKey = (Vec<(i32, i32, u8, u64, Option<u64>)>, Vec<u32>);

pub fn plan_route(
    observation: &Observation,
    forecast: &HazardForecast,
    request: RouteRequest,
    work: &mut WorkBudget,
) -> Result<RouteCertificate, RouteFailure> {
    let context = validation::validate(
        observation,
        forecast,
        request.start_window,
        request.survive_until,
    )?;
    validate_request(&context, forecast, request)?;
    let (lanes, assumptions) = initial_lanes(&context, forecast, request, work)?;
    let wait_boundaries = wait_boundaries(&context, forecast, work)?;
    let mut root = SearchNode {
        steps: Vec::new(),
        lanes,
        assumptions,
    };
    if terminal(forecast, request, &mut root, work)? {
        let mut stopped = root.clone();
        apply_wait(forecast, request.mode, 1, &mut stopped, work)?;
        stopped.steps.push(RouteStep::Wait { updates: 1 });
        if terminal(forecast, request, &mut stopped, work)? {
            return finish(&context, forecast, request, stopped, work);
        }
    }
    let root_risk = risk(forecast, request, &root, work)?;
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
                    forecast,
                    request.mode,
                    direction,
                    updates,
                    &mut next,
                    work,
                ) {
                    Ok(()) => {}
                    Err(RouteFailure::NoRoute) => continue,
                    Err(error) => return Err(error),
                }
                next.steps.push(RouteStep::Move { direction, updates });
                if terminal(forecast, request, &mut next, work)? {
                    return finish(&context, forecast, request, next, work);
                }
                let candidate_risk = risk(forecast, request, &next, work)?;
                queue.push((candidate_risk, order, next));
                order = order.checked_add(1).ok_or(RouteFailure::Capacity)?;
            }
        }
        let ready_ticks = node.lanes.iter().map(|lane| lane.ready).collect::<Vec<_>>();
        for updates in wait_lengths(&wait_boundaries, request, &ready_ticks)? {
            let mut next = node.clone();
            match apply_wait(forecast, request.mode, updates, &mut next, work) {
                Ok(()) => {}
                Err(RouteFailure::NoRoute) => continue,
                Err(error) => return Err(error),
            }
            next.steps.push(RouteStep::Wait { updates });
            if terminal(forecast, request, &mut next, work)? {
                return finish(&context, forecast, request, next, work);
            }
            let candidate_risk = risk(forecast, request, &next, work)?;
            queue.push((candidate_risk, order, next));
            order = order.checked_add(1).ok_or(RouteFailure::Capacity)?;
        }
    }
    Err(RouteFailure::NoRoute)
}

fn risk(
    forecast: &HazardForecast,
    request: RouteRequest,
    node: &SearchNode,
    work: &mut WorkBudget,
) -> Result<u32, RouteFailure> {
    let lanes = node
        .lanes
        .iter()
        .map(|lane| (lane.motion.position, lane.ready))
        .collect::<Vec<_>>();
    let risk = stationary_risk(forecast, &lanes, request.survive_until, request.mode, work)?;
    Ok(
        if matches!(node.steps.last(), Some(RouteStep::Wait { updates }) if *updates > 1) {
            risk.saturating_sub(100_000)
        } else {
            risk
        },
    )
}

fn state_key(node: &SearchNode) -> StateKey {
    let lanes = node
        .lanes
        .iter()
        .map(|lane| {
            (
                lane.motion.position.x,
                lane.motion.position.y,
                direction_key(lane.motion.heading),
                lane.ready.0,
                lane.last_stop.map(|tick| tick.0),
            )
        })
        .collect();
    let assumptions = node.assumptions.iter().map(|id| id.0).collect();
    (lanes, assumptions)
}

const fn direction_key(direction: Direction) -> u8 {
    match direction {
        Direction::North => 0,
        Direction::East => 1,
        Direction::South => 2,
        Direction::West => 3,
    }
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
    forecast: &HazardForecast,
    request: RouteRequest,
    work: &mut WorkBudget,
) -> Result<(Vec<LaneState>, BTreeSet<EntityId>), RouteFailure> {
    let mut lanes = Vec::new();
    let mut assumptions = BTreeSet::new();
    for value in request.start_window.start.0..request.start_window.end.0 {
        let first = Tick(value);
        let segment = stationary_segment(context.actor.position);
        assumptions.extend(check_transition(
            context,
            forecast,
            &segment,
            TickWindow {
                start: Tick(forecast.basis_tick().0 + 1),
                end: first,
            },
            request.mode,
            work,
        )?);
        lanes.push(LaneState {
            motion: Motion::stopped(context, first),
            ready: first,
            last_stop: None,
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

fn dominated(visited: &mut BTreeMap<StateKey, usize>, key: StateKey, depth: usize) -> bool {
    if visited.get(&key).is_some_and(|old| *old <= depth) {
        true
    } else {
        visited.insert(key, depth);
        false
    }
}

#[cfg(test)]
mod search_tests {
    use super::*;

    #[test]
    fn shallower_converging_wait_state_is_not_suppressed() {
        let key = (vec![(1, 2, 0, 10, None)], Vec::new());
        let mut visited = BTreeMap::new();
        assert!(!dominated(&mut visited, key.clone(), 2));
        assert!(!dominated(&mut visited, key.clone(), 1));
        assert!(dominated(&mut visited, key, 2));
    }
}
