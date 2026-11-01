use std::fmt;

use super::CpuController;

impl fmt::Debug for CpuController {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CpuController")
            .field("room_instance", &self.room_instance)
            .field("private_seed", &"<redacted>")
            .field("world", &self.world)
            .field("request_serial", &self.request_serial)
            .field("pending", &self.pending.is_some())
            .field("escaping", &self.escaping)
            .finish()
    }
}

#[cfg(test)]
impl CpuController {
    /// Read-only owner-witness trace; never reconstructs or mutates authority.
    pub(in crate::game::rooms) fn controller_state_for_test(&self) -> String {
        format!(
            "observation={:?} stop_admission={:?} settled_stop={:?} serial={} \
             pending={:?} completed={} escaping={} origin={} teleporter={} \
             pulse={} route={} contact={} continuation={} fence={} intent={:?}",
            self.last_observation_tick,
            self.stop_admission.map(|admission| admission.tick),
            self.settled_stop_tick,
            self.request_serial,
            self.pending.as_ref().map(|pending| pending.decision),
            self.pending
                .as_ref()
                .is_some_and(|pending| pending.completed.is_some()),
            self.escaping,
            self.escape_origin.is_some(),
            self.committed_teleporter.is_some(),
            self.committed_pulse.is_some(),
            self.committed_route.is_some(),
            self.committed_contact.is_some(),
            self.prepared_continuation.is_some() || self.continuation.is_some(),
            self.continuation_fence_required,
            self.memory.and_then(|memory| memory.strategic_intent()),
        )
    }
}
