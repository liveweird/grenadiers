use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::policy::{PolicyError, UtilityMemory};
use crate::cpu::{CpuRng, Observation, WorkBudget};
use crate::game::Tick;

use super::{LairSurvivalPlan, plan_lairs_inner};
use crate::cpu::survival::PlantEscapeOrigin;

/// Rules-v8 policy. It certifies navigation and stopped handoff before trying
/// any optional tactic, so a failed tactic returns useful movement.
pub fn plan_lairs(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<LairSurvivalPlan, PolicyError> {
    plan_lairs_with_origin(
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
pub fn plan_lairs_with_origin(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: Option<&PlantEscapeOrigin>,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<LairSurvivalPlan, PolicyError> {
    plan_lairs_inner(
        observation,
        memory,
        history,
        settled_stop_tick,
        origin,
        rng,
        work,
        8,
        false,
        false,
        false,
        false,
        false,
    )
}

#[allow(clippy::too_many_arguments)]
pub(in crate::cpu::survival) fn plan_lairs_v23(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: Option<&PlantEscapeOrigin>,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<LairSurvivalPlan, PolicyError> {
    plan_lairs_inner(
        observation,
        memory,
        history,
        settled_stop_tick,
        origin,
        rng,
        work,
        9,
        true,
        true,
        false,
        false,
        false,
    )
}

#[allow(clippy::too_many_arguments)]
pub(in crate::cpu::survival) fn plan_lairs_v24(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: Option<&PlantEscapeOrigin>,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<LairSurvivalPlan, PolicyError> {
    plan_lairs_inner(
        observation,
        memory,
        history,
        settled_stop_tick,
        origin,
        rng,
        work,
        9,
        true,
        true,
        true,
        false,
        false,
    )
}

#[allow(clippy::too_many_arguments)]
pub(in crate::cpu::survival) fn plan_lairs_v25(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: Option<&PlantEscapeOrigin>,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<LairSurvivalPlan, PolicyError> {
    plan_lairs_inner(
        observation,
        memory,
        history,
        settled_stop_tick,
        origin,
        rng,
        work,
        9,
        true,
        true,
        true,
        true,
        false,
    )
}
