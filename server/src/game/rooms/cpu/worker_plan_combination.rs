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
    if !matches!(
        job.policy,
        CpuPolicy::UtilityV67 | CpuPolicy::UtilityV69 | CpuPolicy::UtilityV70
    ) || job.escape_origin.is_some()
    {
        return Some(Ok(WorkerPlan::NoReplacement));
    }
    let planner = match job.policy {
        CpuPolicy::UtilityV67 => {
            grenadiers_core::cpu::survival::plan_hard_combination_v67_with_handoff
        }
        CpuPolicy::UtilityV70 => {
            grenadiers_core::cpu::survival::plan_hard_combination_v70_with_handoff
        }
        CpuPolicy::UtilityV69 => {
            grenadiers_core::cpu::survival::plan_hard_combination_v69_with_handoff
        }
        _ => unreachable!(),
    };
    Some(
        planner(
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
