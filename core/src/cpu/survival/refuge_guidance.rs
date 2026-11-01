use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::policy::{PolicyError, UtilityMemory};
use crate::cpu::{CpuRng, Observation, WorkBudget};
use crate::game::Tick;

use super::contact_plant::ContactTopology;
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
        false,
        false,
        true,
        RefugeGuidance::CENTER_THREATENED_ROOT,
        false,
        false,
        ContactTopology::Disabled,
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
        true,
        true,
        true,
        RefugeGuidance::CENTER_THREATENED_ROOT,
        false,
        false,
        ContactTopology::Disabled,
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
        false,
        false,
        true,
        RefugeGuidance::CENTER_THREATENED_ROOT,
        true,
        false,
        ContactTopology::Disabled,
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
        true,
        true,
        true,
        RefugeGuidance::CENTER_THREATENED_ROOT,
        true,
        false,
        ContactTopology::Disabled,
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
        false,
        false,
        true,
        RefugeGuidance::CENTER_THREATENED_ROOT,
        true,
        true,
        ContactTopology::Disabled,
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
        false,
        false,
        true,
        RefugeGuidance::CENTER_THREATENED_ROOT,
        true,
        true,
        ContactTopology::ExistingSources,
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
        true,
        true,
        true,
        RefugeGuidance::CENTER_THREATENED_ROOT,
        true,
        true,
        ContactTopology::ExistingSources,
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
        false,
        false,
        true,
        RefugeGuidance::CENTER_THREATENED_ROOT,
        true,
        true,
        ContactTopology::BlockedBirths,
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
        true,
        true,
        true,
        RefugeGuidance::CENTER_THREATENED_ROOT,
        true,
        true,
        ContactTopology::BlockedBirths,
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
        true,
        true,
        true,
        RefugeGuidance::CENTER_THREATENED_ROOT,
        true,
        true,
        ContactTopology::Disabled,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn plan_policy(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: Option<&PlantEscapeOrigin>,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
    persistent_burning_oil: bool,
    hard_tactics: bool,
    counter_pressure: bool,
    pickup_progress: bool,
    refuge_guidance: RefugeGuidance,
    joint_refuges: bool,
    farm_origin_recovery: bool,
    contact_topology: ContactTopology,
) -> Result<LairSurvivalPlan, PolicyError> {
    if observation.parts().rules.version != 10 {
        return Err(PolicyError::InvalidObservation);
    }
    if observation.respawn_lairs().is_none() {
        return super::planner::plan_inner(
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
            true,
            persistent_burning_oil,
            hard_tactics,
            counter_pressure,
            pickup_progress,
            refuge_guidance,
        )
        .map(|survival| LairSurvivalPlan {
            survival,
            contact: None,
        });
    }
    super::lair_planner::plan_lairs_inner(
        observation,
        memory,
        history,
        settled_stop_tick,
        origin,
        rng,
        work,
        10,
        true,
        true,
        true,
        true,
        persistent_burning_oil,
        hard_tactics,
        counter_pressure,
        pickup_progress,
        refuge_guidance,
        joint_refuges,
        farm_origin_recovery,
        contact_topology,
    )
}
