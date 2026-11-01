use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::policy::{PolicyError, UtilityMemory};
use crate::cpu::{CpuRng, Observation, WorkBudget};
use crate::game::Tick;

use super::{LairSurvivalPlan, PlantEscapeOrigin};

pub fn plan_respawn(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<LairSurvivalPlan, PolicyError> {
    plan_respawn_with_origin(
        observation,
        memory,
        history,
        settled_stop_tick,
        None,
        rng,
        work,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn plan_respawn_with_origin(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: Option<&PlantEscapeOrigin>,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<LairSurvivalPlan, PolicyError> {
    plan_respawn_policy(
        observation,
        memory,
        history,
        settled_stop_tick,
        origin,
        rng,
        work,
        false,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn plan_respawn_v22_with_origin(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: Option<&PlantEscapeOrigin>,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<LairSurvivalPlan, PolicyError> {
    plan_respawn_policy(
        observation,
        memory,
        history,
        settled_stop_tick,
        origin,
        rng,
        work,
        true,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn plan_hard_v28_with_origin(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: Option<&PlantEscapeOrigin>,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<LairSurvivalPlan, PolicyError> {
    plan_spike_policy(
        observation,
        memory,
        history,
        settled_stop_tick,
        origin,
        rng,
        work,
        true,
        true,
        false,
        false,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn plan_counter_pressure_v29_with_origin(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: Option<&PlantEscapeOrigin>,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<LairSurvivalPlan, PolicyError> {
    plan_spike_policy(
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
pub fn plan_respawn_v23_with_origin(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: Option<&PlantEscapeOrigin>,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<LairSurvivalPlan, PolicyError> {
    if observation.parts().rules.version != 9 {
        return Err(PolicyError::InvalidObservation);
    }
    if observation.respawn_lairs().is_none() {
        return super::respawn_disabled::plan_v23(
            observation,
            memory,
            history,
            settled_stop_tick,
            origin,
            rng,
            work,
        )
        .map(|survival| LairSurvivalPlan {
            survival,
            contact: None,
        });
    }
    super::lair_planner::plan_lairs_v23(
        observation,
        memory,
        history,
        settled_stop_tick,
        origin,
        rng,
        work,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn plan_respawn_v24_with_origin(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: Option<&PlantEscapeOrigin>,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<LairSurvivalPlan, PolicyError> {
    if observation.parts().rules.version != 9 {
        return Err(PolicyError::InvalidObservation);
    }
    if observation.respawn_lairs().is_none() {
        return super::respawn_disabled::plan_v24(
            observation,
            memory,
            history,
            settled_stop_tick,
            origin,
            rng,
            work,
        )
        .map(|survival| LairSurvivalPlan {
            survival,
            contact: None,
        });
    }
    super::lair_planner::plan_lairs_v24(
        observation,
        memory,
        history,
        settled_stop_tick,
        origin,
        rng,
        work,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn plan_respawn_v25_with_origin(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: Option<&PlantEscapeOrigin>,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<LairSurvivalPlan, PolicyError> {
    if observation.parts().rules.version != 9 {
        return Err(PolicyError::InvalidObservation);
    }
    if observation.respawn_lairs().is_none() {
        return super::respawn_disabled::plan_v25(
            observation,
            memory,
            history,
            settled_stop_tick,
            origin,
            rng,
            work,
        )
        .map(|survival| LairSurvivalPlan {
            survival,
            contact: None,
        });
    }
    super::lair_planner::plan_lairs_v25(
        observation,
        memory,
        history,
        settled_stop_tick,
        origin,
        rng,
        work,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn plan_spike_pits_v26_with_origin(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: Option<&PlantEscapeOrigin>,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<LairSurvivalPlan, PolicyError> {
    plan_spike_policy(
        observation,
        memory,
        history,
        settled_stop_tick,
        origin,
        rng,
        work,
        false,
        false,
        false,
        false,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn plan_reliable_v27_with_origin(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: Option<&PlantEscapeOrigin>,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<LairSurvivalPlan, PolicyError> {
    plan_spike_policy(
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
        false,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn plan_spike_policy(
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
) -> Result<LairSurvivalPlan, PolicyError> {
    super::refuge_guidance::plan_policy(
        observation,
        memory,
        history,
        settled_stop_tick,
        origin,
        rng,
        work,
        persistent_burning_oil,
        hard_tactics,
        counter_pressure,
        pickup_progress,
        super::steering::RefugeGuidance::default(),
        false,
        false,
    )
}

#[allow(clippy::too_many_arguments)]
fn plan_respawn_policy(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: Option<&PlantEscapeOrigin>,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
    pickup_handoff: bool,
) -> Result<LairSurvivalPlan, PolicyError> {
    if observation.parts().rules.version != 9 {
        return Err(PolicyError::InvalidObservation);
    }
    if observation.respawn_lairs().is_none() {
        return super::respawn_disabled::plan(
            observation,
            memory,
            history,
            settled_stop_tick,
            origin,
            rng,
            work,
            pickup_handoff,
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
        9,
        pickup_handoff,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        super::steering::RefugeGuidance::default(),
        false,
        false,
    )
}
