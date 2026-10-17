use super::*;

impl MatchLifecycle {
    pub(super) fn depart_batch_inner(
        &mut self,
        departures: [Option<DepartureReason>; 2],
    ) -> Result<MatchChange, MatchError> {
        let mut admitted = departures;
        for (index, departure) in admitted.iter_mut().enumerate() {
            if !self.seats[index].occupied {
                *departure = None;
            }
        }
        if admitted == [None, None] {
            return Ok(MatchChange::Unchanged);
        }
        if matches!(self.phase, MatchPhase::MatchResult { .. }) {
            let votes = self.series.rematch_votes();
            for (index, departure) in admitted.iter().enumerate() {
                if departure.is_some() && votes[index] {
                    self.series
                        .set_rematch_vote(SeatId::ALL[index], false, self.tick)?;
                }
            }
            self.remove_departed(admitted)?;
            return Ok(MatchChange::Changed);
        }
        if self.series.actual_round_number() == 0 {
            let next_attempt = if matches!(self.phase, MatchPhase::Countdown { .. }) {
                Some(self.next_attempt()?)
            } else {
                None
            };
            self.remove_departed(admitted)?;
            self.clear_readiness();
            if let Some(next_attempt) = next_attempt {
                self.round_attempt = next_attempt;
                self.phase = MatchPhase::Waiting;
            }
            return Ok(MatchChange::Changed);
        }

        let outcome = self
            .series
            .resolve_departures(admitted, self.tick)?
            .expect("started match departure is terminal");
        let rematch_at = self.series.hold_deadline().expect("terminal result hold");
        self.remove_departed(admitted)?;
        self.clear_readiness();
        self.phase = MatchPhase::MatchResult {
            outcome,
            rematch_at,
        };
        Ok(MatchChange::MatchFinished(outcome))
    }

    pub(super) fn interrupt_inner(
        &mut self,
        reason: InterruptReason,
    ) -> Result<MatchChange, MatchError> {
        if reason == InterruptReason::ParticipantLeft {
            return Err(MatchError::InvalidInterruptReason);
        }
        if matches!(
            self.phase,
            MatchPhase::MatchResult { .. } | MatchPhase::Interrupted { .. }
        ) {
            return Ok(MatchChange::Unchanged);
        }
        self.clear_readiness();
        if self.series.actual_round_number() == 0 {
            self.phase = MatchPhase::Interrupted {
                ended_at: self.tick,
                reason,
            };
            return Ok(MatchChange::Changed);
        }
        let outcome = self
            .series
            .interrupt(reason, self.tick)?
            .expect("started match interruption is terminal");
        let rematch_at = self.series.hold_deadline().expect("terminal result hold");
        self.phase = MatchPhase::MatchResult {
            outcome,
            rematch_at,
        };
        Ok(MatchChange::MatchFinished(outcome))
    }

    pub(super) fn set_rematch_vote_inner(
        &mut self,
        seat: SeatId,
        ready: bool,
    ) -> Result<MatchChange, MatchError> {
        if !matches!(self.phase, MatchPhase::MatchResult { .. }) {
            return Err(MatchError::InvalidPhase);
        }
        if !self.seats[seat.index()].occupied {
            return Err(MatchError::SeatVacant(seat));
        }
        if !self.reconnect.seat(seat).connected() {
            return Err(MatchError::Reconnect(ReconnectError::SeatNotConnected(
                seat,
            )));
        }
        let votes = self.series.rematch_votes();
        let other_index = 1 - seat.index();
        let starts = ready && votes[other_index];
        if starts && !self.reconnect.seat(SeatId::ALL[other_index]).connected() {
            return Err(MatchError::Reconnect(ReconnectError::SeatNotConnected(
                SeatId::ALL[other_index],
            )));
        }
        let next_attempt = starts.then(|| self.next_attempt()).transpose()?;
        let status = self.series.set_rematch_vote(seat, ready, self.tick)?;
        if status == RematchStatus::Started {
            self.round_attempt = next_attempt.expect("both votes preflight epoch");
            self.clear_readiness();
            self.phase = MatchPhase::Waiting;
            Ok(MatchChange::RematchStarted)
        } else if votes[seat.index()] == ready {
            Ok(MatchChange::Unchanged)
        } else {
            Ok(MatchChange::Changed)
        }
    }

    pub(super) fn remove_departed(
        &mut self,
        departures: [Option<DepartureReason>; 2],
    ) -> Result<(), MatchError> {
        for (index, departure) in departures.into_iter().enumerate() {
            if departure.is_some() {
                self.reconnect.release(SeatId::ALL[index], self.tick)?;
                self.seats[index] = SeatState::default();
            }
        }
        Ok(())
    }
}
