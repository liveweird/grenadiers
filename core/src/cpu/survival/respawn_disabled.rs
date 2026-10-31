use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::policy::{PolicyError, UtilityMemory};
use crate::cpu::{CpuRng, Observation, WorkBudget};
use crate::game::Tick;

use super::{PlantEscapeOrigin, SurvivalPlan};

#[allow(clippy::too_many_arguments)]
pub(super) fn plan(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: Option<&PlantEscapeOrigin>,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
    pickup_handoff: bool,
) -> Result<SurvivalPlan, PolicyError> {
    if observation.parts().rules.version != 9 || observation.respawn_lairs().is_some() {
        return Err(PolicyError::InvalidObservation);
    }
    super::planner::plan_inner(
        observation,
        memory,
        history,
        settled_stop_tick,
        origin,
        rng,
        work,
        true,
        pickup_handoff,
        false,
        false,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn plan_v23(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: Option<&PlantEscapeOrigin>,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<SurvivalPlan, PolicyError> {
    if observation.parts().rules.version != 9 || observation.respawn_lairs().is_some() {
        return Err(PolicyError::InvalidObservation);
    }
    super::planner::plan_inner(
        observation,
        memory,
        history,
        settled_stop_tick,
        origin,
        rng,
        work,
        true,
        true,
        true,
        false,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn plan_v24(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: Option<&PlantEscapeOrigin>,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<SurvivalPlan, PolicyError> {
    if observation.parts().rules.version != 9 || observation.respawn_lairs().is_some() {
        return Err(PolicyError::InvalidObservation);
    }
    super::planner::plan_inner(
        observation,
        memory,
        history,
        settled_stop_tick,
        origin,
        rng,
        work,
        true,
        true,
        true,
        true,
    )
}
