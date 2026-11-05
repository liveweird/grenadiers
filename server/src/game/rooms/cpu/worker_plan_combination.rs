use grenadiers_core::{
    cpu::{WorkBudget, policy::PolicyError},
    room::CpuPolicy,
};

use super::{PlanJob, WorkerPlan};

pub(super) fn plan(
    job: &PlanJob,
    work: &mut WorkBudget,
) -> Option<Result<WorkerPlan, PolicyError>> {
    let handoff = job.refuge_handoff.as_ref()?;
    if job.policy != CpuPolicy::UtilityV67 || job.escape_origin.is_some() {
        return Some(Ok(WorkerPlan::NoReplacement));
    }
    Some(
        grenadiers_core::cpu::survival::plan_hard_combination_v67_with_handoff(
            &job.observation,
            &job.memory,
            &job.hazard_memory,
            job.settled_stop_tick,
            handoff,
            work,
        )
        .map(|plan| match plan {
            Some(plan) => WorkerPlan::Ordinary {
                plan: Box::new(plan.survival),
                contact: plan.contact,
            },
            None => WorkerPlan::NoReplacement,
        }),
    )
}
