use super::*;
use crate::room::reconnect::{DisconnectStatus, ReservationState};

impl MatchLifecycle {
    pub(super) fn depart_with_grace_inner(
        &mut self,
        departures: [Option<DepartureReason>; 2],
    ) -> Result<MatchChange, MatchError> {
        let admitted = self.admitted_departures(departures);
        if matches!(self.phase, MatchPhase::MatchResult { .. }) {
            return self.settle_terminal_departures(admitted);
        }
        if self.series.actual_round_number() == 0 {
            return self.settle_prematch_departures(admitted);
        }
        self.validate_explicit_departures(admitted)?;

        let mut changed = self.apply_unfinished_departures(admitted)?;
        let expired = self.reconnect.expire_due(self.tick)?.newly_expired();
        changed |= expired.into_iter().any(|value| value);
        if let Some(change) = self.resolve_unfinished(admitted)? {
            return Ok(change);
        }
        if matches!(
            self.phase,
            MatchPhase::Countdown {
                kind: CountdownKind::InterRound,
                ..
            }
        ) && !self.both_connected()
        {
            let next_attempt = self.next_attempt()?;
            let next_countdown_at = self
                .series
                .hold_deadline()
                .expect("inter-round countdown follows completed round");
            self.round_attempt = next_attempt;
            self.phase = MatchPhase::Result { next_countdown_at };
            changed = true;
        }
        Ok(if changed {
            MatchChange::Changed
        } else {
            MatchChange::Unchanged
        })
    }

    pub(super) fn resume_connection_inner(
        &mut self,
        seat: SeatId,
    ) -> Result<MatchChange, MatchError> {
        if self.series.actual_round_number() == 0
            || !matches!(
                self.phase,
                MatchPhase::Running
                    | MatchPhase::Result { .. }
                    | MatchPhase::Countdown {
                        kind: CountdownKind::InterRound,
                        ..
                    }
                    | MatchPhase::MatchResult { .. }
            )
        {
            return Err(MatchError::InvalidPhase);
        }
        self.reconnect.resume(seat, self.tick)?;
        let expired = self.reconnect.expire_due(self.tick)?.newly_expired();
        if matches!(self.phase, MatchPhase::MatchResult { .. }) {
            self.clear_expired_votes(expired)?;
            return Ok(MatchChange::Changed);
        }
        if let Some(change) = self.resolve_unfinished([None; 2])? {
            return Ok(change);
        }
        Ok(MatchChange::Changed)
    }

    fn admitted_departures(
        &self,
        departures: [Option<DepartureReason>; 2],
    ) -> [Option<DepartureReason>; 2] {
        let mut admitted = departures;
        for (index, departure) in admitted.iter_mut().enumerate() {
            if !self.seats[index].occupied {
                *departure = None;
            }
        }
        admitted
    }

    fn validate_explicit_departures(
        &self,
        admitted: [Option<DepartureReason>; 2],
    ) -> Result<(), MatchError> {
        for (index, departure) in admitted.into_iter().enumerate() {
            if departure == Some(DepartureReason::Explicit)
                && !self
                    .reconnect
                    .seat(SeatId::DUEL[index])
                    .is_some_and(ReservationState::connected)
            {
                return Err(ReconnectError::SeatNotConnected(SeatId::DUEL[index]).into());
            }
        }
        Ok(())
    }

    fn settle_prematch_departures(
        &mut self,
        admitted: [Option<DepartureReason>; 2],
    ) -> Result<MatchChange, MatchError> {
        self.validate_explicit_departures(admitted)?;
        self.reconnect.expire_due(self.tick)?;
        if admitted == [None; 2] {
            return Ok(MatchChange::Unchanged);
        }
        let next_attempt = matches!(self.phase, MatchPhase::Countdown { .. })
            .then(|| self.next_attempt())
            .transpose()?;
        self.remove_departed(admitted)?;
        self.clear_readiness();
        if let Some(next_attempt) = next_attempt {
            self.round_attempt = next_attempt;
            self.phase = MatchPhase::Waiting;
        }
        Ok(MatchChange::Changed)
    }

