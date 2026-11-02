use grenadiers_core::{
    cpu::{
        WorkBudget,
        policy::PolicyError,
        survival::{
            LairSurvivalPlan, plan_contact_punch_v60_with_origin,
            plan_hard_contact_punch_v61_with_origin,
        },
    },
    room::CpuPolicy,
};

use super::PlanJob;

pub(super) fn plan(
    job: &mut PlanJob,
    work: &mut WorkBudget,
) -> Result<LairSurvivalPlan, PolicyError> {
    let planner = match job.policy {
        CpuPolicy::UtilityV60 => plan_contact_punch_v60_with_origin,
        CpuPolicy::UtilityV61 => plan_hard_contact_punch_v61_with_origin,
        _ => unreachable!("contact-punch dispatch is closed to policies 60/61"),
    };
    planner(
        &job.observation,
        &job.memory,
        &job.hazard_memory,
        job.settled_stop_tick,
        job.escape_origin.as_ref(),
        &mut job.rng,
        work,
    )
}
