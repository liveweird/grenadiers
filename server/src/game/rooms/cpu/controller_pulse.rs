use grenadiers_core::cpu::Observation;
use grenadiers_core::cpu::survival::PulseStatus;
use grenadiers_core::game::Tick;

use super::CpuController;

impl CpuController {
    pub(in crate::game::rooms) fn validate_pulse(
        &mut self,
        observation: &Observation,
        current_tick: Tick,
    ) -> Option<PulseStatus> {
        let status = self
            .committed_pulse
            .as_ref()?
            .validate(observation, current_tick);
        if matches!(status, PulseStatus::Completed | PulseStatus::Invalidated) {
            self.committed_pulse = None;
            self.committed_contact = None;
        }
        if status == PulseStatus::Invalidated {
            self.cancel();
            self.memory = Some(
                self.memory
                    .expect("world memory initialized")
                    .without_execution_authority(),
            );
        }
        Some(status)
    }
}
