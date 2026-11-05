use grenadiers_core::{
    cpu::{
        WorkBudget,
        policy::PolicyError,
        survival::{
            LairSurvivalPlan, plan_contact_punch_v60_with_origin,
            plan_hard_contact_punch_v61_with_origin, plan_hard_demolition_v64_with_origin,
            plan_hard_pickup_v66_with_origin, plan_hard_progress_v69_with_origin,
            plan_hard_reactive_v65_with_origin,
        },
    },
    room::CpuPolicy,
};

use super::PlanJob;

pub(super) const fn supports(policy: CpuPolicy) -> bool {
    matches!(
        policy,
        CpuPolicy::UtilityV60
            | CpuPolicy::UtilityV61
            | CpuPolicy::UtilityV62
            | CpuPolicy::UtilityV63
            | CpuPolicy::UtilityV64
            | CpuPolicy::UtilityV65
            | CpuPolicy::UtilityV66
            | CpuPolicy::UtilityV67
            | CpuPolicy::UtilityV68
            | CpuPolicy::UtilityV69
    )
}

pub(super) fn plan(
    job: &mut PlanJob,
    work: &mut WorkBudget,
) -> Result<LairSurvivalPlan, PolicyError> {
    let planner = match job.policy {
        CpuPolicy::UtilityV60 | CpuPolicy::UtilityV62 | CpuPolicy::UtilityV68 => {
            plan_contact_punch_v60_with_origin
        }
        CpuPolicy::UtilityV61 | CpuPolicy::UtilityV63 => plan_hard_contact_punch_v61_with_origin,
        CpuPolicy::UtilityV64 => plan_hard_demolition_v64_with_origin,
        CpuPolicy::UtilityV65 => plan_hard_reactive_v65_with_origin,
        CpuPolicy::UtilityV66 | CpuPolicy::UtilityV67 => plan_hard_pickup_v66_with_origin,
        CpuPolicy::UtilityV69 => plan_hard_progress_v69_with_origin,
        _ => unreachable!("contact-punch dispatch is closed to policies 60-69"),
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
