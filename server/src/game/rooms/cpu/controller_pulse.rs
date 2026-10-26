use grenadiers_core::cpu::Observation;
use grenadiers_core::cpu::policy::UtilityMemory;
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
        }
        if status == PulseStatus::Invalidated {
            self.cancel();
            self.memory = Some(UtilityMemory::new(
                self.memory
                    .as_ref()
                    .expect("world memory initialized")
                    .key(),
            ));
        }
        Some(status)
    }
}
