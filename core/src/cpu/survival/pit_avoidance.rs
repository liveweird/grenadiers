use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::policy::{GoalModes, PolicyError, PressureMode, UtilityMemory};
use crate::cpu::{CpuRng, Observation, WorkBudget};
use crate::game::Tick;

use super::contact_plant::ContactTopology;
use super::joint_refuge::FarmRefugeMode;
use super::steering::RefugeGuidance;
use super::{LairSurvivalPlan, PlantEscapeOrigin};

#[allow(clippy::too_many_arguments)]
pub fn plan_pit_avoidance_v52_with_origin(
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
pub fn plan_hard_pit_avoidance_v53_with_origin(
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
        FarmRefugeMode::Legacy,
        true,
        ContactTopology::BlockedBirths,
        true,
    )
}
