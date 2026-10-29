use grenadiers_core::cpu::{
    Observation,
    policy::{ControlPlan, EscapeExecution, MAX_HOLD_TICKS, PolicyError, UtilityMemory},
};

use super::CpuController;

impl CpuController {
    pub(in crate::game::rooms) fn follow_escape(
        &mut self,
        observation: &Observation,
    ) -> Result<EscapeExecution, PolicyError> {
        if !self.escaping {
            return Ok(EscapeExecution::Completed);
        }
        if self.profile.policy.version() == 19 {
            self.escaping = false;
            self.reset_escape_memory();
            return Ok(EscapeExecution::Invalidated);
        }
        let execution = grenadiers_core::cpu::policy::follow_checked_escape(
            observation,
            self.memory.as_ref().expect("world memory initialized"),
        )?;
        if let EscapeExecution::Continue(plan) | EscapeExecution::Wait(plan) = execution {
            if !(1..=MAX_HOLD_TICKS).contains(&plan.hold_ticks) || plan.intent.manual.is_some() {
                return Err(PolicyError::InvalidObservation);
            }
            return Ok(execution);
        }
        self.escaping = false;
        self.reset_escape_memory();
        Ok(execution)
    }

    pub(in crate::game::rooms) fn commit_escape(&mut self, plan: ControlPlan) {
        self.memory = Some(plan.next_memory);
    }

    fn reset_escape_memory(&mut self) {
        self.memory = Some(UtilityMemory::new(
            self.memory
                .as_ref()
                .expect("world memory initialized")
                .key(),
        ));
    }
}
