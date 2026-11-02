use grenadiers_core::{
    game::SeatId,
    room::{DepartureReason, InterruptReason, MatchChange, RosterMatchLifecycle, SeatControl},
};

use super::{
    error::{RosterRuntimeError, lifecycle},
    runtime::RosterRuntime,
    types::{RosterAction, RosterIdentity, RosterNotice},
};

impl RosterRuntime {
    pub(in crate::game::rooms) const fn lifecycle(&self) -> &RosterMatchLifecycle {
        &self.lifecycle
    }

    pub(in crate::game::rooms) const fn identity(&self) -> &RosterIdentity {
        &self.identity
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(in crate::game::rooms) const fn uses_gameplay19(&self) -> bool {
        matches!(
            self.spike_publication,
            super::runtime::SpikePublicationVersion::V19
        )
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(in crate::game::rooms) const fn uses_gameplay20(&self) -> bool {
        matches!(
            self.spike_publication,
            super::runtime::SpikePublicationVersion::V20
        )
    }

    pub(in crate::game::rooms) fn submit_cpu_intent(
        &mut self,
        seat: SeatId,
        expected_epoch: u64,
        action: RosterAction,
    ) -> Result<Option<RosterNotice>, RosterRuntimeError> {
        self.require_controller(seat, SeatControl::Cpu)?;
        if !matches!(
            action,
            RosterAction::Movement(_) | RosterAction::Bomb(_) | RosterAction::Equipment(_)
        ) {
            return Err(RosterRuntimeError::InvalidPhase);
        }
        let mut next = self.clone();
        let notice = next.apply_action(seat, expected_epoch, action)?;
        *self = next;
        Ok(notice)
    }

    pub(in crate::game::rooms) fn next_cpu_manual_action(&self, seat: SeatId) -> Option<u32> {
        self.require_controller(seat, SeatControl::Cpu).ok()?;
        self.inputs
            .last_manual_action(seat)
            .unwrap_or(0)
            .checked_add(1)
            .filter(|value| grenadiers_core::protocol::Counter::new(*value).is_ok())
    }

    pub(in crate::game::rooms) fn stop_seat_inputs(&mut self, seat: SeatId) {
        self.inputs.stop(seat);
    }

    pub(in crate::game::rooms) fn interrupt(
        &mut self,
        reason: InterruptReason,
    ) -> Result<Option<RosterNotice>, RosterRuntimeError> {
        let mut next = self.clone();
        let update = next.lifecycle.interrupt(reason).map_err(lifecycle)?;
        if update.change() == MatchChange::Unchanged {
            return Ok(None);
        }
        next.inputs.clear_pending();
        let notice = next.publish_current()?;
        *self = next;
        Ok(Some(notice))
    }

    pub(in crate::game::rooms) fn interrupt_and_depart(
        &mut self,
        reason: InterruptReason,
        departures: [Option<DepartureReason>; 4],
    ) -> Result<RosterNotice, RosterRuntimeError> {
        let mut next = self.clone();
        next.cancel_human_rematch_votes()?;
        next.lifecycle.interrupt(reason).map_err(lifecycle)?;
        next.inputs.clear_pending();
        if departures.iter().any(Option::is_some) {
            next.lifecycle
                .depart_with_grace(departures)
                .map_err(lifecycle)?;
            for (index, departure) in departures.into_iter().enumerate() {
                if departure.is_some() {
                    next.inputs.stop(SeatId::ALL[index]);
                }
            }
        }
        let notice = next.publish_current()?;
        *self = next;
        Ok(notice)
    }

    pub(in crate::game::rooms) fn current_notice(
        &mut self,
    ) -> Result<RosterNotice, RosterRuntimeError> {
        let mut next = self.clone();
        let notice = next.publish_current()?;
        *self = next;
        Ok(notice)
    }

    pub(in crate::game::rooms) fn retained_human_count(&self) -> usize {
        self.lifecycle.retained_human_count()
    }

    fn require_controller(
        &self,
        seat: SeatId,
        expected: SeatControl,
    ) -> Result<(), RosterRuntimeError> {
        (self.identity.configuration.controller(seat) == Some(expected))
            .then_some(())
            .ok_or(RosterRuntimeError::InvalidPhase)
    }

    fn cancel_human_rematch_votes(&mut self) -> Result<(), RosterRuntimeError> {
        if !matches!(
            self.lifecycle.phase(),
            grenadiers_core::room::MatchPhase::MatchResult { .. }
        ) {
            return Ok(());
        }
        for seat in self.identity.configuration.human_seats() {
            if self.lifecycle.rematch_vote(seat).map_err(lifecycle)? {
                self.lifecycle
                    .set_rematch_vote(seat, false)
                    .map_err(lifecycle)?;
            }
        }
        Ok(())
    }
}
