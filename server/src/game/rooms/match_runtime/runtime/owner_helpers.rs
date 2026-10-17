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
        let next = self.next_revision()?;
        let room = super::super::publication::room_view(
            &self.identity,
            &self.lifecycle,
            next,
            self.settings_revision,
            None,
        )?;
        self.revision = next;
        Ok(MatchNotice {
            room,
            snapshot: None,
        })
    }

    pub(in crate::game::rooms) fn current_notice(
        &mut self,
    ) -> Result<MatchNotice, MatchRuntimeError> {
        self.publish_current()
    }

    pub(in crate::game::rooms) fn stop_seat_inputs(&mut self, seat: SeatId) {
        self.inputs.stop(seat);
        self.capture(
            self.lifecycle.tick(),
            crate::game::rooms::replay::Operation::InputFence(seat),
        );
    }

    pub(in crate::game::rooms) fn interrupt_and_depart(
        &mut self,
        reason: InterruptReason,
        departures: [Option<DepartureReason>; 2],
    ) -> Result<MatchNotice, MatchRuntimeError> {
        let tick = self.lifecycle.tick();
        self.ensure_revision()?;
        self.lifecycle
            .interrupt(reason)
            .map_err(map_lifecycle_error)?;
        self.inputs.clear_pending();
        self.lifecycle
            .depart_batch(departures)
            .map_err(map_lifecycle_error)?;
        self.release_departed_editor(departures);
        for (seat, departure) in SeatId::ALL.into_iter().zip(departures) {
            if departure.is_some() {
                self.inputs.stop(seat);
            }
        }
        let notice = self.publish_current()?;
        self.capture(
            tick,
            crate::game::rooms::replay::Operation::InterruptAndDepart { reason, departures },
        );
        Ok(notice)
    }
    #[cfg(test)]
    pub(in crate::game::rooms) fn depart_batch(
        &mut self,
        departures: [Option<DepartureReason>; 2],
    ) -> Result<Option<MatchNotice>, MatchRuntimeError> {
        let tick = self.lifecycle.tick();
        self.ensure_revision()?;
        let change = self
            .lifecycle
            .depart_batch(departures)
            .map_err(map_lifecycle_error)?;
        self.release_departed_editor(departures);
        for (seat, departure) in SeatId::ALL.into_iter().zip(departures) {
            if departure.is_some() {
                self.inputs.stop(seat);
            }
        }
        let notice = self.notice_for_change(change, true)?;
        self.capture(
            tick,
            crate::game::rooms::replay::Operation::Depart(departures),
        );
        Ok(notice)
    }

    pub(in crate::game::rooms) fn interrupt(
        &mut self,
        reason: InterruptReason,
    ) -> Result<Option<MatchNotice>, MatchRuntimeError> {
        let tick = self.lifecycle.tick();
        self.ensure_revision()?;
        let change = self
            .lifecycle
            .interrupt(reason)
            .map_err(map_lifecycle_error)?;
        self.inputs.clear_pending();
        let notice = self.notice_for_change(change, true)?;
        self.capture(
            tick,
            crate::game::rooms::replay::Operation::Interrupt(reason),
        );
        Ok(notice)
    }
}
