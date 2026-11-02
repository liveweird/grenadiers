use grenadiers_core::cpu::{
    WorkBudget,
    policy::PolicyError,
    survival::{LairSurvivalPlan, plan_hard_farm_yield_v59_with_origin},
};

use super::PlanJob;

pub(super) fn plan(
    job: &mut PlanJob,
    work: &mut WorkBudget,
) -> Result<LairSurvivalPlan, PolicyError> {
    plan_hard_farm_yield_v59_with_origin(
        &job.observation,
        &job.memory,
        &job.hazard_memory,
        job.settled_stop_tick,
        job.escape_origin.as_ref(),
        &mut job.rng,
        work,
    )
}