    fn apply_unfinished_departures(
        &mut self,
        admitted: [Option<DepartureReason>; 2],
    ) -> Result<bool, MatchError> {
        let mut changed = false;
        // Represent every loss before releasing either explicit departure, so
        // expiry and mixed-batch policy always observe one complete boundary.
        for (index, departure) in admitted.into_iter().enumerate() {
            let seat = SeatId::DUEL[index];
            if departure == Some(DepartureReason::TransportLoss)
                && self.reconnect.seat(seat) == Some(ReservationState::Connected)
            {
                let status = self.reconnect.disconnect(seat, self.tick)?;
                changed |= matches!(status, DisconnectStatus::Started { .. });
            }
        }
        for (index, departure) in admitted.into_iter().enumerate() {
            let seat = SeatId::DUEL[index];
            if departure == Some(DepartureReason::Explicit) {
                self.reconnect.release(seat, self.tick)?;
                self.seats[index] = SeatState::default();
                changed = true;
            }
            if departure.is_some() && self.seats[index].ready {
                self.seats[index].ready = false;
                changed = true;
            }
        }
        Ok(changed)
    }

    fn resolve_unfinished(
        &mut self,
        admitted: [Option<DepartureReason>; 2],
    ) -> Result<Option<MatchChange>, MatchError> {
        let explicit = admitted.map(|departure| departure == Some(DepartureReason::Explicit));
        let terminal = if explicit == [true; 2] {
            Some(None)
        } else if explicit[0] || explicit[1] {
            let opponent = SeatId::DUEL[usize::from(explicit[0])];
            let eligible = matches!(
                self.reconnect.seat(opponent),
                Some(ReservationState::Connected | ReservationState::Disconnected { .. })
            );
            Some(eligible.then_some(opponent))
        } else {
            match self.reconnect.seats() {
                [ReservationState::Expired, ReservationState::Connected] => Some(Some(SeatId::Two)),
                [ReservationState::Connected, ReservationState::Expired] => Some(Some(SeatId::One)),
                [ReservationState::Expired, ReservationState::Expired] => Some(None),
                _ => None,
            }
        };
        let Some(winner) = terminal else {
            return Ok(None);
        };
        let outcome = if let Some(winner) = winner {
            self.series.forfeit(winner, self.tick)?
        } else {
            self.series.abandon(self.tick)?
        }
        .expect("started match terminal fact");
        let rematch_at = self.series.hold_deadline().expect("terminal result hold");
        self.clear_readiness();
        self.phase = MatchPhase::MatchResult {
            outcome,
            rematch_at,
        };
        Ok(Some(MatchChange::MatchFinished(outcome)))
    }

    fn settle_terminal_departures(
        &mut self,
        admitted: [Option<DepartureReason>; 2],
    ) -> Result<MatchChange, MatchError> {
        self.validate_explicit_departures(admitted)?;
        let before = *self.reconnect.seats();
        for (index, departure) in admitted.into_iter().enumerate() {
            let seat = SeatId::DUEL[index];
            match (departure, self.reconnect.seat(seat)) {
                (Some(DepartureReason::Explicit), _)
                | (Some(DepartureReason::TransportLoss), Some(ReservationState::Connected)) => {
                    self.reconnect.release(seat, self.tick)?;
                    self.seats[index] = SeatState::default();
                }
                _ => {}
            }
        }
        let expired = self.reconnect.expire_due(self.tick)?.newly_expired();
        let after = *self.reconnect.seats();
        let affected = [
            before[0] != after[0] || expired[0],
            before[1] != after[1] || expired[1],
        ];
        self.clear_expired_votes(affected)?;
        Ok(if before == after {
            MatchChange::Unchanged
        } else {
            MatchChange::Changed
        })
    }

    fn clear_expired_votes(&mut self, affected: [bool; 2]) -> Result<(), MatchError> {
        let votes = self.series.rematch_votes();
        for index in 0..2 {
            if affected[index] && votes[index] {
                self.series
                    .set_rematch_vote(SeatId::DUEL[index], false, self.tick)?;
            }
        }
        Ok(())
    }
}
