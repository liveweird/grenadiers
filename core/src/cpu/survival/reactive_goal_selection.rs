use crate::cpu::policy::{
    ControlPlan, GoalModes, OpeningMode, PolicyError, ProgressMode, UtilityMemory,
};
use crate::cpu::{CpuRng, Observation, WorkBudget};

pub(super) fn plan(
    observation: &Observation,
    memory: &UtilityMemory,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
    modes: GoalModes,
) -> Result<ControlPlan, PolicyError> {
    if modes.opening == OpeningMode::FarmOpportunity {
        crate::cpu::policy::plan_hard_opportunity_v70(observation, memory, rng, work)
    } else if modes.progress == ProgressMode::ActorMovement {
        crate::cpu::policy::plan_hard_progress_v69(observation, memory, rng, work)
    } else if modes.pickup == crate::cpu::policy::PickupMode::VisibleContention {
        crate::cpu::policy::plan_hard_pickup_contention_v66(observation, memory, rng, work)
    } else {
        crate::cpu::policy::plan_hard_farm_yield_v59(observation, memory, rng, work)
    }
}
