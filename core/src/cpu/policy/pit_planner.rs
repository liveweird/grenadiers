use crate::cpu::{CpuRng, Observation, VisibleActor, WorkBudget};
use crate::game::CellId;

use super::planner::{plan_inner, version::PlannerVersion};
use super::{ControlPlan, PolicyError, UtilityMemory};

#[allow(clippy::too_many_arguments)]
pub(super) fn select_guidance(
    observation: &Observation,
    actor: &VisibleActor,
    current: CellId,
    memory: &UtilityMemory,
    work: &mut WorkBudget,
    pressure_mode: super::PressureMode,
    farm_origin_recovery: bool,
) -> Option<super::goals::v24::GuidanceV24> {
    super::goals::v24::select_pits(
        observation,
        actor,
        current,
        memory,
        work,
        pressure_mode,
        farm_origin_recovery,
    )
}

pub fn plan_farm_pits_v52(
    observation: &Observation,
    memory: &UtilityMemory,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<ControlPlan, PolicyError> {
    plan_rules10(observation, memory, rng, work, PlannerVersion::FarmPitsV52)
}

pub fn plan_hard_farm_pits_v53(
    observation: &Observation,
    memory: &UtilityMemory,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<ControlPlan, PolicyError> {
    plan_rules10(
        observation,
        memory,
        rng,
        work,
        PlannerVersion::HardFarmPitsV53,
    )
}

pub fn plan_spike_pits_cautious_v52(
    observation: &Observation,
    memory: &UtilityMemory,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<ControlPlan, PolicyError> {
    plan_rules10(
        observation,
        memory,
        rng,
        work,
        PlannerVersion::SpikePitsCautiousV52,
    )
}

pub fn plan_hard_pits_cautious_v53(
    observation: &Observation,
    memory: &UtilityMemory,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<ControlPlan, PolicyError> {
    plan_rules10(
        observation,
        memory,
        rng,
        work,
        PlannerVersion::HardPitsCautiousV53,
    )
}

pub(crate) fn plan_hard_farm_exit_pressure_v58(
    observation: &Observation,
    memory: &UtilityMemory,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<ControlPlan, PolicyError> {
    plan_rules10(
        observation,
        memory,
        rng,
        work,
        PlannerVersion::HardFarmExitPressureV58,
    )
}

pub(crate) fn plan_hard_pits_exit_pressure_v58(
    observation: &Observation,
    memory: &UtilityMemory,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<ControlPlan, PolicyError> {
    plan_rules10(
        observation,
        memory,
        rng,
        work,
        PlannerVersion::HardPitsExitPressureV58,
    )
}

fn plan_rules10(
    observation: &Observation,
    memory: &UtilityMemory,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
    version: PlannerVersion,
) -> Result<ControlPlan, PolicyError> {
    if observation.parts().rules.version != 10 {
        return Err(PolicyError::InvalidObservation);
    }
    plan_inner(observation, memory, rng, work, version)
}
