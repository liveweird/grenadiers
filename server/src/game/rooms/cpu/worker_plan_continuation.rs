use super::{PlanJob, WorkerPlan};
use grenadiers_core::cpu::{
    WorkBudget,
    policy::PolicyError,
    survival::{
        OriginRecoveryChoice, plan_continuation_lairs, plan_continuation_lairs_avoiding_pits,
    },
};
use grenadiers_core::room::CpuPolicy;

pub(super) fn plan(
    job: &PlanJob,
    work: &mut WorkBudget,
) -> Option<Result<WorkerPlan, PolicyError>> {
    let supports_continuation = matches!(
        job.policy,
        CpuPolicy::UtilityV44
            | CpuPolicy::UtilityV45
            | CpuPolicy::UtilityV46
            | CpuPolicy::UtilityV47
            | CpuPolicy::UtilityV48
            | CpuPolicy::UtilityV49
            | CpuPolicy::UtilityV50
            | CpuPolicy::UtilityV51
            | CpuPolicy::UtilityV52
            | CpuPolicy::UtilityV53
    );
    let origin = supports_continuation
        .then_some(job.escape_origin.as_ref())
        .flatten()?;
    let planner = if matches!(job.policy, CpuPolicy::UtilityV52 | CpuPolicy::UtilityV53) {
        plan_continuation_lairs_avoiding_pits
    } else {
        plan_continuation_lairs
    };
    let choice = planner(
        &job.observation,
        &job.hazard_memory,
        job.settled_stop_tick,
        &job.memory,
        origin,
        work,
    );
    Some(Ok(match choice {
        Some(OriginRecoveryChoice::Continuation(certificate)) => {
            WorkerPlan::Continuation(certificate)
        }
        Some(OriginRecoveryChoice::Pulse(plan)) => WorkerPlan::Ordinary {
            plan: Box::new(plan.survival),
            contact: plan.contact,
        },
        None => {
            let plan = grenadiers_core::cpu::survival::continuation_recovery_stop(&job.memory);
            WorkerPlan::Ordinary {
                plan: Box::new(plan.survival),
                contact: plan.contact,
            }
        }
    }))
}
