use super::{PlanJob, WorkerPlan};
use grenadiers_core::cpu::{
    WorkBudget,
    survival::{LairSurvivalPlan, TeleporterRecoveryChoice, select_teleporter_recovery},
};
use grenadiers_core::room::CpuPolicy;

pub(super) fn select(job: &PlanJob, plan: LairSurvivalPlan, work: &mut WorkBudget) -> WorkerPlan {
    if !matches!(
        job.policy,
        CpuPolicy::UtilityV46
            | CpuPolicy::UtilityV47
            | CpuPolicy::UtilityV48
            | CpuPolicy::UtilityV49
            | CpuPolicy::UtilityV50
            | CpuPolicy::UtilityV51
            | CpuPolicy::UtilityV52
            | CpuPolicy::UtilityV53
            | CpuPolicy::UtilityV54
            | CpuPolicy::UtilityV55
            | CpuPolicy::UtilityV56
            | CpuPolicy::UtilityV57
            | CpuPolicy::UtilityV58
    ) {
        return ordinary(plan);
    }
    let selector = if matches!(
        job.policy,
        CpuPolicy::UtilityV50
            | CpuPolicy::UtilityV51
            | CpuPolicy::UtilityV52
            | CpuPolicy::UtilityV53
            | CpuPolicy::UtilityV54
            | CpuPolicy::UtilityV55
            | CpuPolicy::UtilityV56
            | CpuPolicy::UtilityV57
            | CpuPolicy::UtilityV58
    ) {
        grenadiers_core::cpu::survival::select_teleporter_recovery_v50
    } else {
        select_teleporter_recovery
    };
    match selector(
        &job.observation,
        &job.hazard_memory,
        job.settled_stop_tick,
        &job.memory,
        job.escape_origin.as_ref(),
        plan,
        work,
    ) {
        TeleporterRecoveryChoice::Ordinary(plan) => ordinary(*plan),
        TeleporterRecoveryChoice::Step(step) => WorkerPlan::TeleporterStep(step),
    }
}

fn ordinary(plan: LairSurvivalPlan) -> WorkerPlan {
    WorkerPlan::Ordinary {
        plan: Box::new(plan.survival),
        contact: plan.contact,
    }
}
