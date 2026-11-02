use crate::cpu::policy::{
    ControlPlan, FarmOriginMode, GoalModes, PolicyError, PressureMode, UtilityMemory,
};
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
    goal_modes: GoalModes,
    farm_origin_recovery: bool,
    avoid_pits: bool,
) -> Result<ControlPlan, PolicyError> {
    let pressure_mode = goal_modes.pressure;
    if avoid_pits && farm_origin_recovery && goal_modes.farm_origin == FarmOriginMode::VisibleYield
    {
        crate::cpu::policy::plan_hard_farm_yield_v59(observation, memory, rng, work)
    } else if avoid_pits && goal_modes.farm_origin == FarmOriginMode::VisibleYield {
        crate::cpu::policy::plan_hard_pits_yield_v59(observation, memory, rng, work)
    } else if avoid_pits && farm_origin_recovery && pressure_mode == PressureMode::ExitCoverage {
        crate::cpu::policy::plan_hard_farm_exit_pressure_v58(observation, memory, rng, work)
    } else if avoid_pits && pressure_mode == PressureMode::ExitCoverage {
        crate::cpu::policy::plan_hard_pits_exit_pressure_v58(observation, memory, rng, work)
    } else if avoid_pits && farm_origin_recovery && pressure_mode.enabled() {
        crate::cpu::policy::plan_hard_farm_pits_v53(observation, memory, rng, work)
    } else if avoid_pits && farm_origin_recovery {
        crate::cpu::policy::plan_farm_pits_v52(observation, memory, rng, work)
    } else if avoid_pits && pressure_mode.enabled() {
        crate::cpu::policy::plan_hard_pits_cautious_v53(observation, memory, rng, work)
    } else if avoid_pits {
        crate::cpu::policy::plan_spike_pits_cautious_v52(observation, memory, rng, work)
    } else if farm_origin_recovery && pressure_mode.enabled() {
        crate::cpu::policy::plan_hard_farm_v39(observation, memory, rng, work)
    } else if farm_origin_recovery {
        crate::cpu::policy::plan_farm_v38(observation, memory, rng, work)
    } else if pressure_mode.enabled() {
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
