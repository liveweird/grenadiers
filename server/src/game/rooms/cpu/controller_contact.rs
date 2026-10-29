use grenadiers_core::cpu::{Observation, WorkBudget};
use grenadiers_core::game::Tick;

use super::CpuController;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::game::rooms) enum ContactStatus {
    Continue,
    Invalidated,
}

impl CpuController {
    pub(in crate::game::rooms) fn validate_contact(
        &mut self,
        observation: &Observation,
        current_tick: Tick,
    ) -> Option<ContactStatus> {
        let contact = self.committed_contact.as_ref()?;
        let valid = current_tick
            .0
            .checked_add(1)
            .is_some_and(|next| next < contact.safe_until().0)
            && observation.parts().tick <= current_tick
            && contact.observation_fits(observation, &mut WorkBudget::new(65_536));
        if valid {
            return Some(ContactStatus::Continue);
        }
        self.committed_contact = None;
        self.cancel();
        Some(ContactStatus::Invalidated)
    }
}
