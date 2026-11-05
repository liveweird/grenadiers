use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::policy::{GoalModes, PolicyError, UtilityMemory};
use crate::cpu::{CpuRng, Observation, WorkBudget};
use crate::game::Tick;

use super::{LairSurvivalPlan, PlantEscapeOrigin};

#[allow(clippy::too_many_arguments)]
pub fn plan_hard_reactive_v65_with_origin(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: Option<&PlantEscapeOrigin>,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<LairSurvivalPlan, PolicyError> {
    super::reactive_plant::plan_hard_reactive_with_modes(
        observation,
        memory,
        history,
        settled_stop_tick,
        origin,
        rng,
        work,
        GoalModes::visible_yield(),
    )
}

#[allow(clippy::too_many_arguments)]
pub fn plan_hard_pickup_v66_with_origin(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: Option<&PlantEscapeOrigin>,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<LairSurvivalPlan, PolicyError> {
    super::reactive_plant::plan_hard_reactive_with_modes(
        observation,
        memory,
        history,
        settled_stop_tick,
        origin,
        rng,
        work,
        GoalModes::visible_yield_contention(),
    )
}
