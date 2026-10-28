use super::*;
use crate::room::series::{MatchResult, RESULT_HOLD_TICKS};
use std::num::NonZeroU32;

impl RosterMatchLifecycle {
    pub(super) fn ensure_human(&self, seat: SeatId) -> Result<(), RosterError> {
        self.ensure_seat(seat)?;
        if self.configuration.controller(seat) == Some(SeatControl::Cpu) {
            Err(RosterError::CpuTarget(seat))
        } else {
            Ok(())
        }
    }

    pub(super) fn ensure_connected_human(&self, seat: SeatId) -> Result<(), RosterError> {
        self.ensure_human(seat)?;
        match self.participants[seat.index()] {
            RosterParticipant::Connected => Ok(()),
            RosterParticipant::NeverAdmitted => Err(RosterError::SeatNeverAdmitted(seat)),
            _ => Err(RosterError::SeatNotConnected(seat)),
        }
    }

    pub(super) fn next_tick(&self) -> Result<Tick, RosterError> {
        self.tick
            .0
            .checked_add(1)
            .map(Tick)
            .ok_or(RosterError::TickOverflow)
    }

    pub(super) fn deadline(&self, delta: u64) -> Result<Tick, RosterError> {
        self.tick
            .0
            .checked_add(delta)
            .map(Tick)
            .ok_or(RosterError::TickOverflow)
    }

    pub(super) fn next_attempt(&self) -> Result<u64, RosterError> {
        self.round_attempt
            .checked_add(1)
            .ok_or(RosterError::RoundAttemptOverflow)
    }

    pub(super) fn clear_readiness(&mut self) {
        self.ready = [false; 4];
    }

    pub(super) fn cancel_countdown(&mut self) -> Result<(), RosterError> {
        self.round_attempt = self.next_attempt()?;
        self.clear_readiness();
        self.phase = if self.actual_round_number == 0 {
            MatchPhase::Waiting
        } else {
            MatchPhase::Result {
                next_countdown_at: self.tick,
            }
        };
        Ok(())
    }

    pub(super) fn initial_ready(&self) -> bool {
        self.can_start_round()
            && self
                .configuration
                .human_seats()
                .all(|seat| match self.participants[seat.index()] {
                    RosterParticipant::Connected => self.ready[seat.index()],
                    RosterParticipant::Departed => true,
                    RosterParticipant::NeverAdmitted | RosterParticipant::Reserved { .. } => false,
                    RosterParticipant::Cpu => unreachable!(),
                })
    }

    pub(super) fn can_start_round(&self) -> bool {
        self.eligible_seats().count() >= 2
            && self
                .participants()
                .iter()
                .any(|state| state.connected_human())
            && self.configuration.human_seats().all(|seat| {
                matches!(
                    self.participants[seat.index()],
                    RosterParticipant::Connected | RosterParticipant::Departed
                )
            })
    }

    pub(super) fn start_round(&mut self) -> Result<RoundAssignment, RosterError> {
        let number = self
            .actual_round_number
            .checked_add(1)
            .and_then(NonZeroU32::new)
            .ok_or(RosterError::RoundNumberOverflow)?;
        let assignment = RoundAssignment::new(number);
        self.actual_round_number = number.get();
        self.active = Some(ActiveRound {
            assignment,
            started_at: self.tick,
        });
        self.clear_readiness();
        self.phase = MatchPhase::Running;
        Ok(assignment)
    }

    pub(super) fn validate_departures(
        &self,
        departures: [Option<DepartureReason>; 4],
    ) -> Result<(), RosterError> {
        for (index, departure) in departures.into_iter().enumerate() {
            let Some(reason) = departure else { continue };
            let seat = SeatId::ALL[index];
            if index >= self.configuration.player_count().get() {
                return Err(RosterError::InvalidDepartureTail(seat));
            }
            self.ensure_human(seat)?;
            match (reason, self.participants[index]) {
                (DepartureReason::Explicit, RosterParticipant::Connected)
                | (DepartureReason::TransportLoss, RosterParticipant::Connected) => {}
                _ => return Err(RosterError::SeatNotConnected(seat)),
            }
        }
        Ok(())
    }

    pub(super) fn resolve_terminal(
        &mut self,
        update: &mut RosterUpdate,
    ) -> Result<(), RosterError> {
        if self.outcome.is_some() {
            return Ok(());
        }
        if self.actual_round_number == 0 {
            if !self
                .participants()
                .iter()
                .any(|participant| participant.retained_human())
            {
                self.clear_readiness();
                self.phase = MatchPhase::Interrupted {
                    ended_at: self.tick,
                    reason: InterruptReason::ParticipantLeft,
                };
                update.change = MatchChange::Changed;
            }
            return Ok(());
        }
        let eligible: Vec<_> = self.eligible_seats().collect();
        let has_retained_human = self
            .participants()
            .iter()
            .any(|participant| participant.retained_human());
        let result = if !has_retained_human || eligible.is_empty() {
            Some(MatchResult::Abandoned)
        } else if eligible.len() == 1 {
            Some(MatchResult::ForfeitWinner(eligible[0]))
        } else {
            None
        };
        if let Some(result) = result {
            let outcome = self.make_outcome(result);
            let newly_ineligible = update.newly_ineligible;
            *update = self.finish_match(outcome)?;
            update.newly_ineligible = newly_ineligible;
        }
        Ok(())
    }

    pub(super) fn make_outcome(&self, result: MatchResult) -> MatchOutcome {
        super::super::series::facts::match_outcome(self.tick, result)
    }

    pub(super) fn finish_match(
        &mut self,
        outcome: MatchOutcome,
    ) -> Result<RosterUpdate, RosterError> {
        let rematch_at = self.deadline(RESULT_HOLD_TICKS)?;
        self.active = None;
        self.outcome = Some(outcome);
        self.clear_readiness();
        self.phase = MatchPhase::MatchResult {
            outcome,
            rematch_at,
        };
        Ok(RosterUpdate::new(MatchChange::MatchFinished(outcome)))
    }
}
