use super::*;
use crate::room::COUNTDOWN_TICKS;
use crate::room::reconnect::RECONNECT_GRACE_TICKS;
use crate::room::series::{MatchResult, RESULT_HOLD_TICKS};

impl RosterMatchLifecycle {
    pub(super) fn occupy_inner(&mut self, seat: SeatId) -> Result<RosterUpdate, RosterError> {
        self.ensure_human(seat)?;
        if self.phase != MatchPhase::Waiting || self.actual_round_number != 0 {
            return Err(RosterError::InvalidPhase);
        }
        if self.participants[seat.index()] != RosterParticipant::NeverAdmitted {
            return Err(RosterError::SeatAlreadyAdmitted(seat));
        }
        self.participants[seat.index()] = RosterParticipant::Connected;
        Ok(RosterUpdate::new(MatchChange::Changed))
    }

    pub(super) fn set_ready_inner(
        &mut self,
        seat: SeatId,
        ready: bool,
    ) -> Result<RosterUpdate, RosterError> {
        self.ensure_connected_human(seat)?;
        if !matches!(
            self.phase,
            MatchPhase::Waiting
                | MatchPhase::Countdown {
                    kind: super::super::CountdownKind::Initial,
                    ..
                }
        ) {
            return Err(RosterError::InvalidPhase);
        }
        let index = seat.index();
        if self.ready[index] == ready {
            return Ok(RosterUpdate::new(MatchChange::Unchanged));
        }
        if matches!(self.phase, MatchPhase::Countdown { .. }) {
            self.cancel_countdown()?;
            return Ok(RosterUpdate::new(MatchChange::Changed));
        }
        self.ready[index] = ready;
        if ready && self.initial_ready() {
            let ends_at = self.deadline(COUNTDOWN_TICKS)?;
            self.phase = MatchPhase::Countdown {
                kind: super::super::CountdownKind::Initial,
                ends_at,
            };
            Ok(RosterUpdate::new(MatchChange::CountdownStarted))
        } else {
            Ok(RosterUpdate::new(MatchChange::Changed))
        }
    }

    pub(super) fn set_settings_inner(
        &mut self,
        editor: SeatId,
        settings: RoomSettings,
    ) -> Result<RosterUpdate, RosterError> {
        self.ensure_seat(editor)?;
        if self.settings_editor() != Some(editor) {
            return Err(RosterError::UnauthorizedSettingsEditor(editor));
        }
        if self.actual_round_number != 0
            || !matches!(
                self.phase,
                MatchPhase::Waiting
                    | MatchPhase::Countdown {
                        kind: super::super::CountdownKind::Initial,
                        ..
                    }
            )
        {
            return Err(RosterError::InvalidPhase);
        }
        if self.settings == settings {
            return Ok(RosterUpdate::new(MatchChange::Unchanged));
        }
        if matches!(self.phase, MatchPhase::Countdown { .. }) {
            self.cancel_countdown()?;
        } else {
            self.clear_readiness();
        }
        self.settings = settings;
        Ok(RosterUpdate::new(MatchChange::Changed))
    }

    pub(super) fn advance_idle_tick_inner(&mut self) -> Result<RosterUpdate, RosterError> {
        if self.phase == MatchPhase::Running {
            return Err(RosterError::InvalidPhase);
        }
        if matches!(self.phase, MatchPhase::Interrupted { .. }) {
            return Ok(RosterUpdate::new(MatchChange::Unchanged));
        }
        let next = self.next_tick()?;
        self.tick = next;
        match self.phase {
            MatchPhase::Countdown { ends_at, .. } if next >= ends_at => {
                if !self.can_start_round() {
                    self.cancel_countdown()?;
                    return Ok(RosterUpdate::new(MatchChange::Changed));
                }
                let assignment = self.start_round()?;
                Ok(RosterUpdate::new(MatchChange::RoundStarted(assignment)))
            }
            MatchPhase::Result { next_countdown_at }
                if next >= next_countdown_at && self.can_start_round() =>
            {
                self.round_attempt = self.next_attempt()?;
                self.phase = MatchPhase::Countdown {
                    kind: super::super::CountdownKind::InterRound,
                    ends_at: self.deadline(COUNTDOWN_TICKS)?,
                };
                Ok(RosterUpdate::new(MatchChange::CountdownStarted))
            }
            _ => Ok(RosterUpdate::new(MatchChange::Changed)),
        }
    }

    pub(super) fn complete_running_tick_inner(
        &mut self,
        result: Option<RoundResult>,
    ) -> Result<RosterUpdate, RosterError> {
        if self.phase != MatchPhase::Running {
            return Err(RosterError::InvalidPhase);
        }
        self.tick = self.next_tick()?;
        match result {
            Some(result) => self.finish_round(result),
            None => Ok(RosterUpdate::new(MatchChange::Changed)),
        }
    }

    pub(super) fn complete_current_tick_inner(
        &mut self,
        result: RoundResult,
    ) -> Result<RosterUpdate, RosterError> {
        if self.phase != MatchPhase::Running {
            return Err(RosterError::InvalidPhase);
        }
        self.finish_round(result)
    }

