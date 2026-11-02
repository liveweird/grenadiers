use super::{PlanJob, WorkerPlan};
use grenadiers_core::cpu::{
    WorkBudget,
    policy::PolicyError,
    survival::{plan_lairs_with_origin, plan_with_origin},
};

pub(super) fn plan(job: &mut PlanJob, work: &mut WorkBudget) -> Result<WorkerPlan, PolicyError> {
    if job.observation.parts().rules.version == 8 {
        plan_lairs_with_origin(
            &job.observation,
            &job.memory,
            &job.hazard_memory,
            job.settled_stop_tick,
            job.escape_origin.as_ref(),
            &mut job.rng,
            work,
        )
        .map(|plan| WorkerPlan::Ordinary {
            plan: Box::new(plan.survival),
            contact: plan.contact,
        })
    } else {
        plan_with_origin(
            &job.observation,
            &job.memory,
            &job.hazard_memory,
            job.settled_stop_tick,
            job.escape_origin.as_ref(),
            &mut job.rng,
            work,
        )
        .map(|plan| WorkerPlan::Ordinary {
            plan: Box::new(plan),
            contact: None,
        })
    }
}
