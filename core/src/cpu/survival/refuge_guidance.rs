use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::policy::{GoalModes, PolicyError, PressureMode, UtilityMemory};
use crate::cpu::{CpuRng, Observation, WorkBudget};
use crate::game::Tick;

use super::contact_plant::ContactTopology;
use super::joint_refuge::FarmRefugeMode;
pub(super) use super::refuge_guidance_policy::plan_policy;
use super::steering::RefugeGuidance;
use super::{LairSurvivalPlan, PlantEscapeOrigin};

#[allow(clippy::too_many_arguments)]
pub fn plan_refuge_guidance_v33_with_origin(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: Option<&PlantEscapeOrigin>,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<LairSurvivalPlan, PolicyError> {
    plan_policy(
        observation,
        memory,
        history,
        settled_stop_tick,
        origin,
        rng,
        work,
        true,
        GoalModes::nearest(PressureMode::Disabled),
        false,
        true,
        RefugeGuidance::CENTER_THREATENED_ROOT,
        FarmRefugeMode::Disabled,
        false,
        ContactTopology::Disabled,
        false,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn plan_refuge_guidance_v34_with_origin(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: Option<&PlantEscapeOrigin>,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<LairSurvivalPlan, PolicyError> {
    plan_policy(
        observation,
        memory,
        history,
        settled_stop_tick,
        origin,
        rng,
        work,
        true,
        GoalModes::nearest(PressureMode::Baseline),
        true,
        true,
        RefugeGuidance::CENTER_THREATENED_ROOT,
        FarmRefugeMode::Disabled,
        false,
        ContactTopology::Disabled,
        false,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn plan_farm_refuges_v36_with_origin(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: Option<&PlantEscapeOrigin>,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<LairSurvivalPlan, PolicyError> {
    plan_policy(
        observation,
        memory,
        history,
        settled_stop_tick,
        origin,
        rng,
        work,
        true,
        GoalModes::nearest(PressureMode::Disabled),
        false,
        true,
        RefugeGuidance::CENTER_THREATENED_ROOT,
        FarmRefugeMode::Legacy,
        false,
        ContactTopology::Disabled,
        false,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn plan_hard_farm_refuges_v37_with_origin(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: Option<&PlantEscapeOrigin>,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<LairSurvivalPlan, PolicyError> {
    plan_policy(
        observation,
        memory,
        history,
        settled_stop_tick,
        origin,
        rng,
        work,
        true,
        GoalModes::nearest(PressureMode::Baseline),
        true,
        true,
        RefugeGuidance::CENTER_THREATENED_ROOT,
        FarmRefugeMode::Legacy,
        false,
        ContactTopology::Disabled,
        false,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn plan_farm_origins_v38_with_origin(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: Option<&PlantEscapeOrigin>,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<LairSurvivalPlan, PolicyError> {
    plan_policy(
        observation,
        memory,
        history,
        settled_stop_tick,
        origin,
        rng,
        work,
        true,
        GoalModes::nearest(PressureMode::Disabled),
        false,
        true,
        RefugeGuidance::CENTER_THREATENED_ROOT,
        FarmRefugeMode::Legacy,
        true,
        ContactTopology::Disabled,
        false,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn plan_contact_topology_v40_with_origin(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: Option<&PlantEscapeOrigin>,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<LairSurvivalPlan, PolicyError> {
    plan_policy(
        observation,
        memory,
        history,
        settled_stop_tick,
        origin,
        rng,
        work,
        true,
        GoalModes::nearest(PressureMode::Disabled),
        false,
        true,
        RefugeGuidance::CENTER_THREATENED_ROOT,
        FarmRefugeMode::Legacy,
        true,
        ContactTopology::ExistingSources,
        false,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn plan_hard_contact_topology_v41_with_origin(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: Option<&PlantEscapeOrigin>,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<LairSurvivalPlan, PolicyError> {
    plan_policy(
        observation,
        memory,
        history,
        settled_stop_tick,
        origin,
        rng,
        work,
        true,
        GoalModes::nearest(PressureMode::Baseline),
        true,
        true,
        RefugeGuidance::CENTER_THREATENED_ROOT,
        FarmRefugeMode::Legacy,
        true,
        ContactTopology::ExistingSources,
        false,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn plan_blocked_births_v42_with_origin(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: Option<&PlantEscapeOrigin>,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<LairSurvivalPlan, PolicyError> {
    plan_policy(
        observation,
        memory,
        history,
        settled_stop_tick,
        origin,
        rng,
        work,
        true,
        GoalModes::nearest(PressureMode::Disabled),
        false,
        true,
        RefugeGuidance::CENTER_THREATENED_ROOT,
        FarmRefugeMode::Legacy,
        true,
        ContactTopology::BlockedBirths,
        false,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn plan_hard_blocked_births_v43_with_origin(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: Option<&PlantEscapeOrigin>,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<LairSurvivalPlan, PolicyError> {
    plan_policy(
        observation,
        memory,
        history,
        settled_stop_tick,
        origin,
        rng,
        work,
        true,
        GoalModes::nearest(PressureMode::Baseline),
        true,
        true,
        RefugeGuidance::CENTER_THREATENED_ROOT,
        FarmRefugeMode::Legacy,
        true,
        ContactTopology::BlockedBirths,
        false,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn plan_hard_farm_origins_v39_with_origin(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: Option<&PlantEscapeOrigin>,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<LairSurvivalPlan, PolicyError> {
    plan_policy(
        observation,
        memory,
        history,
        settled_stop_tick,
        origin,
        rng,
        work,
        true,
        GoalModes::nearest(PressureMode::Baseline),
        true,
        true,
        RefugeGuidance::CENTER_THREATENED_ROOT,
        FarmRefugeMode::Legacy,
        true,
        ContactTopology::Disabled,
        false,
    )
}
