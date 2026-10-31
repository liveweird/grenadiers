use std::collections::BTreeMap;

use crate::cpu::WorkBudget;
use crate::cpu::forecast::HazardForecast;
use crate::game::{CellId, Direction, Position};

use super::route::SearchNode;
use super::route_checks::stationary_risk;
use super::{RouteFailure, RouteRequest, RouteStep};

pub(super) type StateKey = (
    Vec<(
        i32,
        i32,
        u8,
        u64,
        Option<u64>,
        Option<CellId>,
        Option<(CellId, u32)>,
        Option<(u64, u64)>,
    )>,
    Vec<u32>,
    Vec<Vec<(CellId, u8, Option<u64>, u64)>>,
);

pub(super) fn risk(
    forecasts: &[&HazardForecast],
    request: RouteRequest,
    node: &SearchNode,
    escape_origin: Option<CellId>,
    work: &mut WorkBudget,
) -> Result<u32, RouteFailure> {
    let lanes = node
        .lanes
        .iter()
        .map(|lane| (lane.motion.position, lane.ready))
        .collect::<Vec<_>>();
    let mut risk = 0_u32;
    for forecast in forecasts {
        risk = risk.saturating_add(stationary_risk(
            forecast,
            &lanes,
            request.survive_until,
            request.mode,
            work,
        )?);
    }
    if let Some(origin) = escape_origin {
        let center = Position::center(origin);
        let progress = node.lanes.iter().fold(0_u32, |sum, lane| {
            sum.saturating_add(lane.motion.position.x.abs_diff(center.x))
                .saturating_add(lane.motion.position.y.abs_diff(center.y))
        });
        risk = 100_000_u32
            .saturating_sub(progress.min(99_999))
            .saturating_add(risk.min(999));
    }
    Ok(
        if escape_origin.is_none()
            && matches!(node.steps.last(), Some(RouteStep::Wait { updates }) if *updates > 1)
        {
            risk.saturating_sub(100_000)
        } else {
            risk
        },
    )
}

pub(super) fn state_key(node: &SearchNode) -> StateKey {
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
                lane.origin_grace,
                lane.recipient_barrier
                    .map(|barrier| (barrier.cell, barrier.bomb_id.0)),
                lane.motion
                    .storm_deadlines()
                    .map(|bounds| (bounds.lower.0, bounds.upper.0)),
            )
        })
        .collect();
    let assumptions = node.assumptions.iter().map(|id| id.0).collect();
    let pits = node
        .lanes
        .iter()
        .map(|lane| lane.pits.state_key())
        .collect();
    (lanes, assumptions, pits)
}

const fn direction_key(direction: Direction) -> u8 {
    match direction {
        Direction::North => 0,
        Direction::East => 1,
        Direction::South => 2,
        Direction::West => 3,
    }
}

pub(super) fn dominated(
    visited: &mut BTreeMap<StateKey, usize>,
    key: StateKey,
    depth: usize,
) -> bool {
    if visited.get(&key).is_some_and(|old| *old <= depth) {
        true
    } else {
        visited.insert(key, depth);
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shallower_converging_wait_state_is_not_suppressed() {
        let key = (
            vec![(1, 2, 0, 10, None, None, None, None)],
            Vec::new(),
            Vec::new(),
        );
        let mut visited = BTreeMap::new();
        assert!(!dominated(&mut visited, key.clone(), 2));
        assert!(!dominated(&mut visited, key.clone(), 1));
        assert!(dominated(&mut visited, key, 2));
    }

    #[test]
    fn distinct_storm_deadlines_do_not_dominate_each_other() {
        let first = (
            vec![(1, 2, 0, 10, None, None, None, Some((20, 40)))],
            Vec::new(),
            Vec::new(),
        );
        let second = (
            vec![(1, 2, 0, 10, None, None, None, Some((30, 50)))],
            Vec::new(),
            Vec::new(),
        );
        let mut visited = BTreeMap::new();
        assert!(!dominated(&mut visited, first, 1));
        assert!(!dominated(&mut visited, second, 1));
    }
}
