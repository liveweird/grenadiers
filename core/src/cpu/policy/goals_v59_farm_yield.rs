use crate::cpu::{BombKnowledge, Observation, WorkBudget};
use crate::game::{CellId, PickupKind};

use super::pressure::{PressureMaps, PressureRemainder};
use super::{Candidate, cell_index};
use crate::cpu::policy::movement::{STEPS, offset};
use crate::cpu::policy::{StrategicIntent, StrategicObjectiveKind, StrategicTarget};

pub(super) fn retain_previous(
    observation: &Observation,
    previous: StrategicIntent,
    baseline: Candidate,
    maps: PressureMaps<'_>,
    failed_approach: Option<CellId>,
) -> Candidate {
    if previous.objective != StrategicObjectiveKind::Farm
        || previous.target != baseline.target
        || !matches!(baseline.target, StrategicTarget::Block { .. })
    {
        return baseline;
    }
    let Some(previous_approach) = previous.approach_cell else {
        return baseline;
    };
    eligible_candidate(
        observation,
        baseline,
        previous_approach,
        maps,
        failed_approach,
    )
    .filter(|candidate| candidate.distance == baseline.distance)
    .unwrap_or(baseline)
}

pub(super) fn promote_fresh(
    observation: &Observation,
    baseline: Candidate,
    maps: PressureMaps<'_>,
    failed_approach: Option<CellId>,
    remainder: &mut PressureRemainder<'_>,
    parent: &mut WorkBudget,
) -> Candidate {
    let original = baseline;
    let StrategicTarget::Block { cell: target } = baseline.target else {
        return baseline;
    };
    if baseline.objective != StrategicObjectiveKind::Farm
        || !supported_scene(observation, &mut remainder.work)
    {
        let _ = remainder.charge_delta(parent);
        return baseline;
    }
    let mut origins = STEPS
        .into_iter()
        .filter_map(|(_, dc, dr)| offset(target, dc, dr))
        .filter_map(|origin| {
            eligible_candidate(observation, baseline, origin, maps, failed_approach)
        })
        .filter(|candidate| candidate.distance == baseline.distance)
        .collect::<Vec<_>>();
    origins.sort_unstable_by_key(|candidate| (candidate.approach.row, candidate.approach.column));
    let Some(mut best_yield) = remainder.geometry.visible_block_yield(
        baseline.approach,
        remainder.range,
        remainder.kind,
        target,
        &mut remainder.work,
    ) else {
        let _ = remainder.charge_delta(parent);
        return original;
    };
    let mut best = baseline;
    for candidate in origins {
        if candidate.approach == baseline.approach {
            continue;
        }
        let Some(candidate_yield) = remainder.geometry.visible_block_yield(
            candidate.approach,
            remainder.range,
            remainder.kind,
            target,
            &mut remainder.work,
        ) else {
            let _ = remainder.charge_delta(parent);
            return original;
        };
        if candidate_yield > best_yield {
            best = candidate;
            best_yield = candidate_yield;
        }
    }
    if remainder.charge_delta(parent).is_none() {
        return original;
    }
    best
}

fn eligible_candidate(
    observation: &Observation,
    baseline: Candidate,
    approach: CellId,
    maps: PressureMaps<'_>,
    failed_approach: Option<CellId>,
) -> Option<Candidate> {
    if failed_approach == Some(approach)
        || maps.pits.contains(observation, approach)
        || !maps.search.transitable(observation, approach)
        || observation
            .parts()
            .pickups
            .iter()
            .any(|pickup| pickup.cell == approach)
    {
        return None;
    }
    let index = cell_index(observation, approach)?;
    Some(Candidate {
        approach,
        distance: maps.distance[index]?,
        first_step: maps.first_step[index]?,
        ..baseline
    })
}

fn supported_scene(observation: &Observation, work: &mut WorkBudget) -> bool {
    let parts = observation.parts();
    if !matches!(&parts.bombs, BombKnowledge::Visible(bombs) if bombs.is_empty())
        || !parts.flames.is_empty()
        || !parts.oil.is_empty()
    {
        return false;
    }
    for pickup in &parts.pickups {
        if work.try_take(1).is_err() || pickup.kind == PickupKind::Teleporter {
            return false;
        }
    }
    true
}

#[cfg(test)]
#[path = "goals_v59_farm_yield_tests.rs"]
mod tests;
