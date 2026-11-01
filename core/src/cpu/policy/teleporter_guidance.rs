use crate::cpu::{MovementIntent, Observation, ObservedActor, WorkBudget};
use crate::game::{CellId, PickupKind, Tick};

use super::goals::{cell_index, search_map};
use super::movement::{STEPS, direction, offset};
use super::placement::{TurnAlignment, alignment_direction, ordinary_turn_alignment};
use super::{
    STRATEGIC_COMMITMENT_TICKS, STRATEGIC_PROGRESS_TICKS, StrategicIntent, StrategicObjectiveKind,
    StrategicPhase, StrategicSwitchReason, StrategicTarget, UtilityMemory,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TeleporterCandidate {
    pub(crate) movement: MovementIntent,
    pub(crate) waypoint: CellId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TeleporterGuidance {
    pub(crate) target: CellId,
    pub(crate) approach: CellId,
    pub(crate) candidates: [Option<TeleporterCandidate>; 2],
    pub(crate) next_memory: UtilityMemory,
}

#[derive(Clone, Copy)]
struct ApproachChoice {
    cell: CellId,
    retained: Option<(StrategicIntent, bool)>,
    reason: StrategicSwitchReason,
}

pub(crate) fn teleporter_guidance(
    observation: &Observation,
    memory: &UtilityMemory,
    work: &mut WorkBudget,
) -> Option<TeleporterGuidance> {
    let parts = observation.parts();
    work.try_take(1).ok()?;
    let key = memory.key();
    if key.world != parts.world || key.cpu_seat != parts.recipient {
        return None;
    }
    work.try_take(u32::try_from(parts.actors.len()).ok()?)
        .ok()?;
    let actor = parts.actors.iter().find_map(|candidate| match candidate {
        ObservedActor::Visible(actor) if actor.seat == parts.recipient => Some(actor),
        _ => None,
    })?;
    if !actor.alive || actor.voluntary_actions_frozen {
        return None;
    }
    let current = actor.position.cell()?;

    work.try_take(u32::try_from(parts.pickups.len()).ok()?)
        .ok()?;
    let mut targets = parts
        .pickups
        .iter()
        .filter(|pickup| pickup.kind == PickupKind::Teleporter);
    let target = targets.next()?.cell;
    if targets.next().is_some() {
        return None;
    }

    let map = search_map(observation, actor, current, false, work)?;
    let (distance, first_step) = super::goals::v24::reachable(observation, current, &map, work)?;
    let retained = retained_approach(*memory, target);
    let mut approaches = Vec::with_capacity(4);
    for (_, dc, dr) in STEPS {
        work.try_take(1).ok()?;
        let Some(cell) = offset(target, dc, dr) else {
            continue;
        };
        let Some(index) = cell_index(observation, cell) else {
            continue;
        };
        if map.transitable(observation, cell) && distance[index].is_some() {
            approaches.push(cell);
        }
    }
    approaches.sort_unstable_by_key(|cell| {
        let index = cell_index(observation, *cell).unwrap_or(0);
        (
            retained != Some(*cell),
            distance[index].unwrap_or(u16::MAX),
            cell.row,
            cell.column,
        )
    });
    let previous = memory
        .strategic_intent()
        .filter(|intent| intent.target == teleporter_target(target));
    let approach_choice = choose_approach(
        observation,
        actor,
        previous,
        &approaches,
        &distance,
        &first_step,
    )?;
    let approach = approach_choice.cell;
    let approach_index = cell_index(observation, approach)?;
    let (candidates, remaining_distance) = if current == approach {
        (final_candidates(observation, actor, current, target)?, 0)
    } else {
        let waypoint = first_step[approach_index]?;
        (
            navigation_candidates(observation, actor, current, waypoint)?,
            distance[approach_index]?,
        )
    };
    work.try_take(u32::try_from(candidates.iter().flatten().count()).ok()?)
        .ok()?;

    let tick = parts.tick;
    let intent = approach_choice.retained.map_or_else(
        || {
            new_intent(
                tick,
                target,
                approach,
                remaining_distance,
                actor.position,
                approach_choice.reason,
            )
        },
        |(previous, progressed)| StrategicIntent {
            approach_cell: Some(approach),
            observation_tick: tick,
            last_distance: remaining_distance,
            last_cpu_position: actor.position,
            progress_deadline: if progressed {
                add_ticks(tick, STRATEGIC_PROGRESS_TICKS)
            } else {
                previous.progress_deadline
            },
            // The waypoint is deliberately read before carrying the prior
            // intent: route progress targets the next hop, while progress from
            // the neighboring cell targets the Teleporter itself.
            phase: StrategicPhase::Approach,
            ..previous
        },
    );
    Some(TeleporterGuidance {
        target,
        approach,
        candidates,
        next_memory: memory
            .without_execution_authority()
            .with_strategic_intent(intent),
    })
}

fn choose_approach(
    observation: &Observation,
    actor: &crate::cpu::VisibleActor,
    previous: Option<StrategicIntent>,
    approaches: &[CellId],
    distance: &[Option<u16>; crate::game::arena::ARENA_CELL_COUNT],
    first_step: &[Option<CellId>; crate::game::arena::ARENA_CELL_COUNT],
) -> Option<ApproachChoice> {
    let current = actor.position.cell()?;
    let Some(previous) = previous else {
        return Some(ApproachChoice {
            cell: *approaches.first()?,
            retained: None,
            reason: StrategicSwitchReason::Initial,
        });
    };
    let retained = previous.approach_cell?;
    if !approaches.contains(&retained) {
        if previous.last_switch_reason == StrategicSwitchReason::Stalled {
            return None;
        }
        return Some(ApproachChoice {
            cell: *approaches.first()?,
            retained: None,
            reason: StrategicSwitchReason::TargetInvalid,
        });
    }
    let index = cell_index(observation, retained)?;
    let retained_distance = distance[index]?;
    let waypoint = if current == retained {
        match previous.target {
            StrategicTarget::Pickup { cell, .. } => cell,
            _ => return None,
        }
    } else {
        first_step[index]?
    };
    let progressed = retained_distance < previous.last_distance
        || (actor.position != previous.last_cpu_position
            && position_distance(actor.position, crate::game::Position::center(waypoint))
                < position_distance(
                    previous.last_cpu_position,
                    crate::game::Position::center(waypoint),
                ));
    if progressed || observation.parts().tick < previous.progress_deadline {
        return Some(ApproachChoice {
            cell: retained,
            retained: Some((previous, progressed)),
            reason: previous.last_switch_reason,
        });
    }
    if previous.last_switch_reason == StrategicSwitchReason::Stalled {
        return None;
    }
    approaches
        .iter()
        .copied()
        .find(|approach| *approach != retained)
        .map(|alternate| ApproachChoice {
            cell: alternate,
            retained: None,
            reason: StrategicSwitchReason::Stalled,
        })
}

fn new_intent(
    tick: Tick,
    target: CellId,
    approach: CellId,
    distance: u16,
    position: crate::game::Position,
    reason: StrategicSwitchReason,
) -> StrategicIntent {
    StrategicIntent {
        objective: StrategicObjectiveKind::Collect,
        target: teleporter_target(target),
        approach_cell: Some(approach),
        phase: StrategicPhase::Approach,
        last_switch_reason: reason,
        observation_tick: tick,
        score: 10,
        committed_until: add_ticks(tick, STRATEGIC_COMMITMENT_TICKS),
        progress_deadline: add_ticks(tick, STRATEGIC_PROGRESS_TICKS),
        last_distance: distance,
        last_cpu_position: position,
        deferred_until: None,
    }
}

const fn teleporter_target(cell: CellId) -> StrategicTarget {
    StrategicTarget::Pickup {
        cell,
        kind: PickupKind::Teleporter,
    }
}

fn position_distance(left: crate::game::Position, right: crate::game::Position) -> i64 {
    (i64::from(left.x) - i64::from(right.x)).abs() + (i64::from(left.y) - i64::from(right.y)).abs()
}

fn retained_approach(memory: UtilityMemory, target: CellId) -> Option<CellId> {
    memory
        .strategic_intent()
        .filter(|intent| {
            intent.objective == StrategicObjectiveKind::Collect
                && intent.target
                    == (StrategicTarget::Pickup {
                        cell: target,
                        kind: PickupKind::Teleporter,
                    })
        })
        .and_then(|intent| intent.approach_cell)
}

fn navigation_candidates(
    observation: &Observation,
    actor: &crate::cpu::VisibleActor,
    current: CellId,
    waypoint: CellId,
) -> Option<[Option<TeleporterCandidate>; 2]> {
    let desired = direction(current, waypoint)?;
    let movement = candidate(desired, waypoint);
    let correction = match ordinary_turn_alignment(
        actor,
        current,
        desired,
        observation.parts().rules.settings.turn_assist_units() as u32,
    ) {
        TurnAlignment::Ready => None,
        TurnAlignment::Correct(direction) => Some(candidate(direction, current)),
    };
    Some(match correction {
        Some(correction) if correction != movement => [Some(correction), Some(movement)],
        _ => [Some(movement), None],
    })
}

fn final_candidates(
    observation: &Observation,
    actor: &crate::cpu::VisibleActor,
    current: CellId,
    target: CellId,
) -> Option<[Option<TeleporterCandidate>; 2]> {
    let desired = direction(current, target)?;
    let movement = candidate(desired, target);
    let tolerance = observation.parts().rules.settings.turn_assist_units() as u32;
    let correction = alignment_direction(actor, current, tolerance)
        .filter(|direction| *direction != desired)
        .map(|direction| candidate(direction, current));
    Some(match correction {
        // Once the stable neighboring approach is reached, always try the
        // target-specific step first. A center correction is only a fallback
        // if certification rejects that step; preferring the correction near
        // the pickup can alternate away from and back toward the same edge.
        Some(correction) => [Some(movement), Some(correction)],
        None => [Some(movement), None],
    })
}

const fn candidate(direction: crate::game::Direction, waypoint: CellId) -> TeleporterCandidate {
    TeleporterCandidate {
        movement: MovementIntent::Direction(direction),
        waypoint,
    }
}

fn add_ticks(tick: Tick, offset: u64) -> Tick {
    Tick(tick.0.saturating_add(offset))
}

#[cfg(test)]
#[path = "teleporter_guidance_tests.rs"]
mod tests;
