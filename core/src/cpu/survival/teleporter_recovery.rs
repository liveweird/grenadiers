use crate::cpu::forecast::{ForecastLimits, ObservedHazardMemory, forecast};
use crate::cpu::policy::{UtilityMemory, teleporter_guidance, teleporter_guidance_v50};
use crate::cpu::routing::{
    TeleporterStepRequest, certify_teleporter_step, certify_teleporter_step_v50,
};
use crate::cpu::{Observation, WorkBudget};
use crate::game::Tick;

use super::{LairSurvivalPlan, PlantEscapeOrigin, TeleporterStepPlan};

const TELEPORTER_RECOVERY_CAP: u32 = 32_768;

pub enum TeleporterRecoveryChoice {
    Ordinary(Box<LairSurvivalPlan>),
    Step(Box<TeleporterStepPlan>),
}

#[derive(Clone, Copy)]
enum RecoveryMode {
    Legacy,
    V50,
}

#[allow(clippy::too_many_arguments)]
pub fn select_teleporter_recovery(
    observation: &Observation,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    memory: &UtilityMemory,
    origin: Option<&PlantEscapeOrigin>,
    fallback: LairSurvivalPlan,
    work: &mut WorkBudget,
) -> TeleporterRecoveryChoice {
    select(
        observation,
        history,
        settled_stop_tick,
        memory,
        origin,
        fallback,
        work,
        RecoveryMode::Legacy,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn select_teleporter_recovery_v50(
    observation: &Observation,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    memory: &UtilityMemory,
    origin: Option<&PlantEscapeOrigin>,
    fallback: LairSurvivalPlan,
    work: &mut WorkBudget,
) -> TeleporterRecoveryChoice {
    select(
        observation,
        history,
        settled_stop_tick,
        memory,
        origin,
        fallback,
        work,
        RecoveryMode::V50,
    )
}

#[allow(clippy::too_many_arguments)]
fn select(
    observation: &Observation,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    memory: &UtilityMemory,
    origin: Option<&PlantEscapeOrigin>,
    fallback: LairSurvivalPlan,
    work: &mut WorkBudget,
    mode: RecoveryMode,
) -> TeleporterRecoveryChoice {
    if origin.is_some() || work.remaining() == 0 {
        return TeleporterRecoveryChoice::Ordinary(Box::new(fallback));
    }

    let mut recovery_work = WorkBudget::new(work.remaining().min(TELEPORTER_RECOVERY_CAP));
    let guidance = match mode {
        RecoveryMode::Legacy => teleporter_guidance(observation, memory, &mut recovery_work),
        RecoveryMode::V50 => teleporter_guidance_v50(observation, memory, &mut recovery_work),
    };
    let Some(guidance) = guidance else {
        charge_shared(work, recovery_work);
        return TeleporterRecoveryChoice::Ordinary(Box::new(fallback));
    };
    let Ok(hazards) = forecast(
        observation,
        history,
        None,
        ForecastLimits::default(),
        &mut recovery_work,
    ) else {
        charge_shared(work, recovery_work);
        return TeleporterRecoveryChoice::Ordinary(Box::new(fallback));
    };

    // Candidate generation, navigation and the single forecast are complete.
    // Freeze these two slots before proof attempts so a failure cannot reopen
    // search or multiply the bounded allowance.
    let candidates = guidance.candidates;
    let candidate_count = candidates.iter().flatten().count();
    let allowances = candidate_allowances(recovery_work.remaining(), candidate_count);
    for (candidate, allowance) in candidates.into_iter().flatten().zip(allowances) {
        let mut candidate_work = WorkBudget::new(allowance);
        let request = TeleporterStepRequest {
            target: guidance.target,
            movement: candidate.movement,
            settled_stop_tick,
        };
        let step = match mode {
            RecoveryMode::Legacy => {
                certify_teleporter_step(observation, &hazards, request, &mut candidate_work)
            }
            RecoveryMode::V50 => {
                certify_teleporter_step_v50(observation, &hazards, request, &mut candidate_work)
            }
        }
        .ok()
        .filter(|step| {
            step.nominal_makes_progress_toward(candidate.waypoint, &mut candidate_work)
                .unwrap_or(false)
        });
        recovery_work
            .try_take(candidate_work.spent())
            .expect("frozen candidate slots fit the recovery remainder");
        if let Some(step) = step {
            charge_shared(work, recovery_work);
            return TeleporterRecoveryChoice::Step(Box::new(TeleporterStepPlan {
                step,
                next_memory: guidance.next_memory,
            }));
        }
    }

    charge_shared(work, recovery_work);
    TeleporterRecoveryChoice::Ordinary(Box::new(fallback))
}

fn candidate_allowances(remaining: u32, count: usize) -> [u32; 2] {
    match count {
        0 => [0, 0],
        1 => [remaining, 0],
        _ => {
            let first = remaining / 2;
            [first, remaining - first]
        }
    }
}

fn charge_shared(shared: &mut WorkBudget, recovery: WorkBudget) {
    shared
        .try_take(recovery.spent())
        .expect("recovery allowance was capped to remaining shared work");
}

#[cfg(test)]
#[path = "teleporter_recovery_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "teleporter_recovery_v50_tests.rs"]
mod v50_tests;
