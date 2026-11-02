use grenadiers_core::cpu::{
    Intent, Observation, WorkBudget,
    policy::StrategicIntent,
    survival::{
        TeleporterRecoveryChoice, WORK_ALLOWANCE, plan_hard_corner_refuges_v57_with_origin,
        plan_hard_exit_pressure_v58_with_origin, select_teleporter_recovery_v50,
    },
};
use grenadiers_core::game::Tick;
use grenadiers_core::room::CpuPolicy;

use crate::game::rooms::cpu::PlanningBasis;

#[derive(Debug)]
pub(super) struct QueuedPlanProbe {
    pub observation: Observation,
    pub stop: Tick,
    pub work: u32,
    pub intent: Intent,
    pub strategy: Option<StrategicIntent>,
    pub execution: bool,
}

pub(super) fn inspect(basis: &PlanningBasis<'_>) -> QueuedPlanProbe {
    let planner = match basis.policy {
        CpuPolicy::UtilityV57 => plan_hard_corner_refuges_v57_with_origin,
        CpuPolicy::UtilityV58 => plan_hard_exit_pressure_v58_with_origin,
        _ => panic!("unsupported queued-plan witness profile"),
    };
    let mut rng = basis.rng.clone();
    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let planned = planner(
        basis.observation,
        basis.memory,
        basis.hazard_memory,
        basis.settled_stop_tick,
        basis.escape_origin,
        &mut rng,
        &mut work,
    )
    .unwrap();
    let TeleporterRecoveryChoice::Ordinary(planned) = select_teleporter_recovery_v50(
        basis.observation,
        basis.hazard_memory,
        basis.settled_stop_tick,
        basis.memory,
        basis.escape_origin,
        planned,
        &mut work,
    ) else {
        panic!("witness scene has no Teleporter recovery target")
    };
    QueuedPlanProbe {
        observation: basis.observation.clone(),
        stop: basis.settled_stop_tick,
        work: work.spent(),
        intent: planned.survival.control.intent,
        strategy: planned.survival.control.next_memory.strategic_intent(),
        execution: planned.survival.execution.is_some(),
    }
}
