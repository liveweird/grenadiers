use std::collections::VecDeque;

use crate::cpu::{Observation, VisibleActor, WorkBudget};
use crate::game::arena::ARENA_CELL_COUNT;
use crate::game::{CellId, Position, Tick};

use super::super::movement::{STEPS, offset};
use super::super::{
    FAILED_TARGET_DEFER_TICKS, STRATEGIC_COMMITMENT_TICKS, STRATEGIC_PROGRESS_TICKS,
    StrategicIntent, StrategicObjectiveKind, StrategicPhase, StrategicSwitchReason,
    StrategicTarget, UtilityMemory,
};
use super::{cell_index, pickup_value, search_map};

#[path = "goals_v28_pressure.rs"]
mod pressure;

#[path = "goals_v24_order.rs"]
mod order;
use order::{add_ticks, fallback, position_distance, target_order};

#[path = "goals_v24_validation.rs"]
mod validation;
use validation::validate_previous;

#[path = "goals_v24_farm.rs"]
mod farm;

#[path = "goals_v24_versions.rs"]
mod versions;
pub(in crate::cpu::policy) use versions::{
    select, select_pits, select_v25, select_v28, select_v38,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::cpu::policy) struct GuidanceV24 {
    pub first_step: CellId,
    pub intent: StrategicIntent,
    pub failed_target: Option<(StrategicTarget, Tick)>,
}

#[derive(Clone, Copy)]
struct Candidate {
    objective: StrategicObjectiveKind,
    target: StrategicTarget,
    approach: CellId,
    score: u8,
    distance: u16,
    first_step: CellId,
}

pub(in crate::cpu::policy) type DistanceMap = [Option<u16>; ARENA_CELL_COUNT];
pub(in crate::cpu::policy) type FirstStepMap = [Option<CellId>; ARENA_CELL_COUNT];

