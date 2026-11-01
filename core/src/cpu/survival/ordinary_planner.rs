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
    hard_tactics: bool,
    farm_origin_recovery: bool,
) -> Result<ControlPlan, PolicyError> {
    if farm_origin_recovery && hard_tactics {
        crate::cpu::policy::plan_hard_farm_v39(observation, memory, rng, work)
    } else if farm_origin_recovery {
        crate::cpu::policy::plan_farm_v38(observation, memory, rng, work)
    } else if hard_tactics {
        crate::cpu::policy::plan_hard_v28(observation, memory, rng, work)
    } else if observation.parts().rules.version == 10 {
        crate::cpu::policy::plan_spike_pits_v26(observation, memory, rng, work)
    } else if failed_plant_recovery {
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
