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
    )
}

#[allow(clippy::too_many_arguments)]
fn plan_spike_policy(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: Option<&PlantEscapeOrigin>,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
    persistent_burning_oil: bool,
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
    )
}