#[allow(
    clippy::too_many_arguments,
    reason = "Keep archived goal and opt-in recovery switches explicit"
)]
fn select_inner(
    observation: &Observation,
    actor: &VisibleActor,
    current: CellId,
    memory: &UtilityMemory,
    work: &mut WorkBudget,
    failed_plant_recovery: bool,
    range_aware_pressure: bool,
    farm_origin_recovery: bool,
    avoid_pits: bool,
) -> Option<GuidanceV24> {
    let mut map = search_map(observation, actor, current, !range_aware_pressure, work)?;
    let pit_sites = if avoid_pits {
        super::super::pit_avoidance::block_search_map(observation, &mut map, work)?
    } else {
        super::super::pit_avoidance::PitMask::EMPTY
    };
    let (distance, first_step) = reachable(observation, current, &map, work)?;
    let mut candidates = Vec::new();
    for pickup in &observation.parts().pickups {
        if pit_sites.contains(observation, pickup.cell) {
            continue;
        }
        let index = cell_index(observation, pickup.cell)?;
        let Some(score) = pickup_value(observation, actor, pickup.kind) else {
            continue;
        };
        if let Some(distance) = distance[index] {
            candidates.push(Candidate {
                objective: StrategicObjectiveKind::Collect,
                target: StrategicTarget::Pickup {
                    cell: pickup.cell,
                    kind: pickup.kind,
                },
                approach: pickup.cell,
                score,
                distance,
                first_step: first_step[index]?,
            });
        }
    }
    let failed_farm_approach = farm_origin_recovery
        .then(|| memory.failed_farm_approach())
        .flatten()
        .filter(|(_, until)| observation.parts().tick < *until)
        .map(|(cell, _)| cell);
    for &block in &observation.parts().arena.blocks {
        farm::add_firing_candidates(
            observation,
            block,
            StrategicObjectiveKind::Farm,
            StrategicTarget::Block { cell: block },
            2,
            &distance,
            &first_step,
            &pit_sites,
            &mut candidates,
            failed_farm_approach,
        );
    }
    if range_aware_pressure {
        candidates.extend(pressure::candidates(
            observation,
            actor,
            &distance,
            &first_step,
            &map,
            &pit_sites,
            work,
        ));
    }
    pressure::add_legacy_candidates(
        observation,
        actor,
        &distance,
        &first_step,
        &pit_sites,
        &mut candidates,
    );
    candidates.sort_unstable_by_key(|candidate| {
        (
            std::cmp::Reverse(candidate.score),
            candidate.distance,
            candidate.approach.row,
            candidate.approach.column,
            target_order(candidate.target),
        )
    });
    candidates.dedup_by_key(|candidate| candidate.target);
    let retained_failure = memory
        .failed_target()
        .filter(|(_, until)| observation.parts().tick < *until);
    if let Some((failed, _)) = retained_failure {
        candidates.retain(|candidate| candidate.target != failed);
    }

    let previous = memory.strategic_intent();
    let mut reason = StrategicSwitchReason::Initial;
    let chosen = if let Some(previous) = previous {
        let valid = validate_previous(observation, previous.target);
        let retained = candidates
            .iter()
            .copied()
            .find(|candidate| candidate.target == previous.target);
        if let Some(retained) = retained.filter(|_| valid.is_ok()) {
            let progressed = retained.distance < previous.last_distance
                || previous.approach_cell.is_some_and(|approach| {
                    actor.position != previous.last_cpu_position
                        && position_distance(actor.position, Position::center(approach))
                            < position_distance(
                                previous.last_cpu_position,
                                Position::center(approach),
                            )
                });
            let suspended = matches!(
                previous.phase,
                StrategicPhase::Plant
                    | StrategicPhase::Escape
                    | StrategicPhase::Evade
                    | StrategicPhase::Waiting
            );
            let stalled =
                !suspended && observation.parts().tick >= previous.progress_deadline && !progressed;
            let better = candidates.first().copied().filter(|candidate| {
                candidate.target != previous.target
                    && observation.parts().tick >= previous.committed_until
                    && candidate.score >= previous.score.saturating_add(2)
            });
            if stalled {
                reason = StrategicSwitchReason::Stalled;
                candidates
                    .iter()
                    .copied()
                    .find(|candidate| candidate.target != previous.target)
                    .or(Some(retained))
            } else if let Some(better) = better {
                reason = StrategicSwitchReason::BetterOpportunity;
                Some(better)
            } else {
                return Some(GuidanceV24 {
                    first_step: if previous.phase == StrategicPhase::Waiting {
                        current
                    } else {
                        retained.first_step
                    },
                    failed_target: retained_failure,
                    intent: StrategicIntent {
                        approach_cell: Some(retained.approach),
                        observation_tick: observation.parts().tick,
                        last_distance: retained.distance,
                        last_cpu_position: actor.position,
                        progress_deadline: if progressed {
                            add_ticks(observation.parts().tick, STRATEGIC_PROGRESS_TICKS)
                        } else {
                            previous.progress_deadline
                        },
                        phase: match previous.phase {
                            StrategicPhase::Reengage => StrategicPhase::Approach,
                            StrategicPhase::Evade => StrategicPhase::Reengage,
                            phase => phase,
                        },
                        last_switch_reason: if previous.phase == StrategicPhase::Evade {
                            StrategicSwitchReason::Resumed
                        } else {
                            previous.last_switch_reason
                        },
                        ..previous
                    },
                });
            }
        } else {
            reason = if failed_plant_recovery
                && retained_failure.is_some_and(|(target, _)| target == previous.target)
            {
                StrategicSwitchReason::Stalled
            } else {
                valid.err().unwrap_or(StrategicSwitchReason::TargetInvalid)
            };
            candidates.first().copied()
        }
    } else {
        candidates.first().copied()
    };

    if let Some(chosen) = chosen {
        let tick = observation.parts().tick;
        return Some(GuidanceV24 {
            first_step: chosen.first_step,
            failed_target: if reason == StrategicSwitchReason::Stalled {
                previous.map(|intent| (intent.target, add_ticks(tick, FAILED_TARGET_DEFER_TICKS)))
            } else {
                retained_failure
            },
            intent: StrategicIntent {
                objective: chosen.objective,
                target: chosen.target,
                approach_cell: Some(chosen.approach),
                phase: StrategicPhase::Approach,
                last_switch_reason: reason,
                observation_tick: tick,
                score: chosen.score,
                committed_until: add_ticks(tick, STRATEGIC_COMMITMENT_TICKS),
                progress_deadline: add_ticks(tick, STRATEGIC_PROGRESS_TICKS),
                last_distance: chosen.distance,
                last_cpu_position: actor.position,
                deferred_until: (reason == StrategicSwitchReason::Stalled)
                    .then(|| add_ticks(tick, FAILED_TARGET_DEFER_TICKS)),
            },
        });
    }

    let tick = observation.parts().tick;
    let fallback = fallback(current, observation, &map, work);
    Some(GuidanceV24 {
        first_step: fallback.unwrap_or(current),
        failed_target: retained_failure,
        intent: StrategicIntent {
            objective: if fallback.is_some() {
                StrategicObjectiveKind::Explore
            } else {
                StrategicObjectiveKind::Wait
            },
            target: fallback.map_or(StrategicTarget::None, |cell| StrategicTarget::Tile { cell }),
            approach_cell: fallback,
            phase: if fallback.is_some() {
                StrategicPhase::Approach
            } else {
                StrategicPhase::Waiting
            },
            last_switch_reason: if reason == StrategicSwitchReason::Initial {
                StrategicSwitchReason::NoTargets
            } else {
                reason
            },
            observation_tick: tick,
            score: 0,
            committed_until: add_ticks(tick, STRATEGIC_COMMITMENT_TICKS),
            progress_deadline: add_ticks(tick, STRATEGIC_PROGRESS_TICKS),
            last_distance: u16::from(fallback.is_some()),
            last_cpu_position: actor.position,
            deferred_until: None,
        },
    })
}

pub(in crate::cpu::policy) fn reachable(
    observation: &Observation,
    current: CellId,
    map: &super::SearchMap,
    work: &mut WorkBudget,
) -> Option<(DistanceMap, FirstStepMap)> {
    let mut distance: [Option<u16>; ARENA_CELL_COUNT] = [None; ARENA_CELL_COUNT];
    let mut first_step = [None; ARENA_CELL_COUNT];
    let start = cell_index(observation, current)?;
    distance[start] = Some(0);
    first_step[start] = Some(current);
    let mut queue = VecDeque::from([current]);
    while let Some(cell) = queue.pop_front() {
        work.try_take(1).ok()?;
        let index = cell_index(observation, cell)?;
        let next_distance = distance[index]?.saturating_add(1);
        for (_, dc, dr) in STEPS {
            let Some(next) = offset(cell, dc, dr) else {
                continue;
            };
            let Some(next_index) = cell_index(observation, next) else {
                continue;
            };
            if distance[next_index].is_none() && map.transitable(observation, next) {
                distance[next_index] = Some(next_distance);
                first_step[next_index] = Some(if cell == current {
                    next
                } else {
                    first_step[index]?
                });
                queue.push_back(next);
            }
        }
    }
    Some((distance, first_step))
}

#[cfg(test)]
#[path = "goals_v24_tests.rs"]
mod tests;