    fn finish_round(&mut self, result: RoundResult) -> Result<RosterUpdate, RosterError> {
        if let RoundResult::Winner(seat) = result {
            self.ensure_seat(seat)?;
        }
        let active = self.active.ok_or(RosterError::InvalidPhase)?;
        let hold = self.deadline(RESULT_HOLD_TICKS)?;
        let completed = super::super::series::facts::completed_round(
            self.round_attempt,
            active.assignment,
            active.started_at,
            self.tick,
            result,
        );
        let mut terminal = None;
        if let RoundResult::Winner(seat) = result {
            self.scores[seat.index()] = self.scores[seat.index()]
                .checked_add(1)
                .ok_or(RosterError::ScoreOverflow)?;
            if self.scores[seat.index()] == self.settings.wins_to_finish() {
                terminal = Some(self.make_outcome(MatchResult::ScoreWinner(seat)));
            }
        }
        self.active = None;
        self.latest_completed = Some(completed);
        if let Some(outcome) = terminal {
            self.outcome = Some(outcome);
            self.phase = MatchPhase::MatchResult {
                outcome,
                rematch_at: hold,
            };
            Ok(RosterUpdate::new(MatchChange::MatchFinished(outcome)))
        } else {
            self.phase = MatchPhase::Result {
                next_countdown_at: hold,
            };
            Ok(RosterUpdate::new(MatchChange::RoundFinished))
        }
    }

    pub(super) fn depart_with_grace_inner(
        &mut self,
        departures: [Option<DepartureReason>; 4],
    ) -> Result<RosterUpdate, RosterError> {
        self.validate_departures(departures)?;
        let mut update = RosterUpdate::new(MatchChange::Unchanged);
        let was_countdown = matches!(self.phase, MatchPhase::Countdown { .. });
        for (index, departure) in departures.into_iter().enumerate() {
            let Some(reason) = departure else { continue };
            match reason {
                DepartureReason::Explicit => {
                    self.participants[index] = RosterParticipant::Departed;
                    update.newly_ineligible[index] = true;
                }
                DepartureReason::TransportLoss => {
                    self.participants[index] = RosterParticipant::Reserved {
                        expires_at: self.deadline(RECONNECT_GRACE_TICKS)?,
                    };
                }
            }
            self.ready[index] = false;
            self.rematch_votes[index] = false;
            update.change = MatchChange::Changed;
        }
        if was_countdown && !self.can_start_round() && self.outcome.is_none() {
            self.cancel_countdown()?;
        }
        self.resolve_terminal(&mut update)?;
        Ok(update)
    }

    pub(super) fn resume_connection_inner(
        &mut self,
        seat: SeatId,
    ) -> Result<RosterUpdate, RosterError> {
        self.ensure_human(seat)?;
        let RosterParticipant::Reserved { expires_at } = self.participants[seat.index()] else {
            return Err(RosterError::SeatNotReserved(seat));
        };
        if self.tick >= expires_at {
            return Err(RosterError::GraceExpired { seat, expires_at });
        }
        self.participants[seat.index()] = RosterParticipant::Connected;
        Ok(RosterUpdate::new(MatchChange::Changed))
    }

    pub(super) fn expire_reconnect_grace_inner(&mut self) -> Result<RosterUpdate, RosterError> {
        let mut update = RosterUpdate::new(MatchChange::Unchanged);
        for index in 0..self.configuration.player_count().get() {
            if matches!(self.participants[index], RosterParticipant::Reserved { expires_at } if self.tick >= expires_at)
            {
                self.participants[index] = RosterParticipant::Departed;
                self.rematch_votes[index] = false;
                update.newly_ineligible[index] = true;
                update.change = MatchChange::Changed;
            }
        }
        if matches!(self.phase, MatchPhase::Countdown { .. })
            && !self.can_start_round()
            && self.outcome.is_none()
        {
            self.cancel_countdown()?;
        }
        self.resolve_terminal(&mut update)?;
        Ok(update)
    }

    pub(super) fn interrupt_inner(
        &mut self,
        reason: InterruptReason,
    ) -> Result<RosterUpdate, RosterError> {
        if reason == InterruptReason::ParticipantLeft {
            return Err(RosterError::InvalidInterruptReason);
        }
        if self.outcome.is_some() || matches!(self.phase, MatchPhase::Interrupted { .. }) {
            return Ok(RosterUpdate::new(MatchChange::Unchanged));
        }
        self.clear_readiness();
        if self.actual_round_number == 0 {
            self.phase = MatchPhase::Interrupted {
                ended_at: self.tick,
                reason,
            };
            Ok(RosterUpdate::new(MatchChange::Changed))
        } else {
            let outcome = self.make_outcome(MatchResult::Interrupted(reason));
            self.finish_match(outcome)
        }
    }

    pub(super) fn set_rematch_vote_inner(
        &mut self,
        seat: SeatId,
        ready: bool,
    ) -> Result<RosterUpdate, RosterError> {
        self.ensure_connected_human(seat)?;
        if self.outcome.is_none()
            || !matches!(self.phase, MatchPhase::MatchResult { rematch_at, .. } if self.tick >= rematch_at)
        {
            return Err(RosterError::RematchUnavailable);
        }
        if self.eligible_seats().count() < 2 {
            return Err(RosterError::RematchUnavailable);
        }
        let index = seat.index();
        if self.rematch_votes[index] == ready {
            return Ok(RosterUpdate::new(MatchChange::Unchanged));
        }
        self.rematch_votes[index] = ready;
        let all =
            self.configuration
                .human_seats()
                .all(|human| match self.participants[human.index()] {
                    RosterParticipant::Connected => self.rematch_votes[human.index()],
                    RosterParticipant::Reserved { .. } | RosterParticipant::NeverAdmitted => false,
                    RosterParticipant::Departed | RosterParticipant::Cpu => true,
                });
        if all {
            self.round_attempt = self.next_attempt()?;
            self.scores = [0; 4];
            self.actual_round_number = 0;
            self.active = None;
            self.latest_completed = None;
            self.outcome = None;
            self.rematch_votes = [false; 4];
            self.clear_readiness();
            self.phase = MatchPhase::Waiting;
            Ok(RosterUpdate::new(MatchChange::RematchStarted))
        } else {
            Ok(RosterUpdate::new(MatchChange::Changed))
        }
    }
}
