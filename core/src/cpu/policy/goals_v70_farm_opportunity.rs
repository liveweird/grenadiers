use crate::cpu::WorkBudget;

use super::super::super::{OpeningMode, StrategicObjectiveKind, StrategicTarget};
use super::Candidate;

pub(super) fn promote_fresh(
    mode: OpeningMode,
    baseline: Candidate,
    candidates: &[Candidate],
    excluded: Option<StrategicTarget>,
    work: &mut WorkBudget,
) -> Candidate {
    if mode != OpeningMode::FarmOpportunity
        || baseline.score != 4
        || baseline.objective != StrategicObjectiveKind::Attack
        || !matches!(baseline.target, StrategicTarget::Opponent { .. })
    {
        return baseline;
    }
    let allowance = work.remaining().min(512).min(work.remaining() / 16);
    let mut child = WorkBudget::new(allowance);
    let promoted = analyze(baseline, candidates, excluded, &mut child);
    let _ = work.try_take(child.spent());
    promoted.unwrap_or(baseline)
}

fn analyze(
    attack: Candidate,
    candidates: &[Candidate],
    excluded: Option<StrategicTarget>,
    work: &mut WorkBudget,
) -> Option<Candidate> {
    let mut collect = false;
    let mut farm = None;
    for candidate in candidates.iter().copied() {
        work.try_take(1).ok()?;
        if excluded == Some(candidate.target) {
            continue;
        }
        collect |= candidate.objective == StrategicObjectiveKind::Collect;
        if farm.is_none() && candidate.objective == StrategicObjectiveKind::Farm {
            farm = Some(candidate);
        }
    }
    if collect {
        return Some(attack);
    }
    let farm = farm?;
    let farm_value = u32::from(farm.score) * (u32::from(attack.distance) + 1);
    let attack_value = u32::from(attack.score) * (u32::from(farm.distance) + 1);
    Some(if farm_value > attack_value {
        farm
    } else {
        attack
    })
}

#[cfg(test)]
#[path = "goals_v70_farm_opportunity_tests.rs"]
mod tests;
