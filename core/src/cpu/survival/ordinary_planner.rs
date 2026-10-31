use crate::cpu::policy::{ControlPlan, PolicyError, UtilityMemory};
use crate::cpu::{CpuRng, Observation, WorkBudget};

#[allow(clippy::too_many_arguments)]
pub(super) fn plan(
    observation: &Observation,
    memory: &UtilityMemory,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
    respawn: bool,
    opportunistic_planting: bool,
    strategic_goals: bool,
    failed_plant_recovery: bool,
) -> Result<ControlPlan, PolicyError> {
    if failed_plant_recovery {
        crate::cpu::policy::plan_respawn_v25(observation, memory, rng, work)
    } else if strategic_goals {
        crate::cpu::policy::plan_respawn_v24(observation, memory, rng, work)
    } else if opportunistic_planting {
        crate::cpu::policy::plan_respawn_v23(observation, memory, rng, work)
    } else if respawn {
        crate::cpu::policy::plan_respawn(observation, memory, rng, work)
    } else {
        crate::cpu::policy::plan(observation, memory, rng, work)
    }
}
