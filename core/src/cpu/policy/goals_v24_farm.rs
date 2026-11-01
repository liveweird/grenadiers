use crate::cpu::Observation;
use crate::game::CellId;

use super::{Candidate, DistanceMap, FirstStepMap, cell_index};
use crate::cpu::policy::movement::{STEPS, offset};
use crate::cpu::policy::{StrategicObjectiveKind, StrategicTarget};

#[allow(clippy::too_many_arguments)]
pub(super) fn add_firing_candidates(
    observation: &Observation,
    target_cell: CellId,
    objective: StrategicObjectiveKind,
    target: StrategicTarget,
    score: u8,
    distance: &DistanceMap,
    first_step: &FirstStepMap,
    candidates: &mut Vec<Candidate>,
    failed_approach: Option<CellId>,
) {
    let mut best = None;
    for (_, dc, dr) in STEPS {
        let Some(approach) = offset(target_cell, dc, dr) else {
            continue;
        };
        let Some(index) = cell_index(observation, approach) else {
            continue;
        };
        if failed_approach == Some(approach) {
            continue;
        }
        let (Some(distance), Some(first_step)) = (distance[index], first_step[index]) else {
            continue;
        };
        let candidate = Candidate {
            objective,
            target,
            approach,
            score,
            distance,
            first_step,
        };
        if best.is_none_or(|old: Candidate| {
            (
                candidate.distance,
                candidate.approach.row,
                candidate.approach.column,
            ) < (old.distance, old.approach.row, old.approach.column)
        }) {
            best = Some(candidate);
        }
    }
    candidates.extend(best);
}
