use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::policy::{GoalModes, PolicyError, PressureMode, UtilityMemory};
use crate::cpu::{CpuRng, Observation, WorkBudget};
use crate::game::Tick;

use super::contact_plant::ContactTopology;
use super::joint_refuge::FarmRefugeMode;
use super::steering::RefugeGuidance;
use super::{LairSurvivalPlan, PlantEscapeOrigin};

#[allow(clippy::too_many_arguments)]
pub fn plan_corner_refuges_v56_with_origin(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: Option<&PlantEscapeOrigin>,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<LairSurvivalPlan, PolicyError> {
    plan(
        observation,
        memory,
        history,
        settled_stop_tick,
        origin,
        rng,
        work,
        GoalModes::nearest(PressureMode::Disabled),
    )
}

#[allow(clippy::too_many_arguments)]
pub fn plan_hard_corner_refuges_v57_with_origin(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: Option<&PlantEscapeOrigin>,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<LairSurvivalPlan, PolicyError> {
    plan(
        observation,
        memory,
        history,
        settled_stop_tick,
        origin,
        rng,
        work,
        GoalModes::nearest(PressureMode::Baseline),
    )
}

#[allow(clippy::too_many_arguments)]
pub fn plan_hard_exit_pressure_v58_with_origin(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: Option<&PlantEscapeOrigin>,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<LairSurvivalPlan, PolicyError> {
    plan(
        observation,
        memory,
        history,
        settled_stop_tick,
        origin,
        rng,
        work,
        GoalModes::nearest(PressureMode::ExitCoverage),
    )
}

#[allow(clippy::too_many_arguments)]
pub fn plan_hard_farm_yield_v59_with_origin(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: Option<&PlantEscapeOrigin>,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<LairSurvivalPlan, PolicyError> {
    plan(
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
fn plan(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: Option<&PlantEscapeOrigin>,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
    goal_modes: GoalModes,
) -> Result<LairSurvivalPlan, PolicyError> {
    super::refuge_guidance::plan_policy(
        observation,
        memory,
        history,
        settled_stop_tick,
        origin,
        rng,
        work,
        true,
        goal_modes,
        goal_modes.pressure.enabled(),
        true,
        RefugeGuidance::CENTER_THREATENED_ROOT,
        FarmRefugeMode::BlockedAway,
        true,
        ContactTopology::BlockedBirths,
        true,
    )
}
