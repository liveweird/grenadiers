use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::policy::{GoalModes, PolicyError, UtilityMemory};
use crate::cpu::{CpuRng, Observation, WorkBudget};
use crate::game::Tick;

use super::contact_plant::ContactTopology;
use super::joint_refuge::FarmRefugeMode;
use super::steering::RefugeGuidance;
use super::{LairSurvivalPlan, PlantEscapeOrigin};

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
    goal_modes: GoalModes,
    counter_pressure: bool,
    pickup_progress: bool,
    refuge_guidance: RefugeGuidance,
    farm_refuges: FarmRefugeMode,
    farm_origin_recovery: bool,
    contact_topology: ContactTopology,
    avoid_pits: bool,
) -> Result<LairSurvivalPlan, PolicyError> {
    plan_inner(
        observation,
        memory,
        history,
        settled_stop_tick,
        origin,
        rng,
        work,
        persistent_burning_oil,
        goal_modes,
        counter_pressure,
        pickup_progress,
        refuge_guidance,
        farm_refuges,
        farm_origin_recovery,
        contact_topology,
        avoid_pits,
        false,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn plan_policy_v64(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: Option<&PlantEscapeOrigin>,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
    goal_modes: GoalModes,
) -> Result<LairSurvivalPlan, PolicyError> {
    plan_inner(
        observation,
        memory,
        history,
        settled_stop_tick,
        origin,
        rng,
        work,
        true,
        goal_modes,
        true,
        true,
        RefugeGuidance::CENTER_THREATENED_ROOT,
        FarmRefugeMode::BlockedAway,
        true,
        ContactTopology::BlockedBirths,
        true,
        true,
    )
}

#[allow(clippy::too_many_arguments)]
fn plan_inner(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: Option<&PlantEscapeOrigin>,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
    persistent_burning_oil: bool,
    goal_modes: GoalModes,
    counter_pressure: bool,
    pickup_progress: bool,
    refuge_guidance: RefugeGuidance,
    farm_refuges: FarmRefugeMode,
    farm_origin_recovery: bool,
    contact_topology: ContactTopology,
    avoid_pits: bool,
    initial_demolition: bool,
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
            goal_modes,
            counter_pressure,
            pickup_progress,
            refuge_guidance,
            avoid_pits,
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
        goal_modes,
        counter_pressure,
        pickup_progress,
        refuge_guidance,
        farm_refuges,
        farm_origin_recovery,
        contact_topology,
        avoid_pits,
        initial_demolition,
    )
}
