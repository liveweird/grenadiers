use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::policy::{ControlPlan, UtilityMemory};
use crate::cpu::routing::{PunchRequest, RouteLimits, plan_punch};
use crate::cpu::{Intent, ManualIntent, Observation, WorkBudget};
use crate::game::Tick;

use super::{ExecutionCertificate, SurvivalPlan};

pub(super) fn certified_punch(
    observation: &Observation,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    memory: &UtilityMemory,
    work: &mut WorkBudget,
) -> Option<SurvivalPlan> {
    if observation.parts().rules.punch_activation_policy
        != crate::game::PunchActivationPolicy::ManualV1
    {
        return None;
    }
    let execution_tick = observation.parts().tick.0.checked_add(7).map(Tick)?;
    // Keep at least half the remaining allowance for ordinary survival if this
    // optional tactic cannot prove a complete escape. All child work is charged.
    let mut tactic_work = WorkBudget::new(work.remaining() / 2);
    let certificate = plan_punch(
        observation,
        history,
        PunchRequest {
            execution_tick,
            settled_stop_tick,
            limits: RouteLimits::default(),
        },
        &mut tactic_work,
    );
    work.try_take(tactic_work.spent()).ok()?;
    let certificate = certificate.ok()?;
    Some(SurvivalPlan {
        control: ControlPlan {
            intent: Intent::STOP.with_manual(ManualIntent::Equipment),
            hold_ticks: 1,
            next_memory: UtilityMemory::new(memory.key()),
        },
        lease: None,
        execution: Some(ExecutionCertificate::Punch(Box::new(certificate))),
    })
}
