use grenadiers_core::{
    game::SeatId,
    room::{DepartureReason, InterruptReason},
};

use super::{MatchNotice, MatchRuntime, MatchRuntimeError, map_lifecycle_error};

impl MatchRuntime {
    pub(in crate::game::rooms) const fn lifecycle(&self) -> &grenadiers_core::room::MatchLifecycle {
        &self.lifecycle
    }

    pub(in crate::game::rooms) fn initial_notice(
        &mut self,
    ) -> Result<MatchNotice, MatchRuntimeError> {
        self.publish_view()
    }

    pub(in crate::game::rooms) fn current_notice(
        &mut self,
    ) -> Result<MatchNotice, MatchRuntimeError> {
        self.publish_current()
    }

    pub(in crate::game::rooms) fn stop_seat_inputs(&mut self, seat: SeatId) {
        self.inputs.stop(seat);
    }

    pub(in crate::game::rooms) fn interrupt_and_depart(
        &mut self,
        reason: InterruptReason,
        departures: [Option<DepartureReason>; 2],
    ) -> Result<MatchNotice, MatchRuntimeError> {
        self.ensure_revision()?;
        self.lifecycle
            .interrupt(reason)
            .map_err(map_lifecycle_error)?;
        self.inputs.clear_pending();
        self.lifecycle
            .depart_batch(departures)
            .map_err(map_lifecycle_error)?;
        for (seat, departure) in SeatId::ALL.into_iter().zip(departures) {
            if departure.is_some() {
                self.inputs.stop(seat);
            }
        }
        self.publish_current()
    }
}
